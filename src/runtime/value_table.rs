use crate::runtime::executor::{StackFrame, Value};

pub fn load_local(stack_frame: &mut StackFrame, index: usize) {
    let result = stack_frame.pop_op_stack().unwrap();
    stack_frame.set_var_table(index, result);
    stack_frame.next_pc();
}

pub fn store_local(stack_frame: &mut StackFrame, index: usize) {
    let value = stack_frame.get_var_table(index);
    stack_frame.push_op_stack(value.clone());
    stack_frame.next_pc();
}

pub fn push_stack(stack_frame: &mut StackFrame, index: usize) {
    let Some(value_ref) = stack_frame.get_const(index) else { unimplemented!() };
    let final_value = if let Value::Ref(path) = value_ref {
        if path.as_str() == "this" {
            let file_base = stack_frame.file_name.split('.').next().unwrap_or("");
            Value::Ref(file_base.into())
        } else {
            value_ref.clone()
        }
    } else {
        value_ref.clone()
    };
    stack_frame.push_op_stack(final_value);
    stack_frame.next_pc();
}

