use crate::compiler::lexer::Token;
use slotmap::{DefaultKey, SlotMap};
use smol_str::SmolStr;

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] //TODO
pub enum Operand {
    Val(DefaultKey),
    Null,
    ImmBool(bool),
    ImmNum(i64),
    ImmFlot(f64),
    ImmStr(SmolStr),
    Expression(Box<Operand>, Box<Operand>),
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
#[allow(dead_code)] //TODO
pub enum OpCode {
    StackLocal(DefaultKey, Operand), // 栈局部变量加载
    Push(Operand),                   // 将值压入操作栈
    Call(SmolStr),                   // 函数调用 (调用路径)
    Return,                          // 栈顶结果返回
    Add,                             // 从栈顶提取两个操作数相加并将结果压回操作栈
    Sub,                             // -
    Mul,                             // *
    Div,                             // /
    And,                             // &&
    Or,                              // ||
    Rmd,                             // %
    Equ,                             // ==
    SAdd,                            // ++
    SSub,                            // --
    Not,                             // !
    NotEqu,                          // !=
    BigEqu,                          // >=
    LesEqu,                          // <=
    Big,                             // >
    Less,                            // <
    Store,                           // =
    AddS,                            // +=
    SubS,                            // -=
    MulS,                            // *=
    DivS,                            // /=
    RmdS,                            // %=
    BitAnd,                          // &
    BitOr,                           // |
    BitXor,                          // ^
    BAndS,                           // &=
    BOrS,                            // |=
    BXorS,                           // ^=
    BLeft,                           // <<
    BRight,                          // >>
    Ref,                             // .
    AIndex,                          // 数组索引
}

#[derive(Clone, Debug)]
#[allow(dead_code)] //TODO
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

    pub fn append_code(&mut self, code: &mut Vec<OpCode>) {
        self.codes.append(code);
    }

    pub fn alloc_value(&mut self, token: Token, type_: ValueGuessType) -> DefaultKey {
        let va = Value {
            variable: false,
            token,
            type_,
        };
        self.values.insert(va)
    }
}
