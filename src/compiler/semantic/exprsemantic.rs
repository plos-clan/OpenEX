use smol_str::SmolStr;
use crate::compiler::ast::ASTExprTree;
use crate::compiler::ast::ssa_ir::{Operand, ValueGuessType};
use crate::compiler::lexer::TokenType;
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::Semantic;

pub fn expr_semantic(semantic: &mut Semantic, expr: Vec<ASTExprTree>) -> Result<(Operand, ValueGuessType), ParserError> {
    let mut guess_type = ValueGuessType::Unknown;
    let mut operand : Operand = Operand::Null;

    if expr.is_empty() {
        guess_type = ValueGuessType::Null;
    }else {
        for expr in expr {
            match expr {
                ASTExprTree::Literal(mut lit) => {
                    match lit.t_type {
                        TokenType::Number => {
                            guess_type = ValueGuessType::Number;
                            operand = Operand::ImmNum(lit.value_number())
                        },
                        TokenType::LiteralString => {
                            guess_type = ValueGuessType::String;
                            operand = Operand::ImmStr(lit.value::<SmolStr>().unwrap());
                        },
                        TokenType::True => {
                            guess_type = ValueGuessType::Bool;
                            operand = Operand::ImmBool(true);
                        }
                        TokenType::False => {
                            guess_type = ValueGuessType::Bool;
                            operand = Operand::ImmBool(false);
                        }
                        _ => {
                            todo!()
                        }
                    }
                }
                _ => {
                    todo!()
                }
            }
        }
    }

    Ok((operand,guess_type))
}
