use crate::compiler::ast::eof_status::EofStatus;
use crate::compiler::ast::eof_status::EofStatus::{Eof, Next};
use crate::compiler::ast::ASTExprTree::{Call, Expr, Var};
use crate::compiler::ast::{ASTExprTree, ExprOp};
use crate::compiler::lexer::TokenType::LP;
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::ParserError::{IllegalExpression, IllegalKey, MissingCondition};
use crate::compiler::parser::{Parser, ParserError};
use std::iter::Peekable;
use std::vec::IntoIter;

fn prefix_binding_power(token: &mut Token) -> ((), u8) {
    let sem = token.value::<String>().unwrap();
    match sem.as_str() {
        "++" | "--" => ((), 21),
        "!" => ((), 23),
        _ => ((), 0),
    }
}

fn postfix_binding_power(token: &mut Token) -> Option<(u8, ())> {
    let sem = token.value::<String>().unwrap();
    match sem.as_str() {
        "++" | "--" => Some((21, ())),
        "[" => Some((27, ())),
        _ => None,
    }
}

fn binding_power(token: &mut Token) -> Option<(u8, u8)> {
    let sem = token.value::<String>().unwrap();
    match sem.as_str() {
        "=" => Some((2, 1)),
        "&&" | "||" => Some((3, 4)),
        "|" => Some((5, 6)),
        "|=" => Some((6, 5)),
        "^" => Some((7, 8)),
        "^=" => Some((8, 7)),
        "&" => Some((9, 10)),
        "&=" => Some((10, 9)),
        "==" | "!=" => Some((11, 12)),
        ">=" | "<=" | "<" | ">" => Some((13, 14)),
        ">>" | "<<" => Some((15, 16)),
        "+" | "-" => Some((17, 18)),
        "+=" | "-=" => Some((18, 17)),
        "*" | "/" | "%" => Some((19, 20)),
        "*=" | "/=" | "%=" => Some((20, 19)),
        "." => Some((26, 25)),
        _ => None,
    }
}

fn parser_arg(
    token0: Token,
    tokens: &mut Peekable<IntoIter<Token>>,
) -> Result<EofStatus<Vec<Token>>, ParserError> {
    let mut expr: Vec<Token> = vec![];
    let mut parentheses_count: usize = 0;
    tokens.next();
    let has_next: bool;
    loop {
        match tokens.peek() {
            Some(token1) => {
                let mut mut_token = token1.clone();
                match mut_token.t_type {
                    TokenType::Semicolon => {
                        if mut_token.value::<String>().unwrap().as_str() == ","
                            && parentheses_count == 0
                        {
                            has_next = true;
                            break;
                        }
                        expr.push(mut_token);
                        tokens.next();
                    }
                    LP => {
                        if mut_token.value::<String>().unwrap() == "(" {
                            parentheses_count += 1;
                        }
                        expr.push(mut_token);
                        tokens.next();
                    }
                    TokenType::LR => {
                        if mut_token.value::<String>().unwrap() == ")" {
                            if parentheses_count == 0 {
                                has_next = false;
                                break;
                            }
                            parentheses_count -= 1;
                        }
                        expr.push(mut_token);
                        tokens.next();
                    }
                    _ => {
                        expr.push(mut_token);
                        tokens.next();
                    }
                }
            }
            None => return Err(ParserError::MissingCondition(token0)),
        };
    }
    if has_next {
        Ok(Next(expr))
    } else {
        Ok(Eof(expr))
    }
}

fn parser_multi_arguments(
    token0: Token,
    tokens: &mut Peekable<IntoIter<Token>>,
) -> Result<Vec<Vec<Token>>, ParserError> {
    let mut arguments: Vec<Vec<Token>> = Vec::new();
    loop {
        match parser_arg(token0.clone(), tokens)? {
            Next(args) => arguments.push(args),
            Eof(args) => {
                arguments.push(args);
                break;
            }
        }
    }
    Ok(arguments)
}

fn func_call_argument(
    identifier: Token,
    parser: &mut Parser,
    tokens: &mut Peekable<IntoIter<Token>>,
) -> Result<ASTExprTree, ParserError> {
    match tokens.peek() {
        Some(token) => {
            let mut token0 = token.clone();
            if token0.t_type == LP && token0.value::<String>().unwrap().as_str() == "(" {
                let mut arguments: Vec<Vec<ASTExprTree>> = Vec::new();
                for args in parser_multi_arguments(token0, tokens)? {
                    let args0: Vec<ASTExprTree> = match args.is_empty() {
                        true => vec![],
                        false => expr_eval(parser, args)?,
                    };
                    arguments.push(args0);
                }
                Ok(Call {
                    name: identifier,
                    args: arguments,
                })
            } else {
                Ok(Var(identifier))
            }
        }
        None => Ok(Var(identifier)),
    }
}

