pub mod ssa_ir;
pub mod vm_ir;

use crate::compiler::lexer::Token;
use smol_str::SmolStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExprOp {
    Pos,    // 取正 +
    Neg,    // 取负 -
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    And,    // &&
    Or,     // ||
    Rmd,    // %
    Equ,    // ==
    SAdd,   // ++
    SSub,   // --
    Not,    // !
    NotEqu, // !=
    BigEqu, // >=
    LesEqu, // <=
    Big,    // >
    Less,   // <
    Store,  // =
    AddS,   // +=
    SubS,   // -=
    MulS,   // *=
    DivS,   // /=
    RmdS,   // %=
    BitAnd, // &
    BitOr,  // |
    BitXor, // ^
    BAndS,  // &=
    BOrS,   // |=
    BXorS,  // ^=
    BLeft,  // <<
    BRight, // >>
    Ref,    // .
    AIndex, // 数组索引
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTExprTree {
    Literal(Token), // Number | String | Bool
    Var(Token),     // x
    This(Token),    // script current context
    Expr {
        token: Token,
        op: ExprOp,
        left: Box<ASTExprTree>,
        right: Box<ASTExprTree>,
    },
    Unary {
        token: Token,
        op: ExprOp,
        code: Box<ASTExprTree>,
    },
    Call {
        // foo(1, 2)
        name: Box<ASTExprTree>, // 必须为 Var
        args: Vec<ASTExprTree>,
    },
}

#[derive(Debug)]
pub enum ASTStmtTree {
    Root(Vec<ASTStmtTree>),
    Block(Vec<ASTStmtTree>),
    Var {
        // var x = 5;
        name: Token,
        value: Option<ASTExprTree>,
    },
    Expr(ASTExprTree),               // 表达式语句：a + b;
    Return(Option<ASTExprTree>),     // return x;
    Import(Token, SmolStr, SmolStr), // import "library"; (use_name, import_name)
    Context(Vec<ASTStmtTree>),       // 独立上下文
    Loop {
        token: Token,
        cond: ASTExprTree,
        body: Vec<ASTStmtTree>,
        is_easy: bool,
    },
    Function {
        // function identifier() {}
        name: Token,
        sync: bool,
        args: Vec<ASTExprTree>,
        body: Vec<ASTStmtTree>,
    },
    NativeFunction {
        name: Token,
        args: Vec<ASTExprTree>,
    },
    If {
        // if (cond) { then } elif (cond) { elif_ } else { else_ }
        cond: ASTExprTree,
        then_body: Vec<ASTStmtTree>,
        else_body: Vec<ASTStmtTree>,
    },
    Array {
        token: Token,
        elements: Vec<ASTExprTree>,
    },
    ArrayFill {
        token: Token,
        value: ASTExprTree,
        count: ASTExprTree,
    },
    Break(Token),
    Continue(Token),
    Empty, // 空语句需要剔除
}

impl ASTExprTree {
    pub fn token(&self) -> &Token {
        match self {
            Self::Call { name, .. } => name.token(),
            Self::Literal(token)
            | Self::Var(token)
            | Self::This(token)
            | Self::Expr { token, .. }
            | Self::Unary { token, .. } => token,
        }
    }
}
