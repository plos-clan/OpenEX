use crate::compiler::ast::ssa_ir::OpCode::Push;
use crate::compiler::ast::ssa_ir::Operand::ImmNumFlot;
use crate::compiler::ast::ssa_ir::ValueGuessType::{Bool, Float, Null, Number, Ref, String, This, Unknown};
use crate::compiler::ast::ssa_ir::{Code, OpCode, OpCodeTable, Operand, ValueGuessType};
use crate::compiler::ast::{ASTExprTree, ExprOp};
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::optimizer::{expr_optimizer, unary_optimizer};
use crate::compiler::semantic::Semantic;
use smol_str::{SmolStr, SmolStrBuilder, ToSmolStr};

macro_rules! check_bool_expr {
    ($op:expr,$second:expr) => {
        if check_opts(
            $op,
            &[
                ExprOp::Equ,
                ExprOp::NotEqu,
                ExprOp::BigEqu,
                ExprOp::LesEqu,
                ExprOp::Less,
                ExprOp::Big,
            ],
        ) {
            Ok(Bool)
        } else {
            Ok($second)
        }
    };
}

fn astop_to_opcode(astop: &ExprOp) -> OpCode {
    match astop {
        ExprOp::And => OpCode::And(None),
        ExprOp::Or => OpCode::Or(None),
        ExprOp::Not => OpCode::Not(None),
        ExprOp::BLeft => OpCode::BLeft(None),
        ExprOp::BRight => OpCode::BRight(None),
        ExprOp::BitXor => OpCode::BitXor(None),
        ExprOp::BitAnd => OpCode::BitAnd(None),
        ExprOp::BitOr => OpCode::BitOr(None),
        ExprOp::Sub => OpCode::Sub(None),
        ExprOp::Add => OpCode::Add(None),
        ExprOp::Mul => OpCode::Mul(None),
        ExprOp::Div => OpCode::Div(None),
        ExprOp::RmdS => OpCode::RmdS(None),
        ExprOp::AddS => OpCode::AddS(None),
        ExprOp::SubS => OpCode::SubS(None),
        ExprOp::MulS => OpCode::MulS(None),
        ExprOp::DivS => OpCode::DivS(None),
        ExprOp::Ref => OpCode::Ref(None),
        ExprOp::SAdd => OpCode::SAdd(None),
        ExprOp::SSub => OpCode::SSub(None),
        ExprOp::Store => OpCode::Store(None),
        ExprOp::BigEqu => OpCode::BigEqu(None),
        ExprOp::LesEqu => OpCode::LesEqu(None),
        ExprOp::Less => OpCode::Less(None),
        ExprOp::Big => OpCode::Big(None),
        ExprOp::Equ => OpCode::Equ(None),
        ExprOp::NotEqu => OpCode::NotEqu(None),
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

fn check_opts(op: &ExprOp, args: &[ExprOp]) -> bool {
    if matches!(op, ExprOp::Store | ExprOp::Equ | ExprOp::NotEqu) {
        return true;
    }
    for ty in args {
        if op == ty {
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
    op: &ExprOp,
) -> Result<ValueGuessType, ParserError> {
    if first == Unknown || second == Unknown {
        return Ok(Unknown);
    }

    if matches!(op, ExprOp::Store) {
        return Ok(second);
    }

    match first {
        Bool => {
            if guess_check_type(second, &[Bool])
                && check_opts(
                    op,
                    &[
                        ExprOp::Not,
                        ExprOp::And,
                        ExprOp::Or,
                        ExprOp::Equ,
                        ExprOp::NotEqu,
                    ],
                )
            {
                Ok(Bool)
            } else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        String => {
            if guess_check_type(second, &[String, Float, Number, Null])
                && check_opts(op, &[ExprOp::Add])
            {
                if check_opts(op, &[]) {
                    Ok(Bool)
                } else {
                    Ok(String)
                }
            } else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        Number => {
            if guess_check_type(second.clone(), &[Number, Float])
                && check_opts(
                    op,
                    &[
                        ExprOp::Add,
                        ExprOp::Sub,
                        ExprOp::Mul,
                        ExprOp::Div,
                        ExprOp::SAdd,
                        ExprOp::SSub,
                        ExprOp::BigEqu,
                        ExprOp::Big,
                        ExprOp::LesEqu,
                        ExprOp::Less,
                        ExprOp::Equ,
                        ExprOp::NotEqu,
                        ExprOp::AddS,
                        ExprOp::SubS,
                        ExprOp::MulS,
                        ExprOp::DivS,
                        ExprOp::Rmd,
                        ExprOp::RmdS,
                        ExprOp::BitOr,
                        ExprOp::BitAnd,
                        ExprOp::BitXor,
                        ExprOp::BLeft,
                        ExprOp::BRight,
                        ExprOp::BitOr,
                        ExprOp::BitOr,
                        ExprOp::BOrS,
                        ExprOp::BAndS,
                        ExprOp::BXorS,
                    ],
                )
            {
                check_bool_expr!(op, second)
            } else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        Float => {
            if guess_check_type(second, &[Float, Number])
                && check_opts(
                    op,
                    &[
                        ExprOp::Add,
                        ExprOp::Sub,
                        ExprOp::Mul,
                        ExprOp::Div,
                        ExprOp::SAdd,
                        ExprOp::SSub,
                        ExprOp::BigEqu,
                        ExprOp::Big,
                        ExprOp::LesEqu,
                        ExprOp::Less,
                        ExprOp::Equ,
                        ExprOp::NotEqu,
                        ExprOp::AddS,
                        ExprOp::SubS,
                        ExprOp::MulS,
                        ExprOp::DivS,
                    ],
                )
            {
                check_bool_expr!(op, Float)
            } else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        Null => {
            if guess_check_type(second, &[Null]) && check_opts(op, &[ExprOp::Equ, ExprOp::NotEqu]) {
                Ok(Bool)
            } else {
                Err(ParserError::IllegalTypeCombination(token.clone()))
            }
        }
        _ => Err(ParserError::IllegalTypeCombination(token.clone())),
    }
}

fn lower_ref(
    semantic: &mut Semantic,
    expr_tree: &ASTExprTree,
    code: &mut Code,
) -> Result<(SmolStr, OpCodeTable), ParserError> {
    let mut opcode_table = OpCodeTable::new();
    let mut path = SmolStrBuilder::new();

    if let ASTExprTree::Expr {
        token:_,
        op:_op,
        left,
        right,
    } = expr_tree
    {
        let left_tree = left.as_ref();
        let right_tree = right.as_ref();

        if matches!(left_tree, ASTExprTree::Call { .. }) || matches!(left_tree, ASTExprTree::This(_token)) ||
            matches!(left_tree, ASTExprTree::Var(_token)) {
            let mut table = lower_expr(semantic, left_tree, code, None)?.2;
            opcode_table.append_code(&mut table);
        }else {
            unreachable!()
        }

        if let ASTExprTree::Var(token) | ASTExprTree::This(token) = right_tree{
            path.push_str(format!("/{}", token.text()).as_str());
            let code = match right_tree {
                ASTExprTree::Var(token) => {
                    Push(None,Operand::Reference(token.text().to_smolstr()))
                },
                ASTExprTree::This(_token) => {
                    Push(None,Operand::This)
                },
                _ => unreachable!()
            };
            opcode_table.add_opcode(code);
        }else {
            unreachable!()
        }
        opcode_table.add_opcode(OpCode::Ref(None));
    }else {
        unreachable!()
    }

    Ok((path.finish(), opcode_table))
}

fn operand_to_guess(operand: Operand) -> ValueGuessType {
    match operand {
        Operand::ImmBool(_) => Bool,
        Operand::Null => Null,
        Operand::This => This,
        Operand::ImmNum(_) => Number,
        Operand::ImmFlot(_) => Float,
        Operand::ImmStr(_) => String,
        Operand::Reference(_) => Ref,
        _=> Unknown,
    }
}

pub(crate) fn lower_expr(
    semantic: &mut Semantic,
    expr_tree: &ASTExprTree,
    code: &mut Code,
    store: Option<Operand>,
) -> Result<(Operand, ValueGuessType, OpCodeTable), ParserError> {
    let mut opcode_table = OpCodeTable::new();
    match expr_tree {
        ASTExprTree::Literal(lit) => {
            let tk_lit = &mut lit.clone();
            match lit.t_type {
                TokenType::Number => {
                    let operand = Operand::ImmNum(tk_lit.value_number());
                    opcode_table.add_opcode(Push(None, operand.clone()));
                    Ok((operand, Number, opcode_table))
                }
                TokenType::Float => {
                    let operand = Operand::ImmFlot(tk_lit.value_float());
                    opcode_table.add_opcode(Push(None, operand.clone()));
                    Ok((operand, Float, opcode_table))
                }
                TokenType::LiteralString => {
                    let operand = Operand::ImmStr(tk_lit.value::<SmolStr>().unwrap());
                    opcode_table.add_opcode(Push(None, operand.clone()));
                    Ok((operand, String, opcode_table))
                }
                TokenType::True | TokenType::False => {
                    let operand = Operand::ImmBool(lit.t_type == TokenType::True);
                    opcode_table.add_opcode(Push(None, operand.clone()));
                    Ok((operand, Bool, opcode_table))
                }
                TokenType::Null => {
                    opcode_table.add_opcode(Push(None, Operand::Null));
                    Ok((Operand::Null, Null, opcode_table))
                }
                _ => {
                    unreachable!()
                }
            }
        }
        ASTExprTree::Ref(token) => {
            opcode_table.add_opcode(Push(None, Operand::Reference(token.text().to_smolstr())));
            Ok((Operand::Reference(token.text().to_smolstr()), Ref, opcode_table))
        }
        ASTExprTree::This(_token) => {
            opcode_table.add_opcode(Push(None, Operand::This));
            Ok((Operand::This, This, opcode_table))
        }
        ASTExprTree::Unary {
            token: u_token,
            op: u_op,
            code: u_code,
        } => {
            let mut load = lower_expr(semantic, u_code.as_ref(), code,None)?;
            let mut store = lower_expr(semantic, u_code.as_ref(), code,Some(ImmNumFlot))?;
            let g_type = guess_type_unary(u_token, store.1, u_op)?;
            if let Some(operand) = unary_optimizer(u_op, &store.0) {
                opcode_table.add_opcode(Push(None, operand));
            } else {
                opcode_table.append_code(&mut load.2);
                opcode_table.add_opcode(astop_to_opcode(u_op));
                opcode_table.append_code(&mut store.2);
            }
            Ok((store.0, g_type, opcode_table))
        }
        ASTExprTree::Expr {
            token: e_token,
            op: e_op,
            left: e_left,
            right: e_right,
        } => {
            let mut right = lower_expr(semantic, e_right.as_ref(), code, None)?;
            let right_opd = Box::new(right.0.clone());
            let stores = if matches!(e_op,ExprOp::Store) {
                Some(right.0.clone())
            }else {
                None
            };
            let mut left = lower_expr(semantic, e_left.as_ref(), code, stores)?;

            let left_opd = Box::new(left.0.clone());
            let guess_type = guess_type(e_token, left.1, right.1, e_op)?;
            let n_operand;

            if let Some(operand) = expr_optimizer(&left.0, &right.0, e_op) {
                n_operand = operand.clone();
                opcode_table.add_opcode(Push(None, operand));
            } else {
                let opcode = astop_to_opcode(e_op);
                if matches!(e_op,ExprOp::Store) {
                    opcode_table.append_code(&mut right.2);
                    opcode_table.append_code(&mut left.2);
                }else {
                    opcode_table.append_code(&mut left.2);
                    opcode_table.append_code(&mut right.2);
                    opcode_table.add_opcode(opcode.clone());
                }
                n_operand = Operand::Expression(left_opd, right_opd, Box::from(opcode));
            }
            Ok((n_operand, guess_type, opcode_table))
        }
        ASTExprTree::Var(u_token) => {
            let var_name = u_token.clone().value::<SmolStr>().unwrap();
            if !semantic
                .compiler_data()
                .symbol_table
                .check_element(var_name.clone())
            {
                return Err(ParserError::UnableResolveSymbols(u_token.clone()));
            }
            if let Some(key) = code.find_value_key(var_name.clone()) {
                let value = code.find_value(key).unwrap();
                value.variable = true;
                let type_ = value.type_.clone();
                match value.type_ {
                    Ref => {
                        opcode_table.add_opcode(Push(None, Operand::Library(var_name)));
                    }
                    _ => {
                        if let Some(operand) = store{
                            if let ImmNumFlot = operand {
                            }else {
                                value.type_ = operand_to_guess(operand);
                            }
                            opcode_table.add_opcode(OpCode::LoadLocal(None, key, Operand::Val(key)));
                        }else {
                            opcode_table.add_opcode(OpCode::StoreLocal(None, key, Operand::Val(key)));
                        }
                    }
                };
                Ok((Operand::Val(key), type_, opcode_table))
            } else {
                unreachable!()
            }
        }
        ASTExprTree::Call { name, args } => {
            for arg in args {
                let mut expr = lower_expr(semantic, arg, code, None)?;
                opcode_table.append_code(&mut expr.2);
            }

            match name.as_ref() {
                ASTExprTree::Var(token) => {
                    let path = token.clone().value::<SmolStr>().unwrap();
                    opcode_table.add_opcode(OpCode::Call(None, path.clone()));
                    Ok((Operand::Call(path), Unknown, opcode_table))
                }
                ASTExprTree::Expr {
                    token: _token,
                    op: _op,
                    left: _left,
                    right: _right,
                } => {
                    let mut refs = lower_ref(semantic, name.as_ref(), code)?;
                    opcode_table.append_code(&mut refs.1);
                    let cl_str = refs.0.clone();
                    opcode_table.add_opcode(OpCode::Call(None,refs.0));
                    Ok((Operand::Call(cl_str), Unknown, opcode_table))
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
}

// 检查表达式是否含有指定操作码同时检查是否含有 call 操作
pub fn check_expr_operand(operand: &Operand, op_code: &OpCode, call_count: i32) -> bool {
    match operand {
        Operand::Expression(right, left, expr_op) => {
            let mut status = check_expr_operand(right.as_ref(), op_code, call_count + 1);
            status &= check_expr_operand(left.as_ref(), op_code, call_count + 1);
            if expr_op.as_ref() == op_code {
                true
            } else {
                if ((matches!(right.as_ref(), Operand::Call(_))
                    && matches!(left.as_ref(), Operand::Val(_)))
                    || (matches!(left.as_ref(), Operand::Call(_))))
                    && call_count == 0
                {
                    return true;
                }
                status
            }
        }
        Operand::Call(_) | Operand::Val(_) => true,
        _ => false,
    }
}

pub fn expr_semantic(
    semantic: &mut Semantic,
    expr: Option<ASTExprTree>,
    code: &mut Code,
) -> Result<(Operand, ValueGuessType, OpCodeTable), ParserError> {
    let guess_type;
    let operand: Operand;
    let mut opcode_vec = OpCodeTable::new();

    if let Some(expr) = expr {
        let mut exp = lower_expr(semantic, &expr, code, None)?;
        opcode_vec.append_code(&mut exp.2);
        operand = exp.0;
        guess_type = exp.1;
    } else {
        guess_type = Null;
        operand = Operand::Null;
    }

    Ok((operand, guess_type, opcode_vec))
}
