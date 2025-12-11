use crate::compiler::lexer::LexerError::{Eof, UnexpectedCharacter};
use crate::compiler::lexer::TokenType::End;
use smol_str::{SmolStr, SmolStrBuilder};
use std::char;
use std::fmt::Debug;
use std::str::FromStr;

pub struct LexerAnalysis {
    data: String,
    cache: Option<char>,
    data_index: usize,
    now_line: usize,
    now_column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub line: usize,
    pub column: usize,
    pub t_type: TokenType,
    pub index: usize, // 文件内容索引
    data: SmolStr,
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(Option<char>),
    IllegalEscapeChar(char),
    IllegalLiteral,
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Number,
    Float,
    LiteralString,
    Identifier,
    Operator,
    LP,
    LR,
    End,

    // 关键字类型
    For,
    While,
    If,
    Elif,
    Else,
    Return,
    Break,
    Continue,
    Import,
    Function,
    True,
    False,
    Var,
    This,
    Null,
    Native,
}

const KEYWORDS: [(&str, TokenType); 16] = [
    ("for", TokenType::For),
    ("while", TokenType::While),
    ("if", TokenType::If),
    ("elif", TokenType::Elif),
    ("else", TokenType::Else),
    ("return", TokenType::Return),
    ("break", TokenType::Break),
    ("continue", TokenType::Continue),
    ("import", TokenType::Import),
    ("function", TokenType::Function),
    ("true", TokenType::True),
    ("false", TokenType::False),
    ("var", TokenType::Var),
    ("this", TokenType::This),
    ("null", TokenType::Null),
    ("native", TokenType::Native),
];

impl Token {
    pub fn new(
        data: SmolStr,
        line: usize,
        column: usize,
        index: usize,
        t_type: TokenType,
    ) -> Token {
        Token {
            line,
            column,
            data,
            t_type,
            index,
        }
    }

    pub fn value_float(&mut self) -> f64 {
        self.data.parse::<f64>().unwrap()
    }

    pub fn value_number(&mut self) -> i64 {
        if let Some(hex) = self
            .data
            .strip_prefix("0x")
            .or_else(|| self.data.strip_prefix("0X"))
        {
            i64::from_str_radix(hex, 16).unwrap()
        } else if let Some(bin) = self
            .data
            .strip_prefix("0b")
            .or_else(|| self.data.strip_prefix("0B"))
        {
            i64::from_str_radix(bin, 2).unwrap()
        } else if let Some(oct) = self
            .data
            .strip_prefix("0o")
            .or_else(|| self.data.strip_prefix("0O"))
        {
            i64::from_str_radix(oct, 8).unwrap()
        } else {
            self.data.parse::<i64>().unwrap()
        }
    }

    pub fn value<T>(&mut self) -> Option<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Debug,
    {
        Some(self.data.parse::<T>().unwrap())
    }

    pub fn text(&self) -> &str {
        &self.data
    }
}

impl LexerAnalysis {
    pub(crate) fn new(p0: String) -> Self {
        LexerAnalysis {
            data: p0,
            now_line: 0,
            now_column: 0,
            data_index: 0,
            cache: None,
        }
    }

    fn next_char(&mut self) -> char {
        self.cache.take().unwrap_or_else(|| {
            self.data[self.data_index..]
                .chars()
                .next()
                .inspect(|ch| {
                    self.data_index += ch.len_utf8();
                    self.now_column += 1;
                })
                .unwrap_or('\0')
        })
    }

    fn is_lp(&self, c: char) -> bool {
        c == '(' || c == '[' || c == '{'
    }

    fn is_lr(&self, c: char) -> bool {
        c == ')' || c == ']' || c == '}'
    }

    fn match_keyword(&self, identifier_name: String) -> TokenType {
        for (keyword, token_type) in KEYWORDS.iter() {
            if identifier_name == *keyword {
                return *token_type;
            }
        }
        TokenType::Identifier
    }

    fn skip_whitespace(&mut self) -> Result<(), LexerError> {
        loop {
            match self.next_char() {
                '\0' => return Err(Eof),
                ' ' | '\t' => {
                    continue;
                }
                '\r' => {
                    self.now_column -= 1;
                    continue;
                }
                '\n' => {
                    self.now_line += 1;
                    self.now_column = 0;
                    continue;
                }
                c => {
                    self.cache = Some(c);
                    return Ok(());
                }
            }
        }
    }

