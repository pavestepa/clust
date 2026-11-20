use super::*;

impl Parser {
    pub(super) fn end(&mut self) -> Result<()> {
        if self.eat(";") || self.newline() {
            self.lines();
            Ok(())
        } else if self.at("}") || self.at_eof() {
            Ok(())
        } else if self.at("=>") {
            Err(Diagnostic::new(
                "E0104",
                "lambda syntax is not supported; use a named function",
                self.current().span,
            ))
        } else {
            Err(self.error("expected ';', newline, or end of body"))
        }
    }
}

impl Parser {
    pub(super) fn nested<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        if self.depth >= 32 {
            return Err(Diagnostic::new(
                "E0201",
                "syntax nesting limit exceeded",
                self.current().span,
            ));
        }
        self.depth += 1;
        let result = f(self);
        self.depth -= 1;
        result
    }
}

impl Parser {
    pub(super) fn unsupported(&self) -> Result<()> {
        if matches!(&self.current().kind, TokenKind::Word(s) if matches!(s.as_str(), "struct" | "constructor" | "destructor" | "this" | "virtual" | "override" | "public" | "private" | "import" | "export"))
        {
            return Err(self.error("experimental syntax is not part of Clust"));
        }
        let (code, feature) = match &self.current().kind {
            TokenKind::Word(s) => match s.as_str() {
                "async" | "await" | "spawn" | "task_scope" | "transfer" => ("E0100", "concurrency"),
                "dyn" | "virtual" | "override" => ("E0102", "runtime polymorphism or inheritance"),

                "owns" | "type" => ("E0106", "ownership annotations or associated types"),
                _ => return Ok(()),
            },
            TokenKind::Symbol("...") => ("E0101", "variadics"),
            TokenKind::Symbol("?") => ("E0107", "propagation; use match"),
            TokenKind::Symbol(".." | "..=") => ("E0103", "range expressions"),
            _ => return Ok(()),
        };
        Err(Diagnostic::new(
            code,
            format!("{feature} is not supported in Clust 0.1"),
            self.current().span,
        ))
    }
}

impl Parser {
    pub(super) fn statement_end(&mut self) -> Result<()> {
        self.expect(";")?;
        Ok(())
    }
}
