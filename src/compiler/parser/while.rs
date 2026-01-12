use crate::compiler::ast::ASTExprTree::Literal;
use crate::compiler::ast::{ASTExprTree, ASTStmtTree};
use crate::compiler::lexer::TokenType::LP;
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::block::blk_eval;
use crate::compiler::parser::{Parser, ParserError};
use smol_str::SmolStr;

pub fn while_eval(parser: &mut Parser) -> Result<ASTStmtTree, ParserError> {
    let mut token = parser.next_parser_token()?;
    let head = token.clone();
    let cond: ASTExprTree;
    let is_easy;
    match token.t_type {
        LP => {
            if token.text() == "{" {
                let tk_b = token.clone();
                parser.cache = Some(token);
                is_easy = true;
                cond = Literal(Token::new(
                    SmolStr::new("true"),
                    tk_b.line,
                    tk_b.column,
                    tk_b.index,
                    TokenType::True,
                ));
            } else {
                is_easy = false;
                parser.cache = Some(token);
                cond = parser.parser_cond(None)?;
            }
        }
        _ => return Err(ParserError::Expected(token, '(')),
    }

    let result = parser.next_parser_token();
    if matches!(result, Err(ParserError::Eof)) {
        return Err(ParserError::MissingLoopBody(parser.get_last().unwrap()));
    }
    token = result?;
    parser.cache = Some(token);

    let body = blk_eval(parser)?;

    Ok(ASTStmtTree::Loop { token:head,cond, body, is_easy })
}
