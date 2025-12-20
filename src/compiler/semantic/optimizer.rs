use crate::compiler::ast::ssa_ir::Operand;
use crate::compiler::ast::ssa_ir::Operand::{ImmBool, ImmFlot, ImmNum, ImmStr};
use crate::compiler::ast::ExprOp;

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
                Some(ImmFlot(flot + 1.0))
            } else {
                None
            }
        }
        ExprOp::SSub => {
            if let ImmNum(num) = operand {
                Some(ImmNum(num - 1))
            } else if let ImmFlot(flot) = operand {
                Some(ImmFlot(flot - 1.0))
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

// 检查整型是否符合浮点转换安全期间
fn check_not_sflnum(num: i64) -> bool {
    const MAX_SAFE_INT: i64 = (1i64 << 53) - 1;
    const MIN_SAFE_INT: i64 = -MAX_SAFE_INT;
    (MIN_SAFE_INT..=MAX_SAFE_INT).contains(&num)
}

macro_rules! float_safe_check {
    ($a:expr) => {
        if check_not_sflnum(*$a) {
            return None
        }
    };
}

pub fn expr_optimizer(left: &Operand, right: &Operand, op: ExprOp) -> Option<Operand> {
    match (left, right, op) {
        (ImmNum(a), ImmNum(b), ExprOp::Add) => Some(ImmNum(a + b)),
        (ImmNum(a), ImmNum(b), ExprOp::Sub) => Some(ImmNum(a - b)),
        (ImmNum(a), ImmNum(b), ExprOp::Mul) => Some(ImmNum(a * b)),
        (ImmNum(a), ImmNum(b), ExprOp::Div) => Some(ImmNum(a / b)),
        (ImmNum(a), ImmNum(b), ExprOp::Rmd) => Some(ImmNum(a % b)),

        (ImmNum(a), ImmFlot(b), ExprOp::Add) => {
            float_safe_check!(a);
            #[allow(clippy::cast_precision_loss)]
            Some(ImmFlot((*a as f64) + b))
        },
        (ImmNum(a), ImmFlot(b), ExprOp::Sub) => {
            float_safe_check!(a);
            #[allow(clippy::cast_precision_loss)]
            Some(ImmFlot(*a as f64 - b))},
        (ImmNum(a), ImmFlot(b), ExprOp::Mul) =>{
            float_safe_check!(a);
            #[allow(clippy::cast_precision_loss)]
            Some(ImmFlot(*a as f64 * b))
        },
        (ImmNum(a), ImmFlot(b), ExprOp::Div) =>{
            float_safe_check!(a);
            #[allow(clippy::cast_precision_loss)]
            Some(ImmFlot(*a as f64 / b))
        },
        (ImmNum(a), ImmFlot(b), ExprOp::Rmd) => {
            float_safe_check!(a);
            #[allow(clippy::cast_precision_loss)]
            Some(ImmFlot(*a as f64 % b))
        },

        (ImmFlot(a), ImmNum(b), ExprOp::Add) => {
            float_safe_check!(b);
            #[allow(clippy::cast_precision_loss)]
            Some(ImmFlot(a + *b as f64))
        },
        (ImmFlot(a), ImmNum(b), ExprOp::Sub) => {
            float_safe_check!(b);
            #[allow(clippy::cast_precision_loss)]
            Some(ImmFlot(a - *b as f64))
        },
        (ImmFlot(a), ImmNum(b), ExprOp::Mul) => {
            float_safe_check!(b);
            #[allow(clippy::cast_precision_loss)]
            Some(ImmFlot(a * *b as f64))
        },
        (ImmFlot(a), ImmNum(b), ExprOp::Div) => {
            float_safe_check!(b);
            #[allow(clippy::cast_precision_loss)]
            Some(ImmFlot(a / *b as f64))
        },
        (ImmFlot(a), ImmNum(b), ExprOp::Rmd) => {
            float_safe_check!(b);
            #[allow(clippy::cast_precision_loss)]
            Some(ImmFlot(a % *b as f64))
        },
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
        (ImmFlot(a), ImmFlot(b), ExprOp::Equ) => Some(ImmBool((a - b).abs() < f64::EPSILON)),
        (ImmFlot(a), ImmFlot(b), ExprOp::NotEqu) => {
            let result = (a - b).abs() < f64::EPSILON;
            Some(ImmBool(!result))
        },
        (ImmBool(a), ImmBool(b), ExprOp::And) => Some(ImmBool(*a && *b)),
        (ImmBool(a), ImmBool(b), ExprOp::Or) => Some(ImmBool(*a || *b)),
        (ImmBool(a), ImmBool(b), ExprOp::Equ) => Some(ImmBool(a == b)),
        (ImmBool(a), ImmBool(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),

        (ImmStr(a), ImmStr(b), ExprOp::Equ) => Some(ImmBool(a == b)),
        (ImmStr(a), ImmStr(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),

        // 赋值 / 复合赋值 / 引用 / 下标 —— 不做常量折叠
        _ => None,
    }
}
