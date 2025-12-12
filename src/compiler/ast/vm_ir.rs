use crate::compiler::ast::ssa_ir::{Code, LocalMap, OpCode, OpCodeTable, Operand};
use crate::compiler::ast::vm_ir::Types::{Null, Ref, String};
use smol_str::{SmolStr, ToSmolStr};

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] //TODO
pub enum ByteCode {
    Push(usize),        // 将常量表中的元素压入操作栈 (常量表索引)
    Load(usize),        // 栈顶元素加载到局部变量表 (变量表索引)
    Store(usize),       // 将局部变量加载到栈顶 (变量表索引)
    LoadGlobal(usize),  // 栈顶元素加载到全局变量表 (变量表索引)
    StoreGlobal(usize), // 将全局变量加载到栈顶 (变量表索引)
    Nol,                // 空操作
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Rmd,
    Equ,
    NotEqu,
    BigEqu,
    LesEqu,
    Big,
    Less,
    SAdd,
    SSub,
    Not,
    AddS,
    SubS,
    MulS,
    DivS,
    RmdS,
    BitAnd,
    BitOr,
    BitXor,
    BAndS,
    BOrS,
    BXorS,
    BLeft,
    BRight,
    Call(usize),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[allow(dead_code)] //TODO
pub enum Types {
    String,
    Number,
    Float,
    Bool,
    Ref,
    Null,
}

// 常量表, 每一个源文件都有一个
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ConstantTable {
    table_size: usize,
    element: Vec<(Types, SmolStr)>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IrFunction {
    codes: Vec<ByteCode>,
    name: SmolStr,
    args: usize,
}

impl IrFunction {
    pub fn new(name: SmolStr, args: usize) -> IrFunction {
        Self {
            codes: vec![],
            name,
            args,
        }
    }

    #[allow(dead_code)] // TODO
    pub fn append_code(
        &mut self,
        table: OpCodeTable,
        code0: &mut Code,
        locals: &mut LocalMap,
        constant_table: &mut ConstantTable,
    ) {
        for code in table.opcodes {
            match code.1 {
                OpCode::Push(_, imm) => {
                    if let Operand::Val(key) = imm {
                        if let Some(index) = locals.get_index(&key) {
                            self.codes.push(ByteCode::StoreGlobal(*index))
                        } else {
                            unreachable!()
                        }
                    } else {
                        let index = constant_table.add_operand(imm, code0);
                        self.codes.push(ByteCode::Push(index));
                    }
                }
                OpCode::Add(_) => self.codes.push(ByteCode::Add),
                OpCode::Call(_, _imm) => {}
                _ => {
                }
            }
        }
    }
}

#[allow(dead_code)] // TODO
#[derive(Debug, Clone)]
pub(crate) struct VMIRTable {
    constant_table: ConstantTable,
    functions: Vec<IrFunction>,
    codes: Vec<ByteCode>,
}

impl VMIRTable {
    pub fn new() -> VMIRTable {
        Self {
            constant_table: ConstantTable::new(),
            functions: vec![],
            codes: vec![],
        }
    }

    #[allow(dead_code)] // TODO
    pub fn append_code(
        &mut self,
        table: &mut OpCodeTable,
        code0: &mut Code,
        locals: &mut LocalMap,
    ) {
        let opcodes = table.opcodes.clone();
        for code in opcodes {
            match code.1 {
                OpCode::Push(_, imm) => {
                    if let Operand::Val(key) = imm {
                        if let Some(index) = locals.get_index(&key) {
                            self.codes.push(ByteCode::StoreGlobal(*index))
                        } else {
                            unreachable!()
                        }
                    } else {
                        let index = self.constant_table.add_operand(imm, code0);
                        self.codes.push(ByteCode::Push(index));
                    }
                }
                OpCode::Call(_, _imm) => {}
                _ => {}
            }
        }
    }
}

impl ConstantTable {
    pub fn new() -> ConstantTable {
        Self {
            table_size: 0,
            element: Vec::new(),
        }
    }

    pub fn add_const(&mut self, types: Types, data: SmolStr) -> usize {
        self.element.push((types, data));
        let rets = self.table_size;
        self.table_size += 1;
        rets
    }

    pub fn add_operand(&mut self, operand: Operand, _code: &mut Code) -> usize {
        let types: (Types, SmolStr) = match operand {
            Operand::Call(path) | Operand::Reference(path) | Operand::Library(path) => (Ref, path),
            Operand::Null => (Null, "null".to_smolstr()),
            Operand::This => (Ref, "/this".to_smolstr()),
            Operand::ImmStr(imm) => (String, imm),
            _ => unreachable!(),
        };
        self.add_const(types.0, types.1)
    }

    #[allow(dead_code)] // TODO
    pub fn find_const(&mut self, index: usize) -> Option<&mut (Types, SmolStr)> {
        self.element.get_mut(index)
    }
}

pub fn ssa_to_vm(mut code: Code, mut locals: LocalMap) -> VMIRTable {
    let mut vm_table = VMIRTable::new();
    vm_table.append_code(code.clone().get_code_table(), &mut code, &mut locals);

    for func in code.clone().funcs {
        if func.codes.is_none() {
            continue;
        }
        let mut ir_func = IrFunction::new(func.name, func.args);
        ir_func.append_code(
            func.codes.unwrap(),
            &mut code,
            &mut locals,
            &mut vm_table.constant_table,
        );
        vm_table.functions.push(ir_func);
    }
    vm_table
}
