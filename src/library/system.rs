use smol_str::{SmolStr, ToSmolStr};
use std::io;
use std::io::Read;
use std::process::exit;

use crate::compiler::ast::vm_ir::Value;
use crate::library::{LibModule, ModuleFunc, output_capture::print, register_library};
use crate::runtime::RuntimeError;

fn print_impl(value: Value) {
    match value {
        Value::Int(i) => print(format_args!("{i}")),
        Value::Bool(i) => print(format_args!("{i}")),
        Value::Float(i) => print(format_args!("{i}")),
        Value::String(i) => print(format_args!("{i}")),
        Value::Ref(i) => print(format_args!("<ref:{i}>")),
        Value::Null => print(format_args!("null")),
        Value::Array(_i, ele) => {
            print(format_args!("["));
            for var in ele {
                print_impl(var);
                print(format_args!(","));
            }
            print(format_args!("]"));
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
fn system_print(args: &[Value]) -> Result<Value, RuntimeError> {
    let output = args.first().unwrap().clone();
    print_impl(output);
    Ok(Value::Null)
}
fn reg_println() -> ModuleFunc {
    ModuleFunc {
        name: SmolStr::new("print"),
        arity: 1,
        func: system_print,
    }
}

#[allow(clippy::unnecessary_wraps)]
fn system_exit(args: &[Value]) -> Result<Value, RuntimeError> {
    let output = args.first().unwrap().clone();
    if let Value::Int(i) = output {
        let i32_code: i32 = if i > i64::from(i32::MAX) {
            return Err(RuntimeError::PrecisionLoss(
                "exit: exit_code > MAX_INT32".to_smolstr(),
            ));
        } else {
            i32::try_from(i).unwrap()
        };
        exit(i32_code);
    } else {
        Err(RuntimeError::TypeException(
            "exit: exit_code not a number.".to_smolstr(),
        ))
    }
}
fn reg_exit() -> ModuleFunc {
    ModuleFunc {
        name: SmolStr::new("exit"),
        arity: 1,
        func: system_exit,
    }
}

#[allow(clippy::unnecessary_wraps)]
fn system_read(_args: &[Value]) -> Result<Value, RuntimeError> {
    let mut buffer = [0u8; 1];

    let read = match io::stdin().read_exact(&mut buffer) {
        Ok(_) => (buffer[0] as char).to_string(),
        Err(_) => {
            String::new() // 可以返回 EOF
        }
    };

    Ok(Value::String(read.to_smolstr()))
}

fn reg_read() -> ModuleFunc {
    ModuleFunc {
        name: SmolStr::new("read"),
        arity: 0,
        func: system_read,
    }
}

pub fn register_system_lib() {
    let mut system_lib = LibModule {
        name: SmolStr::new("system"),
        functions: vec![],
    };
    system_lib.functions.push(reg_println());
    system_lib.functions.push(reg_exit());
    system_lib.functions.push(reg_read());
    register_library(system_lib);
}
