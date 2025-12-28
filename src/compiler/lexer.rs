use crate::compiler::lexer::LexerError::{Eof, UnexpectedCharacter};
use crate::compiler::lexer::TokenType::End;
use smol_str::{SmolStr, SmolStrBuilder};
use std::char;
use std::fmt::Debug;
use std::str::FromStr;
use dashu::float::{DBig, FBig};
use dashu::float::round::mode::HalfAway;

#[derive(Debug, Clone)]
pub struct LexerAnalysis {
    data: String,
    cache: Option<char>,
    data_index: usize,
    now_line: usize,
    now_column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    From,
}

const KEYWORDS: [(&str, TokenType); 17] = [
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
    ("from", TokenType::From),
];

impl Token {
    #[must_use]
    pub const fn new(
        data: SmolStr,
        line: usize,
        column: usize,
        index: usize,
        t_type: TokenType,
    ) -> Self {
        Self {
            line,
            column,
            t_type,
            index,
            data,
        }
    }

    pub fn value_float(&self) -> FBig<HalfAway, 10> {
        DBig::from_str(&self.data).unwrap()
    }

    pub fn value_number(&self) -> i64 {
        self.data
            .strip_prefix("0x")
            .or_else(|| self.data.strip_prefix("0X"))
            .map_or_else(
                || {
                    self.data
                        .strip_prefix("0b")
                        .or_else(|| self.data.strip_prefix("0B"))
                        .map_or_else(
                            || {
                                self.data
                                    .strip_prefix("0o")
                                    .or_else(|| self.data.strip_prefix("0O"))
                                    .map_or_else(
                                        || self.data.parse::<i64>().unwrap(),
                                        |oct| i64::from_str_radix(oct, 8).unwrap(),
                                    )
                            },
                            |bin| i64::from_str_radix(bin, 2).unwrap(),
                        )
                },
                |hex| i64::from_str_radix(hex, 16).unwrap(),
            )
    }

    pub fn value<T>(&self) -> Option<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Debug,
    {
        self.data.parse::<T>().ok()
    }

    pub fn text(&self) -> &str {
        &self.data
    }
}

const fn is_lp(c: char) -> bool {
    c == '(' || c == '[' || c == '{'
}

const fn is_lr(c: char) -> bool {
    c == ')' || c == ']' || c == '}'
}

