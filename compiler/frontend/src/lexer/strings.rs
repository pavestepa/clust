use super::*;

impl Lexer<'_> {
    pub(super) fn quoted(&mut self, quote: char) {
        let start = self.pos;
        self.bump();
        let mut value = String::new();
        let mut closed = false;
        while let Some(c) = self.peek() {
            if c == quote {
                self.bump();
                closed = true;
                break;
            }
            if matches!(c, '\n' | '\r') {
                break;
            }
            if c == '\\' {
                let escape_start = self.pos;
                self.bump();
                if let Some(decoded) = self.escape() {
                    value.push(decoded);
                } else {
                    self.error(
                        escape_start,
                        "E0004",
                        "invalid escape sequence or Unicode scalar",
                    );
                }
            } else {
                self.bump();
                value.push(c);
            }
        }
        if !closed {
            self.error(
                start,
                "E0005",
                "unterminated literal; raw newlines are forbidden",
            );
        }
        if quote == '"' {
            self.emit(start, TokenKind::String(value));
        } else {
            let mut chars = value.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => self.emit(start, TokenKind::Char(c)),
                _ => self.error(
                    start,
                    "E0006",
                    "char literal must contain exactly one Unicode scalar",
                ),
            }
        }
    }
}

impl Lexer<'_> {
    pub(super) fn escape(&mut self) -> Option<char> {
        // Leave line terminators for normal newline recovery.
        if matches!(self.peek(), Some('\r' | '\n')) {
            return None;
        }
        match self.bump()? {
            '\\' => Some('\\'),
            '"' => Some('"'),
            '\'' => Some('\''),
            'n' => Some('\n'),
            'r' => Some('\r'),
            't' => Some('\t'),
            '0' => Some('\0'),
            'x' => {
                let a = self.peek()?.to_digit(16)?;
                self.bump();
                let b = self.peek()?.to_digit(16)?;
                self.bump();
                let value = a * 16 + b;
                (value <= 127).then(|| char::from_u32(value).unwrap())
            }
            'u' => {
                if self.peek() != Some('{') {
                    return None;
                }
                self.bump();
                let mut value = 0u32;
                let mut count = 0;
                while let Some(digit) = self.peek().and_then(|c| c.to_digit(16)) {
                    self.bump();
                    count += 1;
                    if count <= 6 {
                        value = value * 16 + digit;
                    }
                }
                if self.peek() != Some('}') {
                    return None;
                }
                self.bump();
                if count == 0 || count > 6 {
                    return None;
                }
                char::from_u32(value)
            }
            _ => None,
        }
    }
}

pub(super) fn digits(text: &str, radix: u32) -> bool {
    let mut previous_digit = false;
    for c in text.chars() {
        if c == '_' {
            if !previous_digit {
                return false;
            }
            previous_digit = false;
        } else if c.is_digit(radix) {
            previous_digit = true;
        } else {
            return false;
        }
    }
    previous_digit
}