    fn build_identifier(
        &mut self,
        line: usize,
        column: usize,
        data_index: usize,
    ) -> Result<Token, LexerError> {
        let mut data = String::new();
        loop {
            match self.next_char() {
                c if c.is_alphabetic() || c == '_' || c.is_ascii_digit() => {
                    data.push(c);
                }
                c => {
                    self.cache = Some(c);
                    break;
                }
            }
        }

        let t_type = self.match_keyword(data.clone());

        Ok(Token::new(
            SmolStr::new(data),
            line,
            column,
            data_index,
            t_type,
        ))
    }

    fn build_number(
        &mut self,
        line: usize,
        column: usize,
        data_index: usize,
    ) -> Result<Token, LexerError> {
        let mut data = SmolStrBuilder::new();
        let mut is_float = false;

        let first_char = self.next_char();

        if first_char == '.' {
            let lookahead = self.next_char();

            if lookahead.is_ascii_digit() {
                // 合法浮点：.123
                is_float = true;
                data.push('.');
                data.push(lookahead);
            } else {
                // 非法 .<num> 组合 → 返回 . 并缓存字符
                self.cache = Some(lookahead);
                return Ok(Token::new(
                    ".".into(),
                    line,
                    column,
                    data_index,
                    TokenType::Operator,
                ));
            }
        } else {
            data.push(first_char);
        }

        if first_char == '0' {
            let next = self.next_char();
            data.push(next);

            match next {
                'x' | 'X' => {
                    loop {
                        match self.next_char() {
                            c if c.is_ascii_hexdigit() => data.push(c),
                            c => {
                                self.cache = Some(c);
                                break;
                            }
                        }
                    }

                    return Ok(Token::new(
                        data.finish(),
                        line,
                        column,
                        data_index,
                        TokenType::Number,
                    ));
                }
                'b' | 'B' => {
                    loop {
                        match self.next_char() {
                            c if c == '0' || c == '1' => data.push(c),
                            c => {
                                self.cache = Some(c);
                                break;
                            }
                        }
                    }

                    return Ok(Token::new(
                        data.finish(),
                        line,
                        column,
                        data_index,
                        TokenType::Number,
                    ));
                }
                c if c.is_ascii_digit() => {}
                '.' => {
                    is_float = true;
                }
                _ => return Err(LexerError::IllegalLiteral),
            }
        }

        // ===== 小数 + 科学计数法解析 =====
        loop {
            match self.next_char() {
                c if c.is_ascii_digit() => {
                    data.push(c);
                }
                '.' => {
                    if is_float {
                        return Err(LexerError::IllegalLiteral);
                    }
                    is_float = true;
                    data.push('.');
                }
                'e' | 'E' => {
                    is_float = true;
                    data.push('e');

                    let sign = self.next_char();
                    if sign == '+' || sign == '-' || sign.is_ascii_digit() {
                        data.push(sign);
                    } else {
                        return Err(LexerError::IllegalLiteral);
                    }

                    let mut has_exp_digit = false;
                    loop {
                        match self.next_char() {
                            c if c.is_ascii_digit() => {
                                has_exp_digit = true;
                                data.push(c);
                            }
                            c => {
                                if !has_exp_digit {
                                    return Err(LexerError::IllegalLiteral);
                                }
                                self.cache = Some(c);
                                break;
                            }
                        }
                    }
                    break;
                }
                c => {
                    self.cache = Some(c);
                    break;
                }
            }
        }

        Ok(Token::new(
            data.finish(),
            line,
            column,
            data_index,
            if is_float {
                TokenType::Float
            } else {
                TokenType::Number
            },
        ))
    }

    fn build_string(
        &mut self,
        line: usize,
        column: usize,
        data_index: usize,
    ) -> Result<Token, LexerError> {
        let mut data = SmolStrBuilder::new();
        let mut n_char;
        loop {
            n_char = self.next_char();
            match n_char {
                '"' => break,
                '\\' => {
                    n_char = self.next_char();
                    match n_char {
                        'n' => data.push('\n'),
                        't' => data.push('\t'),
                        'r' => data.push('\r'),
                        '"' => data.push('"'),
                        '\\' => data.push('\\'),
                        _ => return Err(LexerError::IllegalEscapeChar(n_char)),
                    }
                }
                _ => data.push(n_char),
            }
        }

        Ok(Token::new(
            data.finish(),
            line,
            column,
            data_index,
            TokenType::LiteralString,
        ))
    }

