use crate::runtime::executor::{Executor, StackFrame, Value};
use crate::runtime::RuntimeError;

pub fn jump_true(stack_frame: &mut StackFrame, jpc: usize) {
    let top = stack_frame.pop_op_stack().unwrap();
    if let Value::Bool(value) = top {
        if value {
            stack_frame.set_next_pc(jpc);
        } else {
            stack_frame.next_pc();
        }
    } else {
        unreachable!()
    }
}

pub fn jump_false(stack_frame: &mut StackFrame, jpc: usize) {
    let top = stack_frame.pop_op_stack().unwrap();
    if let Value::Bool(value) = top {
        if value {
            stack_frame.next_pc();
        } else {
            stack_frame.set_next_pc(jpc);
        }
    } else {
        unreachable!()
    }
}

pub const fn jump(stack_frame:&mut StackFrame, jpc: usize) {
    stack_frame.set_next_pc(jpc);
}

pub fn call_func(stack_frame: &mut StackFrame,executor: &mut Executor) -> Result<(Option<StackFrame>, bool), RuntimeError> {
    let result = stack_frame.pop_op_stack().unwrap();
    if let Value::Ref(path) = result {
        let panic_path = path.clone();
        if let Some(function) = executor.get_path_func(&path) {
            let func = function.1;
            let codes = func.clone_codes();
            stack_frame.next_pc();
            let native = if func.is_native {
                Some(panic_path)
            } else {
                None
            };
            Ok((
                Some(StackFrame::new(
                    func.name.to_string(),
                    func.filename,
                    codes,
                    func.locals,
                    function.0,
                    native,
                    func.args,
                )),
                false,
            ))
        } else {
            Err(RuntimeError::NoSuchFunctionException(panic_path))
        }
    } else {
        Err(RuntimeError::VMError)
    }
}
