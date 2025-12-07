use crate::compiler::ast::ssa_ir::Operand;
use crate::compiler::ast::ExprOp;
use crate::compiler::ast::ssa_ir::Operand::{ImmBool, ImmFlot, ImmNum, ImmStr};

pub fn unary_optimizer(op: &ExprOp, operand: &Operand) -> Option<Operand> {
    match op {
        ExprOp::Not => {
            if let ImmBool(b) = operand {
                Some(ImmBool(!b))
            } else {
                todo!() // 理论来讲已经过了一遍检查器了, 类型全部合法不会到达这里
            }
        }
        ExprOp::SAdd => {
            if let ImmNum(num) = operand {
                Some(ImmNum(num + 1))
            } else if let ImmFlot(flot) = operand {
                Some(ImmFlot(flot + 1.0))
            } else {
                todo!()
            }
        }
        ExprOp::SSub => {
            if let ImmNum(num) = operand {
                Some(ImmNum(num - 1))
            } else if let ImmFlot(flot) = operand {
                Some(ImmFlot(flot - 1.0))
            } else {
                todo!()
            }
        }
        _ => None,
    }
}

pub fn expr_optimizer(left: &Operand, right: &Operand, op: &ExprOp) -> Option<Operand> {
    match (left, right, op) {
        (ImmNum(a), ImmNum(b), ExprOp::Add) => Some(ImmNum(a + b)),
        (ImmNum(a), ImmNum(b), ExprOp::Sub) => Some(ImmNum(a - b)),
        (ImmNum(a), ImmNum(b), ExprOp::Mul) => Some(ImmNum(a * b)),
        (ImmNum(a), ImmNum(b), ExprOp::Div) => Some(ImmNum(a / b)),
        (ImmNum(a), ImmNum(b), ExprOp::Rmd) => Some(ImmNum(a % b)),

        // 位运算
        (ImmNum(a), ImmNum(b), ExprOp::BitAnd) => Some(ImmNum(a & b)),
        (ImmNum(a), ImmNum(b), ExprOp::BitOr)  => Some(ImmNum(a | b)),
        (ImmNum(a), ImmNum(b), ExprOp::BitXor) => Some(ImmNum(a ^ b)),
        (ImmNum(a), ImmNum(b), ExprOp::BLeft)  => Some(ImmNum(a << b)),
        (ImmNum(a), ImmNum(b), ExprOp::BRight) => Some(ImmNum(a >> b)),

        // 比较
        (ImmNum(a), ImmNum(b), ExprOp::Big)    => Some(ImmBool(a > b)),
        (ImmNum(a), ImmNum(b), ExprOp::Less)   => Some(ImmBool(a < b)),
        (ImmNum(a), ImmNum(b), ExprOp::BigEqu) => Some(ImmBool(a >= b)),
        (ImmNum(a), ImmNum(b), ExprOp::LesEqu) => Some(ImmBool(a <= b)),
        (ImmNum(a), ImmNum(b), ExprOp::Equ)    => Some(ImmBool(a == b)),
        (ImmNum(a), ImmNum(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),

        (ImmFlot(a), ImmFlot(b), ExprOp::Add) => Some(ImmFlot(a + b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Sub) => Some(ImmFlot(a - b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Mul) => Some(ImmFlot(a * b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Div) => Some(ImmFlot(a / b)),

        // 浮点比较
        (ImmFlot(a), ImmFlot(b), ExprOp::Big)    => Some(ImmBool(a > b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Less)   => Some(ImmBool(a < b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::BigEqu) => Some(ImmBool(a >= b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::LesEqu) => Some(ImmBool(a <= b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Equ)    => Some(ImmBool(a == b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),

        (ImmBool(a), ImmBool(b), ExprOp::And) => Some(ImmBool(*a && *b)),
        (ImmBool(a), ImmBool(b), ExprOp::Or)  => Some(ImmBool(*a || *b)),
        (ImmBool(a), ImmBool(b), ExprOp::Equ) => Some(ImmBool(a == b)),
        (ImmBool(a), ImmBool(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),

        (ImmStr(a), ImmStr(b), ExprOp::Equ)    => Some(ImmBool(a == b)),
        (ImmStr(a), ImmStr(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),

        // 赋值 / 复合赋值 / 引用 / 下标 —— 不做常量折叠
        (_, _, ExprOp::Store)
        | (_, _, ExprOp::AddS)
        | (_, _, ExprOp::SubS)
        | (_, _, ExprOp::MulS)
        | (_, _, ExprOp::DivS)
        | (_, _, ExprOp::RmdS)
        | (_, _, ExprOp::BAndS)
        | (_, _, ExprOp::BOrS)
        | (_, _, ExprOp::BXorS)
        | (_, _, ExprOp::Ref)
        | (_, _, ExprOp::AIndex) => None,

        _ => None,
    }
}
