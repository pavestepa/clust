use super::*;

impl Engine<'_> {
    pub(in super::super) fn finish_scope(
        &mut self,
        result: Result<Value>,
        locals: &[usize],
        _construction: bool,
        result_type: &Type,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        if matches!(&result,Err(Signal::Trap(t)) if t.kind==TrapKind::DoublePanic) {
            return result;
        }
        let mut pending = match &result {
            Err(Signal::Trap(t)) => Some(t.clone()),
            _ => None,
        };
        if pending.is_some() {
            self.active_panic = true;
        }
        let mut selected = locals.to_vec();
        selected.sort_by_key(|id| std::cmp::Reverse(frame.locals[*id].initialized_at()));
        for local in selected {
            if !frame.views.contains_key(&local) {
                let slot = frame.locals[local].clone();
                self.drop_place(
                    Reference {
                        slot: slot.clone(),
                        projections: vec![],
                    },
                    slot.ty(),
                    &mut pending,
                    span,
                )
                .map_err(Signal::Trap)?;
                *slot.borrow_mut() = None;
                slot.state().borrow_mut().finish(&[]);
            }
            frame.views.remove(&local);
        }
        if pending.is_some() {
            if let Ok(value) = result {
                self.drop_value(value, result_type, &mut pending, span)
                    .map_err(Signal::Trap)?;
            }
            Err(Signal::Trap(pending.unwrap()))
        } else {
            result
        }
    }
}

impl Engine<'_> {
    pub(in super::super) fn aggregate(
        &mut self,
        fields: &[Expr],
        frame: &mut Frame,
        span: Span,
    ) -> Result<Vec<Value>> {
        let mut values = Vec::new();
        for field in fields {
            match self.expr(field, frame) {
                Ok(value) => values.push(value),
                Err(signal) => {
                    if matches!(&signal,Signal::Trap(t) if t.kind==TrapKind::DoublePanic) {
                        return Err(signal);
                    }
                    let mut pending = match &signal {
                        Signal::Trap(t) => Some(t.clone()),
                        _ => None,
                    };
                    for (value, field) in values.into_iter().zip(fields).rev() {
                        self.drop_value(value, &field.ty, &mut pending, span)
                            .map_err(Signal::Trap)?;
                    }
                    return Err(pending.map(Signal::Trap).unwrap_or(signal));
                }
            }
        }
        Ok(values)
    }
}
