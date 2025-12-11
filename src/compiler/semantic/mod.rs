mod block;
mod expression;
mod function;
mod optimizer;
mod var;
mod r#while;

use crate::compiler::ast::ssa_ir::{Code, OpCode, ValueGuessType};
use crate::compiler::ast::ASTStmtTree;
use crate::compiler::file::SourceFile;
use crate::compiler::lints::Lint::UnusedExpression;
use crate::compiler::parser::symbol_table::ElementType;
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::expression::{check_expr_operand, expr_semantic};
use crate::compiler::semantic::r#while::while_semantic;
use crate::compiler::semantic::var::var_semantic;
use crate::compiler::{Compiler, CompilerData};
use smol_str::SmolStr;
use crate::compiler::semantic::function::{function_semantic, native_function_semantic};

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
        if let ASTStmtTree::Root(stmts) = stmt_tree {
            for stmt in stmts {
                match stmt {
                    ASTStmtTree::Var { name, value } => {
                        let mut opcode = var_semantic(self, name, value, code, true)?;
                        code.get_code_table().append_code(&mut opcode);
                    }
                    ASTStmtTree::Expr(expr) => {
                        let ref_expr = expr.clone();
                        let mut ret_m = expr_semantic(self, Some(expr), code)?;
                        code.get_code_table().append_code(&mut ret_m.2);
                        if !check_expr_operand(&ret_m.0, &OpCode::Store(None), 0) {
                            Compiler::warning_info_expr(
                                self.file,
                                "expression result is unused.",
                                &ref_expr,
                                UnusedExpression,
                            );
                        }
                    }
                    ASTStmtTree::Import(token) => {
                        //TODO 检查库是否存在
                        let name = token.clone().value::<SmolStr>().unwrap();
                        self.compiler_data()
                            .symbol_table
                            .add_element(name, ElementType::Library);
                        code.alloc_value(token, ValueGuessType::Library);
                    }
                    ASTStmtTree::Loop {
                        token: _token,
                        cond,
                        body,
                    } => {
                        let mut ret_m = while_semantic(self, cond, body, code)?;
                        code.get_code_table().append_code(&mut ret_m);
                    }
                    ASTStmtTree::Function { name, args, body } => {
                        function_semantic(self, name, args,body,code)?;
                    }
                    ASTStmtTree::NativeFunction { name, args } => {
                        native_function_semantic(self, name, args,code)?;
                    }
                    _ => todo!(),
                }
            }
        } else {
            unreachable!()
        }
        Ok(code.clone())
    }
}
