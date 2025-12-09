mod expression;
mod optimizer;
mod var;

use crate::compiler::ast::ssa_ir::{Code, OpCode};
use crate::compiler::ast::ASTStmtTree;
use crate::compiler::file::SourceFile;
use crate::compiler::lints::Lint::UnusedExpression;
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::expression::{check_expr_operand, expr_semantic};
use crate::compiler::semantic::var::var_semantic;
use crate::compiler::{Compiler, CompilerData};

pub struct Semantic<'a> {
    file: &'a mut SourceFile,
}

impl<'a> Semantic<'a> {
    pub fn new(file: &'a mut SourceFile) -> Semantic<'a> {
        Self { file }
    }

    pub fn compiler_data(&mut self) -> &mut CompilerData {
        &mut self.file.c_data
    }

    pub fn semantic(&mut self, stmt_tree: ASTStmtTree) -> Result<Code, ParserError> {
        let code = &mut Code::new(true);
        match stmt_tree {
            ASTStmtTree::Root(stmts) => {
                for stmt in stmts {
                    match stmt {
                        ASTStmtTree::Var { name, value } => {
                            let opcode = var_semantic(self, name, value, code)?;
                            code.add_opcode(opcode);
                        }
                        ASTStmtTree::Expr(expr) => {
                            let ref_expr = expr.clone();
                            let ret_m = expr_semantic(self,Some(expr), code)?;
                            if !check_expr_operand(&ret_m.0, &OpCode::Store) {
                                Compiler::warning_info_expr(
                                    self.file,
                                    "expression result is unused.",
                                    &ref_expr,
                                    UnusedExpression,
                                );
                            }
                        }
                        _ => todo!(),
                    }
                }
            }
            _ => todo!(),
        }
        Ok(code.clone())
    }
}
