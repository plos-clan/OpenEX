use smol_str::ToSmolStr;
use crate::runtime::executor::Value;
use crate::runtime::RuntimeError;

pub(crate) fn add_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    use crate::runtime::executor::Value::*;
    if matches!(left, String(_)) || matches!(right, String(_)) {
        let left_str = left.to_string();
        let right_str = right.to_string();
        return Ok(String(left_str + &right_str));
    }

    match (left, right) {
        // Int + Int → Int
        (Int(l), Int(r)) => Ok(Int(l + r)),

        // Int + Float → Float
        (Int(l), Float(r)) => Ok(Float(l as f64 + r)),

        // Float + Int → Float
        (Float(l), Int(r)) => Ok(Float(l + r as f64)),

        // Float + Float → Float
        (Float(l), Float(r)) => Ok(Float(l + r)),

        (String(l), String(r)) => Ok(String(l + &r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{} to {}", auto, auto1).to_smolstr(),
        )),
    }
}

pub(crate) fn sub_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    use crate::runtime::executor::Value::*;

    match (left, right) {
        // Int + Int → Int
        (Int(l), Int(r)) => Ok(Int(l - r)),

        // Int + Float → Float
        (Int(l), Float(r)) => Ok(Float(l as f64 - r)),

        // Float + Int → Float
        (Float(l), Int(r)) => Ok(Float(l - r as f64)),

        // Float + Float → Float
        (Float(l), Float(r)) => Ok(Float(l - r)),

        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{} to {}", auto, auto1).to_smolstr(),
        )),
    }
}