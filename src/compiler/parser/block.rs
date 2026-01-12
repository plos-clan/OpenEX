use crate::compiler::ast::ASTStmtTree;
use crate::compiler::ast::ASTStmtTree::Context;
use crate::compiler::lexer::TokenType::{LP, LR};
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::expression::expr_eval;
use crate::compiler::parser::judgment::if_eval;
use crate::compiler::parser::var::var_eval;
use crate::compiler::parser::r#while::while_eval;
use crate::compiler::parser::{check_char, Parser, ParserError};
use crate::compiler::parser::r#for::for_eval;
use crate::compiler::parser::r#return::return_eval;

fn parser_expr(parser: &mut Parser) -> Result<ASTStmtTree, ParserError> {
    let mut tokens: Vec<Token> = vec![];
    let mut token = parser.next_parser_token()?;
    tokens.push(token);
    loop {
        token = parser.next_parser_token()?;
        if token.t_type == TokenType::End {
            break;
        }
        tokens.push(token);
    }
    expr_eval(parser, tokens)?.map_or(Ok(ASTStmtTree::Empty), |expr| Ok(ASTStmtTree::Expr(expr)))
}

pub fn blk_eval(parser: &mut Parser) -> Result<Vec<ASTStmtTree>, ParserError> {
    let mut token = parser.next_parser_token()?;
    check_char(&token, LP, '{')?;

    let mut stmt: Vec<ASTStmtTree> = vec![];
    loop {
        token = parser.next_parser_token()?;
        match token.t_type {
            TokenType::Function => return Err(ParserError::NotAStatement(token)),
            TokenType::If => {
                parser.last = Some(token);
                stmt.push(if_eval(parser)?);
            }
            TokenType::Var => {
                parser.last = Some(token);
                stmt.push(var_eval(parser)?);
            }
            TokenType::While => {
                parser.last = Some(token);
                stmt.push(while_eval(parser)?);
            }
            TokenType::For => {
                parser.last = Some(token);
                stmt.push(for_eval(parser)?);
            }
            TokenType::End => {
            }
            TokenType::Return => {
                parser.last = Some(token);
                stmt.push(return_eval(parser)?);
            }
            TokenType::Break => {
                stmt.push(ASTStmtTree::Break(token));
                token = parser.next_parser_token()?;
                if token.t_type != TokenType::End {
                    return Err(ParserError::Expected(token,';'));
                }
            }
            TokenType::Continue => {
                stmt.push(ASTStmtTree::Continue(token));
                token = parser.next_parser_token()?;
                if token.t_type != TokenType::End {
                    return Err(ParserError::Expected(token,';'));
                }
            }
            LR => {
                let t = token.clone();
                parser.cache = Some(token);
                if t.text() == "}" {
                    break;
                }
            }
            LP => {
                let t = token.clone();
                parser.cache = Some(token);
                if t.text() == "{" {
                    stmt.push(Context(blk_eval(parser)?));
                } else {
                    stmt.push(parser_expr(parser)?);
                }
            }
            _ => {
                parser.cache = Some(token);
                stmt.push(parser_expr(parser)?);
            } ,
        }
    }
    token = parser.next_parser_token()?;
    check_char(&token, LR, '}')?;
    Ok(stmt)
}
