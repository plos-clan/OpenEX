use crate::compiler::lexer::Token;
use slotmap::{DefaultKey, SlotMap};
use smol_str::SmolStr;

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Val(DefaultKey),
    Null,
    ImmBool(bool),
    ImmNum(i64),
    ImmFlot(f64),
    ImmStr(SmolStr),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueGuessType {
    Bool,
    Number,
    String,
    Float,
    Null,
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Value {
    variable: bool,        // 是否被重赋值
    type_: ValueGuessType, // 猜测类型
    token: Token,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OpCode {
    StackLocal(DefaultKey,Operand),            // 栈局部变量加载
    Add(Operand, Operand, Operand), // 加法 (输出,被加数,加数)
}

#[derive(Clone, Debug)]
pub struct Code {
    codes: Vec<OpCode>,
    values: SlotMap<DefaultKey, Value>,
    stack_size: usize,
    root: bool, // 是否是根脚本上下文 (true: 根上下文|false: 函数上下文)
}

impl Code {
    pub fn new(root: bool) -> Code {
        Self {
            codes: vec![],
            values: SlotMap::new(),
            stack_size: 0,
            root,
        }
    }
    
    pub fn add_opcode(&mut self, opcode: OpCode) {
        self.codes.push(opcode);
    }

    pub fn alloc_value(
        &mut self,
        value: Operand,
        token: Token,
        type_: ValueGuessType,
    ) -> DefaultKey {
        let va = Value {
            variable: false,
            token,
            type_,
        };
        self.values.insert(va)
    }
}
