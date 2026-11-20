mod calls;
mod operators;
mod patterns;
mod places;
mod storage;
mod weak;
use super::*;

impl Engine<'_> {
    pub(super) fn expr(&mut self, expr: &Expr, frame: &mut Frame) -> Result<Value> {
        let value = self.expr_mode(expr, frame, false)?;
        if expr.ty.copyable() && !matches!(expr.ty, Type::StoredView(_)) {
            if let Value::View(view) = value {
                let value = view.0.read().map_err(|kind| trap(kind, expr.span))?;
                return memory::copy_value(&value, self.clock.clone())
                    .map_err(|kind| trap(kind, expr.span));
            }
        }
        Ok(value)
    }
}

impl Engine<'_> {
    pub(super) fn expr_mode(
        &mut self,
        expr: &Expr,
        frame: &mut Frame,
        borrowed: bool,
    ) -> Result<Value> {
        let already_panicking = self.active_panic;
        let first_temporary = frame.temporary_places.len();
        let result = self.expr_body(expr, frame, borrowed);
        let result = match result {
            Err(Signal::Trap(mut failure)) => {
                if already_panicking {
                    failure.kind = TrapKind::DoublePanic;
                }
                self.active_panic = true;
                Err(Signal::Trap(failure))
            }
            result => result,
        };
        if matches!(&result,Err(Signal::Trap(t)) if t.kind==TrapKind::DoublePanic) {
            return result;
        }
        let mut pending = match &result {
            Err(Signal::Trap(t)) => Some(t.clone()),
            _ => None,
        };
        for slot in frame.temporary_places.drain(first_temporary..).rev() {
            self.drop_place(
                Place {
                    slot: slot.clone(),
                    projections: vec![],
                },
                slot.ty(),
                &mut pending,
                expr.span,
            )
            .map_err(Signal::Trap)?;
        }
        if pending.is_some() {
            if let Ok(value) = result {
                self.drop_value(value, &expr.ty, &mut pending, expr.span)
                    .map_err(Signal::Trap)?;
            }
            Err(Signal::Trap(pending.unwrap()))
        } else {
            result
        }
    }
}

impl Engine<'_> {
    pub(super) fn expr_body(
        &mut self,
        expr: &Expr,
        frame: &mut Frame,
        borrowed: bool,
    ) -> Result<Value> {
        self.tick(expr.span)?;
        let span = expr.span;
        if borrowed
            && !matches!(
                expr.kind,
                ExprKind::Block(_) | ExprKind::If { .. } | ExprKind::Match(_)
            )
        {
            return Ok(Value::View(memory::ViewValue(
                self.place(expr, frame)?
                    .captured()
                    .map_err(|kind| trap(kind, span))?,
            )));
        }
        match &expr.kind {
            ExprKind::StoreView(value) => Ok(Value::View(memory::ViewValue(
                self.place(value, frame)?
                    .captured()
                    .map_err(|kind| trap(kind, span))?,
            ))),
            ExprKind::StoredView(_) => self.read_view(expr, frame, span),
            ExprKind::Identity(left, right) => self.identity(left, right, frame, span),
            ExprKind::Take(value) => self.take(value, frame, span),
            ExprKind::View(_)
            | ExprKind::Local(_)
            | ExprKind::Field(..)
            | ExprKind::Index(..)
            | ExprKind::Payload(..) => self.read_place(expr, frame, span),
            ExprKind::ArgumentView { .. } => Err(trap(TrapKind::InvalidIr, span)),
            ExprKind::Borrow { local, place } => {
                let place = self.place(place, frame)?;
                frame
                    .views
                    .insert(*local, place.captured().map_err(|kind| trap(kind, span))?);
                Ok(Value::Unit)
            }
            ExprKind::Block(block) => self.block_mode(block, frame, borrowed),
            ExprKind::Enum { variant, fields } => Ok(Value::Enum {
                variant: *variant,
                fields: self.aggregate(fields, frame, span)?,
            }),
            ExprKind::Tag(value) => self.tag(value, frame, span),
            ExprKind::Match(arms) => self.match_value(arms, frame, borrowed, span),

            ExprKind::Value(value) => Ok(value.clone()),
            ExprKind::Tuple(values) | ExprKind::Array(values) => {
                Ok(Value::Aggregate(self.aggregate(values, frame, span)?))
            }
            ExprKind::Unary("storage_view" | "storage_view_mut", values) => {
                self.storage_view(values, frame, span)
            }
            ExprKind::Unary(op @ ("raw_expose" | "raw_expose_mut" | "raw_recover"), operand) => {
                self.address_conversion(op, operand, expr, frame, span)
            }
            ExprKind::Unary("storage_allocate", count) => self.allocate(count, expr, frame, span),
            ExprKind::Unary(
                op @ ("raw_initialize"
                | "storage_initialize"
                | "storage_replace"
                | "storage_take"
                | "raw_take"
                | "raw_drop"
                | "storage_drop"
                | "storage_drop_range"
                | "storage_detach"
                | "storage_adopt"
                | "storage_release"
                | "storage_check_range"
                | "storage_move_slots"
                | "storage_address"
                | "storage_address_mut"),
                args,
            ) => self.storage_operation(op, args, frame, span),
            ExprKind::Unary("raw_addr" | "raw_addr_mut", value) => {
                memory::RawPointer::new(self.place(value, frame)?)
                    .map(Value::Pointer)
                    .map_err(|kind| trap(kind, span))
            }
            ExprKind::Unary("weak_identity", value) => self.weak_identity(value, frame, span),
            ExprKind::Unary("weak_alive", value) => {
                let token = self.expr(value, frame)?;
                Ok(Value::Bool(self.weak.get(&token, None).is_some()))
            }
            ExprKind::Binary("weak_access", token, callback) => {
                self.weak_access(token, callback, frame, span)
            }
            ExprKind::Unary(operator, value) => self.unary(operator, value, frame, span),
            ExprKind::Binary(operator, left, right) => {
                self.binary_value(operator, left, right, frame, span)
            }
            ExprKind::Cast(value) => cast(self.expr(value, frame)?, &expr.ty)
                .ok_or_else(|| trap(TrapKind::InvalidIr, span)),
            ExprKind::StorageCall { .. }
            | ExprKind::BorrowedCall { .. }
            | ExprKind::New { .. }
            | ExprKind::Call(..)
            | ExprKind::CallIndirect(..) => self.invoke(expr, frame, span),
            ExprKind::Assign {
                place,
                operator,
                value,
            } => self.assign(place, operator, value, frame, span),
            ExprKind::If { condition, yes, no } => {
                let Value::Bool(condition) = self.expr(condition, frame)? else {
                    return Err(trap(TrapKind::InvalidIr, span));
                };
                if condition {
                    self.block_mode(yes, frame, borrowed)
                } else if let Some(no) = no {
                    self.block_mode(no, frame, borrowed)
                } else {
                    Ok(Value::Unit)
                }
            }
            ExprKind::While { condition, body } => {
                loop {
                    self.tick(span)?;
                    let Value::Bool(condition) = self.expr(condition, frame)? else {
                        return Err(trap(TrapKind::InvalidIr, span));
                    };
                    if !condition {
                        break;
                    }
                    match self.block(body, frame) {
                        Ok(_) | Err(Signal::Continue) => {}
                        Err(Signal::Break) => break,
                        other => return other,
                    }
                }
                Ok(Value::Unit)
            }
        }
    }
}
