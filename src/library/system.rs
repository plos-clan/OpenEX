use crate::compiler::ast::vm_ir::Value;
use crate::library::{LibModule, ModuleFunc, output_capture::print, register_library};
use crate::runtime::RuntimeError;
use crate::runtime::context;
use smol_str::{SmolStr, ToSmolStr};
use std::io::Read;
use std::process::exit;
use std::time::Duration;
use std::{io, thread};

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

#[allow(clippy::unnecessary_wraps)]
fn system_thread(args: &[Value]) -> Result<Value, RuntimeError> {
    let output = args.first().unwrap().clone();
    let path = match output {
        Value::String(s) | Value::Ref(s) => s,
        _ => {
            return Err(RuntimeError::TypeException(
                "thread: path not a string.".to_smolstr(),
            ));
        }
    };

    let ret = context::with_context(|ctx| {
        let units = context::get_units(ctx);
        let mut sp = path.split('/');
        let file = sp.next().unwrap_or("");
        let func = sp.next().unwrap_or("");
        if file.is_empty() || func.is_empty() {
            return Err(RuntimeError::TypeException(
                "thread: path should be \"file/func\".".to_smolstr(),
            ));
        }
        let mut target = None;
        for (unit_index, unit) in units.iter().enumerate() {
            if unit.names == file {
                for method in &unit.methods {
                    if method.name.as_str() == func {
                        target = Some((unit_index, unit, method));
                        break;
                    }
                }
                break;
            }
        }
        let Some((unit_index, unit, method)) = target else {
            return Err(RuntimeError::NoSuchFunctionException(path));
        };
        let Some(thread_manager) = context::get_thread_manager(ctx) else {
            return Err(RuntimeError::VMError);
        };
        let globals = context::get_globals(ctx);
        let sync_table = context::get_sync_table(ctx);
        thread_manager.submit_run_thread(unit_index, unit, method, units, globals, sync_table);
        Ok(Value::Null)
    });

    ret.unwrap_or(Err(RuntimeError::VMError))
}
fn reg_thread() -> ModuleFunc {
    ModuleFunc {
        name: SmolStr::new("thread"),
        arity: 1,
        func: system_thread,
    }
}

#[allow(clippy::unnecessary_wraps)]
fn system_thread_exit(args: &[Value]) -> Result<Value, RuntimeError> {
    let _ = args;
    context::request_thread_exit();
    Ok(Value::Null)
}
fn reg_thread_exit() -> ModuleFunc {
    ModuleFunc {
        name: SmolStr::new("thread_exit"),
        arity: 0,
        func: system_thread_exit,
    }
}

#[allow(clippy::unnecessary_wraps)]
fn system_nano_sleep(args: &[Value]) -> Result<Value, RuntimeError> {
    let arg_time = args.first().unwrap().clone();
    let Value::Int(raw_time) = arg_time else {
        return Err(RuntimeError::TypeException(
            "nano_sleep: time not a number.".to_smolstr(),
        ));
    };
    let time = if raw_time >= 0 {
        raw_time as u64
    } else {
        return Err(RuntimeError::TypeException(
            "nano_sleep: time must > 0.".to_smolstr(),
        ));
    };
    thread::sleep(Duration::from_nanos(time));
    Ok(Value::Null)
}

fn reg_nano_sleep() -> ModuleFunc {
    ModuleFunc {
        name: SmolStr::new("nano_sleep"),
        arity: 1,
        func: system_nano_sleep,
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
    system_lib.functions.push(reg_thread());
    system_lib.functions.push(reg_thread_exit());
    system_lib.functions.push(reg_nano_sleep());
    register_library(system_lib);
}
