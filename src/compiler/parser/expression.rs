use std::iter::Peekable;
use std::vec::IntoIter;

use crate::compiler::ast::ASTExprTree::{Call, Expr, Var};
use crate::compiler::ast::{ASTExprTree, ExprOp};
use crate::compiler::lexer::TokenType::LP;
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::ParserError::{IllegalExpression, IllegalKey, MissingCondition};
use crate::compiler::parser::{Parser, ParserError, check_char};

fn prefix_binding_power(token: &Token) -> ((), u8) {
    match token.text() {
        "++" | "--" => ((), 21),
        "!" | "+" | "-" => ((), 23),
        _ => ((), 0),
    }
}

fn postfix_binding_power(token: &Token) -> Option<(u8, ())> {
    match token.text() {
        "++" | "--" => Some((21, ())),
        "[" | "(" => Some((27, ())),
        _ => None,
    }
}

fn binding_power(token: &Token) -> Option<(u8, u8)> {
    match token.text() {
        "=" | "|=" | "&=" | "^=" | "+=" | "-=" | "*=" | "/=" | "%=" => Some((2, 1)),
        "&&" | "||" => Some((3, 4)),
        "|" => Some((5, 6)),
        "^" => Some((7, 8)),
        "&" => Some((9, 10)),
        "==" | "!=" => Some((11, 12)),
        ">=" | "<=" | "<" | ">" => Some((13, 14)),
        ">>" | "<<" => Some((15, 16)),
        "+" | "-" => Some((17, 18)),
        "*" | "/" | "%" => Some((19, 20)),
        "." => Some((30, 29)),
        _ => None,
    }
}

fn build_head_ast_tree(
    parser: &mut Parser,
    tokens: &mut Peekable<IntoIter<Token>>,
    token: Token,
) -> Result<ASTExprTree, ParserError> {
    match token.t_type {
        LP => {
            let t = token;
            if t.text() != "(" {
                return Err(IllegalExpression(t));
            }
            let lhs = expr_bp(parser, tokens, 0);
            let Some(n_token) = tokens.next() else {
                return Err(MissingCondition(t));
            };
            check_char(&n_token, TokenType::LR, ')')?;
            lhs
        }
        TokenType::Operator => {
            let ((), r_bp) = prefix_binding_power(&token);
            let op = match token.text() {
                "+" => ExprOp::Pos,
                "-" => ExprOp::Neg,
                "++" => ExprOp::SAdd,
                "--" => ExprOp::SSub,
                "!" => ExprOp::Not,
                _ => return Err(IllegalExpression(token)),
            };
            parser.last = Some(token.clone());
            let rhs = expr_bp(parser, tokens, r_bp)?;
            Ok(ASTExprTree::Unary {
                token,
                op,
                code: Box::new(rhs),
            })
        }
        TokenType::Number
        | TokenType::True
        | TokenType::False
        | TokenType::LiteralString
        | TokenType::Float
        | TokenType::Null => Ok(ASTExprTree::Literal(token)),
        TokenType::This => Ok(ASTExprTree::This(token)),
        TokenType::Identifier => Ok(Var(token)),
        _ => Err(IllegalKey(token)),
    }
}

macro_rules! match_opcode {
    ($token:expr) => {
        match $token.text() {
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
            "<<" => ExprOp::BLeft,
            ">>" => ExprOp::BRight,
            "." => ExprOp::Ref,
            _ => return Err(IllegalExpression($token)),
        }
    };
}

macro_rules! check_operand {
    ($token:expr) => {
        if $token.t_type != TokenType::Operator
            && $token.t_type != TokenType::LR
            && !($token.t_type == LP && ($token.text() == "[" || $token.text() == "("))
        {
            return Err(IllegalExpression($token));
        }
    };
}

fn expr_bp(
    parser: &mut Parser,
    tokens: &mut Peekable<IntoIter<Token>>,
    min_bp: u8,
) -> Result<ASTExprTree, ParserError> {
    let Some(mut token) = tokens.next() else {
        return Err(IllegalExpression(parser.last.take().unwrap()));
    };

    let mut expr_tree: ASTExprTree = build_head_ast_tree(parser, tokens, token)?;

    loop {
        token = match tokens.peek().cloned() {
            Some(token) => token,
            None => break,
        };

        check_operand!(token);

        if let Some((l_bp, ())) = postfix_binding_power(&token) {
            if l_bp < min_bp {
                break;
            }

            tokens.next();
            if token.t_type == LP {
                if matches!(check_char(&token, LP, '['), Ok(())) {
                    let rhs = expr_bp(parser, tokens, 0)?;
                    match tokens.next() {
                        Some(tk) => {
                            check_char(&tk, TokenType::LR, ']')?;
                            expr_tree = Expr {
                                token,
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
                    check_char(&token, LP, '(')?;
                    let mut arguments: Vec<ASTExprTree> = vec![];
                    loop {
                        let mut sub_tokens: Vec<Token> = vec![];
                        token = tokens.next().ok_or(MissingCondition(token))?;
                        let mut p_count: u64 = 0;
                        let mut done: bool = false;
                        loop {
                            if token.t_type == TokenType::Operator
                                && token.text() == ","
                                && p_count == 0
                            {
                                break;
                            }
                            if token.t_type == LP && token.text() == "(" {
                                p_count += 1;
                            }
                            if token.t_type == TokenType::LR && token.text() == ")" {
                                if p_count == 0 {
                                    done = true;
                                    break;
                                }
                                p_count -= 1;
                            }
                            sub_tokens.push(token.clone());
                            token = tokens.next().ok_or(MissingCondition(token))?;
                        }
                        if let Some(expr) = expr_eval(parser, sub_tokens)? {
                            arguments.push(expr);
                        }
                        if done {
                            break;
                        }
                    }
                    expr_tree = Call {
                        name: Box::new(expr_tree),
                        args: arguments,
                    }
                }
            } else {
                let op = match token.text() {
                    "++" => ExprOp::SAdd,
                    "--" => ExprOp::SSub,
                    _ => return Err(IllegalExpression(token)),
                };

                expr_tree = ASTExprTree::Unary {
                    token,
                    op,
                    code: Box::new(expr_tree),
                };
            }
            continue;
        }

        if let Some((l_bp, r_bp)) = binding_power(&token) {
            if l_bp < min_bp {
                break;
            }
            tokens.next();
            let rhs = expr_bp(parser, tokens, r_bp)?;
            let op = match_opcode!(token);
            expr_tree = Expr {
                token,
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

pub fn expr_eval(
    parser: &mut Parser,
    tokens: Vec<Token>,
) -> Result<Option<ASTExprTree>, ParserError> {
    if tokens.is_empty() {
        return Ok(None);
    }
    let mut into_tokens = tokens.into_iter().peekable();
    Ok(Some(expr_bp(parser, &mut into_tokens, 0)?))
}
