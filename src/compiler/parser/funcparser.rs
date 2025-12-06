use crate::compiler::ast::{ASTExprTree, ASTStmtTree};
use crate::compiler::lexer::TokenType;
use crate::compiler::lexer::TokenType::{LP, LR};
use crate::compiler::parser::blkparser::blk_eval;
use crate::compiler::parser::symbol_table::ContextType::ROOT;
use crate::compiler::parser::ParserError::{Expected, IllegalArgument, NotAStatement};
use crate::compiler::parser::{Parser, ParserError};

fn parser_argument(parser: &mut Parser) -> Result<Vec<ASTExprTree>, ParserError> {
    let mut token = parser.next_parser_token()?;
    parser.check_char(&mut token,LP, '(')?;
    let mut is_split = false;
    let mut arguments: Vec<ASTExprTree> = Vec::new();
    loop {
        token = parser.next_parser_token()?;
        match parser.check_char(&mut token,LR, ')') {
            Ok(_) => break,
            Err(_) => match token.t_type {
                TokenType::Identifier => {
                    if is_split {
                        return Err(Expected(token, ','));
                    }
                    arguments.push(ASTExprTree::Var(token));
                    is_split = true;
                }
                TokenType::Semicolon => {
                    if !is_split {
                        return Err(ParserError::IdentifierExpected(token));
                    }
                    is_split = false;
                }
                _ => {
                    return Err(IllegalArgument(token));
                }
            },
        }
    }
    parser.last = Some(token);
    Ok(arguments)
}

pub fn func_eval(parser: &mut Parser) -> Result<ASTStmtTree, ParserError> {
    let mut token = parser.next_parser_token()?;
    if !parser.file.c_data.symbol_table.in_context(ROOT) { //TODO ?
        return Err(NotAStatement(token));
    }
    token = parser.next_parser_token()?;
    if token.t_type != TokenType::Identifier {
        return Err(ParserError::IdentifierExpected(token));
    }
    let name = token;
    token = parser.next_parser_token()?;

    let args:Vec<ASTExprTree>;
    match token.t_type {
        LP => {
            if token.value::<String>().unwrap() == "{" {
                parser.cache = Some(token);
                args = vec![]
            }else {
                parser.cache = Some(token);
                args = parser_argument(parser)?;
            }
        }
        _=> {
            parser.cache = Some(token);
            args = vec![]
        }
    };

    let result = parser.next_parser_token();
    if let Err(ParserError::EOF) = result {
        return Err(ParserError::MissingFunctionBody(parser.get_last().unwrap()));
    }
    token = result?;
    parser.cache = Some(token);

    let body = blk_eval(parser)?;

    Ok(ASTStmtTree::Function { name, args, body })
}
