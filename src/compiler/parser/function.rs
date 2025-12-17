use crate::compiler::ast::{ASTExprTree, ASTStmtTree};
use crate::compiler::lexer::TokenType;
use crate::compiler::lexer::TokenType::{LP, LR};
use crate::compiler::parser::block::blk_eval;
use crate::compiler::parser::ParserError::{Expected, IdentifierExpected, IllegalArgument};
use crate::compiler::parser::{check_char, Parser, ParserError};

fn parser_argument(parser: &mut Parser) -> Result<Vec<ASTExprTree>, ParserError> {
    let mut token = parser.next_parser_token()?;
    check_char(&token, LP, '(')?;
    let mut is_split = false;
    let mut arguments: Vec<ASTExprTree> = Vec::new();
    loop {
        token = parser.next_parser_token()?;
        match check_char(&token, LR, ')') {
            Ok(()) => break,
            Err(_) => match token.t_type {
                TokenType::Identifier => {
                    if is_split {
                        return Err(Expected(token, ','));
                    }
                    arguments.push(ASTExprTree::Var(token));
                    is_split = true;
                }
                TokenType::Operator => {
                    if !is_split {
                        return Err(IdentifierExpected(token));
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
    parser.next_parser_token()?;
    let mut token = parser.next_parser_token()?;
    let name;
    let is_native;
    match token.t_type {
        TokenType::Identifier => {
            name = token;
            is_native = false;
        }
        TokenType::Native => {
            token = parser.next_parser_token()?;
            if token.t_type != TokenType::Identifier {
                return Err(IdentifierExpected(token));
            }
            name = token;
            is_native = true;
        }
        _ => {
            return Err(IdentifierExpected(token));
        }
    }

    token = parser.next_parser_token()?;

    let args: Vec<ASTExprTree>;
    match token.t_type {
        LP => {
            if token.text() == "{" {
                parser.cache = Some(token);
                args = vec![];
            } else {
                parser.cache = Some(token);
                args = parser_argument(parser)?;
            }
        }
        TokenType::End => {
            if !is_native {
                return Err(ParserError::MissingFunctionBody(token));
            }
            args = vec![];
        }
        _ => {
            parser.cache = Some(token);
            args = vec![];
        }
    }

    let result = parser.next_parser_token();
    if matches!(result, Err(ParserError::Eof)) {
        return Err(ParserError::MissingFunctionBody(parser.get_last().unwrap()));
    }
    token = result?;
    if token.t_type == TokenType::End && is_native {
        Ok(ASTStmtTree::NativeFunction {name,args})
    } else {
        parser.cache = Some(token);
        let body = blk_eval(parser)?;
        Ok(ASTStmtTree::Function { name, args, body })
    }
}
