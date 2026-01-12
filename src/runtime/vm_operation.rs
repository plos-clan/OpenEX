use crate::compiler::ast::vm_ir::Value;
use crate::compiler::ast::vm_ir::Value::{Bool, Float, Int, Null, String};
use crate::runtime::executor::StackFrame;
use crate::runtime::RuntimeError;
use dashu::float::{Context, DBig};
use smol_str::{format_smolstr, ToSmolStr};

pub fn get_ref(stack_frame: &mut StackFrame) {
    let ref1 = stack_frame.pop_op_stack();
    let ref2 = stack_frame.pop_op_stack();
    if let Value::Ref(ref_top) = ref1
        && let Value::Ref(ref_bak) = ref2
    {
        let all_ref = format_smolstr!("{}/{}", ref_bak, ref_top);
        stack_frame.push_op_stack(Value::Ref(all_ref));
    } else {
        unreachable!()
    }
    stack_frame.next_pc();
}

pub fn add_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        // Int + Int → Int
        (Int(l), Int(r)) => Ok(Int(l + r)),

        // Int + Float → Float
        (Int(l), Float(r)) => Ok(Float(DBig::from(l) + r)),

        // Float + Int → Float
        (Float(l), Int(r)) => Ok(Float(l + DBig::from(r))),

        // Float + Float → Float
        (Float(l), Float(r)) => Ok(Float(l + r)),

        (String(l), String(r)) => Ok(String(format_smolstr!("{l}{r}"))),
        // any + String → String
        (lhs, String(r)) => Ok(String(format_smolstr!("{lhs}{r}"))),
        // String + any → String
        (String(l), rhs) => Ok(String(format_smolstr!("{l}{rhs}"))),
        (auto, auto1) => Err(RuntimeError::TypeException(format_smolstr!(
            "{auto} to {auto1}"
        ))),
    }
}

pub fn sub_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Int(l - r)),
        (Int(l), Float(r)) => Ok(Float(DBig::from(l) - r)),
        (Float(l), Int(r)) => Ok(Float(l - DBig::from(r))),
        (Float(l), Float(r)) => Ok(Float(l - r)),
        (auto, auto1) => Err(RuntimeError::TypeException(format_smolstr!(
            "{auto} to {auto1}"
        ))),
    }
}

pub fn mul_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Int(l * r)),
        (Int(l), Float(r)) => Ok(Float(DBig::from(l) * r)),
        (Float(l), Int(r)) => Ok(Float(l * DBig::from(r))),
        (Float(l), Float(r)) => Ok(Float(l * r)),
        (auto, auto1) => Err(RuntimeError::TypeException(format_smolstr!(
            "{auto} to {auto1}"
        ))),
    }
}

pub fn div_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Int(l / r)),
        (Int(l), Float(r)) => {
            let context = Context::new(30);
            Ok(Float(context.div(DBig::from(l).repr(), r.repr()).value()))
        }
        (Float(l), Int(r)) => {
            let context = Context::new(30);
            Ok(Float(context.div(l.repr(), DBig::from(r).repr()).value()))
        }
        (Float(l), Float(r)) => Ok(Float(l / r)),
        (auto, auto1) => Err(RuntimeError::TypeException(format_smolstr!(
            "{auto} to {auto1}"
        ))),
    }
}

pub fn rmd_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Int(l % r)),
        (Int(l), Float(r)) => {
            let context = Context::new(30);
            Ok(Float(context.rem(DBig::from(l).repr(), r.repr()).value()))
        }
        (Float(l), Int(r)) => {
            let context = Context::new(30);
            Ok(Float(context.rem(l.repr(), DBig::from(r).repr()).value()))
        }
        (Float(l), Float(r)) => Ok(Float(l % r)),
        (auto, auto1) => Err(RuntimeError::TypeException(format_smolstr!(
            "{auto} to {auto1}"
        ))),
    }
}

pub fn equ_value(stack_frame: &mut StackFrame) {
    let right = stack_frame.pop_op_stack();
    let left = stack_frame.pop_op_stack();
    let value = match (left, right) {
        (Int(l), Int(r)) => Bool(l == r),
        (Float(l), Float(r)) => Bool(l == r),
        (String(l), String(r)) => Bool(l.as_str() == r.as_str()),
        (Null, Null) => Bool(true),
        (Bool(l), Bool(r)) => Bool(l == r),
        _ => Bool(false),
    };
    stack_frame.push_op_stack(value);
    stack_frame.next_pc();
}

pub fn not_equ_value(stack_frame: &mut StackFrame) {
    let right = stack_frame.pop_op_stack();
    let left = stack_frame.pop_op_stack();
    let value = match (left, right) {
        (Int(l), Int(r)) => Bool(l != r),
        (Float(l), Float(r)) => Bool(l != r),
        (String(l), String(r)) => Bool(l.as_str() != r.as_str()),
        (Null, Null) => Bool(false),
        (Bool(l), Bool(r)) => Bool(l != r),
        _ => Bool(true),
    };
    stack_frame.push_op_stack(value);
    stack_frame.next_pc();
}

pub fn not_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let var = stack_frame.pop_op_stack();
    let value = match var {
        Bool(l) => Bool(!l),
        auto => {
            return Err(RuntimeError::TypeException(format_smolstr!(
                "{auto} to bool"
            )));
        }
    };
    stack_frame.push_op_stack(value);
    stack_frame.next_pc();
    Ok(())
}

