use super::*;

impl Lexer<'_> {
    pub(super) fn comment(&mut self) {
        let start = self.pos;
        self.pos += 2;
        let mut depth = 1usize;
        while self.peek().is_some() {
            if self.rest().starts_with("/*") {
                self.pos += 2;
                depth += 1;
            } else if self.rest().starts_with("*/") {
                self.pos += 2;
                depth -= 1;
                if depth == 0 {
                    return;
                }
            } else if matches!(self.peek(), Some('\r' | '\n')) {
                self.newline();
            } else {
                self.bump();
            }
        }
        self.error(start, "E0002", "unterminated block comment");
    }
}
