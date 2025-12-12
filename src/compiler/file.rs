use crate::compiler::ast::vm_ir::{ssa_to_vm, VMIRTable};
use crate::compiler::lexer::{LexerAnalysis, LexerError, Token};
use crate::compiler::lints::Lint;
use crate::compiler::parser::symbol_table::SymbolTable;
use crate::compiler::parser::ParserError::LexError;
use crate::compiler::parser::{Parser, ParserError};
use crate::compiler::semantic::Semantic;
use crate::compiler::{Compiler, CompilerData};
use std::collections::HashSet;

#[derive(Debug,Clone)]
#[allow(dead_code)] // TODO
pub struct SourceFile {
    pub name: String,
    data: String,
    pub lexer: LexerAnalysis,
    pub(crate) c_data: CompilerData,
    ir_table: VMIRTable,
}

impl SourceFile {
    pub fn new(name: String, data: String, lints: HashSet<Lint>) -> SourceFile {
        let data0 = data.clone();

        SourceFile {
            name,
            data,
            lexer: LexerAnalysis::new(data0),
            c_data: CompilerData {
                symbol_table: SymbolTable::new(),
                lints,
            },
            ir_table: VMIRTable::new(),
        }
    }

    pub fn peek_token(&mut self) -> Result<Token, ParserError> {
        match self.lexer.next_token() {
            Ok(lexeme) => Ok(lexeme),
            Err(err) => match err {
                LexerError::Eof => Err(ParserError::Eof),
                err => Err(LexError(err)),
            },
        }
    }

    pub fn has_warnings(&mut self, lint: Lint) -> bool {
        self.c_data.lints.contains(&lint)
    }

    pub fn get_data(&self) -> &str {
        &self.data
    }

    pub fn compiler(&mut self,compiler: &mut Compiler) -> Result<(), ParserError> {
        let parser = Parser::new(self);
        let ast_tree = parser.parser()?;
        let mut semantic = Semantic::new(self,compiler);
        let ssa_ir = semantic.semantic(ast_tree)?;
        let vm_ir = ssa_to_vm(ssa_ir.0,ssa_ir.1);
        dbg!(vm_ir);
        Ok(())
    }
}
