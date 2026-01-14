use smol_str::{SmolStr, ToSmolStr};

use crate::compiler::Compiler;
use crate::compiler::ast::vm_ir::{ByteCode, Value};
use crate::runtime::thread::ThreadManager;

pub mod executor;
pub mod thread;
mod vm_operation;
mod vm_table_opt;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum RuntimeError {
    NoSuchFunctionException(SmolStr), // 找不到函数
    TypeException(SmolStr),           // 类型检查错误
    PrecisionLoss(SmolStr),           // 精度转换损失
    IndexOutOfBounds(SmolStr),        // 索引越界
    VMError,                          // 解释器内部错误
}

pub struct MethodInfo {
    pub name: SmolStr,
    pub r_name: SmolStr,
    pub codes: Vec<ByteCode>,
    pub locals: usize, // 局部变量表
    pub is_native: bool,
    pub args: usize, // 形参个数
}

pub struct MetadataUnit<'a> {
    pub constant_table: &'a [Value],
    pub methods: Vec<MethodInfo>,
    pub names: &'a str,
    pub globals: usize,           // 全局变量表
    pub root_code: Vec<ByteCode>, // 全局代码
    pub library: bool,
}

pub struct GlobalStore {
    globals: Vec<Vec<Value>>,
}

impl GlobalStore {
    pub fn new(units: &[MetadataUnit<'_>]) -> Self {
        let globals = units
            .iter()
            .map(|unit| vec![Value::Null; unit.globals])
            .collect();
        Self { globals }
    }

    pub fn empty() -> Self {
        Self {
            globals: Vec::new(),
        }
    }

    pub fn get(&self, unit_index: usize, index: usize) -> Option<&Value> {
        self.globals
            .get(unit_index)
            .and_then(|unit| unit.get(index))
    }

    pub fn get_mut(&mut self, unit_index: usize, index: usize) -> Option<&mut Value> {
        self.globals
            .get_mut(unit_index)
            .and_then(|unit| unit.get_mut(index))
    }
}

impl MethodInfo {
    pub const fn get_codes(&self) -> &[ByteCode] {
        self.codes.as_slice()
    }
}

pub fn initialize_executor(compiler: &mut Compiler) {
    let mut metadata: Vec<MetadataUnit> = Vec::new();
    for file in compiler.get_files() {
        let vm_ir = file.ir_table.as_ref().unwrap();
        let mut methods: Vec<MethodInfo> = vec![];

        for func in vm_ir.get_functions() {
            methods.push(MethodInfo {
                name: func.name.clone(),
                r_name: func.filename.split('.').next().unwrap().to_smolstr(),
                locals: func.locals,
                codes: func.clone_codes().unwrap_or_default(),
                is_native: func.is_native,
                args: func.args,
            });
        }

        metadata.push(MetadataUnit {
            constant_table: vm_ir.get_constant_table(),
            names: file.name.split('.').next().unwrap(),
            methods,
            globals: vm_ir.get_locals_len(),
            root_code: vm_ir.clone_codes(),
            library: file.is_library,
        });
    }

    let (main_index, main_metadata) = metadata
        .iter()
        .enumerate()
        .find(|(_, file)| !file.library)
        .unwrap();

    let main_method = &MethodInfo {
        name: "<main_root>".to_smolstr(),
        r_name: SmolStr::new(main_metadata.names),
        locals: main_metadata.globals,
        codes: main_metadata.root_code.clone(),
        is_native: false,
        args: 0,
    };

    let mut globals = GlobalStore::new(&metadata);
    std::thread::scope(|scope| {
        let thread_manager = ThreadManager::new(scope);
        thread_manager.submit_join_thread(
            main_index,
            main_metadata,
            main_method,
            metadata.as_slice(),
            &mut globals,
        );
    });
}
