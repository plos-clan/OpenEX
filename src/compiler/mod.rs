use crate::compiler::ast::ASTExprTree;
use crate::compiler::file::SourceFile;
use crate::compiler::lexer::{LexerError, Token};
use crate::compiler::lints::Lint;
use crate::compiler::parser::symbol_table::SymbolTable;
use crate::compiler::parser::ParserError;
use std::collections::HashSet;

mod ast;
pub mod file;
pub mod lexer;
#[allow(unused)]
pub mod lints;
mod parser;
mod semantic;

pub struct CompilerData {
    symbol_table: SymbolTable,
    lints: HashSet<Lint>,
}

pub struct Compiler {
    files: Vec<SourceFile>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler { files: vec![] }
    }

    pub fn get_version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
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
        let message: String = match lex_error {
            LexerError::UnexpectedCharacter(c) => {
                format!("unexpected character {}", c.unwrap())
            }
            LexerError::IllegalLiteral => String::from("illegal literal"),
            LexerError::IllegalEscapeChar(char) => {
                format!("illegal escape character {}", char)
            }
            LexerError::Eof => String::from("EOF"),
        };
        Self::dump_error_info(
            message,
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
                Self::dump_lexer_error(lex_error, file);
                return;
            }
            ParserError::Eof => {
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
                message = format!("'{}' expected.", c);
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
        }

        Self::dump_error_info(message, line, column, file);
    }

    pub fn warning_info_expr(source_file: &mut SourceFile, msg: &str, expr: &ASTExprTree,lint: Lint) {
        if !source_file.has_warnings(lint) {
            let token: &Token = match expr {
                ASTExprTree::Var(token)
                | ASTExprTree::Literal(token)
                | ASTExprTree::This(token) => token,
                ASTExprTree::Call { name: e_name, .. } =>{
                    match e_name.as_ref() { 
                        ASTExprTree::Var(token ) => token,
                        _=> unreachable!()
                    }
                },
                ASTExprTree::Unary { token: u_token, .. } => u_token,
                ASTExprTree::Expr { token: e_token, .. } => e_token,
            };
            println!("warning: {}", msg);
            println!(
                "{}",
                Self::highlight_line_and_column(source_file.get_data(), token.line, token.column)
            );
        }
    }

    pub fn compile(&mut self) {
        for file in &mut self.files {
            file.compiler().unwrap_or_else(|error| {
                Self::dump_parser_error(error, file);
            })
        }
    }
}
