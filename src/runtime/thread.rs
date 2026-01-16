use std::thread::Scope;

use crate::runtime::executor::interpretive;
use crate::runtime::{MetadataUnit, MethodInfo, SharedGlobals, SharedSync};

pub struct ThreadManager<'scope, 'env> {
    scope: &'scope Scope<'scope, 'env>,
}

impl<'scope, 'env> ThreadManager<'scope, 'env> {
    pub const fn new(scope: &'scope Scope<'scope, 'env>) -> Self {
        Self { scope }
    }

    pub fn submit_run_thread(
        &self,
        unit_index: usize,
        metadata: &'env MetadataUnit,
        unit: &'env MethodInfo,
        units: &'env [MetadataUnit],
        globals: SharedGlobals,
        sync_table: SharedSync,
    ) {
        let globals = globals.clone();
        let thread_manager = self as *const _ as usize;
        let sync_table = sync_table.clone();
        self.scope.spawn(move || {
            interpretive(
                unit.get_codes(),
                metadata.constant_table,
                metadata.names,
                units,
                unit_index,
                metadata.globals,
                globals,
                sync_table,
                Some(thread_manager),
            );
        });
    }

    pub fn submit_join_thread(
        &self,
        unit_index: usize,
        metadata: &'env MetadataUnit,
        unit: &'env MethodInfo,
        units: &'env [MetadataUnit],
        globals: SharedGlobals,
        sync_table: SharedSync,
    ) {
        let globals = globals.clone();
        let thread_manager = self as *const _ as usize;
        let sync_table = sync_table.clone();
        self.scope
            .spawn(move || {
                interpretive(
                    unit.get_codes(),
                    metadata.constant_table,
                    metadata.names,
                    units,
                    unit_index,
                    metadata.globals,
                    globals,
                    sync_table,
                    Some(thread_manager),
                );
            })
            .join()
            .unwrap();
    }
}
