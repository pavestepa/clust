use super::*;

impl Flow {
    pub(super) fn read(&mut self, value: &a::Expr, target: &Place) {
        if !self.live || self.stopped {
            return;
        }
        if let Some(p) = as_place(value) {
            if overlaps(&p, target) {
                self.used = true;
            }
            self.address(value, target);
            return;
        }
        match &value.kind {
            a::ExprKind::Assign {
                operator,
                place,
                value,
            } => {
                self.address(place, target);
                if *operator != "=" {
                    self.read(place, target);
                }
                self.read(value, target);
                if as_place(place).is_some_and(|p| covers(&p, target)) {
                    self.live = false;
                }
            }
            a::ExprKind::If {
                condition,
                then_block,
                else_branch,
            } => {
                self.read(condition, target);
                let mut yes = *self;
                yes.block(then_block, target);
                let mut no = *self;
                if let Some(no_expr) = else_branch {
                    no.read(no_expr, target);
                }
                *self = yes.join(no);
            }
            a::ExprKind::While { condition, body } => {
                self.read(condition, target);
                let mut run = *self;
                run.block(body, target);
                self.used |= run.used;
            }
            a::ExprKind::Match { value, arms } => {
                self.read(value, target);
                let mut merged = Self {
                    live: false,
                    used: self.used,
                    stopped: true,
                };
                for arm in arms {
                    let mut next = *self;
                    if let Some(g) = &arm.guard {
                        next.read(g, target);
                    }
                    next.read(&arm.value, target);
                    merged = merged.join(next);
                }
                *self = merged;
            }
            a::ExprKind::Branch(body) => self.block(body, target),
            a::ExprKind::Binary {
                operator,
                left,
                right,
            } => {
                self.read(left, target);
                if ["&&", "||"].contains(operator) {
                    let mut rhs = *self;
                    rhs.read(right, target);
                    *self = self.join(rhs);
                } else {
                    self.read(right, target);
                }
            }
            a::ExprKind::Tuple(v) | a::ExprKind::Array(v) => {
                for e in v {
                    self.read(e, target)
                }
            }
            a::ExprKind::Construct { fields, .. } => {
                for (_, e) in fields {
                    self.read(e, target)
                }
            }
            a::ExprKind::Call {
                callee, arguments, ..
            } => {
                self.read(callee, target);
                for e in arguments {
                    self.read(e, target)
                }
            }
            a::ExprKind::New { arguments, .. } => {
                for e in arguments {
                    self.read(e, target)
                }
            }
            a::ExprKind::Group(value)
            | a::ExprKind::Unary { value, .. }
            | a::ExprKind::Member { value, .. }
            | a::ExprKind::Specialize { value, .. }
            | a::ExprKind::Cast { value, .. } => self.read(value, target),
            a::ExprKind::Index { value, index } => {
                self.read(value, target);
                self.read(index, target)
            }
            _ => (),
        }
    }
}
