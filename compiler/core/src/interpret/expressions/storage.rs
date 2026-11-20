use super::*;
impl Engine<'_> {
    pub(super) fn storage_view(
        &mut self,
        values: &Expr,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        let ExprKind::Tuple(args) = &values.kind else {
            return Err(trap(TrapKind::InvalidIr, span));
        };
        let storage = self
            .place(&args[0], frame)?
            .read()
            .map_err(|kind| trap(kind, span))?;
        let index = self.expr(&args[1], frame)?;
        self.allocations
            .storage_view(storage, index)
            .map_err(|kind| trap(kind, span))
    }
    pub(super) fn address_conversion(
        &mut self,
        op: &&'static str,
        operand: &Expr,
        expr: &Expr,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        let value = self.expr(operand, frame)?;
        if *op == "raw_recover" {
            self.allocations.recover(value, &expr.ty)
        } else {
            self.allocations
                .expose(value, *op == "raw_expose_mut", &expr.ty)
        }
        .map_err(|kind| trap(kind, span))
    }
    pub(super) fn allocate(
        &mut self,
        count: &Expr,
        expr: &Expr,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        let Value::Integer(count) = self.expr(count, frame)? else {
            return Err(trap(TrapKind::InvalidIr, span));
        };
        let Type::Pointer { pointee, .. } = &expr.ty else {
            return Err(trap(TrapKind::InvalidIr, span));
        };
        self.allocations
            .allocate(pointee, count.bits() as u64, self.clock.clone())
            .map_err(|kind| trap(kind, span))
    }
    pub(super) fn storage_operation(
        &mut self,
        op: &&'static str,
        args: &Expr,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        if matches!(*op, "raw_drop" | "storage_drop" | "storage_drop_range") {
            let Type::Tuple(fields) = &args.ty else {
                return Err(trap(TrapKind::InvalidIr, span));
            };
            let Type::Pointer { pointee, .. } = &fields[0] else {
                return Err(trap(TrapKind::InvalidIr, span));
            };
            let value = self.expr(args, frame)?;
            if *op == "storage_drop_range" {
                let (pointer, start, count) = self
                    .allocations
                    .drop_range_arguments(value)
                    .map_err(|kind| trap(kind, span))?;
                let mut pending = None;
                for index in start..start + count {
                    let argument = Value::Aggregate(vec![
                        pointer.clone(),
                        Value::Integer(Integer::from_bits(
                            clust_semantics::IntegerType::Usize,
                            index as u128,
                        )),
                    ]);
                    let payload = self
                        .allocations
                        .operation(
                            if *op == "raw_drop" {
                                "raw_take"
                            } else {
                                "storage_take"
                            },
                            argument,
                        )
                        .map_err(|kind| trap(kind, span))?;
                    self.drop_value(payload, pointee, &mut pending, span)
                        .map_err(Signal::Trap)?;
                }
                return pending.map_or(Ok(Value::Unit), |e| Err(Signal::Trap(e)));
            }
            let value = self
                .allocations
                .operation(
                    if *op == "raw_drop" {
                        "raw_take"
                    } else {
                        "storage_take"
                    },
                    value,
                )
                .map_err(|kind| trap(kind, span))?;
            let mut pending = None;
            self.drop_value(value, pointee, &mut pending, span)
                .map_err(Signal::Trap)?;
            return pending.map_or(Ok(Value::Unit), |e| Err(Signal::Trap(e)));
        }
        let args = self.expr(args, frame)?;
        self.allocations
            .operation(op, args)
            .map_err(|kind| trap(kind, span))
    }
}
