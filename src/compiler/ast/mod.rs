pub mod ssa_ir;
pub(crate) mod vm_ir;

use crate::compiler::lexer::Token;

#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord)]
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

#[derive(Debug,Clone,PartialEq)]
#[allow(dead_code)] //TODO
pub enum ASTExprTree {
    Literal(Token), // Number | String | Bool
    Var(Token),     // x
    Ref(Token),
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
#[allow(dead_code)] //TODO
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
        token: Token,
        cond: ASTExprTree,
        body: Vec<ASTStmtTree>,
    },
    Function {
        // function identifier() {}
        name: Token,
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
    Break(Token),
    Continue(Token),
    Empty, // 空语句需要剔除
}

impl ASTExprTree {
    pub fn token(&self) -> &Token {
        match self {
            ASTExprTree::Literal(token)
            | ASTExprTree::Var(token)
            | ASTExprTree::Ref(token)
            | ASTExprTree::This(token) => token,

            ASTExprTree::Expr { token, .. } => token,
            ASTExprTree::Unary { token, .. } => token,
            ASTExprTree::Call { name, .. } => name.token(),
        }
    }
}