impl LexerAnalysis {
    pub(crate) const fn new(p0: String) -> Self {
        Self {
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

    fn match_keyword(identifier_name: &str) -> TokenType {
        for (keyword, token_type) in &KEYWORDS {
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
                ' ' | '\t' => {}
                '\r' => {
                    self.now_column -= 1;
                }
                '\n' => {
                    self.now_line += 1;
                    self.now_column = 0;
                }
                c => {
                    self.cache = Some(c);
                    return Ok(());
                }
            }
        }
    }

    fn build_identifier(&mut self, line: usize, column: usize, data_index: usize) -> Token {
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

        let t_type = Self::match_keyword(data.as_str());

        Token::new(SmolStr::new(data), line, column, data_index, t_type)
    }

    fn build_number_hex(
        &mut self,
        line: usize,
        column: usize,
        data_index: usize,
        data: &mut SmolStrBuilder,
        next: char,
    ) -> Token {
        data.push(next);
        loop {
            match self.next_char() {
                c if c.is_ascii_hexdigit() => data.push(c),
                c => {
                    self.cache = Some(c);
                    break;
                }
            }
        }

        Token::new(data.finish(), line, column, data_index, TokenType::Number)
    }

    fn build_number_binary(
        &mut self,
        line: usize,
        column: usize,
        data_index: usize,
        data: &mut SmolStrBuilder,
        next: char,
    ) -> Token {
        data.push(next);
        loop {
            match self.next_char() {
                c if c == '0' || c == '1' => data.push(c),
                c => {
                    self.cache = Some(c);
                    break;
                }
            }
        }
        Token::new(
            data.finish(),
            line,
            column,
            data_index,
            TokenType::Number,
        )
    }

    fn build_number(
        &mut self,
        line: usize,
        column: usize,
        data_index: usize,
    ) -> Result<Token, LexerError> {
        let mut data = SmolStrBuilder::new();
        let mut is_float = TokenType::Number;

        let first_char = self.next_char();

        if first_char == '.' {
            let lookahead = self.next_char();

            if lookahead.is_ascii_digit() {
                // 合法浮点：.123
                is_float = TokenType::Float;
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
            match next {
                'x' | 'X' => {
                    return Ok(self.build_number_hex(line, column, data_index, &mut data, next));
                }
                'b' | 'B' => return Ok(self.build_number_binary(line, column, data_index, &mut data, next)),
                c if c.is_ascii_digit() => {
                    data.push(next);
                }
                '.' => {
                    data.push(next);
                    is_float = TokenType::Float;
                }
                _ => {
                    self.cache = Some(next);
                    return Ok(Token::new(
                        data.finish(),
                        line,
                        column,
                        data_index,
                        TokenType::Number,
                    ));
                }
            }
        }

        // ===== 小数 + 科学计数法解析 =====
        loop {
            match self.next_char() {
                c if c.is_ascii_digit() => {
                    data.push(c);
                }
                '.' => {
                    if let TokenType::Float = is_float {
                        return Err(LexerError::IllegalLiteral);
                    }
                    is_float = TokenType::Float;
                    data.push('.');
                }
                'e' | 'E' => {
                    is_float = TokenType::Float;
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
            is_float,
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
    ) -> Token {
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
        Token::new(data.finish(), line, column, data_index, TokenType::Operator)
    }

    fn build_semicolon_op_double(
        &mut self,
        line: usize,
        column: usize,
        data_index: usize,
        c: char,
    ) -> Token {
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
        Token::new(data.finish(), line, column, data_index, TokenType::Operator)
    }

    fn build_semicolon_op_easy(
        &mut self,
        line: usize,
        column: usize,
        data_index: usize,
        c: char,
    ) -> Token {
        let c0 = self.next_char();
        let mut data = SmolStrBuilder::new();
        if c0 == '=' {
            data.push(c);
            data.push('=');
        } else {
            self.cache = Some(c0);
            data.push(c);
        }
        Token::new(data.finish(), line, column, data_index, TokenType::Operator)
    }

    fn build_opt_skip_text(
        &mut self,
        line: usize,
        column: usize,
        data_index: usize,
        mut c: char,
    ) -> Result<Token, LexerError> {
        let mut str_builder = SmolStrBuilder::new();
        str_builder.push(c);
        c = self.next_char();
        match c {
            '/' => {
                loop {
                    c = self.next_char();
                    if c == '\n' || c == '\0' {
                        break;
                    }
                }
                self.cache = Some(c);
                self.next_token()
            }
            '*' => {
                loop {
                    c = self.next_char();
                    if c == '*' {
                        c = self.next_char();
                        if c == '/' {
                            break;
                        } else if c == '\0' {
                            return Err(Eof);
                        }
                    } else if c == '\0' {
                        return Err(Eof);
                    }
                }
                self.next_token()
            }
            '=' => {
                str_builder.push(c);
                Ok(Token::new(
                    str_builder.finish(),
                    line,
                    column,
                    data_index,
                    TokenType::Operator,
                ))
            }
            _ => {
                self.cache = Some(c);
                Ok(Token::new(
                    str_builder.finish(),
                    line,
                    column,
                    data_index,
                    TokenType::Operator,
                ))
            }
        }
    }

    const fn is_sem(c: char) -> bool {
        c == ',' || c == ':' || c == '?'
    }

    pub const fn get_now_line(&self) -> usize {
        self.now_line
    }

    pub const fn get_now_column(&self) -> usize {
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
                Ok(self.build_identifier(line, column, data_index))
            }
            c if c.is_ascii_digit() || c == '.' => {
                self.cache = Some(c);
                self.build_number(line, column, data_index)
            }
            c if is_lp(c) => Ok(Token::new(
                str_builder.finish(),
                line,
                column,
                data_index,
                TokenType::LP,
            )),
            c if is_lr(c) => Ok(Token::new(
                str_builder.finish(),
                line,
                column,
                data_index,
                TokenType::LR,
            )),
            c if Self::is_sem(c) => Ok(Token::new(
                str_builder.finish(),
                line,
                column,
                data_index,
                TokenType::Operator,
            )),
            '/' => self.build_opt_skip_text(line, column, data_index, '/'),
            '"' => self.build_string(line, column, data_index),
            '+' => Ok(self.build_semicolon_op_in(line, column, data_index, '+')),
            '-' => Ok(self.build_semicolon_op_in(line, column, data_index, '-')),
            '*' => Ok(self.build_semicolon_op_easy(line, column, data_index, '*')),
            '%' => Ok(self.build_semicolon_op_easy(line, column, data_index, '%')),
            '>' => Ok(self.build_semicolon_op_in(line, column, data_index, '>')),
            '<' => Ok(self.build_semicolon_op_in(line, column, data_index, '<')),
            '=' => Ok(self.build_semicolon_op_easy(line, column, data_index, '=')),
            '!' => Ok(self.build_semicolon_op_easy(line, column, data_index, '!')),
            '&' => Ok(self.build_semicolon_op_double(line, column, data_index, '&')),
            '|' => Ok(self.build_semicolon_op_double(line, column, data_index, '|')),
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
