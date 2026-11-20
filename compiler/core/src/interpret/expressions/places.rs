use super::*;
impl Engine<'_> {
    pub(super) fn read_view(
        &mut self,
        expr: &Expr,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        let place = self.place(expr, frame)?;
        let value = place.read().map_err(|kind| trap(kind, span))?;
        if expr.ty.copyable() {
            memory::copy_value(&value, self.clock.clone()).map_err(|kind| trap(kind, span))
        } else {
            Err(trap(TrapKind::InvalidIr, span))
        }
    }
    pub(super) fn identity(
        &mut self,
        left: &Expr,
        right: &Expr,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        let Value::Class(left) = self
            .place(left, frame)?
            .read()
            .map_err(|kind| trap(kind, span))?
        else {
            return Err(trap(TrapKind::InvalidIr, span));
        };
        let Value::Class(right) = self
            .place(right, frame)?
            .read()
            .map_err(|kind| trap(kind, span))?
        else {
            return Err(trap(TrapKind::InvalidIr, span));
        };
        Ok(Value::Bool(left.same_identity(&right)))
    }
    pub(super) fn take(&mut self, value: &Expr, frame: &mut Frame, span: Span) -> Result<Value> {
        let place = self.place(value, frame)?;
        let value = place.read().map_err(|kind| trap(kind, span))?;
        place.forget().map_err(|kind| trap(kind, span))?;
        Ok(value)
    }
    pub(super) fn read_place(
        &mut self,
        expr: &Expr,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        let place = self.place(expr, frame)?;
        let value = place.read().map_err(|kind| trap(kind, span))?;
        if !expr.ty.copyable() {
            place.forget().map_err(|kind| trap(kind, span))?;
            Ok(value)
        } else {
            memory::copy_value(&value, self.clock.clone()).map_err(|kind| trap(kind, span))
        }
    }
    pub(super) fn assign(
        &mut self,
        place: &Expr,
        operator: &&'static str,
        value: &Expr,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        let ty = place.ty.clone();
        let place = self.place(place, frame)?;
        let old = if *operator != "=" {
            Some(place.read().map_err(|kind| trap(kind, span))?)
        } else {
            None
        };
        let value = self.expr(value, frame)?;
        let value = if let Some(old) = old {
            binary(operator.trim_end_matches('='), old, value, self.checks)
                .map_err(|kind| trap(kind, span))?
        } else {
            value
        };
        let mut pending = None;
        self.drop_place(place.clone(), &ty, &mut pending, span)
            .map_err(Signal::Trap)?;
        if pending.is_some() {
            self.drop_value(value, &ty, &mut pending, span)
                .map_err(Signal::Trap)?;
            return Err(Signal::Trap(pending.unwrap()));
        }
        place.write(value).map_err(|kind| trap(kind, span))?;
        Ok(Value::Unit)
    }
}
