use crate::compiler::ast::ASTStmtTree::Block;
use crate::compiler::ast::ASTStmtTree;
use crate::compiler::lexer::TokenType;
use crate::compiler::parser::block::blk_eval;
use crate::compiler::parser::{Parser, ParserError};

fn next_parser_if(parser: &mut Parser,else_body:&mut Vec<ASTStmtTree>) -> Result<(),ParserError> {
    match parser_elif(parser){
        Ok(elif_stmt) => {
            if let Some(elif_stmt) = elif_stmt {
                else_body.push(elif_stmt);
            }
        },
        Err(parser_error) =>{
            match parser_error {
                ParserError::Eof => {}
                _ => return Err(parser_error),
            }
        },
    };

    match parser_else(parser){
        Ok(else_stmt) => {
            if let Some(else_stmt) = else_stmt {
                else_body.push(else_stmt);
            }
            Ok(())
        }
        Err(parser_error) =>{
            match parser_error {
                ParserError::Eof => {
                    Ok(())
                }
                _ => Err(parser_error),
            }
        },
    }
}

fn parser_elif(parser: &mut Parser) -> Result<Option<ASTStmtTree>, ParserError> {
    let token = parser.next_parser_token()?;
    if token.t_type != TokenType::Elif {
        parser.cache = Some(token);
        return Ok(None);
    }
    let cond = parser.parser_cond()?;
    let then = blk_eval(parser)?;

    let mut else_body: Vec<ASTStmtTree> = vec![];

    next_parser_if(parser, &mut else_body)?;

    Ok(Some(ASTStmtTree::If {
        cond,
        then_body: then,
        else_body,
    }))
}

fn parser_else(parser: &mut Parser) -> Result<Option<ASTStmtTree>, ParserError> {
    let token = parser.next_parser_token()?;

    if token.t_type != TokenType::Else {
        parser.cache = Some(token);
        return Ok(None);
    }
    let then = blk_eval(parser)?;
    Ok(Some(Block(then)))
}

pub fn if_eval(parser: &mut Parser) -> Result<ASTStmtTree, ParserError> {
    let cond = parser.parser_cond()?;
    let then = blk_eval(parser)?;

    let mut else_body: Vec<ASTStmtTree> = vec![];

    next_parser_if(parser, &mut else_body)?;

    Ok(ASTStmtTree::If {
        cond,
        then_body: then,
        else_body,
    })
}
