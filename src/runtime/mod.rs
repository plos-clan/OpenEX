use smol_str::SmolStr;
use crate::library::ModuleFunc;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] //TODO
pub enum CodeImm {
    Float(f64),
    Int(i64),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] //TODO
pub enum ByteCode {
    Push(CodeImm),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[allow(dead_code)] //TODO
pub enum Types {
    String(SmolStr),
    Number(i64),
    Bool(bool),
    Null,
}

pub enum RuntimeError {
}