pub fn big_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Bool(l > r)),
        (Int(l), Float(r)) => Ok(Bool(DBig::from(l) > r)),
        (Float(l), Int(r)) => Ok(Bool(l > DBig::from(r))),
        (Float(l), Float(r)) => Ok(Bool(l > r)),
        (auto, auto1) => Err(RuntimeError::TypeException(format_smolstr!(
            "{auto} to {auto1}"
        ))),
    }
}

pub fn less_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Bool(l < r)),
        (Int(l), Float(r)) => Ok(Bool(DBig::from(l) < r)),
        (Float(l), Int(r)) => Ok(Bool(l < DBig::from(r))),
        (Float(l), Float(r)) => Ok(Bool(l < r)),
        (auto, auto1) => Err(RuntimeError::TypeException(format_smolstr!(
            "{auto} to {auto1}"
        ))),
    }
}

pub fn self_add_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let var = stack_frame.pop_op_stack();
    match var {
        Int(i) => stack_frame.push_op_stack(Int(i + 1)),
        Float(f) => stack_frame.push_op_stack(Float(f + DBig::from(1))),
        auto => {
            return Err(RuntimeError::TypeException(format_smolstr!(
                "{auto} to int or float"
            )));
        }
    }
    stack_frame.next_pc();
    Ok(())
}

pub fn self_sub_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let var = stack_frame.pop_op_stack();
    match var {
        Int(i) => stack_frame.push_op_stack(Int(i - 1)),
        Float(f) => stack_frame.push_op_stack(Float(f - DBig::from(1))),
        auto => {
            return Err(RuntimeError::TypeException(format_smolstr!(
                "{auto} to int or float"
            )));
        }
    }
    stack_frame.next_pc();
    Ok(())
}

pub fn less_equ_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Bool(l <= r)),
        (Int(l), Float(r)) => Ok(Bool(DBig::from(l) <= r)),
        (Float(l), Int(r)) => Ok(Bool(l <= DBig::from(r))),
        (Float(l), Float(r)) => Ok(Bool(l <= r)),
        (auto, auto1) => Err(RuntimeError::TypeException(format_smolstr!(
            "{auto} to {auto1}"
        ))),
    }
}

pub fn neg_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let var = stack_frame.pop_op_stack();
    let value = match var {
        Int(l) => Int(-l),
        Float(f) => Float(-f),
        auto => {
            return Err(RuntimeError::TypeException(format_smolstr!(
                "{auto} to float or number"
            )));
        }
    };
    stack_frame.push_op_stack(value);
    stack_frame.next_pc();
    Ok(())
}

pub fn and_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let v1 = stack_frame.pop_op_stack();
    let v2 = stack_frame.pop_op_stack();
    if let Bool(b1) = v1
        && let Bool(b2) = v2
    {
        stack_frame.push_op_stack(Bool(b1 && b2));
        stack_frame.next_pc();
        Ok(())
    } else {
        Err(RuntimeError::TypeException("unknown to bool.".to_smolstr()))
    }
}

pub fn or_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let v1 = stack_frame.pop_op_stack();
    let v2 = stack_frame.pop_op_stack();
    if let Bool(b1) = v1
        && let Bool(b2) = v2
    {
        stack_frame.push_op_stack(Bool(b1 || b2));
        stack_frame.next_pc();
        Ok(())
    } else {
        Err(RuntimeError::TypeException("unknown to bool.".to_smolstr()))
    }
}

pub fn bit_left_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let right = stack_frame.pop_op_stack();
    let left = stack_frame.pop_op_stack();
    match (left,right) {
        (Int(i), Int(r)) => {
            stack_frame.push_op_stack(Int(i << r));
        },
        _=> return Err(RuntimeError::TypeException("bit left need number".to_smolstr())),
    };
    stack_frame.next_pc();
    Ok(())
}

pub fn bit_right_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let right = stack_frame.pop_op_stack();
    let left = stack_frame.pop_op_stack();
    match (left,right) {
        (Int(i), Int(r)) => {
            stack_frame.push_op_stack(Int(i >> r));
        },
        _=> return Err(RuntimeError::TypeException("bit right need number".to_smolstr())),
    };
    stack_frame.next_pc();
    Ok(())
}

pub fn bit_and_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let right = stack_frame.pop_op_stack();
    let left = stack_frame.pop_op_stack();
    match (left,right) {
        (Int(i), Int(r)) => {
            stack_frame.push_op_stack(Int(i & r));
        },
        _=> return Err(RuntimeError::TypeException("bit and need number".to_smolstr())),
    };
    stack_frame.next_pc();
    Ok(())
}

pub fn bit_or_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let right = stack_frame.pop_op_stack();
    let left = stack_frame.pop_op_stack();
    match (left,right) {
        (Int(i), Int(r)) => {
            stack_frame.push_op_stack(Int(i | r));
        },
        _=> return Err(RuntimeError::TypeException("bit or need number".to_smolstr())),
    };
    stack_frame.next_pc();
    Ok(())
}

pub fn bit_xor_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let right = stack_frame.pop_op_stack();
    let left = stack_frame.pop_op_stack();
    match (left,right) {
        (Int(i), Int(r)) => {
            stack_frame.push_op_stack(Int(i ^ r));
        },
        _=> return Err(RuntimeError::TypeException("bit xor need number".to_smolstr())),
    };
    stack_frame.next_pc();
    Ok(())
}
