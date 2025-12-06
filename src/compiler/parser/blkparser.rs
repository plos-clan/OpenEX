use crate::compiler::ast::ASTStmtTree;
use crate::compiler::ast::ASTStmtTree::Context;
use crate::compiler::lexer::TokenType::{LP, LR};
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::exprparser::expr_eval;
use crate::compiler::parser::ifparser::if_eval;
use crate::compiler::parser::varparser::var_eval;
use crate::compiler::parser::whileparser::while_eval;
use crate::compiler::parser::{Parser, ParserError};
use crate::compiler::parser::retparser::return_eval;

fn parser_expr(parser: &mut Parser) -> Result<ASTStmtTree, ParserError> {
    let mut token1 = parser.next_parser_token()?;
    let mut tokens: Vec<Token> = vec![token1.clone()];
    loop {
        token1 = parser.next_parser_token()?;
        if token1.t_type == TokenType::END {
            break;
        }
        tokens.push(token1);
    }
    Ok(ASTStmtTree::Expr(expr_eval(parser, tokens)?))
}

pub fn blk_eval(parser: &mut Parser) -> Result<Vec<ASTStmtTree>, ParserError> {
    let mut token = parser.next_parser_token()?;
    parser.check_char(&mut token, LP, '{')?;

    let mut stmt: Vec<ASTStmtTree> = vec![];
    loop {
        token = parser.next_parser_token()?;
        match token.t_type {
            TokenType::Function => return Err(ParserError::NotAStatement(token)),
            TokenType::If => {
                parser.last = Some(token);
                stmt.push(if_eval(parser)?)
            }
            TokenType::Var => {
                parser.last = Some(token);
                stmt.push(var_eval(parser)?)
            }
            TokenType::While => {
                parser.last = Some(token);
                stmt.push(while_eval(parser)?)
            }
            TokenType::END => {
            }
            TokenType::Return => {
                parser.last = Some(token);
                stmt.push(return_eval(parser)?)
            }
            TokenType::Break => {
                stmt.push(ASTStmtTree::Break(token));
                token = parser.next_parser_token()?;
                if token.t_type != TokenType::END {
                    return Err(ParserError::Expected(token,';'));
                }
            }
            TokenType::Continue => {
                stmt.push(ASTStmtTree::Continue(token));
                token = parser.next_parser_token()?;
                if token.t_type != TokenType::END {
                    return Err(ParserError::Expected(token,';'));
                }
            }
            LR => {
                let mut t = token.clone();
                parser.cache = Some(token);
                if t.value::<String>().unwrap() == "}" {
                    break;
                }
            }
            LP => {
                let mut t = token.clone();
                parser.cache = Some(token);
                if t.value::<String>().unwrap() == "{" {
                    stmt.push(Context(blk_eval(parser)?))
                } else {
                    stmt.push(parser_expr(parser)?)
                }
            }
            _ => stmt.push(parser_expr(parser)?),
        }
    }
    token = parser.next_parser_token()?;
    parser.check_char(&mut token, LR, '}')?;
    Ok(stmt)
}
