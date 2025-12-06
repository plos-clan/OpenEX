use crate::compiler::lexer::{LexerAnalysis, LexerError, Token};
use crate::compiler::parser::symbol_table::SymbolTable;
use crate::compiler::parser::{Parser, ParserError};
use crate::compiler::{CompileStatus, CompilerData};
use crate::compiler::parser::ParserError::LexError;

pub struct SourceFile {
    pub name: String,
    data: String,
    pub lexer: LexerAnalysis,
    pub(crate) c_data: CompilerData,
}

impl SourceFile {
    pub fn new(name: String, data: String) -> SourceFile {
        let data0 = data.clone();

        SourceFile {
            name,
            data,
            lexer: LexerAnalysis::new(data0),
            c_data: CompilerData {
                symbol_table: SymbolTable::new(),
                astree: None,
                tokens: vec![],
                status: CompileStatus::Source,
            }
        }
    }

    pub fn peek_token(&mut self) -> Result<Token, ParserError> {
        match self.lexer.next_token() {
            Ok(lexeme) => Ok(lexeme),
            Err(err) => match err {
                LexerError::EOF => Err(ParserError::EOF),
                err => Err(LexError(err)),
            },
        }
    }

    pub fn get_data(&self) -> &str {
        &self.data
    }

    pub fn compiler(&mut self) -> Result<(), ParserError> {
        let parser = Parser::new(self);
        let ast_tree = parser.parser()?;
        dbg!(ast_tree);
        Ok(())
    }
}
