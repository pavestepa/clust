use super::*;

impl Contract {
    pub(super) fn expr(&mut self, value: &a::Expr, state: &mut State) -> Result<()> {
        use a::ExprKind as E;
        match &value.kind {
            E::Closure(_) => return Err(error(value.span, "E0405", "closure was not lowered")),

            E::Call {
                callee, arguments, ..
            } if matches!(&callee.kind,E::Member {value,name} if name=="new" && matches!(&value.kind,E::Name(n) if n=="super")) =>
            {
                if !self.constructor || !self.base {
                    return Err(error(
                        value.span,
                        "E0393",
                        "super.new is only valid inside a derived constructor",
                    ));
                }
                if self.repeated {
                    return Err(error(
                        value.span,
                        "E0393",
                        "base initialization cannot occur in a loop or match guard",
                    ));
                }
                if state.counts.contains(&1) {
                    return Err(error(
                        value.span,
                        "E0393",
                        "direct base may already be initialized on this path",
                    ));
                }
                for argument in arguments {
                    self.expr(argument, state)?;
                }
                // Argument evaluation may itself contain a base call.
                if state.counts.contains(&1) {
                    return Err(error(
                        value.span,
                        "E0393",
                        "direct base is initialized by a constructor argument",
                    ));
                }
                state.counts = BTreeSet::from([1]);
            }
            E::Name(name)
                if name == "this"
                    || (state.fields.contains(name) && !state.locals.contains(name)) =>
            {
                if self.constructor && self.base && state.counts.contains(&0) {
                    return Err(error(
                        value.span,
                        "E0393",
                        "initialize the direct base before accessing self or derived fields",
                    ));
                }
            }
            E::If {
                condition,
                then_block,
                else_branch,
            } => {
                self.expr(condition, state)?;
                let mut yes = state.clone();
                let mut no = state.clone();
                self.block(then_block, &mut yes)?;
                if let Some(other) = else_branch {
                    self.expr(other, &mut no)?;
                }
                state.counts = yes.counts.union(&no.counts).copied().collect();
            }
            E::While { condition, body } => {
                let previous = self.repeated;
                self.repeated = true;
                self.expr(condition, state)?;
                self.block(body, &mut state.clone())?;
                self.repeated = previous;
            }
            E::Match { value, arms } => {
                self.expr(value, state)?;
                let mut counts = BTreeSet::new();
                for arm in arms {
                    let mut branch = state.clone();
                    bind(&arm.pattern, &mut branch.locals);
                    if let Some(guard) = &arm.guard {
                        let previous = self.repeated;
                        self.repeated = true;
                        self.expr(guard, &mut branch)?;
                        self.repeated = previous;
                    }
                    self.expr(&arm.value, &mut branch)?;
                    counts.extend(branch.counts);
                }
                state.counts = counts;
            }
            E::Binary {
                operator,
                left,
                right,
            } => {
                self.expr(left, state)?;
                if matches!(*operator, "&&" | "||") {
                    let mut branch = state.clone();
                    self.expr(right, &mut branch)?;
                    state.counts.extend(branch.counts);
                } else {
                    self.expr(right, state)?;
                }
            }
            E::Group(value)
            | E::Unary { value, .. }
            | E::Cast { value, .. }
            | E::Specialize { value, .. }
            | E::Member { value, .. } => self.expr(value, state)?,
            E::Tuple(values)
            | E::Array(values)
            | E::New {
                arguments: values, ..
            } => {
                for value in values {
                    self.expr(value, state)?;
                }
            }
            E::Construct { fields, .. } => {
                for (_, value) in fields {
                    self.expr(value, state)?;
                }
            }
            E::Call {
                callee, arguments, ..
            } => {
                self.expr(callee, state)?;
                for arg in arguments {
                    self.expr(arg, state)?;
                }
            }
            E::Assign { place, value, .. } => {
                self.expr(place, state)?;
                self.expr(value, state)?;
            }
            E::Index { value, index } => {
                self.expr(value, state)?;
                self.expr(index, state)?;
            }
            E::Branch(body) => self.block(body, state)?,
            E::Name(_) | E::Literal(_) => {}
        }
        Ok(())
    }
}
