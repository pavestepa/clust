use super::*;

impl Engine<'_> {
    pub(super) fn place(&mut self, expr: &Expr, frame: &mut Frame) -> Result<Place> {
        let place = match &expr.kind {
            ExprKind::StoredView(storage) => {
                let Value::View(value) = self.expr(storage, frame)? else {
                    return Err(trap(TrapKind::InvalidIr, expr.span));
                };
                value.0
            }
            ExprKind::BorrowedCall { .. }
            | ExprKind::Unary("storage_view" | "storage_view_mut", _)
            | ExprKind::CallIndirect(_, _, crate::hir::CallContract { view: Some(_), .. }) => {
                let Value::View(value) = self.expr_mode(expr, frame, false)? else {
                    return Err(trap(TrapKind::InvalidIr, expr.span));
                };
                value.0
            }
            ExprKind::View(id) => frame
                .views
                .get(id)
                .cloned()
                .ok_or_else(|| trap(TrapKind::InvalidIr, expr.span))?,
            ExprKind::Local(local) => Place {
                slot: frame.locals[*local].clone(),
                projections: Vec::new(),
            },
            ExprKind::Index(base, index) => {
                let mut place = self.place(base, frame)?;
                let index = index_value(self.expr(index, frame)?, expr.span)?;
                place.projections.push(crate::mir::Projection::Field(index));
                place
            }
            ExprKind::Payload(base, variant) => {
                let mut place = self.place(base, frame)?;
                place
                    .projections
                    .push(crate::mir::Projection::Variant(*variant));
                return Ok(place);
            }
            ExprKind::Field(base, index) => {
                let mut place = self.place(base, frame)?;
                place
                    .projections
                    .push(crate::mir::Projection::Field(*index));
                place
            }
            _ => {
                let value = self.expr(expr, frame)?;
                let slot = memory::typed_slot(expr.ty.clone(), self.clock.clone());
                let place = Place {
                    slot: slot.clone(),
                    projections: vec![],
                };
                place.write(value).map_err(|kind| trap(kind, expr.span))?;
                frame.temporary_places.push(slot);
                place
            }
        };
        let place = place.normalized().map_err(|kind| trap(kind, expr.span))?;
        // Check bounds before evaluating the RHS; each index is evaluated once.
        let _ = place.read().map_err(|kind| trap(kind, expr.span))?;
        Ok(place)
    }
}
