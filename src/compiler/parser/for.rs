use smol_str::SmolStr;
use crate::compiler::ast::{ASTExprTree, ASTStmtTree};
use crate::compiler::ast::ASTExprTree::Literal;
use crate::compiler::ast::ASTStmtTree::Loop;
use crate::compiler::lexer::TokenType::{End, Var, LP};
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::expression::expr_eval;
use crate::compiler::parser::var::var_eval;
use crate::compiler::parser::{check_char, Parser, ParserError};
use crate::compiler::parser::block::blk_eval;

pub fn for_eval(parser: &mut Parser) -> Result<ASTStmtTree, ParserError> {
    let mut token = parser.next_parser_token()?;
    let head = token.clone();
    let mut ctxt_stmt: Vec<ASTStmtTree> = Vec::new();
    let f_cond: ASTExprTree;
    let f_expr: Option<ASTExprTree>;
    let mut is_easy = true;
    check_char(&token, LP, '(')?;

    token = parser.next_parser_token()?;
    if token.t_type == Var {
        is_easy = false;
        ctxt_stmt.push(var_eval(parser)?);
    } else if token.t_type != End {
        return Err(ParserError::IllegalArgument(token));
    }

    let mut cond: Vec<Token> = Vec::new();
    loop {
        token = parser.next_parser_token()?;
        if token.t_type == End {
            break;
        }
        cond.push(token);
    }
    f_cond = if let Some(cond) = expr_eval(parser, cond)? {
        is_easy = false;
        cond
    } else {
        Literal {
            0: Token::new(
                SmolStr::new("true"),
                head.line,
                head.column,
                head.index,
                TokenType::True,
            )
        }
    };

    f_expr = match parser.parser_cond(Some(token)) {
        Ok(expr) => Some(expr),
        Err(err) => match err {
            ParserError::MissingCondition(_token) => { None }
            _ => return Err(err),
        }
    };

    let result = parser.next_parser_token();
    if matches!(result, Err(ParserError::Eof)) {
        return Err(ParserError::MissingLoopBody(parser.get_last().unwrap()));
    }
    token = result?;
    parser.cache = Some(token);
    let mut body = blk_eval(parser)?;

    if let Some(expr) = f_expr {
        is_easy = false;
        body.push(ASTStmtTree::Expr(expr));
    }

    ctxt_stmt.push(Loop {
        token: head,
        cond: f_cond,
        body,
        is_easy,
    });

    Ok(ASTStmtTree::Context {
        0: ctxt_stmt,
    })
}
