use crate::source::{Diagnostic, SourceId, Span};

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Word(String),
    Integer(String),
    Float(String),
    String(String),
    Char(char),
    Symbol(&'static str),
    Newline,
    Eof,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug)]
pub struct Lexed {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(source: SourceId, text: &str) -> Lexed {
    Lexer {
        source,
        text,
        pos: 0,
        tokens: Vec::new(),
        diagnostics: Vec::new(),
    }
    .run()
}

struct Lexer<'a> {
    source: SourceId,
    text: &'a str,
    pos: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl Lexer<'_> {
    fn run(mut self) -> Lexed {
        if self.text.starts_with('\u{feff}') {
            self.bump();
        }
        while let Some(c) = self.peek() {
            let start = self.pos;
            match c {
                ' ' | '\t' | '\x0c' => {
                    self.bump();
                }
                '\r' | '\n' => self.newline(),
                '/' if self.rest().starts_with("//") => {
                    while self.peek().is_some_and(|c| c != '\r' && c != '\n') {
                        self.bump();
                    }
                }
                '/' if self.rest().starts_with("/*") => self.comment(),
                'a'..='z' | 'A'..='Z' | '_' => {
                    while self
                        .peek()
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        self.bump();
                    }
                    self.emit(
                        start,
                        TokenKind::Word(self.text[start..self.pos].to_owned()),
                    );
                }
                '0'..='9' => self.number(),
                '"' | '\'' => self.quoted(c),
                '`' => {
                    self.bump();
                    while self.peek().is_some_and(|c| c != '`') {
                        self.bump();
                    }
                    if self.peek() == Some('`') {
                        self.bump();
                    }
                    self.error(
                        start,
                        "E0105",
                        "interpolation is not supported in the current Clust lexer",
                    );
                }
                _ => {
                    const SYMBOLS: &[&str] = &[
                        "->", "<<=", ">>=", "...", "..=", "=>", "==", "!=", "<=", ">=", "&&", "||",
                        "<<", ">>", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "..", "(", ")",
                        "{", "}", "[", "]", ",", ";", ":", ".", "+", "-", "*", "/", "%", "&", "|",
                        "^", "!", "=", "<", ">", "#", "?",
                    ];
                    if let Some(symbol) = SYMBOLS
                        .iter()
                        .find(|symbol| self.rest().starts_with(**symbol))
                    {
                        self.pos += symbol.len();
                        self.emit(start, TokenKind::Symbol(symbol));
                    } else {
                        self.bump();
                        let code = if c.is_alphabetic() { "E0110" } else { "E0001" };
                        self.error(
                            start,
                            code,
                            "unexpected character; identifiers must use ASCII",
                        );
                    }
                }
            }
        }
        self.emit(self.pos, TokenKind::Eof);
        Lexed {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }
}

mod comments;
mod cursor;
mod numbers;
mod strings;
use self::strings::digits;
