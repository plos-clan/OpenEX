use crate::compiler::ast::ASTStmtTree;
use crate::compiler::file::SourceFile;
use crate::compiler::lexer::{LexerError, Token};
use crate::compiler::parser::symbol_table::SymbolTable;
use crate::compiler::parser::ParserError;

pub mod file;
pub mod lexer;
mod parser;
mod ast;
mod semantic;

#[derive(PartialEq)]
pub enum CompileStatus {
    Source,
    Parse,
    IR,
    Execute,
    Error,
}

pub struct CompilerData {
    symbol_table: SymbolTable,
    astree: Option<ASTStmtTree>,
    tokens: Vec<Token>,
    status: CompileStatus,
}

pub struct Compiler {
    files: Vec<SourceFile>,
    version: String,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler { files: vec![], version: "OpenEX RustEdition v0.0.1".to_string() }
    }

    pub fn get_version(&self) -> &String {
        &self.version
    }

    pub fn add_file(&mut self, file: SourceFile) {
        self.files.push(file);
    }

    fn highlight_line_and_column(data: &str, target_line: usize, target_column: usize) -> String {
        let target_line_content = data.lines().nth(target_line);
        let line_content = match target_line_content {
            Some(s) => s,
            None => return format!("error: unknown line {}", target_line),
        };
        let line_number_str = format!("{}", target_line + 1);
        let line_prefix = format!("{:<4} | ", line_number_str);
        let mut output = format!("{}{}\n", line_prefix, line_content);
        let indicator_prefix_len = line_prefix.len();

        let mut padding = line_content
            .chars()
            .take(target_column)
            .map(|_c| 1)
            .sum::<usize>();
        let indicator_spaces = " ".repeat(indicator_prefix_len);
        padding = if padding > 0 { padding - 1 } else { padding };
        let column_padding = " ".repeat(padding);

        output.push_str(&format!("{}{}^\n", indicator_spaces, column_padding));

        output
    }

    fn dump_error_info(message: String, line: usize, column: usize, file: &SourceFile) {
        eprintln!(
            "SyntaxError({}-line: {} column: {}): {}",
            file.name,
            line + 1,
            column,
            message
        );
        eprintln!(
            "{}",
            Self::highlight_line_and_column(file.get_data(), line, column)
        );
    }

    fn dump_lexer_error(lex_error: LexerError, file: &SourceFile) {
        let message;
        match lex_error {
            LexerError::UnexpectedCharacter(_c, msg) => {
                message = msg.to_string();
            }
            LexerError::IllegalLiteral(msg) => {
                message = msg.to_string();
            }
            _ => {
                message = String::new();
            }
        }
        Self::dump_error_info(
            message,
            file.lexer.get_now_line(),
            file.lexer.get_now_column(),
            file,
        );
    }

    fn dump_parser_error(error: ParserError, file: &SourceFile) {
        let line:usize;
        let column:usize;
        let message:String;

        match error {
            ParserError::LexError(lex_error) => {
                Self::dump_lexer_error(lex_error, file);
                return;
            }
            ParserError::EOF => {
                return;
            }
            ParserError::IdentifierExpected(token) => {
                line = token.line;
                column = token.column;
                message = "<identifier> expected.".parse().unwrap();
            }
            ParserError::NotAStatement(token) => {
                line = token.line;
                column = token.column;
                message = "<statement> expected.".parse().unwrap();
            }
            ParserError::Expected(token,c) => {
                line = token.line;
                column = token.column;
                message = format!("'{}' expected.",c);
            }
            ParserError::MissingStatement(token) => {
                line = token.line;
                column = token.column;
                message = String::from("statement is incomplete.");
            }
            ParserError::IllegalArgument(token) => {
                line = token.line;
                column = token.column;
                message = String::from("illegal argument.");
            }
            ParserError::MissingFunctionBody(token) => {
                line = token.line;
                column = token.column;
                message = String::from("missing function body.");
            }
            ParserError::MissingCondition(token) => {
                line = token.line;
                column = token.column;
                message = String::from("missing condition.");
            }
            ParserError::IllegalExpression(token) => {
                line = token.line;
                column = token.column;
                message = String::from("illegal combination of expressions.");
            }
            ParserError::IllegalKey(key) => {
                line = key.line;
                column = key.column;
                message = String::from("illegal key.");
            }
            ParserError::BackOutsideLoop(token) => {
                line = token.line;
                column = token.column;
                message = String::from("back statement outside loop.");
            }
            ParserError::SymbolDefined(name) => {
                line = name.line;
                column = name.column;
                message = String::from("type already defined."); // 沿用 Pro 版本的彩蛋
            }
            ParserError::IllegalTypeCombination(token) => {
                line = token.line;
                column = token.column;
                message = String::from("illegal type combination.");
            }
        }

        Self::dump_error_info(
            message,
            line,
            column,
            file,
        );
    }

    pub fn compile(&mut self) {
        for file in &mut self.files {
            file.compiler().unwrap_or_else(|error| {
                Self::dump_parser_error(error,file);
                
            })
        }
    }
}
