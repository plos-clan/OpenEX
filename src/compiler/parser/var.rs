use crate::compiler::ast::ASTStmtTree::Var;
use crate::compiler::ast::{ASTExprTree, ASTStmtTree};
use crate::compiler::lexer::TokenType::{End, LP, LR, Operator};
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::expression::expr_eval;
use crate::compiler::parser::{Parser, ParserError, check_char};

fn parse_fill_len_expr(parser: &mut Parser) -> Result<ASTExprTree, ParserError> {
    let mut token;
    let mut p_count = 0;
    let mut sub_exp: Vec<Token> = Vec::new();
    loop {
        token = parser.next_parser_token()?;
        if token.t_type == Operator && token.text() == "," && p_count == 0 {
            return Err(ParserError::IllegalExpression(token));
        }
        if token.t_type == End && p_count == 0 {
            return Err(ParserError::IllegalExpression(token));
        }

        if token.t_type == LP && (token.text() == "[" || token.text() == "(" || token.text() == "{")
        {
            p_count += 1;
        }

        if token.t_type == LR {
            if token.text() == "]" && p_count == 0 {
                break;
            }
            p_count -= 1;
        }

        sub_exp.push(token);
    }

    expr_eval(parser, sub_exp)?.ok_or(ParserError::IllegalTypeCombination(token))
}

fn try_parse_fill_count(expr: &ASTExprTree) -> Result<Option<usize>, ParserError> {
    let ASTExprTree::Literal(token) = expr else {
        return Ok(None);
    };
    if token.t_type != TokenType::Number {
        return Ok(None);
    }
    let count = token.value_number();
    let count =
        usize::try_from(count).map_err(|_| ParserError::IllegalTypeCombination(token.clone()))?;
    Ok(Some(count))
}

pub fn var_eval(parser: &mut Parser) -> Result<ASTStmtTree, ParserError> {
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
        });
    }
    check_char(&token, Operator, '=')?;

    token = parser.next_parser_token()?;
    if token.t_type == LP && token.text() == "[" {
        let mut p_count = 0;
        let mut cone: Vec<ASTExprTree> = Vec::new();
        let mut done = false;
        loop {
            let mut sub_exp: Vec<Token> = Vec::new();
            loop {
                token = parser.next_parser_token()?;
                if token.t_type == Operator && token.text() == "," && p_count == 0 {
                    break;
                }
                if token.t_type == End && p_count == 0 {
                    let fill_expr = match expr_eval(parser, sub_exp)? {
                        None => return Err(ParserError::IllegalTypeCombination(token)),
                        Some(expr) => expr,
                    };
                    let len_expr = parse_fill_len_expr(parser)?;
                    if let Some(count) = try_parse_fill_count(&len_expr)? {
                        return Ok(ASTStmtTree::Array {
                            token: var_name,
                            elements: vec![fill_expr; count],
                        });
                    }
                    return Ok(ASTStmtTree::ArrayFill {
                        token: var_name,
                        value: fill_expr,
                        count: len_expr,
                    });
                }

                if token.t_type == LP
                    && (token.text() == "[" || token.text() == "(" || token.text() == "{")
                {
                    p_count += 1;
                }

                if token.t_type == LR {
                    if token.text() == "]" && p_count == 0 {
                        done = true;
                        break;
                    }
                    p_count -= 1;
                }

                sub_exp.push(token);
            }

            cone.push(match expr_eval(parser, sub_exp)? {
                None => {
                    return Err(ParserError::IllegalTypeCombination(token));
                }
                Some(expr) => expr,
            });
            if done {
                break;
            }
        }

        Ok(ASTStmtTree::Array {
            token: var_name,
            elements: cone,
        })
    } else {
        parser.cache = Some(token);
        let mut cone: Vec<Token> = vec![];

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
}
