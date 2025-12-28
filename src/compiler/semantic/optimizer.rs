use crate::compiler::ast::ssa_ir::Operand;
use crate::compiler::ast::ssa_ir::Operand::{ImmBool, ImmFlot, ImmNum, ImmStr, Library, Reference};
use crate::compiler::ast::ExprOp;
use dashu::float::{Context, DBig};
use smol_str::SmolStrBuilder;

pub fn unary_optimizer(op: ExprOp, operand: &Operand) -> Option<Operand> {
    match op {
        ExprOp::Not => {
            if let ImmBool(b) = operand {
                Some(ImmBool(!b))
            } else {
                None
            }
        }
        ExprOp::SAdd => {
            if let ImmNum(num) = operand {
                Some(ImmNum(num + 1))
            } else if let ImmFlot(flot) = operand {
                Some(ImmFlot(flot + DBig::from(1)))
            } else {
                None
            }
        }
        ExprOp::SSub => {
            if let ImmNum(num) = operand {
                Some(ImmNum(num - 1))
            } else if let ImmFlot(flot) = operand {
                Some(ImmFlot(flot - DBig::from(1)))
            } else {
                None
            }
        }
        ExprOp::Neg => {
            if let ImmNum(num) = operand {
                Some(ImmNum(-num))
            } else if let ImmFlot(flot) = operand {
                Some(ImmFlot(-flot))
            } else {
                None
            }
        }
        _ => None,
    }
}


pub fn expr_optimizer(left: &Operand, right: &Operand, op: ExprOp) -> Option<Operand> {
    match (left, right, op) {
        (ImmNum(a), ImmNum(b), ExprOp::Add) => Some(ImmNum(a + b)),
        (ImmNum(a), ImmNum(b), ExprOp::Sub) => Some(ImmNum(a - b)),
        (ImmNum(a), ImmNum(b), ExprOp::Mul) => Some(ImmNum(a * b)),
        (ImmNum(a), ImmNum(b), ExprOp::Div) => Some(ImmNum(a / b)),
        (ImmNum(a), ImmNum(b), ExprOp::Rmd) => Some(ImmNum(a % b)),

        (ImmNum(a), ImmFlot(b), ExprOp::Add) => Some(ImmFlot(DBig::from(*a) + b)),
        (ImmNum(a), ImmFlot(b), ExprOp::Sub) => Some(ImmFlot(DBig::from(*a) - b)),
        (ImmNum(a), ImmFlot(b), ExprOp::Mul) => Some(ImmFlot(DBig::from(*a) * b)),
        (ImmNum(a), ImmFlot(b), ExprOp::Div) => {
            let context = Context::new(30);
            Some(ImmFlot(
                context.div(DBig::from(*a).repr(), b.repr()).value(),
            ))
        }
        (ImmNum(a), ImmFlot(b), ExprOp::Rmd) => {
            let context = Context::new(30);
            Some(ImmFlot(
                context.rem(DBig::from(*a).repr(), b.repr()).value(),
            ))
        }

        (ImmFlot(a), ImmNum(b), ExprOp::Add) => Some(ImmFlot(a + DBig::from(*b))),
        (ImmFlot(a), ImmNum(b), ExprOp::Sub) => Some(ImmFlot(a - DBig::from(*b))),
        (ImmFlot(a), ImmNum(b), ExprOp::Mul) => Some(ImmFlot(a * DBig::from(*b))),
        (ImmFlot(a), ImmNum(b), ExprOp::Div) => {
            let context = Context::new(30);
            Some(ImmFlot(
                context.div(a.repr(), DBig::from(*b).repr()).value(),
            ))
        }
        (ImmFlot(a), ImmNum(b), ExprOp::Rmd) => {
            let context = Context::new(30);
            Some(ImmFlot(
                context.rem(a.repr(), DBig::from(*b).repr()).value(),
            ))
        }
        // 位运算
        (ImmNum(a), ImmNum(b), ExprOp::BitAnd) => Some(ImmNum(a & b)),
        (ImmNum(a), ImmNum(b), ExprOp::BitOr) => Some(ImmNum(a | b)),
        (ImmNum(a), ImmNum(b), ExprOp::BitXor) => Some(ImmNum(a ^ b)),
        (ImmNum(a), ImmNum(b), ExprOp::BLeft) => Some(ImmNum(a << b)),
        (ImmNum(a), ImmNum(b), ExprOp::BRight) => Some(ImmNum(a >> b)),

        // 比较
        (ImmNum(a), ImmNum(b), ExprOp::Big) => Some(ImmBool(a > b)),
        (ImmNum(a), ImmNum(b), ExprOp::Less) => Some(ImmBool(a < b)),
        (ImmNum(a), ImmNum(b), ExprOp::BigEqu) => Some(ImmBool(a >= b)),
        (ImmNum(a), ImmNum(b), ExprOp::LesEqu) => Some(ImmBool(a <= b)),
        (ImmNum(a), ImmNum(b), ExprOp::Equ) => Some(ImmBool(a == b)),
        (ImmNum(a), ImmNum(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),

        (ImmFlot(a), ImmFlot(b), ExprOp::Add) => Some(ImmFlot(a + b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Sub) => Some(ImmFlot(a - b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Mul) => Some(ImmFlot(a * b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Div) => Some(ImmFlot(a / b)),

        // 浮点比较
        (ImmFlot(a), ImmFlot(b), ExprOp::Big) => Some(ImmBool(a > b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Less) => Some(ImmBool(a < b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::BigEqu) => Some(ImmBool(a >= b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::LesEqu) => Some(ImmBool(a <= b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Equ) => Some(ImmBool(a == b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),
        (ImmBool(a), ImmBool(b), ExprOp::And) => Some(ImmBool(*a && *b)),
        (ImmBool(a), ImmBool(b), ExprOp::Or) => Some(ImmBool(*a || *b)),
        (ImmBool(a), ImmBool(b), ExprOp::Equ) => Some(ImmBool(a == b)),
        (ImmBool(a), ImmBool(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),

        (ImmStr(a), ImmStr(b), ExprOp::Equ) => Some(ImmBool(a == b)),
        (ImmStr(a), ImmStr(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),

        (Reference(str) | Library(str), Reference(str1) | Library(str1), ExprOp::Ref) => {
            let mut ref_build = SmolStrBuilder::new();
            ref_build.push_str(str1.as_str());
            ref_build.push('/');
            ref_build.push_str(str.as_str());
            Some(Reference(ref_build.finish()))
        }
        // 赋值 / 复合赋值 / 引用 / 下标 —— 不做常量折叠
        _ => None,
    }
}
