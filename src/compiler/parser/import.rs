use crate::compiler::ast::ASTStmtTree;
use crate::compiler::lexer::TokenType::{Identifier, LiteralString, End};
use crate::compiler::parser::{Parser, ParserError};

pub fn import_eval(parser: &mut Parser) -> Result<ASTStmtTree, ParserError> {
    let mut token = parser.next_parser_token()?;

    if token.t_type != LiteralString && token.t_type != Identifier {
        return  Err(ParserError::Expected(token, '"'))
    }
    let library = token;
    token = parser.next_parser_token()?;
    if token.t_type != End {
        return  Err(ParserError::Expected(token, ';'))
    }

    Ok(ASTStmtTree::Import(library))
}
