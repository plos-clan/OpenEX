use crate::library::{register_library, LibModule, ModuleFunc, output_capture::print};
use crate::runtime::executor::Value;
use crate::runtime::RuntimeError;
use smol_str::SmolStr;

fn system_print(args:&[Value]) -> Result<Value,RuntimeError> {
    let output = args.first().unwrap().clone();
    match output {
        Value::Int(i) => print(format_args!("{i}")),
        Value::Bool(i) => print(format_args!("{i}")),
        Value::Float(i) => print(format_args!("{i}")),
        Value::String(i) => print(format_args!("{i}")),
        Value::Ref(i) => print(format_args!("<ref:{i}>")),
        Value::Null => print(format_args!("null")),
    }
    Ok(Value::Null)
}
fn reg_println() -> ModuleFunc{
    ModuleFunc {
        name: SmolStr::new("print"),
        arity: 1,
        func: system_print,
    }
}

pub fn register_system_lib() {
    let mut system_lib = LibModule {
        name: SmolStr::new("system"),
        functions: vec![]
    };
    system_lib.functions.push(reg_println());
    register_library(system_lib);
}
