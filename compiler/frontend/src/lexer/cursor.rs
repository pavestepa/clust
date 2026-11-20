use super::*;

impl Lexer<'_> {
    pub(super) fn rest(&self) -> &str {
        &self.text[self.pos..]
    }
}

impl Lexer<'_> {
    pub(super) fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }
}

impl Lexer<'_> {
    pub(super) fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }
}

impl Lexer<'_> {
    pub(super) fn span(&self, start: usize) -> Span {
        Span {
            source: self.source,
            start,
            end: self.pos,
        }
    }
}

impl Lexer<'_> {
    pub(super) fn emit(&mut self, start: usize, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            span: self.span(start),
        });
    }
}

impl Lexer<'_> {
    pub(super) fn error(&mut self, start: usize, code: &'static str, message: &str) {
        self.diagnostics
            .push(Diagnostic::new(code, message, self.span(start)));
    }
}

impl Lexer<'_> {
    pub(super) fn newline(&mut self) {
        let start = self.pos;
        if self.bump() == Some('\r') && self.peek() == Some('\n') {
            self.bump();
        }
        self.emit(start, TokenKind::Newline);
    }
}
