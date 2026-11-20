use super::*;

impl Constructor {
    pub(super) fn complete(&self, state: &State, span: Span) -> Result<()> {
        let missing = self
            .fields
            .difference(&state.initialized)
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(error(
                span,
                "E0393",
                format!(
                    "constructor target is not fully initialized; missing fields: {}",
                    missing.join(", ")
                ),
            ));
        }
        Ok(())
    }
}

impl Constructor {
    pub(super) fn field<'a>(&self, value: &'a a::Expr, state: &State) -> Option<&'a str> {
        match &value.kind {
            a::ExprKind::Group(v) => self.field(v, state),
            a::ExprKind::Name(name)
                if self.fields.contains(name) && !state.locals.contains(name) =>
            {
                Some(name)
            }
            a::ExprKind::Member { value, name } if is_this(value) => {
                self.fields.contains(name).then_some(name)
            }
            _ => None,
        }
    }
}

impl Constructor {
    pub(super) fn block(&self, block: &a::Block, state: &mut State) -> Result<()> {
        let outer = state.locals.clone();
        for statement in &block.statements {
            if !state.reachable {
                break;
            }
            match &statement.kind {
                a::StatementKind::Binding { pattern, value, .. } => {
                    self.expr(value, state)?;
                    bindings(pattern, &mut state.locals);
                }
                a::StatementKind::Expression(value) => self.expr(value, state)?,
                a::StatementKind::Return(value) => {
                    if let Some(value) = value {
                        self.expr(value, state)?;
                    }
                    self.complete(state, statement.span)?;
                    state.reachable = false;
                }
                a::StatementKind::Break | a::StatementKind::Continue => state.reachable = false,
                a::StatementKind::Unsafe(body) => self.block(body, state)?,
            }
        }
        state.locals = outer;
        Ok(())
    }
}
