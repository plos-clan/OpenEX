use crate::compiler::ast::ssa_ir::OpCode::Push;
use crate::compiler::ast::ssa_ir::{LocalMap, OpCode, OpCodeTable, Operand, ValueAlloc};
use crate::compiler::ast::ASTStmtTree;
use crate::compiler::lints::Lint::UnusedExpression;
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::expression::{check_expr_operand, expr_semantic, lower_expr};
use crate::compiler::semantic::judgment::judgment_semantic;
use crate::compiler::semantic::loop_back::loop_back_semantic;
use crate::compiler::semantic::r#while::while_semantic;
use crate::compiler::semantic::var::{array_semantic, var_semantic};
use crate::compiler::semantic::Semantic;
use crate::compiler::Compiler;

pub fn block_semantic(
    semantic: &mut Semantic,
    stmt_tree: Vec<ASTStmtTree>,
    code: &mut ValueAlloc,
    locals: &mut LocalMap,
) -> Result<OpCodeTable, ParserError> {
    let mut opcodes = OpCodeTable::new();
    for stmt in stmt_tree {
        match stmt {
            ASTStmtTree::Root(_) => {
                unreachable!()
            }
            ASTStmtTree::Block(stmts) => {
                opcodes.append_code(&block_semantic(semantic, stmts, code,locals)?);
            }
            ASTStmtTree::Var { name, value } => {
                let opcode = var_semantic(semantic, name, value, code, false,locals)?;
                opcodes.append_code(&opcode);
            }
            ASTStmtTree::Array { token, elements } => {
                let ret_m =
                    array_semantic(semantic, token, elements, code, locals, false)?;
                opcodes.append_code(&ret_m);
            }
            ASTStmtTree::Expr(expr) => {
                let ref_expr = expr.clone();
                let ret_m = expr_semantic(semantic, Some(expr), code)?;
                if !check_expr_operand(&ret_m.0, &OpCode::Store(None), 0) {
                    Compiler::warning_info_expr(
                        semantic.file,
                        "expression result is unused.",
                        &ref_expr,
                        UnusedExpression,
                    );
                }
                opcodes.append_code(&ret_m.2);
            }
            ASTStmtTree::If {cond,then_body,else_body} => {
                let ret_m = judgment_semantic(semantic, &cond, then_body, else_body, code, locals)?;
                opcodes.append_code(&ret_m);
            }
            ASTStmtTree::Loop {
                token: _token,
                cond,
                body,
                is_easy,
            } => {
                let ret_m = while_semantic(semantic, &cond, body, code, locals, is_easy)?;
                opcodes.append_code(&ret_m);
            }
            ASTStmtTree::Break(token) =>{
                let ret_m = loop_back_semantic(semantic, true, token)?;
                opcodes.append_code(&ret_m);
            },
            ASTStmtTree::Continue(token) =>{
                let ret_m = loop_back_semantic(semantic, false, token)?;
                opcodes.append_code(&ret_m);
            },
            ASTStmtTree::Return(expr) => {
                if let Some(expr) = expr {
                    let ref_expr = lower_expr(semantic, &expr, code, None)?;
                    opcodes.append_code(&ref_expr.2);
                    opcodes.add_opcode(OpCode::Return(None));
                }else {
                    opcodes.add_opcode(Push(None,Operand::Null));
                    opcodes.add_opcode(OpCode::Return(None));
                }
            }
            _ => todo!(),
        }
    }
    Ok(opcodes)
}
