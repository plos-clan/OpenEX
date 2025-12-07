pub mod eof_status;
pub mod ssa_ir;

use crate::compiler::lexer::Token;
use smol_str::SmolStr;

#[derive(Debug)]
pub enum ExprOp {
    Add,           // +
    Sub,           // -
    Mul,           // *
    Div,           // /
    And,           // &&
    Or,            // ||
    Rmd,           // %
    Equ,           // ==
    SAdd,          // ++
    SSub,          // --
    Not,           // !
    NotEqu,        // !=
    BigEqu,        // >=
    LesEqu,        // <=
    Big,           // >
    Less,          // <
    Store,         // =
    AddS,          // +=
    SubS,          // -=
    MulS,          // *=
    DivS,          // /=
    RmdS,          // %=
    BitAnd,        // &
    BitOr,         // |
    BitXor,        // ^
    BAndS,         // &=
    BOrS,          // |=
    BXorS,         // ^=
    BLeft,         // <<
    BRight,        // >>
    Ref,           // .
    AIndex,        // 数组索引
}

#[derive(Debug)]
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
        name: Token,
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
    Expr(ASTExprTree),           // 表达式语句：a + b;
    Return(Option<ASTExprTree>), // return x;
    Import(Token),               // import "library";
    Context(Vec<ASTStmtTree>),   // 独立上下文
    Loop {
        cond: ASTExprTree,
        body: Vec<ASTStmtTree>,
    },
    Function {
        // function identifier() {}
        name: Token,
        args: Vec<ASTExprTree>,
        body: Vec<ASTStmtTree>,
    },
    If {
        // if (cond) { then } elif (cond) { elif_ } else { else_ }
        cond: ASTExprTree,
        then_body: Vec<ASTStmtTree>,
        else_body: Vec<ASTStmtTree>,
    },
    Break(Token),
    Continue(Token),
    Empty, // 空语句需要剔除
}
