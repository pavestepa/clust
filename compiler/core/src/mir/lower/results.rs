use super::*;
impl Builder {
    pub(super) fn expr(&mut self, expr: &Expr) -> Operand {
        let result = self.expr_mode(expr, None);
        if expr.ty.copyable() {
            if let Operand::Local(local) = result {
                if self.function.locals[local].view.is_some() {
                    return self.evaluate(
                        Operation::Copy(Place {
                            local,
                            projections: vec![Projection::Dereference],
                        }),
                        &expr.ty,
                        expr.span,
                    );
                }
            }
        }
        result
    }
}

impl Builder {
    pub(super) fn result_into(
        &mut self,
        value: Operand,
        result: LocalId,
        borrowed: Option<loans::Access>,
        span: Span,
    ) {
        if self.current.is_none() {
            return;
        }
        let operation = if borrowed.is_some() {
            let Operand::Local(local) = value else {
                unreachable!()
            };
            Operation::Borrow(Place {
                local,
                projections: vec![Projection::Dereference],
            })
        } else {
            Operation::Use(value)
        };
        self.evaluate_into(operation, Place::local(result), span);
    }
}
