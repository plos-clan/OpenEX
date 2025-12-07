use crate::compiler::ast::ssa_ir::OpCode::Push;
use crate::compiler::ast::ssa_ir::ValueGuessType::{Bool, Float, Null, Number, String, Unknown};
use crate::compiler::ast::ssa_ir::{Code, OpCode, Operand, ValueGuessType};
use crate::compiler::ast::{ASTExprTree, ExprOp};
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::optimizer::{expr_optimizer, unary_optimizer};
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

fn guess_type_unary(
    token: &Token,
    first: ValueGuessType,
    op: &ExprOp,
) -> Result<ValueGuessType, ParserError> {
    match first {
        Bool => {
            if matches!(op, ExprOp::Not) {
                Ok(Bool)
            } else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        Number | Float => {
            if matches!(op, ExprOp::SAdd) || matches!(op, ExprOp::SSub) {
                Ok(first)
            } else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        Null => Err(ParserError::IllegalTypeCombination(token.clone())),
        _ => Ok(Unknown),
    }
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
            if guess_check_type(second.clone(), &[Number, Float]) {
                Ok(second)
            } else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        Float => {
            if guess_check_type(second, &[Float, Number]) {
                Ok(Float)
            } else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        Null => {
            if guess_check_type(second, &[Null]) {
                Ok(Null)
            } else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        _ => Err(ParserError::IllegalTypeCombination(token.clone())),
    }
}

fn lower_expr(
    expr_tree: &ASTExprTree,
) -> Result<(Operand, ValueGuessType, Vec<OpCode>), ParserError> {
    match expr_tree {
        ASTExprTree::Literal(lit) => {
            let tk_lit = &mut lit.clone();
            match lit.t_type {
                TokenType::Number => {
                    let operand = Operand::ImmNum(tk_lit.value_number());
                    Ok((operand.clone(), Number, vec![Push(operand)]))
                }
                TokenType::LiteralString => {
                    let operand = Operand::ImmStr(tk_lit.value::<SmolStr>().unwrap());
                    Ok((operand.clone(), String, vec![Push(operand)]))
                }
                TokenType::True | TokenType::False => {
                    let operand = Operand::ImmBool(lit.t_type == TokenType::True);
                    Ok((operand.clone(), Bool, vec![Push(operand)]))
                }
                TokenType::Null => Ok((Operand::Null, Bool, vec![Push(Operand::Null)])),
                _ => {
                    todo!()
                }
            }
        }
        ASTExprTree::Unary {
            token: u_token,
            op: u_op,
            code: u_code,
        } => {
            let mut a = lower_expr(u_code.as_ref())?;
            let g_type = guess_type_unary(u_token, a.1, u_op)?;
            let mut op_code;
            if let Some(operand) = unary_optimizer(u_op, &a.0) {
                op_code = vec![Push(operand)];
            } else {
                op_code = vec![];
                op_code.append(&mut a.2);
                op_code.push(astop_to_opcode(u_op));
            }
            Ok((a.0, g_type, op_code))
        }
        ASTExprTree::Expr {
            token: e_token,
            op: e_op,
            left: e_left,
            right: e_right,
        } => {
            let mut left = lower_expr(e_left.as_ref())?;
            let mut right = lower_expr(e_right.as_ref())?;
            let left_opd = Box::new(left.0.clone());
            let right_opd = Box::new(right.0.clone());
            let guess_type = guess_type(e_token, left.1, right.1)?;
            let mut op_code = vec![];
            let n_operand;

            if let Some(operand) = expr_optimizer(&left.0, &right.0, e_op) {
                n_operand = operand.clone();
                op_code.push(Push(operand));
            } else {
                if !matches!(left.0, Operand::Expression(_, _)) {
                    op_code.append(&mut left.2);
                }
                if !matches!(right.0, Operand::Expression(_, _)) {
                    op_code.append(&mut right.2);
                }
                op_code.push(astop_to_opcode(e_op));
                n_operand = Operand::Expression(left_opd, right_opd);
            }

            Ok((n_operand, guess_type, op_code))
        }
        _ => {
            todo!()
        }
    }
}

pub fn expr_semantic(
    expr: Option<ASTExprTree>,
    code: &mut Code,
) -> Result<(Operand, ValueGuessType), ParserError> {
    let guess_type;
    let operand: Operand;

    if let Some(expr) = expr {
        let mut exp = lower_expr(&expr)?;
        code.append_code(&mut exp.2);
        operand = exp.0;
        guess_type = exp.1;
    } else {
        guess_type = Null;
        operand = Operand::Null;
    }

    Ok((operand, guess_type))
}
