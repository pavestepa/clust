use super::*;

impl Rewrite<'_> {
    pub(super) fn block(&mut self, block: &mut Block) {
        self.scopes.push(BTreeSet::new());
        for statement in &mut block.statements {
            match &mut statement.kind {
                StatementKind::Binding {
                    pattern, ty, value, ..
                } => {
                    if let Some(ty) = ty {
                        self.ty(ty);
                    }
                    self.expr(value);
                    self.pattern(pattern);
                }
                StatementKind::Expression(value) => self.expr(value),
                StatementKind::Return(Some(value)) => self.expr(value),
                StatementKind::Unsafe(block) => self.block(block),
                _ => {}
            }
        }
        self.scopes.pop();
    }
}

impl Rewrite<'_> {
    pub(super) fn expr(&mut self, value: &mut Expr) {
        fn expression_path(value: &Expr) -> Option<Vec<String>> {
            match &value.kind {
                ExprKind::Name(name) => Some(vec![name.clone()]),
                ExprKind::Member { value, name } => {
                    let mut path = expression_path(value)?;
                    path.push(name.clone());
                    Some(path)
                }
                _ => None,
            }
        }
        if let Some(mut path) = expression_path(value) {
            let original = path.clone();
            self.path(&mut path, value.span);
            if path != original {
                let mut rewritten = Expr {
                    kind: ExprKind::Name(path[0].clone()),
                    span: value.span,
                };
                for name in path.into_iter().skip(1) {
                    rewritten = Expr {
                        kind: ExprKind::Member {
                            value: Box::new(rewritten),
                            name,
                        },
                        span: value.span,
                    };
                }
                *value = rewritten;
                return;
            }
        }
        match &mut value.kind {
            ExprKind::Closure(function) => self.function(function),
            ExprKind::Name(name) => self.name(name),
            ExprKind::Literal(_) => {}
            ExprKind::Group(value)
            | ExprKind::Unary { value, .. }
            | ExprKind::Member { value, .. } => self.expr(value),
            ExprKind::Specialize { value, arguments } => {
                self.expr(value);
                self.arguments(arguments);
            }
            ExprKind::Tuple(values) | ExprKind::Array(values) => {
                for value in values {
                    self.expr(value);
                }
            }
            ExprKind::Construct { ty, fields } => {
                self.ty(ty);
                for (_, value) in fields {
                    self.expr(value);
                }
            }
            ExprKind::New { ty, arguments } => {
                self.ty(ty);
                for value in arguments {
                    self.expr(value);
                }
            }
            ExprKind::Call {
                callee,
                types,
                arguments,
            } => {
                self.expr(callee);
                self.arguments(types);
                for value in arguments {
                    self.expr(value);
                }
            }
            ExprKind::Index { value, index } => {
                self.expr(value);
                self.expr(index);
            }
            ExprKind::Binary { left, right, .. } => {
                self.expr(left);
                self.expr(right);
            }
            ExprKind::Assign { place, value, .. } => {
                self.expr(place);
                self.expr(value);
            }
            ExprKind::Cast { value, ty } => {
                self.expr(value);
                self.ty(ty);
            }
            ExprKind::If {
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
            ExprKind::While { condition, body } => {
                self.expr(condition);
                self.block(body);
            }
            ExprKind::Branch(block) => self.block(block),
            ExprKind::Match { value, arms } => {
                self.expr(value);
                for arm in arms {
                    self.scopes.push(BTreeSet::new());
                    self.pattern(&mut arm.pattern);
                    if let Some(guard) = &mut arm.guard {
                        self.expr(guard);
                    }
                    self.expr(&mut arm.value);
                    self.scopes.pop();
                }
            }
        }
    }
}
