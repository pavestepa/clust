use super::*;

impl Lexer<'_> {
    pub(super) fn number(&mut self) {
        let start = self.pos;
        let base = if self.rest().starts_with("0x") {
            16
        } else if self.rest().starts_with("0o") {
            8
        } else if self.rest().starts_with("0b") {
            2
        } else {
            10
        };
        if base != 10 {
            self.pos += 2;
            while self
                .peek()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                self.bump();
            }
            if !digits(&self.text[start + 2..self.pos], base) {
                self.error(
                    start,
                    "E0003",
                    "invalid digits or separators in integer literal",
                );
            }
            self.emit(
                start,
                TokenKind::Integer(self.text[start..self.pos].to_owned()),
            );
            return;
        }
        while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '_') {
            self.bump();
        }
        let whole_end = self.pos;
        let mut valid = digits(&self.text[start..whole_end], 10);
        let mut float = false;
        if self.peek() == Some('.')
            && self
                .rest()
                .as_bytes()
                .get(1)
                .is_some_and(u8::is_ascii_digit)
        {
            float = true;
            self.bump();
            let fraction = self.pos;
            while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '_') {
                self.bump();
            }
            valid &= digits(&self.text[fraction..self.pos], 10);
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            float = true;
            self.bump();
            if matches!(self.peek(), Some('+' | '-')) {
                self.bump();
            }
            let exponent = self.pos;
            while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '_') {
                self.bump();
            }
            valid &= digits(&self.text[exponent..self.pos], 10);
        }
        if self
            .peek()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        {
            valid = false;
            while self
                .peek()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                self.bump();
            }
        }
        if !valid {
            self.error(
                start,
                "E0003",
                "invalid numeric literal; suffixes are not supported",
            );
        }
        let spelling = self.text[start..self.pos].to_owned();
        self.emit(
            start,
            if float {
                TokenKind::Float(spelling)
            } else {
                TokenKind::Integer(spelling)
            },
        );
    }
}
