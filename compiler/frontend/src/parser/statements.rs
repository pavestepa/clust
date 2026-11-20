use super::*;

impl Parser {
    pub(super) fn block(&mut self) -> Result<Block> {
        self.block_with_tail(false)
    }
}

impl Parser {
    pub(super) fn block_with_tail(&mut self, allow_tail: bool) -> Result<Block> {
        self.nested(|p| {
            p.lines();
            let start = p.expect("{")?;
            let mut statements = Vec::new();
            p.separators();
            while !p.at("}") {
                if p.at_eof() {
                    return Err(p.error("unterminated block"));
                }
                statements.push(p.statement(allow_tail)?);
                p.separators();
            }
            let end = p.expect("}")?;
            Ok(Block {
                statements,
                span: start.join(end),
            })
        })
    }
}

impl Parser {
    pub(super) fn statement(&mut self, allow_tail: bool) -> Result<Statement> {
        self.unsupported()?;
        let start = self.current().span;
        let kind = if self.word("let") || self.word("var") {
            let mutable = self.eat_word("var");
            if !mutable {
                self.bump();
            }
            let pattern = self.pattern()?;
            let ty = if self.eat(":") {
                self.lines();
                Some(self.ty()?)
            } else {
                None
            };
            self.expect("=")?;
            self.lines();
            let value = self.expr(0, false, false)?;
            self.statement_end()?;
            StatementKind::Binding {
                mutable,
                pattern,
                ty,
                value,
            }
        } else if self.eat_word("return") {
            if self.constructor {
                return Err(self.error("constructors cannot return early"));
            }
            let value = if self.newline() || self.at(";") || self.at("}") || self.at_eof() {
                None
            } else {
                Some(self.expr(0, false, false)?)
            };
            self.statement_end()?;
            StatementKind::Return(value)
        } else if self.eat_word("break") {
            self.statement_end()?;
            StatementKind::Break
        } else if self.eat_word("continue") {
            self.statement_end()?;
            StatementKind::Continue
        } else if self.eat_word("unsafe") {
            return Err(
                self.error("Clust has no unsafe blocks; unsafe implementation belongs in Rust")
            );
        } else {
            let value = self.expr(0, false, false)?;
            if !matches!(
                value.kind,
                ExprKind::If { .. }
                    | ExprKind::While { .. }
                    | ExprKind::Match { .. }
                    | ExprKind::Branch(_)
            ) {
                if allow_tail {
                    let saved = (self.pos, self.last);
                    self.lines();
                    if !self.at("}") {
                        (self.pos, self.last) = saved;
                        self.statement_end()?;
                    }
                } else {
                    self.statement_end()?;
                }
            }
            StatementKind::Expression(value)
        };
        Ok(Statement {
            kind,
            span: start.join(self.last),
        })
    }
}
