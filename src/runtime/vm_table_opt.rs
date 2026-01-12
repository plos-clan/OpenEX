use crate::compiler::ast::vm_ir::Value;
use crate::runtime::executor::{RunState, StackFrame};
use crate::runtime::{MetadataUnit, RuntimeError};
use smol_str::ToSmolStr;

pub fn push_stack(stack_frame: &mut StackFrame, index: usize) {
    let Some(value_ref) = stack_frame.get_const(index) else {
        unimplemented!()
    };
    let final_value = if let Value::Ref(path) = value_ref {
        if path.as_str() == "this" {
            let file_base = stack_frame.r_name;
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

pub fn load_local(stack_frame: &mut StackFrame, index: usize) {
    let result = stack_frame.pop_op_stack();
    stack_frame.set_local(index, result);
    stack_frame.next_pc();
}

pub fn store_local(stack_frame: &mut StackFrame, index: usize) {
    let value = stack_frame.get_local(index);
    stack_frame.push_op_stack(value.clone());
    stack_frame.next_pc();
}

pub fn jump_true(stack_frame: &mut StackFrame, jpc: usize) {
    let top = stack_frame.pop_op_stack();
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
    let top = stack_frame.pop_op_stack();
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

pub const fn jump(stack_frame: &mut StackFrame, jpc: usize) {
    stack_frame.set_next_pc(jpc);
}

pub fn call_func<'a>(
    stack_frame: &mut StackFrame,
    units: &'a [MetadataUnit],
) -> Result<RunState<'a>, RuntimeError> {
    let result = stack_frame.pop_op_stack();

    if let Value::Ref(path) = result {
        let panic_path = path.clone();

        let mut sp = path.split('/');
        let file = sp.next().unwrap();
        let func_name = sp.next().unwrap();

        for unit in units {
            if unit.names == file {
                for func in &unit.methods {
                    if func.name == func_name {
                        let codes = func.get_codes();
                        stack_frame.next_pc();
                        let native = if func.is_native { Some(path) } else { None };
                        return Ok(RunState::CallRequest(StackFrame::new(
                            func.locals,
                            codes,
                            unit.constant_table,
                            func.name.as_str(),
                            func.r_name.as_str(),
                            native,
                            func.args,
                        )));
                    }
                }
            }
        }
        Err(RuntimeError::NoSuchFunctionException(panic_path))
    } else {
        Err(RuntimeError::VMError)
    }
}

pub fn load_array_local(stack_frame: &mut StackFrame, len: usize, index: usize) {
    let mut elements: Vec<Value> = Vec::new();
    for _ in 0..len {
        elements.push(stack_frame.pop_op_stack());
    }
    let reversed_values: Vec<Value> = elements.into_iter().rev().collect();
    let result = Value::Array(len, reversed_values);
    stack_frame.set_local(index, result);
    stack_frame.next_pc();
}

pub fn set_index_array(stack_frame: &mut StackFrame, index: usize) -> Result<(), RuntimeError> {
    let arr_index = stack_frame.pop_op_stack();
    let value = stack_frame.pop_op_stack();
    let result = stack_frame.get_local_mut(index);

    if let Value::Array(len, elements) = result
        && let Value::Int(a_index) = arr_index
    {
        let usize_index = usize::try_from(a_index).unwrap();
        if usize_index >= *len {
            return Err(RuntimeError::IndexOutOfBounds(
                format_args!("Index {a_index} out of bounds for length {len}").to_smolstr(),
            ));
        }
        elements[usize_index] = value;
        stack_frame.next_pc();
        Ok(())
    } else {
        Err(RuntimeError::TypeException(
            "cannot set unknown type for array.".to_smolstr(),
        ))
    }
}

pub fn get_index_array(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let index = stack_frame.pop_op_stack();
    let array = stack_frame.pop_op_stack();
    if let Value::Int(index) = index
        && let Value::Array { 0: len, 1: element } = array
    {
        if usize::try_from(index).unwrap() >= len {
            return Err(RuntimeError::IndexOutOfBounds(
                format_args!("Index {index} out of bounds for length {len}").to_smolstr(),
            ));
        }
        let result = element
            .get(usize::try_from(index).unwrap())
            .unwrap()
            .clone();
        stack_frame.push_op_stack(result);
        stack_frame.next_pc();
        Ok(())
    } else {
        Err(RuntimeError::TypeException(
            "cannot get_index unknown type.".to_smolstr(),
        ))
    }
}
