use crate::compiler::ast::ASTStmtTree::Var;
use crate::compiler::ast::ASTStmtTree;
use crate::compiler::lexer::TokenType::{Semicolon, End};
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::expression::expr_eval;
use crate::compiler::parser::{Parser, ParserError};

pub fn var_eval(parser:&mut Parser) -> Result<ASTStmtTree, ParserError> {
    let mut token = parser.next_parser_token()?;
    if token.t_type != TokenType::Identifier {
        return Err(ParserError::IdentifierExpected(token));
    }
    let var_name = token;
    token = parser.next_parser_token()?;
    if token.t_type == End {
        return Ok(Var {
            name: var_name,
            value: None,
        })
    }
    parser.check_char(&mut token, Semicolon, '=')?;

    let mut cone:Vec<Token> = vec![];
    loop {
        token = parser.next_parser_token()?;
        if token.t_type == End {
            break;
        }
        cone.push(token);
    }

    Ok(Var {
        name: var_name,
        value: expr_eval(parser, cone)?,
    })
}
