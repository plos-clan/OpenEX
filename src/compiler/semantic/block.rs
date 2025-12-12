use crate::compiler::ast::ssa_ir::{Code, LocalMap, OpCode, OpCodeTable};
use crate::compiler::ast::ASTStmtTree;
use crate::compiler::lints::Lint::UnusedExpression;
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::expression::{check_expr_operand, expr_semantic};
use crate::compiler::semantic::var::var_semantic;
use crate::compiler::semantic::Semantic;
use crate::compiler::Compiler;

pub fn block_semantic(
    semantic: &mut Semantic,
    stmt_tree: Vec<ASTStmtTree>,
    code: &mut Code,
    locals: &mut LocalMap,
) -> Result<OpCodeTable, ParserError> {
    let mut opcodes = OpCodeTable::new();
    for stmt in stmt_tree {
        match stmt {
            ASTStmtTree::Root(_) => {
                unreachable!()
            }
            ASTStmtTree::Block(stmts) => {
                opcodes.append_code(&mut block_semantic(semantic, stmts, code,locals)?);
            }
            ASTStmtTree::Var { name, value } => {
                let mut opcode = var_semantic(semantic, name, value, code, false,locals)?;
                opcodes.append_code(&mut opcode);
            }
            ASTStmtTree::Expr(expr) => {
                let ref_expr = expr.clone();
                let mut ret_m = expr_semantic(semantic, Some(expr), code)?;
                if !check_expr_operand(&ret_m.0, &OpCode::Store(None), 0) {
                    Compiler::warning_info_expr(
                        semantic.file,
                        "expression result is unused.",
                        &ref_expr,
                        UnusedExpression,
                    );
                }
                opcodes.append_code(&mut ret_m.2);
            }
            _ => todo!(),
        }
    }
    Ok(opcodes)
}
