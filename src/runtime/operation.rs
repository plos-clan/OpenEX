use crate::runtime::executor::Value;
use crate::runtime::executor::Value::*;
use crate::runtime::RuntimeError;
use smol_str::ToSmolStr;

pub(crate) fn add_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
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
    match (left, right) {
        (Int(l), Int(r)) => Ok(Int(l - r)),
        (Int(l), Float(r)) => Ok(Float(l as f64 - r)),
        (Float(l), Int(r)) => Ok(Float(l - r as f64)),
        (Float(l), Float(r)) => Ok(Float(l - r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{} to {}", auto, auto1).to_smolstr(),
        )),
    }
}

pub(crate) fn mul_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Int(l * r)),
        (Int(l), Float(r)) => Ok(Float(l as f64 * r)),
        (Float(l), Int(r)) => Ok(Float(l * r as f64)),
        (Float(l), Float(r)) => Ok(Float(l * r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{} to {}", auto, auto1).to_smolstr(),
        )),
    }
}

pub(crate) fn div_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Int(l / r)),
        (Int(l), Float(r)) => Ok(Float(l as f64 / r)),
        (Float(l), Int(r)) => Ok(Float(l / r as f64)),
        (Float(l), Float(r)) => Ok(Float(l / r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{} to {}", auto, auto1).to_smolstr(),
        )),
    }
}

pub(crate) fn self_add_value(var: Value) -> Result<Value, RuntimeError> {
    match var {
        Int(i) => Ok(Int(i + 1)),
        Float(f) => Ok(Float(f + 1.0)),
        auto => Err(RuntimeError::TypeException(
            format!("{} to int or float", auto).to_smolstr(),
        )),
    }
}

pub(crate) fn self_sub_value(var: Value) -> Result<Value, RuntimeError> {
    match var {
        Int(i) => Ok(Int(i - 1)),
        Float(f) => Ok(Float(f - 1.0)),
        auto => Err(RuntimeError::TypeException(
            format!("{} to int or float", auto).to_smolstr(),
        )),
    }
}

pub(crate) fn big_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Bool(l > r)),
        (Int(l), Float(r)) => Ok(Bool(l as f64 > r)),
        (Float(l), Int(r)) => Ok(Bool(l > r as f64)),
        (Float(l), Float(r)) => Ok(Bool(l > r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{} to {}", auto, auto1).to_smolstr(),
        )),
    }
}

pub(crate) fn less_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Bool(l < r)),
        (Int(l), Float(r)) => Ok(Bool((l as f64) < r)),
        (Float(l), Int(r)) => Ok(Bool(l < r as f64)),
        (Float(l), Float(r)) => Ok(Bool(l < r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{} to {}", auto, auto1).to_smolstr(),
        )),
    }
}
