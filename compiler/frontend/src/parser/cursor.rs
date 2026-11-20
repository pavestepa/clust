use super::*;

impl Parser {
    pub(super) fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }
}

impl Parser {
    pub(super) fn at_eof(&self) -> bool {
        self.current().kind == TokenKind::Eof
    }
}

impl Parser {
    pub(super) fn at(&self, symbol: &str) -> bool {
        matches!(&self.current().kind, TokenKind::Symbol(s) if *s == symbol)
    }
}

impl Parser {
    pub(super) fn word(&self, word: &str) -> bool {
        matches!(&self.current().kind, TokenKind::Word(s) if s == word)
    }
}

impl Parser {
    pub(super) fn newline(&self) -> bool {
        self.current().kind == TokenKind::Newline
    }
}

impl Parser {
    pub(super) fn bump(&mut self) -> Token {
        let token = self.current().clone();
        self.last = token.span;
        if !self.at_eof() {
            self.pos += 1;
        }
        token
    }
}

impl Parser {
    pub(super) fn eat(&mut self, symbol: &str) -> bool {
        if self.at(symbol) {
            self.bump();
            true
        } else {
            false
        }
    }
}

impl Parser {
    pub(super) fn eat_word(&mut self, word: &str) -> bool {
        if self.word(word) {
            self.bump();
            true
        } else {
            false
        }
    }
}

impl Parser {
    pub(super) fn lines(&mut self) {
        while self.newline() {
            self.bump();
        }
    }
}

impl Parser {
    pub(super) fn separators(&mut self) {
        while self.newline() || self.at(";") {
            self.bump();
        }
    }
}

impl Parser {
    pub(super) fn error(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new("E0200", message, self.current().span)
    }
}

impl Parser {
    pub(super) fn expect(&mut self, symbol: &str) -> Result<Span> {
        if self.eat(symbol) {
            Ok(self.last)
        } else {
            Err(self.error(format!("expected '{symbol}'")))
        }
    }
}

impl Parser {
    pub(super) fn expect_word(&mut self, word: &str) -> Result<Span> {
        if self.eat_word(word) {
            Ok(self.last)
        } else {
            Err(self.error(format!("expected '{word}'")))
        }
    }
}

impl Parser {
    pub(super) fn path_segment(&mut self) -> Result<String> {
        for word in ["self", "weak"] {
            if self.eat_word(word) {
                return Ok(word.into());
            }
        }
        self.name()
    }
}

impl Parser {
    pub(super) fn name(&mut self) -> Result<String> {
        self.unsupported()?;
        if let TokenKind::Word(name) = &self.current().kind {
            const RESERVED: &[&str] = &[
                "fn",
                "let",
                "var",
                "return",
                "break",
                "continue",
                "if",
                "else",
                "while",
                "loop",
                "match",
                "class",
                "struct",
                "enum",
                "interface",
                "constructor",
                "destructor",
                "const",
                "new",
                "unsafe",
                "extern",
                "mod",
                "use",
                "export",
                "from",
                "public",
                "private",
                "weak",
                "ref",
                "as",
                "true",
                "false",
                "this",
            ];
            if RESERVED.contains(&name.as_str()) {
                return Err(self.error("expected identifier, found reserved keyword"));
            }
            let name = name.clone();
            self.bump();
            Ok(name)
        } else {
            Err(self.error("expected identifier"))
        }
    }
}
