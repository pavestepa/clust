use super::*;
impl Engine<'_> {
    pub(super) fn weak_identity(
        &mut self,
        value: &Expr,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        let ExprKind::ArgumentView { place, .. } = &value.kind else {
            return Err(trap(TrapKind::InvalidIr, span));
        };
        let Value::Class(object) = self
            .place(place, frame)?
            .read()
            .map_err(|kind| trap(kind, span))?
        else {
            return Err(trap(TrapKind::InvalidIr, span));
        };
        self.weak.create(object).map_err(|kind| trap(kind, span))
    }
    pub(super) fn weak_access(
        &mut self,
        token: &Expr,
        callback: &Expr,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        let token = self.expr(token, frame)?;
        let Value::Function(id) = self.expr(callback, frame)? else {
            return Err(trap(TrapKind::InvalidIr, span));
        };
        let Type::Function { parameters, .. } = &callback.ty else {
            return Err(trap(TrapKind::InvalidIr, span));
        };
        if let Some((target, temporary)) = self.weak.access(
            &token,
            &parameters[0].1,
            &self.program.classes,
            self.clock.clone(),
        ) {
            let view = memory::class_reference(target.clone(), self.clock.clone());
            let value = self.call(id, vec![Argument::Borrowed(view)], span);
            if temporary {
                self.weak.invalidate(&target);
            }
            let value = value?;
            Ok(Value::Enum {
                variant: 1,
                fields: vec![value],
            })
        } else {
            Ok(Value::Enum {
                variant: 0,
                fields: vec![],
            })
        }
    }
}
