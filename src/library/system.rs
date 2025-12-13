use crate::library::{register_library, LibModule, ModuleFunc};
use crate::runtime::executor::Value;
use crate::runtime::RuntimeError;
use smol_str::SmolStr;

fn system_print(args:Vec<Value>) -> Result<Value,RuntimeError> {
    let output = args.first().unwrap().clone();
    let out_str = match output {
        Value::Int(i) => format!("{}", i),
        Value::Bool(i) => format!("{}", i),
        Value::Float(i) => format!("{}", i),
        Value::String(i) => i,
        Value::Ref(i) => format!("<ref:{}>", i),
        Value::Null => String::from("null"),
    };
    print!("{}",out_str);
    Ok(Value::Null)
}
fn reg_println() -> ModuleFunc{
    ModuleFunc {
        name: SmolStr::new("print"),
        arity: 1,
        func: system_print,
    }
}

pub(crate) fn register_system_lib() {
    let mut system_lib = LibModule {
        name: SmolStr::new("system"),
        functions: vec![]
    };
    system_lib.functions.push(reg_println());
    register_library(system_lib);
}
