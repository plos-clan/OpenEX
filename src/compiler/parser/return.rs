use crate::compiler::ast::ASTStmtTree;
use crate::compiler::ast::ASTStmtTree::Return;
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::{Parser, ParserError};
use crate::compiler::parser::expression::expr_eval;

pub fn return_eval(parser: &mut Parser) -> Result<ASTStmtTree, ParserError> {
    let mut token = parser.next_parser_token()?;
    let mut tokens: Vec<Token> = vec![token.clone()];
    loop {
        token = parser.next_parser_token()?;
        if token.t_type == TokenType::End {
            break;
        }
        tokens.push(token);
    }
    Ok(Return(expr_eval(parser, tokens)?))
}

