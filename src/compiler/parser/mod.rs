mod blkparser;
mod exprparser;
mod funcparser;
mod ifparser;
mod impparser;
mod retparser;
pub mod symbol_table;
mod varparser;
mod whileparser;

use crate::compiler::ast::{ASTExprTree, ASTStmtTree};
use crate::compiler::file::SourceFile;
use crate::compiler::lexer::TokenType::LP;
use crate::compiler::lexer::{LexerError, Token, TokenType};
use crate::compiler::parser::exprparser::expr_eval;
use crate::compiler::parser::funcparser::func_eval;
use crate::compiler::parser::ifparser::if_eval;
use crate::compiler::parser::impparser::import_eval;
use crate::compiler::parser::varparser::var_eval;
use crate::compiler::parser::whileparser::while_eval;

pub enum ParserError {
    NotAStatement(Token),          // 不是一个语句
    LexError(LexerError),          // 词法分析错误
    IdentifierExpected(Token),     // 需要标识符
    Expected(Token, char),         // 需要指定字符
    MissingFunctionBody(Token),    // 缺少函数体
    MissingStatement(Token),       // 语句定义不完整
    MissingCondition(Token),       // 缺少条件表达式
    IllegalArgument(Token),        // 非法参数组合
    IllegalExpression(Token),      // 非法的表达式组合
    IllegalKey(Token),             // 非法的关键字
    BackOutsideLoop(Token),        // 循环退出语句位于循环体外
    SymbolDefined(Token),          // 类型已被定义
    IllegalTypeCombination(Token), //非法类型组合
    EOF,
}

pub struct Parser<'a> {
    cache: Option<Token>,
    last: Option<Token>,
    file: &'a mut SourceFile,
}

impl<'a> Parser<'a> {
    pub fn new(file: &'a mut SourceFile) -> Parser<'a> {
        Parser {
            cache: None,
            last: None,
            file,
        }
    }

    pub fn check_char(
        &mut self,
        token: &mut Token,
        type_: TokenType,
        c: char,
    ) -> Result<(), ParserError> {
        if !(token.t_type == type_ && token.value::<String>().unwrap() == c.to_string()) {
            return Err(ParserError::Expected(token.clone(), c));
        }
        Ok(())
    }

    fn next_parser_token(&mut self) -> Result<Token, ParserError> {
        match self.cache.take() {
            None => self.file.peek_token(),
            Some(token) => Ok(token),
        }
    }

    // 解析 () 括号内的表达式 - 需要括号
    pub fn parser_cond(&mut self) -> Result<ASTExprTree, ParserError> {
        let mut token = self.next_parser_token()?;
        self.check_char(&mut token, LP, '(')?;
        let last_token = token;
        let mut parentheses_count: usize = 0;
        let mut cond: Vec<Token> = Vec::new();

        loop {
            token = self.next_parser_token()?;
            match token.t_type {
                LP => {
                    if token.value::<String>().unwrap() == "(" {
                        parentheses_count += 1;
                    }
                    cond.push(token);
                }
                TokenType::LR => {
                    if token.value::<String>().unwrap() == ")" {
                        if parentheses_count == 0 {
                            break;
                        }
                        parentheses_count -= 1;
                    }
                    cond.push(token);
                }
                _ => {
                    cond.push(token);
                }
            }
        }
        self.last = Some(last_token.clone());
        if let Some(expr) = expr_eval(self, cond)? {
            Ok(expr)
        } else {
            Err(ParserError::MissingCondition(last_token))
        }
    }

    fn parse_step(&mut self) -> Result<ASTStmtTree, ParserError> {
        let root_token = self.next_parser_token()?;

        match root_token.t_type {
            TokenType::Function => {
                let saved_token = root_token.clone();
                self.cache = Some(root_token);
                Ok(func_eval(self).map_err(|e| match e {
                    ParserError::EOF => ParserError::MissingStatement(saved_token),
                    _ => e,
                })?)
            }
            TokenType::Import => {
                let saved_token = root_token.clone();
                Ok(import_eval(self).map_err(|e| match e {
                    ParserError::EOF => ParserError::MissingStatement(saved_token),
                    _ => e,
                })?)
            }
            TokenType::If => {
                let saved_token = root_token.clone();
                Ok(if_eval(self).map_err(|e| match e {
                    ParserError::EOF => ParserError::MissingStatement(saved_token),
                    _ => e,
                })?)
            }
            TokenType::Var => {
                let saved_token = root_token.clone();
                Ok(var_eval(self).map_err(|e| match e {
                    ParserError::EOF => ParserError::MissingStatement(saved_token),
                    _ => e,
                })?)
            }
            TokenType::While => {
                let saved_token = root_token.clone();
                Ok(while_eval(self).map_err(|e| match e {
                    ParserError::EOF => ParserError::MissingStatement(saved_token),
                    _ => e,
                })?)
            }
            TokenType::END => Ok(ASTStmtTree::Empty),
            TokenType::Continue | TokenType::Break => Err(ParserError::BackOutsideLoop(root_token)),
            _ => {
                let mut token;
                let mut tokens: Vec<Token> = vec![root_token.clone()];
                loop {
                    token = self.next_parser_token()?;
                    if token.t_type == TokenType::END {
                        break;
                    }
                    tokens.push(token);
                }
                if let Some(expr) = expr_eval(self, tokens)? {
                    Ok(ASTStmtTree::Expr(expr))
                } else {
                    Ok(ASTStmtTree::Empty)
                }
            }
        }
    }

    pub fn get_last(&mut self) -> Option<Token> {
        self.last.clone()
    }

    pub fn parser(mut self) -> Result<ASTStmtTree, ParserError> {
        let mut root_tree: Vec<ASTStmtTree> = vec![];
        loop {
            match self.parse_step() {
                Ok(node) => {
                    if let ASTStmtTree::Empty = node {
                    } else {
                        root_tree.push(node);
                    }
                }
                Err(ParserError::EOF) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(ASTStmtTree::Root(root_tree))
    }
}
