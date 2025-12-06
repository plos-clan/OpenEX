use crate::compiler::ast::ASTStmtTree;
use crate::compiler::ast::ASTStmtTree::Return;
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::{Parser, ParserError};
use crate::compiler::parser::exprparser::expr_eval;

pub fn return_eval(parser: &mut Parser) -> Result<ASTStmtTree, ParserError> {
    let mut token1 = parser.next_parser_token()?;
    let mut tokens: Vec<Token> = vec![token1.clone()];
    loop {
        token1 = parser.next_parser_token()?;
        if token1.t_type == TokenType::END {
            break;
        }
        tokens.push(token1);
    }
    Ok(Return(expr_eval(parser, tokens)?))
}

