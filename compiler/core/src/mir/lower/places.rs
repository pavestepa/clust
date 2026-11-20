use super::*;

impl Builder {
    pub(super) fn place(&mut self, expr: &Expr) -> Place {
        match &expr.kind {
            ExprKind::StoredView(storage) => {
                let source = self.place(storage);
                let id = self.temp(&expr.ty, expr.span);
                self.function.locals[id].view = Some(loans::Access::Readonly);
                self.evaluate_into(Operation::LoadView(source), Place::local(id), expr.span);
                Place {
                    local: id,
                    projections: vec![Projection::Dereference],
                }
            }
            ExprKind::BorrowedCall { .. }
            | ExprKind::Unary("storage_view" | "storage_view_mut", _)
            | ExprKind::CallIndirect(_, _, h::CallContract { view: Some(_), .. }) => {
                let Operand::Local(id) = self.expr_mode(expr, None) else {
                    unreachable!()
                };
                Place {
                    local: id,
                    projections: vec![Projection::Dereference],
                }
            }
            ExprKind::Local(id) => Place::local(*id),
            ExprKind::View(id) => Place {
                local: *id,
                projections: vec![Projection::Dereference],
            },
            ExprKind::Field(base, index) => {
                let mut place = self.place(base);
                place.projections.push(Projection::Field(*index));
                place
            }
            ExprKind::Payload(base, variant) => {
                let mut place = self.place(base);
                place.projections.push(Projection::Variant(*variant));
                place
            }
            ExprKind::Index(base, index) => {
                let mut place = self.place(base);
                if let ExprKind::Value(Value::Integer(value)) = &index.kind {
                    place
                        .projections
                        .push(Projection::ConstantIndex(value.bits() as u64));
                    self.check_place(&place, expr.span);
                    return place;
                }
                let value = self.expr(index);
                let id = self.temp(&index.ty, index.span);
                self.evaluate_into(Operation::Use(value), Place::local(id), index.span);
                place.projections.push(Projection::Index(id));
                self.check_place(&place, expr.span);
                place
            }
            _ => {
                let value = self.expr(expr);
                let id = self.temp(&expr.ty, expr.span);
                self.evaluate_into(Operation::Use(value), Place::local(id), expr.span);
                Place::local(id)
            }
        }
    }
}
