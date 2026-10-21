use std::fmt;

use crate::messages as msg;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Str(String),
    Ident(String),

    Let,
    If, Elseif, Else, Then, End,
    For, In, Do, Loop, Break, Continue,
    Match,
    Echo, EchoDot,
    True, False, Nil,
    And, Or, Not,
    Return,

    Plus, Minus, Star, Slash, Percent,
    PlusPlus,
    Eq, EqEq, NotEq, Lt, LtEq, Gt, GtEq,
    Caret, Backslash, Arrow,
    LParen, RParen, LBrace, RBrace,
    LBracket, RBracket,
    Comma, Dot, Colon,

    Newline,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Self { chars: src.chars().collect(), pos: 0, line: 1, col: 1 }
    }

    fn peek(&self) -> Option<char> { self.chars.get(self.pos).copied() }
    fn peek2(&self) -> Option<char> { self.chars.get(self.pos + 1).copied() }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
        Some(c)
    }

    pub fn tokenize(&mut self) -> Result<Vec<(Token, usize, usize)>, String> {
        let mut out = Vec::new();
        loop {
            let (line, col) = (self.line, self.col);
            match self.next_token()? {
                Some(tok) => out.push((tok, line, col)),
                None => break,
            }
        }
        out.push((Token::Eof, self.line, self.col));
        Ok(out)
    }

    fn next_token(&mut self) -> Result<Option<Token>, String> {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') => { self.advance(); }
                Some('-') if self.peek2() == Some('-') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' { break; }
                        self.advance();
                    }
                }
                _ => break,
            }
        }

        let c = match self.peek() {
            Some(c) => c,
            None => return Ok(None),
        };

        if c == '\n' {
            self.advance();
            return Ok(Some(Token::Newline));
        }

        if c.is_ascii_digit() {
            let mut s = String::new();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() || ch == '_' || (ch == '.' && self.peek2().map_or(false, |d| d.is_ascii_digit())) {
                    s.push(ch); self.advance();
                } else { break; }
            }
            let clean = s.replace('_', "");
            let n: f64 = clean.parse().map_err(|_| msg::lex_bad_number(&s))?;
            return Ok(Some(Token::Number(n)));
        }

        if c.is_alphabetic() || c == '_' {
            let mut s = String::new();
            while let Some(ch) = self.peek() {
                if ch.is_alphanumeric() || ch == '_' { s.push(ch); self.advance(); }
                else { break; }
            }
            return Ok(Some(match s.as_str() {
                "let" => Token::Let,
                "if" => Token::If,
                "elseif" => Token::Elseif,
                "else" => Token::Else,
                "then" => Token::Then,
                "end" => Token::End,
                "for" => Token::For,
                "in" => Token::In,
                "do" => Token::Do,
                "loop" => Token::Loop,
                "break" => Token::Break,
                "continue" => Token::Continue,
                "match" => Token::Match,
                "true" => Token::True,
                "false" => Token::False,
                "nil" => Token::Nil,
                "and" => Token::And,
                "or" => Token::Or,
                "not" => Token::Not,
                "return" => Token::Return,
                "echo" => {
                    if self.peek() == Some('.') { self.advance(); Token::EchoDot }
                    else { Token::Echo }
                }
                _ => Token::Ident(s),
            }));
        }

        if c == '"' {
            self.advance();
            let mut s = String::new();
            loop {
                match self.peek() {
                    None => return Err(msg::lex_unclosed_string(self.line)),
                    Some('"') => { self.advance(); break; }
                    Some('\\') => {
                        self.advance();
                        match self.advance() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some('"') => s.push('"'),
                            Some('\\') => s.push('\\'),
                            Some('$') => s.push('$'),
                            Some(other) => return Err(msg::lex_unknown_escape(other)),
                            None => return Err(msg::lex_unclosed_string_generic()),
                        }
                    }
                    Some(ch) => { s.push(ch); self.advance(); }
                }
            }
            return Ok(Some(Token::Str(s)));
        }

        self.advance();
        Ok(Some(match c {
            '+' => if self.peek() == Some('+') { self.advance(); Token::PlusPlus } else { Token::Plus },
            '-' => if self.peek() == Some('>') { self.advance(); Token::Arrow } else { Token::Minus },
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '=' => if self.peek() == Some('=') { self.advance(); Token::EqEq } else { Token::Eq },
            '!' => if self.peek() == Some('=') { self.advance(); Token::NotEq }
                   else { return Err(msg::lex_bang_without_eq(self.line, self.col)); },
            '<' => if self.peek() == Some('=') { self.advance(); Token::LtEq } else { Token::Lt },
            '>' => if self.peek() == Some('=') { self.advance(); Token::GtEq } else { Token::Gt },
            '^' => Token::Caret,
            '\\' => Token::Backslash,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            '.' => Token::Dot,
            ':' => Token::Colon,
            other => return Err(msg::lex_unknown_char(self.line, self.col, other)),
        }))
    }
}