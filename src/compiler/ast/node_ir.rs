use crate::compiler::ast::ssa_ir::Operand;
use crate::compiler::lexer::Token;

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] // TODO
pub struct NodeFunction {
    pub name: Token,
    pub args: usize, // 形参个数
    pub codes: Vec<NodeIr>,
    pub is_native: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] // TODO
pub enum OptIR {
    Push(Operand),

    Pos,
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Rmd,

    Store,
    AddStore,
    SubStore,
    MulStore,
    DivStore,
    RmdStore,

    BitAnd,
    BitOr,
    BitXor,
    BitLeft,
    BitRight,

    Equ,
    NotEqu,
    LessEqu,
    BigEqu,
    Less,
    Big,

    And,
    Or,
    Not,

    SuperAdd,
    SuperSub,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] // TODO
pub enum NodeIr {
    LoadVar {
        name: Token,
        vars: Vec<OptIR>,
    },
    Return(Vec<OptIR>),
    LoopBack(bool), // 是否是 continue
    Loop {
        cond: Vec<OptIR>,
        body: Vec<NodeIr>,
    },
    Expr(Vec<OptIR>),
    Condition {
        cond: Vec<OptIR>,
        body: Vec<NodeIr>,
        else_body: Vec<NodeIr>,
    },
}