    fn build_semicolon_op_in(
        &mut self,
        line: usize,
        column: usize,
        data_index: usize,
        c: char,
    ) -> Result<Token, LexerError> {
        let c0 = self.next_char();
        let mut data = SmolStrBuilder::new();
        match c0 {
            c1 if c1 == c => {
                data.push(c);
                data.push(c);
            }
            '=' => {
                data.push(c);
                data.push('=');
            }
            _ => {
                self.cache = Some(c0);
                data.push(c);
            }
        }
        Ok(Token::new(
            data.finish(),
            line,
            column,
            data_index,
            TokenType::Operator,
        ))
    }

    fn build_semicolon_op_double(
        &mut self,
        line: usize,
        column: usize,
        data_index: usize,
        c: char,
    ) -> Result<Token, LexerError> {
        let c0 = self.next_char();
        let mut data = SmolStrBuilder::new();
        match c0 {
            c1 if c1 == c => {
                data.push(c);
                data.push(c);
            }
            _ => {
                self.cache = Some(c0);
                data.push(c);
            }
        }
        Ok(Token::new(
            data.finish(),
            line,
            column,
            data_index,
            TokenType::Operator,
        ))
    }

    fn build_semicolon_op_easy(
        &mut self,
        line: usize,
        column: usize,
        data_index: usize,
        c: char,
    ) -> Result<Token, LexerError> {
        let c0 = self.next_char();
        let mut data = SmolStrBuilder::new();
        match c0 {
            '=' => {
                data.push(c);
                data.push('=');
            }
            _ => {
                self.cache = Some(c0);
                data.push(c);
            }
        }
        Ok(Token::new(
            data.finish(),
            line,
            column,
            data_index,
            TokenType::Operator,
        ))
    }

    fn is_sem(&self, c: char) -> bool {
        c == ',' || c == ':' || c == '?'
    }

    pub fn get_now_line(&self) -> usize {
        self.now_line
    }

    pub fn get_now_column(&self) -> usize {
        self.now_column
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace()?;

        let start: char = self.next_char();
        let line = self.now_line;
        let column = self.now_column;
        let data_index = self.data_index;
        let mut str_builder = SmolStrBuilder::new();
        str_builder.push(start);
        match start {
            '\0' => Err(Eof),
            c if c.is_alphabetic() || c == '_' => {
                self.cache = Some(c);
                self.build_identifier(line, column, data_index)
            }
            c if c.is_ascii_digit() || c == '.' => {
                self.cache = Some(c);
                self.build_number(line, column, data_index)
            }
            c if self.is_lp(c) => Ok(Token::new(
                str_builder.finish(),
                line,
                column,
                data_index,
                TokenType::LP,
            )),
            c if self.is_lr(c) => Ok(Token::new(
                str_builder.finish(),
                line,
                column,
                data_index,
                TokenType::LR,
            )),
            c if self.is_sem(c) => Ok(Token::new(
                str_builder.finish(),
                line,
                column,
                data_index,
                TokenType::Operator,
            )),
            '"' => self.build_string(line, column, data_index),
            '+' => self.build_semicolon_op_in(line, column, data_index, '+'),
            '-' => self.build_semicolon_op_in(line, column, data_index, '-'),
            '*' => self.build_semicolon_op_easy(line, column, data_index, '*'),
            '>' => self.build_semicolon_op_in(line, column, data_index, '>'),
            '<' => self.build_semicolon_op_in(line, column, data_index, '<'),
            '=' => self.build_semicolon_op_easy(line, column, data_index, '='),
            '!' => self.build_semicolon_op_easy(line, column, data_index, '!'),
            '&' => self.build_semicolon_op_double(line, column, data_index, '&'),
            '|' => self.build_semicolon_op_double(line, column, data_index, '|'),
            ';' => Ok(Token::new(
                str_builder.finish(),
                line,
                column,
                data_index,
                End,
            )),
            _ => Err(UnexpectedCharacter(Some(start))),
        }
    }
}
