use crate::runtime::executor::Value;
use crate::runtime::executor::Value::{String, Int, Float, Bool, Null};
use crate::runtime::RuntimeError;
use smol_str::ToSmolStr;

pub fn add_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    if matches!(left, String(_)) || matches!(right, String(_)) {
        let left_str = left.to_string();
        let right_str = right.to_string();
        return Ok(String(left_str + &right_str));
    }

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
                return Err(RuntimeError::PrecisionLoss(
                    format_args!("{l} not in safe int.").to_smolstr(),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l as f64 + r))
        }

        // Float + Int → Float
        (Float(l), Int(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&r) {
                return Err(RuntimeError::PrecisionLoss(
                    format_args!("{r} not in safe int.").to_smolstr(),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l + r as f64))
        },

        // Float + Float → Float
        (Float(l), Float(r)) => Ok(Float(l + r)),

        (String(l), String(r)) => Ok(String(l + &r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{auto} to {auto1}").to_smolstr(),
        )),
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
                    format_args!("{l} not in safe int.").to_smolstr(),
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
                    format_args!("{r} not in safe int.").to_smolstr(),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l - r as f64))
        },
        (Float(l), Float(r)) => Ok(Float(l - r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{auto} to {auto1}").to_smolstr(),
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
                    format_args!("{l} not in safe int.").to_smolstr(),
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
                    format_args!("{r} not in safe int.").to_smolstr(),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l * r as f64))
        },
        (Float(l), Float(r)) => Ok(Float(l * r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{auto} to {auto1}").to_smolstr(),
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
                    format_args!("{l} not in safe int.").to_smolstr(),
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
                    format_args!("{r} not in safe int.").to_smolstr(),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Float(l / r as f64))
        },
        (Float(l), Float(r)) => Ok(Float(l / r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{auto} to {auto1}").to_smolstr(),
        )),
    }
}

pub fn self_add_value(var: Value) -> Result<Value, RuntimeError> {
    match var {
        Int(i) => Ok(Int(i + 1)),
        Float(f) => Ok(Float(f + 1.0)),
        auto => Err(RuntimeError::TypeException(
            format!("{auto} to int or float").to_smolstr(),
        )),
    }
}

pub fn self_sub_value(var: Value) -> Result<Value, RuntimeError> {
    match var {
        Int(i) => Ok(Int(i - 1)),
        Float(f) => Ok(Float(f - 1.0)),
        auto => Err(RuntimeError::TypeException(
            format!("{auto} to int or float").to_smolstr(),
        )),
    }
}

pub fn big_value(left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Int(l), Int(r)) => Ok(Bool(l > r)),
        (Int(l), Float(r)) => {
            const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
            const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;

            if !(MIN_SAFE_INT..=MAX_SAFE_INT).contains(&l) {
                return Err(RuntimeError::PrecisionLoss(
                    format_args!("{l} not in safe int.").to_smolstr(),
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
                    format_args!("{r} not in safe int.").to_smolstr(),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Bool(l > r as f64))
        },
        (Float(l), Float(r)) => Ok(Bool(l > r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{auto} to {auto1}").to_smolstr(),
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
                    format_args!("{l} not in safe int.").to_smolstr(),
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
                    format_args!("{r} not in safe int.").to_smolstr(),
                ));
            }

            #[allow(clippy::cast_precision_loss)]
            Ok(Bool(l < r as f64))
        },
        (Float(l), Float(r)) => Ok(Bool(l < r)),
        (auto, auto1) => Err(RuntimeError::TypeException(
            format!("{auto} to {auto1}").to_smolstr(),
        )),
    }
}

pub fn equ_value(left: Value, right: Value) -> Value {
    match (left, right) {
        (Int(l), Int(r)) => Bool(l == r),
        (Float(l), Float(r)) => Bool((l - r).abs() < f64::EPSILON),
        (String(l), String(r)) => Bool(l.as_str() == r.as_str()),
        (Null, Null) => Bool(true),
        (Bool(l), Bool(r)) => Bool(l == r),
        _ => Bool(false),
    }
}

pub fn not_equ_value(left: Value, right: Value) -> Value {
    match (left, right) {
        (Int(l), Int(r)) => Bool(l != r),
        (Float(l), Float(r)) => {
            let is_equal = (l - r).abs() < f64::EPSILON;
            Bool(!is_equal)
        },
        (String(l), String(r)) => Bool(l.as_str() != r.as_str()),
        (Null, Null) => Bool(false),
        (Bool(l), Bool(r)) => Bool(l != r),
        _ => Bool(true),
    }
}

pub fn not_value(var: Value) -> Result<Value, RuntimeError> {
    match var {
        Bool(l) => Ok(Bool(!l)),
        auto => Err(RuntimeError::TypeException(
            format!("{auto} to bool").to_smolstr(),
        )),
    }
}
