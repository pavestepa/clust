use super::*;
impl Engine<'_> {
    pub(super) fn invoke(&mut self, expr: &Expr, frame: &mut Frame, span: Span) -> Result<Value> {
        let (id, arguments) = match &expr.kind {
            ExprKind::Call(id, arguments) => (*id, arguments),
            ExprKind::StorageCall {
                function,
                arguments,
                ..
            }
            | ExprKind::BorrowedCall {
                function,
                arguments,
                ..
            } => (*function, arguments),
            ExprKind::New {
                constructor,
                arguments,
            } => (*constructor, arguments),
            ExprKind::CallIndirect(callee, arguments, _) => {
                let Value::Function(id) = self.expr(callee, frame)? else {
                    return Err(trap(TrapKind::InvalidIr, span));
                };
                (id, arguments)
            }
            _ => unreachable!(),
        };
        let mut args = Vec::new();
        for (index, value) in arguments.iter().enumerate() {
            let argument = match &value.kind {
                ExprKind::ArgumentView { place, .. } => self
                    .place(place, frame)
                    .and_then(|p| p.captured().map_err(|kind| trap(kind, span)))
                    .map(Argument::Borrowed),
                _ => self.expr(value, frame).map(Argument::Owned),
            };
            match argument {
                Ok(argument) => args.push(argument),
                Err(signal) => {
                    if matches!(&signal,Signal::Trap(t) if t.kind==TrapKind::DoublePanic) {
                        return Err(signal);
                    }
                    let mut pending = match &signal {
                        Signal::Trap(t) => Some(t.clone()),
                        _ => None,
                    };
                    for (argument, expression) in args.into_iter().zip(&arguments[..index]).rev() {
                        if let Argument::Owned(value) = argument {
                            self.drop_value(value, &expression.ty, &mut pending, span)
                                .map_err(Signal::Trap)?;
                        }
                    }
                    return Err(pending.map(Signal::Trap).unwrap_or(signal));
                }
            }
        }
        if matches!(&expr.kind, ExprKind::New { .. }) {
            let name = &expr.ty.class_key();
            let object =
                memory::allocate_class(name, &self.program.classes[name], self.clock.clone());
            args.insert(
                0,
                Argument::Borrowed(memory::class_reference(object.clone(), self.clock.clone())),
            );
            match self.call(id, args, span) {
                Ok(_) => {
                    let value = Value::Class(object);
                    memory::complete_class(&value).map_err(|kind| trap(kind, span))?;
                    Ok(value)
                }
                Err(Signal::Trap(error)) => {
                    if error.kind == TrapKind::DoublePanic {
                        return Err(Signal::Trap(error));
                    }
                    let mut pending = Some(error);
                    self.drop_value(Value::Class(object), &expr.ty, &mut pending, span)
                        .map_err(Signal::Trap)?;
                    Err(Signal::Trap(pending.unwrap()))
                }
                Err(signal) => Err(signal),
            }
        } else {
            self.call(id, args, span)
        }
    }
}
