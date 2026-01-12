use std::collections::HashSet;

use crate::compiler::ast::ASTExprTree;
use crate::compiler::file::SourceFile;
use crate::compiler::lexer::{LexerError, Token};
use crate::compiler::lints::Lint;
use crate::compiler::parser::ParserError;
use crate::compiler::parser::symbol_table::SymbolTable;

pub mod ast;
pub mod file;
pub mod lexer;
pub mod lints;
pub mod parser;
mod semantic;

#[derive(Debug, Clone)]
pub struct CompilerData {
    symbol_table: SymbolTable,
    lints: HashSet<Lint>,
}

#[derive(Debug, Clone)]
pub struct Compiler {
    files: Vec<SourceFile>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    #[must_use]
    pub const fn new() -> Self {
        Self { files: vec![] }
    }

    #[must_use]
    pub const fn get_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    pub fn add_file(&mut self, file: SourceFile) {
        self.files.push(file);
    }

    pub const fn get_files(&mut self) -> &mut Vec<SourceFile> {
        &mut self.files
    }

    #[must_use]
    /// # Panics
    pub fn find_file(&self, path: &str) -> Option<&SourceFile> {
        for file in &self.files {
            if file.name.as_str().split('.').next().unwrap() == path {
                return Some(file);
            }
        }
        None
    }

    fn highlight_line_and_column(data: &str, target_line: usize, target_column: usize) -> String {
        let target_line_content = data.lines().nth(target_line);
        let Some(line_content) = target_line_content else {
            return format!("error: unknown line {target_line}");
        };
        let line_number_str = format!("{}", target_line + 1);
        let line_prefix = format!("{line_number_str:<4} | ");
        let mut output = format!("{line_prefix}{line_content}\n");
        let indicator_prefix_len = line_prefix.len();

        let mut padding = line_content
            .chars()
            .take(target_column)
            .map(|_c| 1)
            .sum::<usize>();
        let indicator_spaces = " ".repeat(indicator_prefix_len);
        padding = if padding > 0 { padding - 1 } else { padding };
        let column_padding = " ".repeat(padding);

        output.push_str(indicator_spaces.as_str());
        output.push_str(column_padding.as_str());
        output.push('^');

        output
    }

    fn dump_error_info(message: &str, line: usize, column: usize, file: &SourceFile) {
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

    fn dump_lexer_error(lex_error: &LexerError, file: &SourceFile) {
        let message: String = match lex_error {
            LexerError::UnexpectedCharacter(c) => {
                format!("unexpected character {}", c.unwrap())
            }
            LexerError::IllegalLiteral => String::from("illegal literal"),
            LexerError::IllegalEscapeChar(char) => {
                format!("illegal escape character {char}")
            }
            LexerError::Eof => String::from("EOF"),
        };
        Self::dump_error_info(
            &message,
            file.lexer.get_now_line(),
            file.lexer.get_now_column(),
            file,
        );
    }

    fn dump_parser_error(error: ParserError, file: &SourceFile) {
        let line: usize;
        let column: usize;
        let message: String;

        match error {
            ParserError::LexError(lex_error) => {
                Self::dump_lexer_error(&lex_error, file);
                return;
            }
            ParserError::Eof | ParserError::Empty => {
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
            ParserError::Expected(token, c) => {
                line = token.line;
                column = token.column;
                message = format!("'{c}' expected.");
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
            ParserError::UnableResolveSymbols(token) => {
                line = token.line;
                column = token.column;
                message = String::from("unable to resolve symbols.");
            }
            ParserError::NoNativeImplement(token) => {
                line = token.line;
                column = token.column;
                message = String::from("no native implement.");
            }
            ParserError::NotFoundLibrary(token) => {
                line = token.line;
                column = token.column;
                message = String::from("not found import library.");
            }
            ParserError::MissingLoopBody(token) => {
                line = token.line;
                column = token.column;
                message = String::from("missing loop body.");
            }
        }

        Self::dump_error_info(&message, line, column, file);
    }

    pub fn warning_info_expr(source_file: &SourceFile, msg: &str, expr: &ASTExprTree, lint: Lint) {
        if !source_file.has_warnings(lint) {
            let token: &Token = match expr {
                ASTExprTree::Var(token)
                | ASTExprTree::Literal(token)
                | ASTExprTree::This(token)
                | ASTExprTree::Expr { token, .. }
                | ASTExprTree::Unary { token, .. } => token,
                ASTExprTree::Call { name: e_name, .. } => match e_name.as_ref() {
                    ASTExprTree::Var(token) => token,
                    _ => {
                        return;
                    }
                },
            };
            println!("warning: {msg}");
            println!(
                "{}",
                Self::highlight_line_and_column(source_file.get_data(), token.line, token.column)
            );
        }
    }

    /// # Errors
    /// # Panics
    #[warn(clippy::result_unit_err)]
    pub fn compile(&mut self) -> Result<(), ()> {
        let mut compiler = self.clone();
        for file in &mut self.files {
            if file.compiled {
                continue;
            }
            let vm_ir = file.compiler(&mut compiler);
            let vm_ir = match vm_ir {
                Ok(_) => vm_ir.unwrap(),
                Err(error) => {
                    Self::dump_parser_error(error, file);
                    return Err(());
                }
            };
            file.ir_table = Some(Box::new(vm_ir));
            file.compiled = true;
        }
        Ok(())
    }
}
