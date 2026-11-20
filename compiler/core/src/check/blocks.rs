use super::*;

impl Checker<'_> {
    pub(super) fn block(
        &mut self,
        source: &a::Block,
        expected: Option<&Type>,
        value_context: bool,
    ) -> Result<h::Block> {
        let storage_start = self.locals.len();
        self.continuations.push((source.clone(), 0));
        self.scopes.push(BTreeMap::new());
        let mut statements = Vec::new();
        let mut result = Type::Unit;
        let mut diverges = false;
        for (index, statement) in source.statements.iter().enumerate() {
            self.continuations.last_mut().unwrap().1 = index;
            let last = index + 1 == source.statements.len();
            let checked = match &statement.kind {
                a::StatementKind::Binding {
                    mutable,
                    pattern,
                    ty,
                    value,
                } => {
                    if views::has_ref(pattern) {
                        statements.push(self.view_binding(pattern, value, ty.as_ref())?);
                        continue;
                    }
                    if !matches!(
                        pattern.kind,
                        a::PatternKind::Binding { by_ref: false, .. } | a::PatternKind::Wildcard
                    ) && uses::binds(pattern)
                        && self.needs_pattern_view(value)
                    {
                        statements.push(self.view_binding(
                            &views::inferred_pattern(pattern, *mutable),
                            value,
                            ty.as_ref(),
                        )?);
                        continue;
                    }
                    let ty = ty
                        .as_ref()
                        .map(|ty| resolve_type_with(ty, self.constants, self.nominals))
                        .transpose()?;
                    let hint = self.storage_hints.get(&storage::key(pattern.span)).cloned();
                    if let (Some(annotation), Some(hint)) = (&ty, &hint) {
                        same(annotation, hint, pattern.span)?;
                    }
                    let value = self.expr(value, hint.as_ref().or(ty.as_ref()), true)?;
                    if let Some(hint) = &hint {
                        let joined = storage::union(hint, &value.ty, pattern.span)?;
                        if joined != *hint {
                            self.storage_requests
                                .insert(storage::key(pattern.span), joined);
                        } else if value.ty != *hint && !self.request_storage_mode(&value, hint)? {
                            self.same_storage_mode(hint, &value.ty, value.span)?;
                        }
                    }
                    // NMM §7.1: a binding of an existing non-Copy place is an alias
                    // (R, or W for `var`); it never becomes a move by absence of
                    // later uses. Ownership transfers only in owning contexts.
                    if !value.ty.copyable() && views::place(&value) {
                        let a::StatementKind::Binding {
                            value: source,
                            ty: annotation,
                            ..
                        } = &statement.kind
                        else {
                            unreachable!()
                        };
                        statements.push(self.view_binding(
                            &views::inferred_pattern(pattern, *mutable),
                            source,
                            annotation.as_ref(),
                        )?);
                        continue;
                    }
                    let value_diverges = expression_diverges(&value);
                    let id = self.locals.len();
                    self.locals.push(h::Local {
                        view: None,
                        name: "<binding value>".into(),
                        ty: value.ty.clone(),
                        mutable: false,
                        span: pattern.span,
                    });
                    let reference = h::Expr {
                        kind: h::ExprKind::Local(id),
                        ty: value.ty.clone(),
                        span: pattern.span,
                    };
                    self.storage_sources.insert(id, vec![value.clone()]);
                    let mut bindings = vec![h::Statement::Initialize(id, value)];
                    self.scopes.push(BTreeMap::new());
                    let first = self.locals.len();
                    let (_, irrefutable, _) = self.pattern(pattern, &reference, &mut bindings)?;
                    if !irrefutable {
                        return Err(error(
                            pattern.span,
                            "E0344",
                            "binding requires an irrefutable pattern",
                        ));
                    }
                    if *mutable {
                        for local in &mut self.locals[first..] {
                            local.mutable = true;
                        }
                    }
                    let names = self.scopes.pop().unwrap();
                    if matches!(pattern.kind, a::PatternKind::Binding { by_ref: false, .. }) {
                        self.storage_roots.extend(names.values().copied());
                    }
                    self.scopes.last_mut().unwrap().extend(names);
                    diverges |= value_diverges;
                    h::Statement::Evaluate(h::Expr {
                        kind: h::ExprKind::Block(h::Block {
                            construction: vec![],
                            storage: vec![],
                            statements: bindings,
                            result: Type::Unit,
                            diverges: value_diverges,
                            span: pattern.span,
                        }),
                        ty: Type::Unit,
                        span: pattern.span,
                    })
                }
                a::StatementKind::Expression(value) => {
                    let checked = self.expr(
                        value,
                        if last && value_context {
                            expected
                        } else {
                            None
                        },
                        last && value_context,
                    )?;
                    if !diverges && last && value_context {
                        if let Some(expected) = expected {
                            self.request_storage_mode(&checked, expected)?;
                        }
                        result = checked.ty.clone();
                    }
                    diverges |= expression_diverges(&checked);
                    h::Statement::Evaluate(checked)
                }
                a::StatementKind::Return(value) => {
                    let expected = self.actual_result.as_ref().unwrap_or(&self.result).clone();
                    let value = value
                        .as_ref()
                        .map(|v| self.expr(v, Some(&expected), true))
                        .transpose()?;
                    if let Some(value) = &value {
                        if !expression_diverges(value) {
                            same(&self.result, &value.ty, value.span)?;
                            self.request_storage_mode(value, &expected)?;
                            if !self.actual_result.as_ref().is_some_and(|previous| {
                                previous.contains_views()
                                    && !value.ty.contains_views()
                                    && !self.mono.borrow().pending_results.is_empty()
                            }) {
                                self.actual_result =
                                    Some(if let Some(previous) = &self.actual_result {
                                        storage::union(previous, &value.ty, value.span)?
                                    } else {
                                        value.ty.clone()
                                    });
                            }
                        }
                    }
                    if value.is_none() && expected != Type::Unit {
                        return Err(error(
                            statement.span,
                            "E0304",
                            "return without value requires unit result",
                        ));
                    }
                    diverges = true;
                    h::Statement::Return(value)
                }
                a::StatementKind::Break | a::StatementKind::Continue => {
                    if self.loops == 0 {
                        return Err(error(
                            statement.span,
                            "E0305",
                            "loop control outside a loop",
                        ));
                    }
                    diverges = true;
                    if matches!(statement.kind, a::StatementKind::Break) {
                        h::Statement::Break
                    } else {
                        h::Statement::Continue
                    }
                }
                a::StatementKind::Unsafe(source) => {
                    self.unsafe_depth += 1;
                    let checked = self.block(source, None, false);
                    self.unsafe_depth -= 1;
                    let block = checked?;
                    diverges |= block.diverges;
                    h::Statement::Evaluate(h::Expr {
                        ty: block.result.clone(),
                        span: statement.span,
                        kind: h::ExprKind::Block(block),
                    })
                }
            };
            statements.push(checked);
        }
        self.scopes.pop();
        self.continuations.pop();
        if value_context && !diverges {
            if let Some(expected) = expected {
                same(expected, &result, source.span)?;
            }
        }
        Ok(h::Block {
            construction: vec![],
            storage: (storage_start..self.locals.len()).collect(),
            statements,
            result,
            diverges,
            span: source.span,
        })
    }
}
