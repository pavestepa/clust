use super::*;
impl Builder {
    pub(super) fn conditional(
        &mut self,
        expr: &Expr,
        condition: &Expr,
        yes: &h::Block,
        no: &Option<h::Block>,
        borrowed: Option<loans::Access>,
    ) -> Operand {
        let condition = self.expr(condition);
        let yes_id = self.block_id(yes.span);
        let no_id = self.block_id(expr.span);
        let end = self.block_id(expr.span);
        let result = self.temp(&expr.ty, expr.span);
        self.function.locals[result].view = borrowed;
        self.end(Terminator::Branch {
            condition,
            yes: yes_id,
            no: no_id,
        });
        self.current = Some(yes_id);
        let value = self.block_mode(yes, borrowed);
        let yes_reaches = self.current.is_some();
        self.result_into(value, result, borrowed, expr.span);
        self.end(Terminator::Goto(end));
        self.current = Some(no_id);
        let value = no.as_ref().map_or(Operand::Constant(Value::Unit), |body| {
            self.block_mode(body, borrowed)
        });
        let no_reaches = self.current.is_some();
        self.result_into(value, result, borrowed, expr.span);
        self.end(Terminator::Goto(end));
        self.current = (yes_reaches || no_reaches).then_some(end);
        Operand::Local(result)
    }
    pub(super) fn while_loop(&mut self, expr: &Expr, condition: &Expr, body: &h::Block) -> Operand {
        let temporary_start = self.function.locals.len();
        let test = self.block_id(condition.span);
        let run = self.block_id(body.span);
        let end = self.block_id(expr.span);
        self.end(Terminator::Goto(test));
        self.current = Some(test);
        let condition = self.expr(condition);
        self.end_temporaries(temporary_start, Some(&condition), expr.span);
        self.end(Terminator::Branch {
            condition,
            yes: run,
            no: end,
        });
        self.loops
            .push((test, end, self.scopes.len(), temporary_start));
        self.current = Some(run);
        self.block(body);
        self.end(Terminator::Goto(test));
        self.loops.pop();
        self.current = Some(end);
        Operand::Constant(Value::Unit)
    }
    pub(super) fn match_arms(
        &mut self,
        expr: &Expr,
        arms: &[h::MatchArm],
        borrowed: Option<loans::Access>,
    ) -> Operand {
        let end = self.block_id(expr.span);
        let result = self.temp(&expr.ty, expr.span);
        self.function.locals[result].view = borrowed;
        let mut reaches = false;
        for arm in arms {
            let condition = self.expr(&arm.condition);
            let body = self.block_id(arm.body.span);
            let next = self.block_id(expr.span);
            self.end(Terminator::Branch {
                condition,
                yes: body,
                no: next,
            });
            self.current = Some(body);
            for binding in &arm.guard_bindings {
                self.statement(binding, expr.span);
            }
            if let Some(guard) = &arm.guard {
                let condition = self.expr(guard);
                let accepted = self.block_id(guard.span);
                self.end(Terminator::Branch {
                    condition,
                    yes: accepted,
                    no: next,
                });
                self.current = Some(accepted);
            }
            for binding in &arm.bindings {
                self.statement(binding, expr.span);
            }
            let value = self.block_mode(&arm.body, borrowed);
            reaches |= self.current.is_some();
            self.result_into(value, result, borrowed, expr.span);
            self.end(Terminator::Goto(end));
            self.current = Some(next);
        }
        self.end(Terminator::Unreachable);
        self.current = reaches.then_some(end);
        Operand::Local(result)
    }
    pub(super) fn short_circuit(
        &mut self,
        expr: &Expr,
        operator: &&'static str,
        left: &Expr,
        right: &Expr,
    ) -> Operand {
        let condition = self.expr(left);
        let rhs = self.block_id(right.span);
        let constant = self.block_id(expr.span);
        let end = self.block_id(expr.span);
        let result = self.temp(&Type::Bool, expr.span);
        let (yes, no) = if *operator == "&&" {
            (rhs, constant)
        } else {
            (constant, rhs)
        };
        self.end(Terminator::Branch { condition, yes, no });
        self.current = Some(rhs);
        let value = self.expr(right);
        self.evaluate_into(Operation::Use(value), Place::local(result), right.span);
        self.end(Terminator::Goto(end));
        self.current = Some(constant);
        self.evaluate_into(
            Operation::Use(Operand::Constant(Value::Bool(*operator == "||"))),
            Place::local(result),
            expr.span,
        );
        self.end(Terminator::Goto(end));
        self.current = Some(end);
        Operand::Local(result)
    }
    pub(super) fn assignment(
        &mut self,
        expr: &Expr,
        place: &Expr,
        operator: &&'static str,
        value: &Expr,
    ) -> Operand {
        let mut target = self.place(place);
        if matches!(target.projections.first(), Some(Projection::Dereference)) {
            let id = self.temp(&place.ty, place.span);
            self.function.locals[id].view = Some(loans::Access::Mutable);
            self.evaluate_into(
                Operation::CapturePlace(target),
                Place::local(id),
                place.span,
            );
            target = Place {
                local: id,
                projections: vec![Projection::Dereference],
            };
        }
        self.check_place(&target, place.span);
        let old = (*operator != "=")
            .then(|| self.evaluate(Operation::Copy(target.clone()), &place.ty, place.span));
        let value = self.expr(value);
        let operation = if let Some(old) = old {
            Operation::Binary(operator.trim_end_matches('='), old, value)
        } else {
            Operation::Use(value)
        };
        self.evaluate_into(operation, target, expr.span);
        Operand::Constant(Value::Unit)
    }
}
