use crate::compiler::ast::vm_ir::Value;
use crate::compiler::ast::vm_ir::Value::{Bool, Float, Int, String, Null};
use crate::runtime::executor::StackFrame;
use crate::runtime::RuntimeError;
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
        (Int(l), Float(r)) => {
            // 检查 i64 是否能被 f64 无损表示
            // f64 能精确表示的正负整数范围是 -(2^53 - 1) 到 (2^53 - 1)
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&l) {
                return Err(RuntimeError::PrecisionLoss(format_smolstr!(
                    "{l} not in safe int."
                )));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l as f64 + r))
        }

        // Float + Int → Float
        (Float(l), Int(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&r) {
                return Err(RuntimeError::PrecisionLoss(format_smolstr!(
                    "{r} not in safe int."
                )));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l + r as f64))
        }

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
        (Int(l), Float(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&l) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{l} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l as f64 - r))
        },
        (Float(l), Int(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&r) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{r} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l - r as f64))
        },
        (Float(l), Float(r)) => Ok(Float(l - r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format_smolstr!("{auto} to {auto1}"),
        )),
    }
}

pub fn mul_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Int(l * r)),
        (Int(l), Float(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&l) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{l} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l as f64 * r))
        },
        (Float(l), Int(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&r) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{r} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l * r as f64))
        },
        (Float(l), Float(r)) => Ok(Float(l * r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format_smolstr!("{auto} to {auto1}"),
        )),
    }
}

pub fn div_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Int(l / r)),
        (Int(l), Float(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&l) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{l} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l as f64 / r))
        },
        (Float(l), Int(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&r) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{r} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l / r as f64))
        },
        (Float(l), Float(r)) => Ok(Float(l / r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format_smolstr!("{auto} to {auto1}"),
        )),
    }
}

pub fn rmd_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Int(l % r)),
        (Int(l), Float(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&l) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{l} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l as f64 % r))
        },
        (Float(l), Int(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&r) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{r} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l % r as f64))
        },
        (Float(l), Float(r)) => Ok(Float(l % r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format_smolstr!("{auto} to {auto1}"),
        )),
    }
}


pub fn equ_value(stack_frame: &mut StackFrame) {
    let right = stack_frame.pop_op_stack();
    let left = stack_frame.pop_op_stack();
    let value = match (left, right) {
        (Int(l), Int(r)) => Bool(l == r),
        (Float(l), Float(r)) => Bool((l - r).abs() < f64::EPSILON),
        (String(l), String(r)) => Bool(l.as_str() == r.as_str()),
        (Null, Null) => Bool(true),
        (Bool(l), Bool(r)) => Bool(l == r),
        _ => Bool(false),
    };
    stack_frame.push_op_stack(value);
    stack_frame.next_pc();
}

pub fn not_equ_value(stack_frame:&mut StackFrame) {
    let right = stack_frame.pop_op_stack();
    let left = stack_frame.pop_op_stack();
    let value = match (left, right) {
        (Int(l), Int(r)) => Bool(l != r),
        (Float(l), Float(r)) => {
            let is_equal = (l - r).abs() < f64::EPSILON;
            Bool(!is_equal)
        },
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
        auto => return Err(RuntimeError::TypeException(
            format_smolstr!("{auto} to bool"),
        )),
    };
    stack_frame.push_op_stack(value);
    stack_frame.next_pc();
    Ok(())
}

pub fn big_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Bool(l > r)),
        (Int(l), Float(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&l) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{l} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Bool(l as f64 > r))
        },
        (Float(l), Int(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&r) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{r} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Bool(l > r as f64))
        },
        (Float(l), Float(r)) => Ok(Bool(l > r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format_smolstr!("{auto} to {auto1}"),
        )),
    }
}

pub fn less_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Bool(l < r)),
        (Int(l), Float(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&l) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{l} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Bool((l as f64) < r))
        },
        (Float(l), Int(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&r) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{r} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Bool(l < r as f64))
        },
        (Float(l), Float(r)) => Ok(Bool(l < r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format_smolstr!("{auto} to {auto1}"),
        )),
    }
}

pub fn self_add_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let var = stack_frame.pop_op_stack();
    match var {
        Int(i) => stack_frame.push_op_stack(Int(i + 1)),
        Float(f) => stack_frame.push_op_stack(Float(f + 1.0)),
        auto => return Err(RuntimeError::TypeException(
            format_smolstr!("{auto} to int or float"),
        )),
    }
    stack_frame.next_pc();
    Ok(())
}

pub fn self_sub_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let var = stack_frame.pop_op_stack();
    match var {
        Int(i) => stack_frame.push_op_stack(Int(i - 1)),
        Float(f) => stack_frame.push_op_stack(Float(f - 1.0)),
        auto => return Err(RuntimeError::TypeException(
            format_smolstr!("{auto} to int or float"),
        )),
    }
    stack_frame.next_pc();
    Ok(())
}

pub fn less_equ_value(left: Value, right: Value)-> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Bool(l <= r)),
        (Int(l), Float(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&l) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{l} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Bool((l as f64) <= r))
        },
        (Float(l), Int(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&r) {
                return Err(RuntimeError::PrecisionLoss(
                    format_smolstr!("{r} not in safe int."),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Bool(l <= r as f64))
        },
        (Float(l), Float(r)) => Ok(Bool(l <= r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format_smolstr!("{auto} to {auto1}"),
        )),
    }
}

pub fn neg_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let var = stack_frame.pop_op_stack();
    let value = match var {
        Int(l) => Int(-l),
        Float(f) => Float(-f),
        auto => return Err(RuntimeError::TypeException(
            format_smolstr!("{auto} to float or number"),
        )),
    };
    stack_frame.push_op_stack(value);
    stack_frame.next_pc();
    Ok(())
}

pub fn and_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let v1 = stack_frame.pop_op_stack();
    let v2 = stack_frame.pop_op_stack();
    if let Bool(b1) = v1 && let Bool(b2) = v2 {
        stack_frame.push_op_stack(Bool(b1 && b2));
        stack_frame.next_pc();
        Ok(())
    }else { 
        Err(RuntimeError::TypeException("unknown to bool.".to_smolstr()))
    }
}

pub fn or_value(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let v1 = stack_frame.pop_op_stack();
    let v2 = stack_frame.pop_op_stack();
    if let Bool(b1) = v1 && let Bool(b2) = v2 {
        stack_frame.push_op_stack(Bool(b1 || b2));
        stack_frame.next_pc();
        Ok(())
    }else {
        Err(RuntimeError::TypeException("unknown to bool.".to_smolstr()))
    }
}