fn expr_bp(
    parser: &mut Parser,
    tokens: &mut Peekable<IntoIter<Token>>,
    min_bp: u8,
) -> Result<ASTExprTree, ParserError> {
    let mut token = match tokens.next() {
        Some(token) => token,
        None => return Err(IllegalExpression(parser.last.take().unwrap())),
    };

    let mut expr_tree: ASTExprTree = match token.t_type {
        LP => {
            let mut t = token.clone();
            if t.value::<String>().unwrap().as_str() != "(" {
                return Err(IllegalExpression(token));
            }
            let lhs = expr_bp(parser, tokens, 0);
            let mut n_token = match tokens.next() {
                Some(token) => token,
                None => return Err(MissingCondition(token)),
            };
            parser.check_char(&mut n_token, TokenType::LR, ')')?;
            lhs?
        }
        TokenType::Semicolon => {
            let ((), r_bp) = prefix_binding_power(&mut token);
            let op = match token.value::<String>().unwrap().as_str() {
                "++" => ExprOp::SAdd,
                "--" => ExprOp::SSub,
                "!" => ExprOp::Not,
                _ => return Err(IllegalExpression(token)),
            };
            parser.last = Some(token.clone());
            let rhs = expr_bp(parser, tokens, r_bp)?;
            ASTExprTree::Unary {
                op,
                code: Box::new(rhs),
            }
        }
        TokenType::Number
        | TokenType::True
        | TokenType::False
        | TokenType::LiteralString
        | TokenType::Null => ASTExprTree::Literal(token),
        TokenType::Identifier => func_call_argument(token, parser, tokens)?,
        _ => return Err(IllegalKey(token)),
    };

    loop {
        token = match tokens.peek().cloned() {
            Some(token0) => token0,
            None => break,
        };

        if token.t_type != TokenType::Semicolon && token.t_type != TokenType::LR {
            if !(token.t_type == LP && token.value::<String>().unwrap().as_str() == "[") {
                return Err(IllegalExpression(token));
            }
        }

        if let Some((l_bp, ())) = postfix_binding_power(&mut token) {
            if l_bp < min_bp {
                break;
            }

            tokens.next();
            if token.t_type == LP {
                parser.check_char(&mut token, LP, '[')?;
                let rhs = expr_bp(parser, tokens, 0)?;
                match tokens.next() {
                    Some(mut tk) => {
                        parser.check_char(&mut tk, TokenType::LR, ']')?;
                        expr_tree = Expr {
                            op: ExprOp::AIndex,
                            left: Box::new(expr_tree),
                            right: Box::new(rhs),
                        };
                    }
                    None => {
                        return Err(MissingCondition(token));
                    }
                }
            } else {
                let op = match token.value::<String>().unwrap().as_str() {
                    "++" => ExprOp::SAdd,
                    "--" => ExprOp::SSub,
                    _ => return Err(IllegalExpression(token)),
                };

                expr_tree = ASTExprTree::Unary {
                    op,
                    code: Box::new(expr_tree),
                };
            }
            continue;
        }

        if let Some((l_bp, r_bp)) = binding_power(&mut token) {
            if l_bp < min_bp {
                break;
            }
            tokens.next();
            let rhs = expr_bp(parser, tokens, r_bp)?;
            let op = match token.value::<String>().unwrap().as_str() {
                "+" => ExprOp::Add,
                "-" => ExprOp::Sub,
                "*" => ExprOp::Mul,
                "/" => ExprOp::Div,
                "==" => ExprOp::Equ,
                "!=" => ExprOp::NotEqu,
                ">=" => ExprOp::BigEqu,
                "<=" => ExprOp::LesEqu,
                "=" => ExprOp::Store,
                ">" => ExprOp::Big,
                "<" => ExprOp::Less,
                "&&" => ExprOp::And,
                "||" => ExprOp::Or,
                "%" => ExprOp::Rmd,
                "+=" => ExprOp::AddS,
                "-=" => ExprOp::SubS,
                "*=" => ExprOp::MulS,
                "/=" => ExprOp::DivS,
                "%=" => ExprOp::RmdS,
                "&" => ExprOp::BitAnd,
                "^" => ExprOp::BitXor,
                "|" => ExprOp::BitOr,
                "&=" => ExprOp::BAndS,
                "|=" => ExprOp::BOrS,
                "^=" => ExprOp::BXorS,
                ">>" => ExprOp::BLeft,
                "<<" => ExprOp::BRight,
                "." => ExprOp::Ref,
                _ => return Err(IllegalExpression(token)),
            };
            expr_tree = Expr {
                op,
                left: Box::new(expr_tree),
                right: Box::new(rhs),
            };
            continue;
        }
        break;
    }

    Ok(expr_tree)
}

pub fn expr_eval(parser: &mut Parser, tokens: Vec<Token>) -> Result<Vec<ASTExprTree>, ParserError> {
    if tokens.len() == 0 {
        return Ok(Vec::new());
    }
    let mut into_tokens = tokens.into_iter().peekable();
    let mut expr_tree: Vec<ASTExprTree> = vec![];
    expr_tree.push(expr_bp(parser, &mut into_tokens, 0)?);
    Ok(expr_tree)
}
