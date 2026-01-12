mod block;
mod expression;
mod function;
mod judgment;
mod loop_back;
mod optimizer;
mod var;
mod r#while;

use crate::compiler::ast::ssa_ir::{Code, LocalMap, OpCode, ValueAlloc, ValueGuessType};
use crate::compiler::ast::ASTStmtTree;
use crate::compiler::file::SourceFile;
use crate::compiler::lints::Lint::UnusedExpression;
use crate::compiler::parser::symbol_table::ElementType;
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::expression::{check_expr_operand, expr_semantic};
use crate::compiler::semantic::function::{function_semantic, native_function_semantic};
use crate::compiler::semantic::judgment::judgment_semantic;
use crate::compiler::semantic::r#while::while_semantic;
use crate::compiler::semantic::var::{array_semantic, var_semantic};
use crate::compiler::{Compiler, CompilerData};
use crate::compiler::semantic::block::block_semantic;

pub struct Semantic<'a> {
    file: &'a mut SourceFile,
    compiler: &'a mut Compiler,
}

impl<'a> Semantic<'a> {
    pub const fn new(file: &'a mut SourceFile, compiler: &'a mut Compiler) -> Self {
        Self { file, compiler }
    }

    pub const fn compiler_data(&mut self) -> &mut CompilerData {
        &mut self.file.c_data
    }

    pub fn semantic(&mut self, stmt_tree: ASTStmtTree) -> Result<(Code, LocalMap), ParserError> {
        let code = &mut Code::new(true);
        let mut global = LocalMap::new();
        let value_alloc = &mut ValueAlloc::new();

        if let ASTStmtTree::Root(stmts) = stmt_tree {
            for stmt in stmts {
                match stmt {
                    ASTStmtTree::Var { name, value } => {
                        let opcode =
                            var_semantic(self, name, value, value_alloc, true, &mut global)?;
                        code.get_code_table().append_code(&opcode);
                    }
                    ASTStmtTree::Expr(expr) => {
                        let ref_expr = expr.clone();
                        let ret_m = expr_semantic(self, Some(expr), value_alloc)?;
                        code.get_code_table().append_code(&ret_m.2);
                        if !check_expr_operand(&ret_m.0, &OpCode::Store(None), 0) {
                            Compiler::warning_info_expr(
                                self.file,
                                "expression result is unused.",
                                &ref_expr,
                                UnusedExpression,
                            );
                        }
                    }
                    ASTStmtTree::Import(token,use_name, imp_name) => {
                        let lib_name = imp_name.clone();
                        if self.compiler.find_file(lib_name.as_str()).is_none() {
                            return Err(ParserError::NotFoundLibrary(token));
                        }
                        let name = use_name;
                        self.compiler_data()
                            .symbol_table
                            .add_element(name, ElementType::Library(imp_name));
                        value_alloc.alloc_value(token, ValueGuessType::Ref);
                    }
                    ASTStmtTree::Loop {
                        token: _token,
                        cond,
                        body,
                        is_easy,
                    } => {
                        let ret_m =
                            while_semantic(self, &cond, body, value_alloc, &mut global, is_easy)?;
                        code.get_code_table().append_code(&ret_m);
                    }
                    ASTStmtTree::Function { name, args, body } => {
                        function_semantic(self, name, args, body, code, value_alloc)?;
                    }
                    ASTStmtTree::NativeFunction { name, args } => {
                        native_function_semantic(self, name, &args, code)?;
                    }
                    ASTStmtTree::If {
                        cond,
                        then_body,
                        else_body,
                    } => {
                        let ret_m = judgment_semantic(
                            self,
                            &cond,
                            then_body,
                            else_body,
                            value_alloc,
                            &mut global,
                        )?;
                        code.get_code_table().append_code(&ret_m);
                    }
                    ASTStmtTree::Array { token, elements } => {
                        let ret_m =
                            array_semantic(self, token, elements, value_alloc, &mut global, true)?;
                        code.get_code_table().append_code(&ret_m);
                    }
                    ASTStmtTree::Context(stmts) => {
                        let ret_m = block_semantic(self, stmts, value_alloc, &mut global)?;
                        code.get_code_table().append_code(&ret_m);
                    }
                    _ => todo!(),
                }
            }
        } else {
            unreachable!()
        }
        Ok((code.clone(), global))
    }
}
