use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crate::runtime::thread::ThreadManager;
use crate::runtime::{MetadataUnit, SharedGlobals};

struct LockState {
    owner: Option<thread::ThreadId>,
    count: usize,
}

pub struct FunctionLock {
    state: Mutex<LockState>,
    cvar: Condvar,
}

impl FunctionLock {
    fn new() -> Self {
        Self {
            state: Mutex::new(LockState {
                owner: None,
                count: 0,
            }),
            cvar: Condvar::new(),
        }
    }

    fn lock(&self) {
        let tid = thread::current().id();
        let mut state = self.state.lock().unwrap();
        loop {
            match state.owner {
                None => {
                    state.owner = Some(tid);
                    state.count = 1;
                    return;
                }
                Some(owner) if owner == tid => {
                    state.count += 1;
                    return;
                }
                _ => {
                    state = self.cvar.wait(state).unwrap();
                }
            }
        }
    }

    fn unlock(&self) {
        let tid = thread::current().id();
        let mut state = self.state.lock().unwrap();
        if state.owner == Some(tid) {
            state.count = state.count.saturating_sub(1);
            if state.count == 0 {
                state.owner = None;
                self.cvar.notify_one();
            }
        }
    }
}

pub struct SyncTable {
    locks: HashMap<(usize, usize), Arc<FunctionLock>>,
}

pub type SharedSync = Arc<SyncTable>;

impl SyncTable {
    pub fn new(units: &[MetadataUnit<'_>]) -> Self {
        let mut locks = HashMap::new();
        for (unit_index, unit) in units.iter().enumerate() {
            for (func_index, func) in unit.methods.iter().enumerate() {
                if func.sync {
                    locks.insert((unit_index, func_index), Arc::new(FunctionLock::new()));
                }
            }
        }
        Self { locks }
    }

    pub fn shared_new(units: &[MetadataUnit<'_>]) -> SharedSync {
        Arc::new(Self::new(units))
    }

    pub fn lock_if_sync(&self, unit_index: usize, func_index: usize) -> bool {
        if let Some(lock) = self.locks.get(&(unit_index, func_index)) {
            lock.lock();
            true
        } else {
            false
        }
    }

    pub fn unlock(&self, unit_index: usize, func_index: usize) {
        if let Some(lock) = self.locks.get(&(unit_index, func_index)) {
            lock.unlock();
        }
    }
}

pub struct RuntimeContext {
    units_ptr: *const MetadataUnit<'static>,
    units_len: usize,
    globals: SharedGlobals,
    sync_table: SharedSync,
    thread_manager: Option<usize>,
}

thread_local! {
    static CONTEXT: RefCell<Option<RuntimeContext>> = RefCell::new(None);
    static THREAD_EXIT: Cell<bool> = Cell::new(false);
}

pub fn set_context(
    units: &[MetadataUnit<'_>],
    globals: SharedGlobals,
    sync_table: SharedSync,
    thread_manager: Option<usize>,
) {
    let units_ptr: *const MetadataUnit<'static> = units.as_ptr() as *const MetadataUnit<'static>;
    let units_len = units.len();
    CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(RuntimeContext {
            units_ptr,
            units_len,
            globals,
            sync_table,
            thread_manager,
        });
    });
}

pub fn clear_context() {
    CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = None;
    });
}

pub fn with_context<T>(f: impl FnOnce(&RuntimeContext) -> T) -> Option<T> {
    CONTEXT.with(|ctx| ctx.borrow().as_ref().map(f))
}

pub fn get_units(ctx: &RuntimeContext) -> &[MetadataUnit<'_>] {
    unsafe { std::slice::from_raw_parts(ctx.units_ptr, ctx.units_len) }
}

pub fn get_globals(ctx: &RuntimeContext) -> SharedGlobals {
    ctx.globals.clone()
}

pub fn get_sync_table(ctx: &RuntimeContext) -> SharedSync {
    ctx.sync_table.clone()
}

pub fn get_thread_manager(ctx: &RuntimeContext) -> Option<&ThreadManager<'_, '_>> {
    ctx.thread_manager
        .map(|ptr| unsafe { &*(ptr as *const ThreadManager<'_, '_>) })
}

pub fn request_thread_exit() {
    THREAD_EXIT.set(true);
}

pub fn take_thread_exit() -> bool {
    THREAD_EXIT.with(|flag| {
        let v = flag.get();
        flag.set(false);
        v
    })
}
