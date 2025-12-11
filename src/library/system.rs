use crate::library::{register_library, LibModule, ModuleFunc};
use crate::runtime::{RuntimeError, Types};
use smol_str::SmolStr;

fn system_print(args:Vec<Types>) -> Result<Types,RuntimeError> {
    Ok(Types::Null)
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
