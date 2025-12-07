mod var;
mod expression;
mod optimizer;

use crate::compiler::ast::ssa_ir::Code;
use crate::compiler::ast::ASTStmtTree;
use crate::compiler::CompilerData;
use crate::compiler::file::SourceFile;
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::var::var_semantic;

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
                        ASTStmtTree::Var {
                            name: v_name,
                            value: v_value,
                        } => {
                            let opcode = var_semantic(self, v_name, v_value, code)?;
                            code.add_opcode(opcode);
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
