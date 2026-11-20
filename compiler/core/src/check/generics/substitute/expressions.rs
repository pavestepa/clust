use super::*;

impl Substitution {
    pub(super) fn block(&mut self, block: &mut a::Block) {
        self.shadowed.push(Default::default());
        for statement in &mut block.statements {
            match &mut statement.kind {
                a::StatementKind::Binding {
                    pattern, ty, value, ..
                } => {
                    if let Some(ty) = ty {
                        self.ty(ty);
                    }
                    self.expr(value);
                    self.pattern(pattern);
                }
                a::StatementKind::Expression(value) | a::StatementKind::Return(Some(value)) => {
                    self.expr(value)
                }
                a::StatementKind::Unsafe(block) => self.block(block),
                _ => (),
            }
        }
        self.shadowed.pop();
    }
}

impl Substitution {
    pub fn expr(&mut self, expr: &mut a::Expr) {
        if let a::ExprKind::Name(name) = &expr.kind {
            if !self.shadowed.iter().any(|scope| scope.contains(name)) {
                if let Some(
                    Type::Record { name: actual, .. }
                    | Type::Class { name: actual, .. }
                    | Type::Enum { name: actual, .. },
                ) = self.types.get(name)
                {
                    *expr = a::Expr {
                        kind: a::ExprKind::Name(actual.clone()),
                        span: expr.span,
                    };
                    return;
                }
                if let Some(value) = self.constants.get(name) {
                    *expr = number(*value, expr.span);
                    return;
                }
            }
        }
        match &mut expr.kind {
            a::ExprKind::Specialize { value, arguments } => {
                self.expr(value);
                self.arguments(arguments);
            }
            a::ExprKind::Group(value)
            | a::ExprKind::Unary { value, .. }
            | a::ExprKind::Member { value, .. } => self.expr(value),
            a::ExprKind::Tuple(values) | a::ExprKind::Array(values) => {
                for value in values {
                    self.expr(value);
                }
            }
            a::ExprKind::Construct { ty, fields } => {
                self.ty(ty);
                for (_, value) in fields {
                    self.expr(value);
                }
            }
            a::ExprKind::New { ty, arguments } => {
                self.ty(ty);
                for arg in arguments {
                    self.expr(arg);
                }
            }
            a::ExprKind::Call {
                callee,
                types,
                arguments,
            } => {
                self.expr(callee);
                self.arguments(types);
                for arg in arguments {
                    self.expr(arg);
                }
            }
            a::ExprKind::Index { value, index } => {
                self.expr(value);
                self.expr(index);
            }
            a::ExprKind::Binary { left, right, .. }
            | a::ExprKind::Assign {
                place: left,
                value: right,
                ..
            } => {
                self.expr(left);
                self.expr(right);
            }
            a::ExprKind::Cast { value, ty } => {
                self.expr(value);
                self.ty(ty);
            }
            a::ExprKind::If {
                condition,
                then_block,
                else_branch,
            } => {
                self.expr(condition);
                self.block(then_block);
                if let Some(branch) = else_branch {
                    self.expr(branch);
                }
            }
            a::ExprKind::While { condition, body } => {
                self.expr(condition);
                self.block(body);
            }
            a::ExprKind::Branch(block) => self.block(block),
            a::ExprKind::Match { value, arms } => {
                self.expr(value);
                for arm in arms {
                    self.shadowed.push(Default::default());
                    self.pattern(&mut arm.pattern);
                    if let Some(guard) = &mut arm.guard {
                        self.expr(guard);
                    }
                    self.expr(&mut arm.value);
                    self.shadowed.pop();
                }
            }
            _ => (),
        }
    }
}
