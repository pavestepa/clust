use super::*;

impl Parser {
    pub(super) fn string(&mut self) -> Result<String> {
        if let TokenKind::String(value) = &self.current().kind {
            let value = value.clone();
            self.bump();
            Ok(value)
        } else {
            Err(self.error("expected string literal"))
        }
    }
}
