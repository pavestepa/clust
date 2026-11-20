use super::*;

impl Engine<'_> {
    pub(super) fn tick(&mut self, span: Span) -> Result<()> {
        if self.fuel == 0 {
            return Err(trap(TrapKind::StepLimit, span));
        }
        self.fuel -= 1;
        Ok(())
    }
}

impl Engine<'_> {
    pub(super) fn call(
        &mut self,
        id: FunctionId,
        arguments: Vec<Argument>,
        span: Span,
    ) -> Result<Value> {
        self.tick(span)?;
        if self.depth >= 32 {
            return Err(trap(TrapKind::CallDepth, span));
        }
        let function = self
            .program
            .functions
            .get(id)
            .ok_or_else(|| trap(TrapKind::InvalidIr, span))?;
        if function.rust_adapter.is_some() {
            return Err(trap(TrapKind::ForeignFunction, span));
        }
        if function.parameters.len() != arguments.len() {
            return Err(trap(TrapKind::InvalidIr, span));
        }
        let mut frame = Frame {
            returned_view: function.result_view.is_some(),
            return_staging: None,
            temporary_places: Vec::new(),
            views: Default::default(),
            locals: function
                .locals
                .iter()
                .map(|local| memory::typed_slot(local.ty.clone(), self.clock.clone()))
                .collect(),
        };
        for (id, value) in function.parameters.iter().zip(arguments) {
            match value {
                Argument::Owned(value) => {
                    Place {
                        slot: frame.locals[*id].clone(),
                        projections: vec![],
                    }
                    .write(value)
                    .map_err(|kind| trap(kind, span))?;
                }
                Argument::Borrowed(place) => {
                    frame.views.insert(*id, place);
                }
            }
        }
        self.depth += 1;
        let result = self.block(&function.body, &mut frame);
        let parameters: Vec<_> = function
            .parameters
            .iter()
            .copied()
            .filter(|id| function.locals[*id].view.is_none())
            .collect();
        let result = self.finish_scope(result, &parameters, false, &Type::Unit, &mut frame, span);
        self.depth -= 1;
        match result {
            Err(Signal::Return) => Ok(frame
                .return_staging
                .take()
                .map(|p| p.0)
                .unwrap_or(Value::Unit)),
            Ok(_) if function.result == Type::Unit => Ok(Value::Unit),
            Ok(_) => Err(trap(TrapKind::InvalidIr, span)),
            Err(Signal::Trap(mut failure)) => {
                if failure.kind != TrapKind::DoublePanic {
                    if let Some((value, ty)) = frame.return_staging.take() {
                        let mut pending = Some(failure.clone());
                        if let Err(error) = self.drop_value(value, &ty, &mut pending, span) {
                            failure = error;
                        }
                    }
                }
                Err(Signal::Trap(failure))
            }
            other => other,
        }
    }
}
