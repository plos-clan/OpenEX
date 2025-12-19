use crate::compiler::ast::vm_ir::{ssa_to_vm, VMIRTable};
use crate::compiler::lexer::{LexerAnalysis, LexerError, Token};
use crate::compiler::lints::Lint;
use crate::compiler::parser::symbol_table::SymbolTable;
use crate::compiler::parser::ParserError::LexError;
use crate::compiler::parser::{Parser, ParserError};
use crate::compiler::semantic::Semantic;
use crate::compiler::{Compiler, CompilerData};
use std::collections::HashSet;
use smol_str::ToSmolStr;

#[derive(Debug,Clone)]
#[allow(dead_code)] // TODO
pub struct SourceFile {
    pub name: String,
    data: String,
    pub(crate) compiled: bool,
    pub(crate) is_library: bool,
    pub lexer: LexerAnalysis,
    pub(crate) c_data: CompilerData,
    pub(crate) ir_table: Option<Box<VMIRTable>>,
}

impl SourceFile {
    pub fn new(name: String, data: String, lints: HashSet<Lint>, is_library: bool) -> Self {
        let data0 = data.clone();

        Self {
            name,
            data,
            lexer: LexerAnalysis::new(data0),
            c_data: CompilerData {
                symbol_table: SymbolTable::new(),
                lints,
            },
            ir_table: None,
            is_library,
            compiled: false,
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

    pub fn has_warnings(&self, lint: Lint) -> bool {
        self.c_data.lints.contains(&lint)
    }

    pub fn get_data(&self) -> &str {
        &self.data
    }

    pub fn compiler(&mut self,compiler: &mut Compiler) -> Result<VMIRTable, ParserError> {

        let parser = Parser::new(self);
        let ast_tree = parser.parser()?;
        let mut semantic = Semantic::new(self,compiler);
        let ssa_ir = semantic.semantic(ast_tree)?;
        let vm_ir = ssa_to_vm(ssa_ir.0, &ssa_ir.1, &self.name.to_smolstr());
        Ok(vm_ir)

    }
}
