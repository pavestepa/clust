use super::*;
impl Engine<'_> {
    pub(super) fn tag(&mut self, value: &Expr, frame: &mut Frame, span: Span) -> Result<Value> {
        let Value::Enum { variant, .. } = self
            .place(value, frame)?
            .read()
            .map_err(|kind| trap(kind, span))?
        else {
            return Err(trap(TrapKind::InvalidIr, span));
        };
        Ok(Value::Integer(Integer::from_bits(
            clust_semantics::IntegerType::U32,
            variant as u128,
        )))
    }
    pub(super) fn match_value(
        &mut self,
        arms: &[MatchArm],
        frame: &mut Frame,
        borrowed: bool,
        span: Span,
    ) -> Result<Value> {
        for arm in arms {
            if self.expr(&arm.condition, frame)? != Value::Bool(true) {
                continue;
            }
            self.block(
                &Block {
                    construction: vec![],
                    storage: vec![],
                    statements: arm.guard_bindings.clone(),
                    result: Type::Unit,
                    diverges: false,
                    span,
                },
                frame,
            )?;
            if let Some(guard) = &arm.guard {
                if self.expr(guard, frame)? != Value::Bool(true) {
                    continue;
                }
            }
            self.block(
                &Block {
                    construction: vec![],
                    storage: vec![],
                    statements: arm.bindings.clone(),
                    result: Type::Unit,
                    diverges: false,
                    span,
                },
                frame,
            )?;
            return self.block_mode(&arm.body, frame, borrowed);
        }
        Err(trap(TrapKind::InvalidIr, span))
    }
}
