use super::*;

impl Checker<'_> {
    pub(super) fn match_expression(
        &mut self,
        value: &a::Expr,
        arms: &[a::Arm],
        expected: Option<&Type>,
        value_context: bool,
        span: Span,
    ) -> Result<h::Expr> {
        let match_storage_start = self.locals.len();
        let borrow =
            arms.iter().any(|arm| views::has_ref(&arm.pattern)) || self.needs_pattern_view(value);
        let value = self.expr(value, None, true)?;
        let borrow = borrow || (!value.ty.copyable() && views::view_root(&value));
        let mut prelude = Vec::new();
        let scrutinee = if borrow {
            if !views::place(&value) && !self.pending_result_place(&value) {
                return Err(error(
                    span,
                    "E0382",
                    "borrowed match requires an existing place",
                ));
            }
            let place = self.freeze_place(value.clone(), &mut prelude);
            let id = self.locals.len();
            self.locals.push(h::Local {
                name: "<match address>".into(),
                ty: place.ty.clone(),
                view: Some(crate::mir::loans::Access::Readonly),
                mutable: false,
                span,
            });
            prelude.push(h::Statement::Evaluate(h::Expr {
                kind: h::ExprKind::Borrow {
                    local: id,
                    place: Box::new(place.clone()),
                },
                ty: Type::Unit,
                span,
            }));
            place
        } else {
            let id = self.locals.len();
            self.locals.push(h::Local {
                view: None,
                name: "<match value>".into(),
                ty: value.ty.clone(),
                mutable: false,
                span,
            });
            prelude.push(h::Statement::Initialize(id, value.clone()));
            h::Expr {
                kind: h::ExprKind::Local(id),
                ty: value.ty.clone(),
                span,
            }
        };
        let mut variants = match &value.ty {
            Type::Enum { variants, .. } => vec![false; variants.len()],
            _ => Vec::new(),
        };
        let mut exhaustive = matches!(&value.ty,Type::Enum {variants,..} if variants.is_empty());
        let mut booleans = [false; 2];
        let mut checked = Vec::new();
        let mut result = expected.cloned();
        for arm in arms {
            if exhaustive {
                return Err(error(arm.span, "E0341", "unreachable match arm"));
            }
            let arm_storage_start = self.locals.len();
            let pattern = if borrow {
                views::inferred_pattern(&arm.pattern, false)
            } else {
                arm.pattern.clone()
            };
            self.scopes.push(BTreeMap::new());
            let previous = self.pattern_views;
            self.pattern_views = borrow;
            let mut bindings = Vec::new();
            let (condition, irrefutable, variant) =
                self.pattern(&pattern, &scrutinee, &mut bindings)?;
            let mut guard_bindings = Vec::new();
            let guard = if let Some(guard) = &arm.guard {
                // Guard bindings are readonly and exist before mutable/consuming body bindings.
                self.scopes.push(BTreeMap::new());
                let mut guard_pattern = pattern.clone();
                readonly_pattern(&mut guard_pattern);
                if !scrutinee.ty.copyable() {
                    guard_pattern = views::inferred_pattern(&guard_pattern, false);
                }
                self.pattern_views = true;
                self.pattern(&guard_pattern, &scrutinee, &mut guard_bindings)?;
                let guard = self.expr(guard, Some(&Type::Bool), true)?;
                self.scopes.pop();
                self.pattern_views = borrow;
                if !readonly_guard(&guard) {
                    return Err(error(guard.span, "E0343", "match guard must only read"));
                }
                Some(guard)
            } else {
                if let a::PatternKind::Literal {
                    value: a::Literal::Bool(b),
                    ..
                } = arm.pattern.kind
                {
                    booleans[b as usize] = true;
                }
                if let Some(index) = variant {
                    variants[index] = true;
                }
                exhaustive = irrefutable
                    || booleans.iter().all(|b| *b)
                    || !variants.is_empty() && variants.iter().all(|v| *v);
                None
            };
            self.pattern_views = previous;
            let mut body = self.branch(&arm.value, result.as_ref(), value_context)?;
            body.storage = (arm_storage_start..self.locals.len()).collect();
            if !body.diverges && value_context {
                if let Some(ty) = &result {
                    result = Some(storage::union(ty, &body.result, arm.span)?);
                } else {
                    result = Some(body.result.clone());
                }
            }
            self.scopes.pop();
            checked.push(h::MatchArm {
                guard_bindings,
                bindings,
                condition,
                guard,
                body,
            });
        }
        if !exhaustive {
            return Err(error(
                span,
                "E0340",
                "non-exhaustive match; add an unguarded fallback pattern",
            ));
        }
        let result = if value_context {
            result.unwrap_or(Type::Unit)
        } else {
            Type::Unit
        };
        for arm in &checked {
            if value_context && !arm.body.diverges {
                self.request_expression_mode(span, &arm.body.result, &result)?;
            }
        }
        let diverges = checked.iter().all(|arm| arm.body.diverges);
        prelude.push(h::Statement::Evaluate(h::Expr {
            kind: h::ExprKind::Match(checked),
            ty: result.clone(),
            span,
        }));
        let block = h::Block {
            construction: vec![],
            storage: (match_storage_start..self.locals.len()).collect(),
            statements: prelude,
            result: result.clone(),
            diverges,
            span,
        };
        Ok(h::Expr {
            kind: h::ExprKind::Block(block),
            ty: result,
            span,
        })
    }
}

mod binding;
mod guards;
use self::guards::readonly_guard;
use self::guards::readonly_pattern;
