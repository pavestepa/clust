mod expression;
mod item;
mod iteration;
mod pattern;
mod types;

use crate::{
    ast::*,
    lexer::{lex, Token, TokenKind},
    source::{Diagnostic, SourceId, Span},
};

type Result<T> = std::result::Result<T, Diagnostic>;

pub struct Parsed {
    pub module: Option<Module>,
    pub diagnostics: Vec<Diagnostic>,
}

pub use parse as parse_clust;

/// Parse a Clust source file.
pub fn parse(source: SourceId, text: &str) -> Parsed {
    let lexed = lex(source, text);
    if !lexed.diagnostics.is_empty() {
        return Parsed {
            module: None,
            diagnostics: lexed.diagnostics,
        };
    }
    let last = lexed.tokens[0].span;
    let mut parser = Parser {
        tokens: lexed.tokens,
        pos: 0,
        depth: 0,
        last,
        edits: Vec::new(),
        constructor: false,
    };
    match parser.module() {
        Ok(module) => Parsed {
            module: Some(module),
            diagnostics: Vec::new(),
        },
        Err(error) => Parsed {
            module: None,
            diagnostics: vec![error],
        },
    }
}

struct Parser {
    constructor: bool,
    tokens: Vec<Token>,
    pos: usize,
    depth: usize,
    last: Span,
    edits: Vec<(usize, Token)>,
}

impl Parser {
    fn module(&mut self) -> Result<Module> {
        let mut items = Vec::new();
        self.separators();
        while !self.at_eof() {
            items.push(self.item()?);
            self.separators();
        }
        Ok(Module { items })
    }
}

mod boundaries;
mod cursor;
mod statements;
