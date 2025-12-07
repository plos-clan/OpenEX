use crate::compiler::ast::ssa_ir::OpCode::Push;
use crate::compiler::ast::ssa_ir::ValueGuessType::{Bool, Float, Null, Number, String, Unknown};
use crate::compiler::ast::ssa_ir::{Code, OpCode, Operand, ValueGuessType};
use crate::compiler::ast::{ASTExprTree, ExprOp};
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::Semantic;
use smol_str::SmolStr;

fn astop_to_opcode(astop: &ExprOp) -> OpCode {
    match astop {
        ExprOp::And => OpCode::And,
        ExprOp::Or => OpCode::Or,
        ExprOp::Not => OpCode::Not,
        ExprOp::BLeft => OpCode::BLeft,
        ExprOp::BRight => OpCode::BRight,
        ExprOp::BitXor => OpCode::BitXor,
        ExprOp::BitAnd => OpCode::BitAnd,
        ExprOp::BitOr => OpCode::BitOr,
        ExprOp::Sub => OpCode::Sub,
        ExprOp::Add => OpCode::Add,
        ExprOp::Mul => OpCode::Mul,
        ExprOp::Div => OpCode::Div,
        ExprOp::RmdS => OpCode::RmdS,
        ExprOp::AddS => OpCode::AddS,
        ExprOp::SubS => OpCode::SubS,
        ExprOp::MulS => OpCode::MulS,
        ExprOp::DivS => OpCode::DivS,
        ExprOp::Ref => OpCode::Ref,
        ExprOp::SAdd => OpCode::SAdd,
        ExprOp::SSub => OpCode::SSub,
        ExprOp::Store => OpCode::Store,
        _ => todo!(),
    }
}

fn guess_check_type(src: ValueGuessType, args: &[ValueGuessType]) -> bool {
    for ty in args {
        if src == *ty {
            return true;
        }
    }
    false
}

fn guess_type(
    token: &Token,
    first: ValueGuessType,
    second: ValueGuessType,
) -> Result<ValueGuessType, ParserError> {
    if first == second {
        return Ok(first);
    }
    if first == Unknown || second == Unknown {
        return Ok(Unknown);
    }

    match first {
        Bool => {
            if guess_check_type(second, &[Bool]) {
                Ok(Bool)
            } else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        String => {
            if guess_check_type(second, &[String, Float, Number, Null]) {
                Ok(String)
            } else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        Number => {
            if guess_check_type(second.clone(), &[Number]) {
                Ok(Number)
            }else if guess_check_type(second, &[Float]) {
                Ok(Float)
            }else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        Float => {
            if guess_check_type(second, &[Float,Number]) {
                Ok(Float)
            }else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        Null => {
            if guess_check_type(second, &[Null]) {
                Ok(Null)
            }else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        _ => Err(ParserError::IllegalTypeCombination(token.clone())),
    }
}

fn lower_expr(
    semantic: &mut Semantic,
    expr_tree: &ASTExprTree,
    code: &mut Code,
) -> Result<(Operand, ValueGuessType), ParserError> {
    match expr_tree {
        ASTExprTree::Literal(lit) => {
            let tk_lit = &mut lit.clone();
            match lit.t_type {
                TokenType::Number => Ok((
                    Operand::ImmNum(tk_lit.value_number()),
                    Number,
                )),
                TokenType::LiteralString => Ok((
                    Operand::ImmStr(tk_lit.value::<SmolStr>().unwrap()),
                    String,
                )),
                TokenType::True => Ok((Operand::ImmBool(true), Bool)),
                TokenType::False => Ok((Operand::ImmBool(false), Bool)),
                TokenType::Null => Ok((Operand::Null, Null)),
                _ => {
                    todo!()
                }
            }
        }
        ASTExprTree::Unary {
            token: _u_token,
            op: u_op,
            code: u_code,
        } => {
            let a = lower_expr(semantic, u_code.as_ref(), code)?;
            code.add_opcode(astop_to_opcode(u_op));
            Ok(a)
        }
        ASTExprTree::Expr {
            token: e_token,
            op: e_op,
            left: e_left,
            right: e_right,
        } => {
            let left = lower_expr(semantic, e_left.as_ref(), code)?;
            let right = lower_expr(semantic, e_right.as_ref(), code)?;
            let left_opd = Box::new(left.0.clone());
            let right_opd = Box::new(right.0.clone());
            if !matches!(left.0, Operand::ExprOperand(_, _)) {
                code.add_opcode(Push(left.0));
            }
            if !matches!(right.0, Operand::ExprOperand(_, _)) {
                code.add_opcode(Push(right.0));
            }
            code.add_opcode(astop_to_opcode(e_op));
            Ok((
                Operand::ExprOperand(left_opd, right_opd),
                guess_type(e_token, left.1, right.1)?,
            ))
        }
        _ => {
            todo!()
        }
    }
}

pub fn expr_semantic(
    semantic: &mut Semantic,
    expr: Option<ASTExprTree>,
    code: &mut Code,
) -> Result<(Operand, ValueGuessType), ParserError> {
    let mut guess_type = Unknown;
    let operand: Operand;

    if let Some(expr) = expr {
        let exp = lower_expr(semantic, &expr, code)?;
        operand = exp.0;
        guess_type = exp.1;
    } else {
        guess_type = ValueGuessType::Null;
        operand = Operand::Null;
    }

    Ok((operand, guess_type))
}
