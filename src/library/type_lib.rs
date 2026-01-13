use dashu::float::DBig;
use smol_str::{SmolStr, ToSmolStr};
use std::str::FromStr;

use crate::compiler::ast::vm_ir::Value;
use crate::library::{LibModule, ModuleFunc, register_library};
use crate::runtime::RuntimeError;

#[allow(clippy::unnecessary_wraps)]
fn type_to_number(args: &[Value]) -> Result<Value, RuntimeError> {
    let auto = args.first().unwrap().clone();
    if let Value::String(raw_str) = auto {
        let i = raw_str.as_str().parse::<i64>().unwrap();
        Ok(Value::Int(i))
    } else if let Value::Float(raw_float) = args.first().unwrap() {
        let i = raw_float.trunc().to_int().value().try_into().unwrap();
        Ok(Value::Int(i))
    } else {
        Err(RuntimeError::TypeException(
            "to_number: auto not a string or float.".to_smolstr(),
        ))
    }
}

fn reg_to_number() -> ModuleFunc {
    ModuleFunc {
        name: SmolStr::new("to_number"),
        arity: 1,
        func: type_to_number,
    }
}

#[allow(clippy::unnecessary_wraps)]
fn type_to_float(args: &[Value]) -> Result<Value, RuntimeError> {
    let auto = args.first().unwrap().clone();
    if let Value::String(raw_str) = auto {
        Ok(Value::Float(DBig::from_str(raw_str.as_str()).unwrap()))
    } else if let Value::Int(raw_number) = args.first().unwrap() {
        Ok(Value::Float(DBig::from(*raw_number)))
    } else {
        Err(RuntimeError::TypeException(
            "to_float: auto not a string or number.".to_smolstr(),
        ))
    }
}

fn reg_to_float() -> ModuleFunc {
    ModuleFunc {
        name: SmolStr::new("to_float"),
        arity: 1,
        func: type_to_float,
    }
}

#[allow(clippy::unnecessary_wraps)]
fn type_check_type(args: &[Value]) -> Result<Value, RuntimeError> {
    let auto = args.first().unwrap().clone();
    match auto {
        Value::String(_) => Ok(Value::String("string".to_smolstr())),
        Value::Float(_) => Ok(Value::String("float".to_smolstr())),
        Value::Int(_) => Ok(Value::String("number".to_smolstr())),
        Value::Bool(_) => Ok(Value::String("bool".to_smolstr())),
        Value::Array(..) => Ok(Value::String("array".to_smolstr())),
        Value::Ref(_) => Ok(Value::String("ref".to_smolstr())),
        Value::Null => Ok(Value::String("null".to_smolstr())),
    }
}

fn reg_check_type() -> ModuleFunc {
    ModuleFunc {
        name: SmolStr::new("check_type"),
        arity: 1,
        func: type_check_type,
    }
}

#[allow(clippy::unnecessary_wraps)]
fn type_array_length(args: &[Value]) -> Result<Value, RuntimeError> {
    let Value::Array(len, _) = args.first().unwrap().clone() else {
        return Err(RuntimeError::TypeException(
            "array_length: arg is not array.".to_smolstr(),
        ));
    };
    Ok(Value::Int(i64::try_from(len).unwrap()))
}

fn reg_array_length() -> ModuleFunc {
    ModuleFunc {
        name: SmolStr::new("array_length"),
        arity: 1,
        func: type_array_length,
    }
}

#[allow(clippy::unnecessary_wraps)]
fn type_array_fill(args: &[Value]) -> Result<Value, RuntimeError> {
    let Some(value) = args.first().cloned() else {
        return Err(RuntimeError::TypeException(
            "array_fill: missing value.".to_smolstr(),
        ));
    };
    let Some(count) = args.get(1) else {
        return Err(RuntimeError::TypeException(
            "array_fill: missing length.".to_smolstr(),
        ));
    };
    let Value::Int(len) = count else {
        return Err(RuntimeError::TypeException(
            "array_fill: length is not int.".to_smolstr(),
        ));
    };
    let len = usize::try_from(*len)
        .map_err(|_| RuntimeError::TypeException("array_fill: length is negative.".to_smolstr()))?;
    Ok(Value::Array(len, vec![value; len]))
}

fn reg_array_fill() -> ModuleFunc {
    ModuleFunc {
        name: SmolStr::new("array_fill"),
        arity: 2,
        func: type_array_fill,
    }
}

#[allow(clippy::unnecessary_wraps)]
fn type_length(args: &[Value]) -> Result<Value, RuntimeError> {
    if let Value::String(raw_str) = args.first().unwrap() {
        return Ok(Value::Int(raw_str.len() as i64));
    }

    let Value::Array(len, _) = args.first().unwrap().clone() else {
        return Err(RuntimeError::TypeException(
            "_length: arg is not array or string".to_smolstr(),
        ));
    };
    Ok(Value::Int(i64::try_from(len).unwrap()))
}

fn reg_length() -> ModuleFunc {
    ModuleFunc {
        name: SmolStr::new("_length"),
        arity: 1,
        func: type_length,
    }
}

pub fn register_type_lib() {
    let mut type_lib = LibModule {
        name: SmolStr::new("type"),
        functions: vec![],
    };
    type_lib.functions.push(reg_to_number());
    type_lib.functions.push(reg_to_float());
    type_lib.functions.push(reg_check_type());
    type_lib.functions.push(reg_array_length());
    type_lib.functions.push(reg_array_fill());
    type_lib.functions.push(reg_length());
    register_library(type_lib);
}
