use crate::runtime::executor::StackFrame;

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

