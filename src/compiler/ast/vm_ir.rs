use crate::compiler::ast::ssa_ir::{Code, LocalMap, OpCode, OpCodeTable, Operand};
use crate::compiler::ast::vm_ir::Types::{Bool, Float, Null, Number, Ref, String};
use smol_str::{SmolStr, ToSmolStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] //TODO
pub enum ByteCode {
    Push(usize),        // 将常量表中的元素压入操作栈 (常量表索引)
    Load(usize),        // 栈顶元素加载到局部变量表 (变量表索引)
    Store(usize),       // 将局部变量加载到栈顶 (变量表索引)
    LoadGlobal(usize),  // 栈顶元素加载到全局变量表 (变量表索引)
    StoreGlobal(usize), // 将全局变量加载到栈顶 (变量表索引)
    Jump(usize),        // 无条件跳转 (pc位置)
    JumpTrue(usize),    // 栈顶条件跳转 (pc位置)
    JumpFalse(usize),   // 栈顶反转条件跳转 (pc位置)
    Call,               // 函数调用 (要求栈上最少有两个引用)
    Nol,                // 空操作
    GetRef,             // 拼接引用路径
    Return,             // 退出当前栈帧 (并将栈顶元素压入父栈帧操作栈)
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantTable {
    table_size: usize,
    element: Vec<(Types, SmolStr)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFunction {
    pub codes: Vec<ByteCode>,
    pub name: SmolStr,
    pub filename: SmolStr,
    pub args: usize,
    pub locals: usize,   // 局部变量表大小
    pub is_native: bool, // 是否是本地函数
}

impl IrFunction {
    pub fn clone_codes(&self) -> Vec<ByteCode> {
        self.codes.clone()
    }
}

fn opcode_to_vmir(code: OpCode) -> ByteCode {
    match code {
        OpCode::Add(_) => ByteCode::Add,
        OpCode::Sub(_) => ByteCode::Sub,
        OpCode::Mul(_) => ByteCode::Mul,
        OpCode::Div(_) => ByteCode::Div,
        OpCode::And(_) => ByteCode::And,
        OpCode::Or(_) => ByteCode::Or,
        OpCode::Not(_) => ByteCode::Not,
        OpCode::LesEqu(_) => ByteCode::LesEqu,
        OpCode::Less(_) => ByteCode::Less,
        OpCode::BigEqu(_) => ByteCode::BigEqu,
        OpCode::Big(_) => ByteCode::Big,
        OpCode::SAdd(_) => ByteCode::SAdd,
        OpCode::SSub(_) => ByteCode::SSub,
        OpCode::NotEqu(_) => ByteCode::NotEqu,
        OpCode::AddS(_) => ByteCode::AddS,
        OpCode::SubS(_) => ByteCode::SubS,
        OpCode::MulS(_) => ByteCode::MulS,
        OpCode::DivS(_) => ByteCode::DivS,
        OpCode::BitAnd(_) => ByteCode::BitAnd,
        OpCode::BitOr(_) => ByteCode::BitOr,
        OpCode::BitXor(_) => ByteCode::BitXor,
        OpCode::BAndS(_) => ByteCode::BAndS,
        OpCode::BOrS(_) => ByteCode::BOrS,
        OpCode::BLeft(_) => ByteCode::BLeft,
        OpCode::BRight(_) => ByteCode::BRight,
        OpCode::Equ(_) => ByteCode::Equ,
        OpCode::Call(_, _imm) => ByteCode::Call,
        OpCode::Ref(_) => ByteCode::GetRef,
        OpCode::Nop(_) => ByteCode::Nol,
        OpCode::Return(_) => ByteCode::Return,
        c => {
            dbg!(c);
            todo!()
        }
    }
}

impl IrFunction {
    pub const fn new(
        name: SmolStr,
        args: usize,
        locals: usize,
        filename: SmolStr,
        is_native: bool,
    ) -> Self {
        Self {
            codes: vec![],
            name,
            args,
            locals,
            filename,
            is_native,
        }
    }

    pub fn append_code(
        &mut self,
        table: OpCodeTable,
        code0: &mut Code,
        locals: &LocalMap,
        globals: &LocalMap,
        constant_table: &mut ConstantTable,
    ) {
        for code in table.opcodes {
            match code.1 {
                OpCode::Push(_, imm) => {
                    if let Operand::Val(key) = imm {
                        let index = locals.get_index(key).unwrap();
                        self.codes.push(ByteCode::StoreGlobal(*index));
                    } else {
                        let index = constant_table.add_operand(imm, code0);
                        self.codes.push(ByteCode::Push(index));
                    }
                }
                OpCode::LoadLocal(_, key, _) => {
                    let index = locals.get_index(key).unwrap();
                    self.codes.push(ByteCode::Load(*index));
                }
                OpCode::StoreLocal(_, key, _) => {
                    let index = locals.get_index(key).unwrap();
                    self.codes.push(ByteCode::Store(*index));
                }
                OpCode::LoadGlobal(_, key, _) => {
                    let index = globals.get_index(key).unwrap();
                    self.codes.push(ByteCode::LoadGlobal(*index));
                }
                OpCode::StoreGlobal(_, key, _) => {
                    let index = globals.get_index(key).unwrap();
                    self.codes.push(ByteCode::StoreGlobal(*index));
                }
                OpCode::Jump(_, addr) => {
                    self.codes.push(ByteCode::Jump(addr.unwrap().offset));
                }
                OpCode::JumpTrue(_, addr, _) => {
                    let addr_some = addr.unwrap();
                    self.codes.push(ByteCode::JumpTrue(addr_some.offset));
                }
                OpCode::JumpFalse(_, addr, _) => {
                    let addr_some = addr.unwrap();
                    self.codes.push(ByteCode::JumpFalse(addr_some.offset));
                }
                c => {
                    self.codes.push(opcode_to_vmir(c));
                }
            }
        }
    }
}

#[allow(dead_code)] // TODO
#[derive(Debug, Clone)]
pub struct VMIRTable {
    constant_table: ConstantTable,
    functions: Vec<IrFunction>,
    codes: Vec<ByteCode>,
    globals: usize, // 全局变量表大小
}

impl VMIRTable {
    pub const fn new() -> Self {
        Self {
            constant_table: ConstantTable::new(),
            functions: vec![],
            codes: vec![],
            globals: 0,
        }
    }

    pub fn get_functions(&self) -> Vec<IrFunction> {
        self.functions.clone()
    }

    pub fn get_constant_table(&self) -> ConstantTable {
        self.constant_table.clone()
    }

    pub const fn get_locals_len(&self) -> usize {
        self.globals
    }

    pub fn clone_codes(&self) -> Vec<ByteCode> {
        self.codes.clone()
    }

    pub fn append_code(
        &mut self,
        table: &OpCodeTable,
        code0: &mut Code,
        locals: &LocalMap,
    ) {
        let opcodes = table.opcodes.clone();
        self.globals = locals.now_index;
        for code in opcodes {
            match code.1 {
                OpCode::Push(_, imm) => {
                    if let Operand::Val(key) = imm {
                        if let Some(index) = locals.get_index(key) {
                            self.codes.push(ByteCode::StoreGlobal(*index));
                        } else {
                            unreachable!()
                        }
                    } else {
                        let index = self.constant_table.add_operand(imm, code0);
                        self.codes.push(ByteCode::Push(index));
                    }
                }
                OpCode::LoadLocal(_, key, _) => {
                    let index = locals.get_index(key).unwrap();
                    self.codes.push(ByteCode::LoadGlobal(*index));
                }
                OpCode::StoreLocal(_, key, _) => {
                    let index = locals.get_index(key).unwrap();
                    self.codes.push(ByteCode::StoreGlobal(*index));
                }
                OpCode::Jump(_, addr) => {
                    self.codes.push(ByteCode::Jump(addr.unwrap().offset));
                }
                OpCode::JumpTrue(_, addr, _) => {
                    let addr_some = addr.unwrap();
                    self.codes.push(ByteCode::JumpTrue(addr_some.offset));
                }
                OpCode::JumpFalse(_, addr, _) => {
                    let addr_some = addr.unwrap();
                    self.codes.push(ByteCode::JumpFalse(addr_some.offset));
                }
                c => {
                    self.codes.push(opcode_to_vmir(c));
                }
            }
        }
    }
}

impl ConstantTable {
    pub const fn new() -> Self {
        Self {
            table_size: 0,
            element: Vec::new(),
        }
    }

    pub fn get_const(&self, index: usize) -> Option<(Types, SmolStr)> {
        self.element.get(index).cloned()
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
            Operand::Null => (Null, SmolStr::new_static("null")),
            Operand::This => (Ref, SmolStr::new_static("this")),
            Operand::ImmStr(imm) => (String, imm),
            Operand::ImmNum(imm) => (Number, imm.to_smolstr()),
            Operand::ImmFlot(imm) => (Float, imm.to_smolstr()),
            Operand::ImmBool(imm) => (Bool, imm.to_smolstr()),
            _ => unreachable!(),
        };
        self.add_const(types.0, types.1)
    }
}

pub fn ssa_to_vm(mut code: Code, locals: &LocalMap, filename: &SmolStr) -> VMIRTable {
    let mut vm_table = VMIRTable::new();
    vm_table.append_code(code.clone().get_code_table(), &mut code, locals);

    for func in code.clone().funcs {
        let mut ir_func = IrFunction::new(
            func.name,
            func.args,
            func.locals.now_index,
            filename.clone(),
            func.codes.is_none(),
        );
        if func.codes.is_some() {
            ir_func.append_code(
                func.codes.unwrap(),
                &mut code,
                &func.locals,
                locals,
                &mut vm_table.constant_table,
            );
        }
        vm_table.functions.push(ir_func);
    }
    vm_table
}
