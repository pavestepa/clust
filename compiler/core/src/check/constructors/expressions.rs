use super::*;

impl Constructor {
    pub(super) fn expr(&self, value: &a::Expr, state: &mut State) -> Result<()> {
        if !state.reachable {
            return Ok(());
        }
        if let Some(field) = self.field(value, state) {
            if !state.initialized.contains(field) {
                return Err(error(
                    value.span,
                    "E0393",
                    format!("constructor reads uninitialized field '{field}'"),
                ));
            }
            return Ok(());
        }
        match &value.kind {
            a::ExprKind::Closure(_) => {
                return Err(error(value.span, "E0405", "closure was not lowered"))
            }

            a::ExprKind::Name(name)
                if name == "this"
                    || (self.methods.contains(name) && !state.locals.contains(name)) =>
            {
                self.complete(state, value.span)?
            }
            a::ExprKind::Assign {
                operator,
                place,
                value: rhs,
            } => {
                if let Some(field) = self.field(place, state) {
                    if *operator != "=" {
                        self.expr(place, state)?;
                    }
                    self.expr(rhs, state)?;
                    if state.reachable {
                        state.initialized.insert(field.into());
                    }
                } else {
                    self.expr(place, state)?;
                    self.expr(rhs, state)?;
                }
            }
            a::ExprKind::Group(v)
            | a::ExprKind::Unary { value: v, .. }
            | a::ExprKind::Cast { value: v, .. }
            | a::ExprKind::Specialize { value: v, .. } => self.expr(v, state)?,
            a::ExprKind::Member { value: v, .. } => self.expr(v, state)?,
            a::ExprKind::Index { value: v, index } => {
                self.expr(v, state)?;
                self.expr(index, state)?;
            }
            a::ExprKind::Binary {
                operator,
                left,
                right,
            } => {
                self.expr(left, state)?;
                if *operator == "&&" || *operator == "||" {
                    let mut conditional = state.clone();
                    self.expr(right, &mut conditional)?;
                    // The right operand is not necessarily evaluated.
                } else {
                    self.expr(right, state)?;
                }
            }
            a::ExprKind::Tuple(values)
            | a::ExprKind::Array(values)
            | a::ExprKind::New {
                arguments: values, ..
            } => {
                for value in values {
                    self.expr(value, state)?;
                }
            }
            a::ExprKind::Construct { fields, .. } => {
                for (_, value) in fields {
                    self.expr(value, state)?;
                }
            }
            a::ExprKind::Call {
                callee, arguments, ..
            } => {
                if let a::ExprKind::Name(name) = &callee.kind {
                    if self.methods.contains(name) && !state.locals.contains(name) {
                        self.complete(state, callee.span)?;
                    }
                }
                self.expr(callee, state)?;
                for value in arguments {
                    self.expr(value, state)?;
                }
            }
            a::ExprKind::If {
                condition,
                then_block,
                else_branch,
            } => {
                self.expr(condition, state)?;
                let mut yes = state.clone();
                self.block(then_block, &mut yes)?;
                let mut no = state.clone();
                if let Some(other) = else_branch {
                    self.expr(other, &mut no)?;
                }
                join(state, yes, no);
            }
            a::ExprKind::While { condition, body } => {
                self.expr(condition, state)?;
                let mut iteration = state.clone();
                self.block(body, &mut iteration)?;
                // A loop may execute zero times, and break paths do not prove
                // initialization on all exits. Typed CFG checking can refine it.
            }
            a::ExprKind::Branch(body) => self.block(body, state)?,
            a::ExprKind::Match {
                value: scrutinee,
                arms,
            } => {
                self.expr(scrutinee, state)?;
                let before = state.clone();
                let mut joined: Option<State> = None;
                let mut exhaustive = false;
                for arm in arms {
                    let mut branch = before.clone();
                    bindings(&arm.pattern, &mut branch.locals);
                    if let Some(guard) = &arm.guard {
                        self.expr(guard, &mut branch)?;
                    }
                    self.expr(&arm.value, &mut branch)?;
                    branch.locals = before.locals.clone();
                    exhaustive |= arm.guard.is_none()
                        && matches!(
                            arm.pattern.kind,
                            a::PatternKind::Wildcard | a::PatternKind::Binding { .. }
                        );
                    if let Some(previous) = joined.take() {
                        let mut result = before.clone();
                        join(&mut result, previous, branch);
                        joined = Some(result);
                    } else {
                        joined = Some(branch);
                    }
                }
                if let Some(after) = joined {
                    if exhaustive {
                        *state = after;
                    } else {
                        join(state, before, after);
                    }
                }
            }
            a::ExprKind::Name(_) | a::ExprKind::Literal(_) => {}
        }
        Ok(())
    }
}
