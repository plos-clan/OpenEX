use crate::compiler::ast::vm_ir::VMIRTable;
use crate::compiler::parser::ParserError;
use crate::library::find_library;
use crate::runtime::executor::{run_executor, Executor, StackFrame};
use crate::runtime::RuntimeError;
use crossbeam_channel::{unbounded, Sender};
use smol_str::SmolStr;
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    sender: Sender<Job>,
}

static THREAD_POOL: LazyLock<ThreadPool> = LazyLock::new(|| ThreadPool::new(4));

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = unbounded::<Job>();

        for _ in 0..size {
            let rx = receiver.clone();
            thread::spawn(move || {
                loop {
                    let job = rx.recv().unwrap();
                    job();
                }
            });
        }

        ThreadPool { sender }
    }

    pub fn submit<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(job)).unwrap();
    }

    pub fn submit_with_join<F, R>(&self, job: F) -> thread::JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        thread::spawn(job)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // TODO
pub struct OpenEXThread {
    name: String,
    pub(crate) call_stack: Vec<StackFrame>,
}

impl OpenEXThread {
    pub fn new(name: String) -> OpenEXThread {
        Self {
            name,
            call_stack: vec![],
        }
    }

    pub fn get_mut_frame(&mut self, index: usize) -> &mut StackFrame {
        self.call_stack.get_mut(index).unwrap()
    }

    pub fn push_frame(&mut self, stack_frame: StackFrame) {
        self.call_stack.push(stack_frame);
    }

    fn run_exec(&mut self, executor: &mut Executor, tables: Arc<VMIRTable>, filename: SmolStr) {
        self.push_frame(StackFrame::new(
            "root".to_string(),
            filename,
            tables.clone_codes(),
            tables.get_locals_len(),
            tables.get_constant_table(),
            None,
            0,
        ));

        loop {
            let frame_index = match self.call_stack.len() {
                0 => break, // 栈空，程序结束
                n => n - 1,
            };
            let stack_frame = self.get_mut_frame(frame_index);

            if let Some(path) = stack_frame.is_native() {
                let mut sp = path.split('/');
                let file = sp.next().unwrap();
                let func = sp.next().unwrap();

                let mut argument = vec![];
                for _i in 0..stack_frame.get_args() {
                    argument.push(stack_frame.pop_op_stack().unwrap());
                }

                match find_library(file, |f| {
                    if let Some(lib) = f
                        && let Some(func) = lib.find_func(&SmolStr::new(func))
                    {
                        match (func.func)(argument) {
                            Ok(value) => {
                                self.call_stack.last_mut().unwrap().push_op_stack(value);
                                self.call_stack.pop().unwrap();
                                Ok(())
                            }
                            Err(_) => Err(ParserError::Empty),
                        }
                    } else {
                        Err(ParserError::Empty)
                    }
                }) {
                    Ok(_lib) => {}
                    Err(_) => {
                        eprintln!(
                            "RuntimeError: {:?}",
                            RuntimeError::NoSuchFunctionException(path)
                        );
                        for frame in self.call_stack.iter_mut() {
                            let name = frame.get_frame_name();
                            eprintln!("\t at <{}>::{}", name.1, name.0)
                        }
                        break;
                    }
                }
            } else {

                match run_executor(frame_index, executor, self) {
                    Ok(frame_o) => {
                        // 有函数调用
                        if let Some(frame) = frame_o.0 {
                            self.push_frame(frame);
                            // 将父栈帧参数压入子栈帧
                            let call_stack = &mut self.call_stack;
                            let mut stack_frame = call_stack.pop().unwrap();
                            if stack_frame.get_args() > 0 {
                                let mut parent_frame = call_stack.pop().unwrap();
                                for _i in 0..stack_frame.get_args() {
                                    let value = parent_frame.pop_op_stack().unwrap();
                                    stack_frame.push_op_stack(value);
                                }
                                call_stack.push(parent_frame);
                            }
                            call_stack.push(stack_frame);
                        } else {
                            let frame = self.call_stack.pop().unwrap();
                            if frame_o.1
                                && let Some(ret_var) = frame.get_op_stack_top()
                            {
                                self.call_stack.last_mut().unwrap().push_op_stack(ret_var);
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!("RuntimeError: {:?}", error);
                        for frame in self.call_stack.iter_mut() {
                            let name = frame.get_frame_name();
                            eprintln!("\t at <{}>::{}", name.1, name.0)
                        }
                        break;
                    }
                }
            }
        }
    }
}

fn make_executor_job(
    executor: Arc<RwLock<Executor>>,
    name_l: Mutex<SmolStr>,
    table_l: Arc<VMIRTable>,
    filename: SmolStr,
) -> impl FnOnce() + Send + 'static {
    move || {
        let sync_n = name_l.lock().unwrap().clone();
        let mut ex_rd = executor.read().unwrap().clone();
        let mut exec = executor.write().unwrap();
        let thread = exec.add_thread(OpenEXThread::new(sync_n.to_string()));
        thread.run_exec(&mut ex_rd, table_l.clone(), filename.clone());
    }
}

#[allow(dead_code)] //TODO
pub fn add_thread_run(
    executor: Arc<RwLock<Executor>>,
    name: SmolStr,
    table: &VMIRTable,
    filename: SmolStr,
) {
    let name_l = Mutex::new(name);
    let table_l = Arc::new(table.clone());
    THREAD_POOL.submit(make_executor_job(executor, name_l, table_l, filename));
}

pub fn add_thread_join(
    executor: Arc<RwLock<Executor>>,
    name: SmolStr,
    table: &VMIRTable,
    filename: SmolStr,
) {
    let name_l = Mutex::new(name);
    let table_l = Arc::new(table.clone());
    let handle =
        THREAD_POOL.submit_with_join(make_executor_job(executor, name_l, table_l, filename));
    handle.join().unwrap();
}
