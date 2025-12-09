
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
