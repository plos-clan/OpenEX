use crate::compiler::ast::ssa_ir::{Code, OpCode, OpCodeTable, Operand};
use crate::compiler::ast::vm_ir::Types::{Null, Ref};
use smol_str::{SmolStr, ToSmolStr};

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] //TODO
pub enum ByteCode {
    Push(usize),
    Nol,
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


#[allow(dead_code)] // TODO
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
    pub fn append_code(&mut self, table: OpCodeTable, code0: &mut Code) {
        for code in table.opcodes {
            match code.1 {
                OpCode::Push(_, imm) => {
                    let index = self.constant_table.add_operand(imm,code0);
                    self.codes.push(ByteCode::Push(index));
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
            _ => unreachable!()
        };
        self.add_const(types.0,types.1)
    }


    #[allow(dead_code)] // TODO
    pub fn find_const(&mut self, index: usize) -> Option<&mut (Types, SmolStr)> {
        self.element.get_mut(index)
    }
}
