use super::*;

impl Builder {
    pub(super) fn end_storage(&mut self, from: usize, span: Span) {
        if self.current.is_none() {
            return;
        }
        let locals = self.scopes[from..]
            .iter()
            .rev()
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        self.end_locals(locals, span);
    }
}

impl Builder {
    pub(super) fn end_locals(&mut self, locals: Vec<LocalId>, span: Span) {
        if locals.is_empty() || self.current.is_none() {
            return;
        }
        let next = self.block_id(span);
        self.end(Terminator::EndStorage { locals, next });
        self.current = Some(next);
    }
}

impl Builder {
    pub(super) fn end_temporaries(&mut self, from: LocalId, keep: Option<&Operand>, span: Span) {
        let locals = (from..self.function.locals.len())
            .filter(|id| !matches!(keep, Some(Operand::Local(keep)) if keep == id))
            .collect();
        self.end_locals(locals, span);
    }
}

impl Builder {
    pub(super) fn block(&mut self, body: &h::Block) -> Operand {
        self.block_mode(body, None)
    }
}

impl Builder {
    pub(super) fn block_mode(
        &mut self,
        body: &h::Block,
        borrowed: Option<loans::Access>,
    ) -> Operand {
        let parent = *self.cleanup_scopes.last().unwrap();
        let scope = if body.construction.is_empty() {
            self.function.cleanup.scope(parent)
        } else {
            let scope = self.function.cleanup.construction_scope(parent);
            for (index, local) in body.construction.iter().enumerate() {
                self.function.cleanup.field_order.insert(*local, index);
            }
            scope
        };
        self.cleanup_scopes.push(scope);
        for local in &body.storage {
            self.function.cleanup.assign(*local, scope);
        }
        self.scopes.push(body.storage.clone());
        let mut result = Operand::Constant(Value::Unit);
        for (index, statement) in body.statements.iter().enumerate() {
            if self.current.is_none() {
                break;
            }
            let temporary_start = self.function.locals.len();
            let expression_scope = self.function.cleanup.scope(scope);
            self.cleanup_scopes.push(expression_scope);
            result = if index + 1 == body.statements.len() && borrowed.is_some() {
                match statement {
                    Statement::Evaluate(value) => self.expr_mode(value, borrowed),
                    _ => self.statement(statement, body.span),
                }
            } else {
                self.statement(statement, body.span)
            };
            self.cleanup_scopes.pop();
            let keep_result = index + 1 == body.statements.len() && body.result != Type::Unit;
            if keep_result {
                if let Operand::Local(local) = &result {
                    self.function.cleanup.assign(*local, parent);
                }
            }
            self.end_temporaries(temporary_start, keep_result.then_some(&result), body.span);
        }
        self.end_storage(self.scopes.len() - 1, body.span);
        self.scopes.pop();
        self.cleanup_scopes.pop();
        if body.result == Type::Unit {
            Operand::Constant(Value::Unit)
        } else {
            result
        }
    }
}

impl Builder {
    pub(super) fn statement(&mut self, statement: &Statement, span: Span) -> Operand {
        match statement {
            Statement::Initialize(id, value) => {
                let operand = self.expr(value);
                self.evaluate_into(Operation::Use(operand), Place::local(*id), value.span);
            }
            Statement::Evaluate(value) => return self.expr(value),
            Statement::Return(value) => {
                let result = if let (Some(contract), Some(value)) =
                    (self.function.result_view.clone(), value)
                {
                    self.expr_mode(value, Some(contract.access))
                } else {
                    value
                        .as_ref()
                        .map_or(Operand::Constant(Value::Unit), |v| self.expr_mode(v, None))
                };
                if let Operand::Local(local) = &result {
                    self.function.cleanup.assign(*local, 0);
                }
                self.end_temporaries(self.temporary_start, Some(&result), span);
                self.end_storage(0, span);
                let result = if self.function.result_view.is_some() {
                    let Operand::Local(id) = result else {
                        unreachable!()
                    };
                    Operand::Address(Place {
                        local: id,
                        projections: vec![Projection::Dereference],
                    })
                } else {
                    result
                };
                self.end(Terminator::Return(result));
            }
            Statement::Break | Statement::Continue => {
                let (test, end, depth, temporary_start) = *self.loops.last().unwrap();
                self.end_temporaries(temporary_start, None, span);
                self.end_storage(depth, span);
                self.end(Terminator::Goto(if matches!(statement, Statement::Break) {
                    end
                } else {
                    test
                }));
            }
        }
        let _ = span;
        Operand::Constant(Value::Unit)
    }
}
