use crate::compiler::ast::ASTStmtTree;
use crate::compiler::lexer::TokenType::{End, From, Identifier, LiteralString};
use crate::compiler::parser::{Parser, ParserError};
use smol_str::ToSmolStr;

pub fn import_eval(parser: &mut Parser) -> Result<ASTStmtTree, ParserError> {
    let mut token = parser.next_parser_token()?;

    if token.t_type == LiteralString {
        let name = token.text().to_smolstr();
        return Ok(ASTStmtTree::Import(token, name.clone(), name));
    }

    if token.t_type != Identifier {
        return Err(ParserError::IdentifierExpected(token));
    }
    let library = token;
    token = parser.next_parser_token()?;
    if token.t_type != End {
        if token.t_type != From {
            return Err(ParserError::MissingStatement(token));
        }
        token = parser.next_parser_token()?;
        if token.t_type != Identifier && token.t_type != LiteralString {
            return Err(ParserError::Expected(token, '"'));
        }
        let imp_name = token.text().to_smolstr();
        let use_name = library.text().to_smolstr();
        return Ok(ASTStmtTree::Import(library, use_name, imp_name));
    }
    let name = library.text().to_smolstr();
    Ok(ASTStmtTree::Import(library, name.clone(), name))
}
