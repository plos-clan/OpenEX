use crate::compiler::Compiler;
use crate::compiler::ast::vm_ir::Value;
use crate::compiler::file::SourceFile;
use crate::compiler::lints::Lint;
use crate::compiler::parser::ParserError;
use crate::library::system::register_system_lib;
use crate::library::type_lib::register_type_lib;
use crate::runtime::RuntimeError;
use smol_str::SmolStr;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::sync::{LazyLock, RwLock};

pub mod output_capture;
mod system;
mod type_lib;

static MODULES: LazyLock<RwLock<BTreeMap<SmolStr, LibModule>>> =
    LazyLock::new(|| RwLock::new(BTreeMap::new()));

pub type NativeFunc = fn(&[Value]) -> Result<Value, RuntimeError>;

#[derive(Debug, Clone, Hash)]
pub struct ModuleFunc {
    pub name: SmolStr,
    pub arity: usize,
    pub func: NativeFunc,
}

#[derive(Debug, Clone)]
pub struct LibModule {
    name: SmolStr,
    functions: Vec<ModuleFunc>,
}

impl LibModule {
    pub fn find_func(&mut self, func_name: &SmolStr) -> Option<&mut ModuleFunc> {
        self.functions
            .iter_mut()
            .find(|entry| entry.name.as_str() == func_name.as_str())
    }
}

fn register_library(library: LibModule) {
    if library.functions.is_empty() {
        println!("warn: {} no functions found", library.name);
    }
    MODULES
        .write()
        .unwrap()
        .insert(library.name.clone(), library);
}

pub fn find_library(
    name: &str,
    f: impl FnOnce(Option<&mut LibModule>) -> Result<Value, ParserError>,
) -> Result<Value, ParserError> {
    let mut map = MODULES.write().unwrap();
    let ret_m = map.get_mut(name);
    f(ret_m)
}

pub fn load_libraries(
    compiler: &mut Compiler,
    path: Option<SmolStr>,
    lints: &HashSet<Lint>,
) -> std::io::Result<()> {
    let lib_path = path.map_or_else(|| SmolStr::new("./lib"), |path| path);
    for entry in fs::read_dir(lib_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let mut buf = Vec::new();
            File::open(&path)?.read_to_end(&mut buf)?;

            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("<invalid>")
                .to_string();
            let data = SmolStr::new(std::str::from_utf8(&buf).expect("error: file not UTF-8"));
            compiler.add_file(SourceFile::new(name, data.to_string(), lints.clone(), true));
        }
    }

    register_system_lib();
    register_type_lib();

    compiler.compile().expect("error: library has error.");
    Ok(())
}
