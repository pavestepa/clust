use super::*;

impl Checker<'_> {
    pub(in super::super) fn pattern(
        &mut self,
        pattern: &a::Pattern,
        value: &h::Expr,
        bindings: &mut Vec<h::Statement>,
    ) -> Result<(h::Expr, bool, Option<usize>)> {
        if matches!(value.ty, Type::StoredView(_)) {
            let value = storage::read(value.clone());
            return self.pattern(&views::inferred_pattern(pattern, false), &value, bindings);
        }
        let span = pattern.span;
        let yes = || h::Expr {
            kind: h::ExprKind::Value(Value::Bool(true)),
            ty: Type::Bool,
            span,
        };
        match &pattern.kind {
            a::PatternKind::Wildcard => Ok((yes(), true, None)),
            a::PatternKind::Binding {
                name,
                mutable,
                by_ref: false,
            } => {
                if self.scopes.last().unwrap().contains_key(name) {
                    return Err(error(span, "E0301", "duplicate binding in pattern"));
                }
                let id = self.local(name.clone(), value.ty.clone(), *mutable, span)?;
                self.storage_sources.insert(id, vec![value.clone()]);
                let value = if value.ty.copyable() && value.ty.owns_storage() && !self.pattern_views
                {
                    h::Expr {
                        kind: h::ExprKind::Take(Box::new(value.clone())),
                        ..value.clone()
                    }
                } else {
                    value.clone()
                };
                bindings.push(h::Statement::Initialize(id, value));
                Ok((yes(), true, None))
            }
            a::PatternKind::Binding {
                name,
                mutable,
                by_ref: true,
            } => {
                if !(super::super::views::view_root(value)
                    || self.pattern_views && views::place(value))
                {
                    return Err(error(
                        span,
                        "E0900",
                        "ref pattern requires source view lowering for this context",
                    ));
                }
                if self.scopes.last().unwrap().contains_key(name) {
                    return Err(error(span, "E0301", "duplicate binding in pattern"));
                }
                if *mutable {
                    self.mutable_place(value)?;
                }
                let id = self.local(name.clone(), value.ty.clone(), *mutable, span)?;
                self.storage_sources.insert(id, vec![value.clone()]);
                self.locals[id].view = Some(if *mutable {
                    crate::mir::loans::Access::Mutable
                } else {
                    crate::mir::loans::Access::Readonly
                });
                bindings.push(h::Statement::Evaluate(h::Expr {
                    kind: h::ExprKind::Borrow {
                        local: id,
                        place: Box::new(value.clone()),
                    },
                    ty: Type::Unit,
                    span,
                }));
                Ok((yes(), true, None))
            }
            a::PatternKind::Literal {
                negative,
                value: literal_value,
            } => {
                if !matches!(value.ty, Type::Bool | Type::Char | Type::Integer(_)) {
                    return Err(error(
                        span,
                        "E0342",
                        "literal pattern requires bool, char or integer",
                    ));
                }
                let (kind, ty) = literal(literal_value, Some(&value.ty), *negative, span)?;
                same(&value.ty, &ty, span)?;
                Ok((
                    h::Expr {
                        kind: h::ExprKind::Binary(
                            "==",
                            Box::new(value.clone()),
                            Box::new(h::Expr { kind, ty, span }),
                        ),
                        ty: Type::Bool,
                        span,
                    },
                    false,
                    None,
                ))
            }
            a::PatternKind::Variant { path, fields } => {
                if let Type::Record {
                    name,
                    fields: definition,
                    ..
                } = &value.ty
                {
                    if path != std::slice::from_ref(name) && path != &[self.nominals.origin(name)] {
                        return Err(error(span, "E0342", "struct pattern type mismatch"));
                    }
                    let a::PatternFields::Named(fields, rest) = fields else {
                        return Err(error(span, "E0342", "struct pattern requires named fields"));
                    };
                    let mut assigned = std::collections::BTreeSet::new();
                    let mut condition = yes();
                    let mut irrefutable = true;
                    for (name, pattern) in fields {
                        let index =
                            definition
                                .iter()
                                .position(|f| f.name == *name)
                                .ok_or_else(|| {
                                    error(pattern.span, "E0342", "unknown struct pattern field")
                                })?;
                        if !assigned.insert(index) {
                            return Err(error(
                                pattern.span,
                                "E0342",
                                "duplicate struct pattern field",
                            ));
                        }
                        let field = &definition[index];
                        self.check_field_access(&value.ty, field, pattern.span)?;
                        let projection = h::Expr {
                            kind: h::ExprKind::Field(Box::new(value.clone()), index),
                            ty: field.ty.clone(),
                            span: pattern.span,
                        };
                        let (test, all, _) = self.pattern(pattern, &projection, bindings)?;
                        irrefutable &= all;
                        condition = h::Expr {
                            kind: h::ExprKind::Binary("&&", Box::new(condition), Box::new(test)),
                            ty: Type::Bool,
                            span,
                        };
                    }
                    if !rest && assigned.len() != definition.len() {
                        return Err(error(span, "E0342", "missing struct pattern fields"));
                    }
                    return Ok((condition, irrefutable, None));
                }

                let Some((ty, index, definition)) = self.enum_pattern(&value.ty, path, span)?
                else {
                    return Err(error(span, "E0342", "unknown enum pattern"));
                };
                same(&value.ty, &ty, span)?;
                let patterns = match (fields, &definition.shape) {
                    (a::PatternFields::Unit, h::VariantShape::Unit) => Vec::new(),
                    (a::PatternFields::Tuple(patterns), h::VariantShape::Tuple)
                        if patterns.len() == definition.fields.len() =>
                    {
                        patterns.clone()
                    }
                    (a::PatternFields::Named(patterns, rest), h::VariantShape::Named(names)) => {
                        let mut result = vec![
                            a::Pattern {
                                kind: a::PatternKind::Wildcard,
                                span
                            };
                            names.len()
                        ];
                        let mut assigned = std::collections::BTreeSet::new();
                        for (name, pattern) in patterns {
                            let i = names
                                .iter()
                                .position(|n| n == name)
                                .ok_or_else(|| error(span, "E0342", "unknown variant field"))?;
                            if !assigned.insert(i) {
                                return Err(error(
                                    span,
                                    "E0342",
                                    "duplicate variant pattern field",
                                ));
                            }
                            result[i] = pattern.clone();
                        }
                        if !rest && assigned.len() != names.len() {
                            return Err(error(span, "E0342", "missing variant pattern fields"));
                        }
                        result
                    }
                    _ => return Err(error(span, "E0342", "variant pattern shape mismatch")),
                };
                let payload = h::Expr {
                    kind: h::ExprKind::Payload(Box::new(value.clone()), index),
                    ty: Type::Tuple(definition.fields.clone()),
                    span,
                };
                let mut test = yes();
                let mut unconditional = true;
                for (i, pattern) in patterns.iter().enumerate() {
                    let projection = h::Expr {
                        kind: h::ExprKind::Field(Box::new(payload.clone()), i),
                        ty: definition.fields[i].clone(),
                        span,
                    };
                    let (part, all, _) = self.pattern(pattern, &projection, bindings)?;
                    unconditional &= all;
                    test = h::Expr {
                        kind: h::ExprKind::Binary("&&", Box::new(test), Box::new(part)),
                        ty: Type::Bool,
                        span,
                    };
                }
                let tag = h::Expr {
                    kind: h::ExprKind::Tag(Box::new(value.clone())),
                    ty: Type::Integer(I::U32),
                    span,
                };
                let target = h::Expr {
                    kind: h::ExprKind::Value(Value::Integer(Integer::from_bits(
                        I::U32,
                        index as u128,
                    ))),
                    ty: Type::Integer(I::U32),
                    span,
                };
                let condition = h::Expr {
                    kind: h::ExprKind::Binary("==", Box::new(tag), Box::new(target)),
                    ty: Type::Bool,
                    span,
                };
                let condition = h::Expr {
                    kind: h::ExprKind::Binary("&&", Box::new(condition), Box::new(test)),
                    ty: Type::Bool,
                    span,
                };
                let single = matches!(ty,Type::Enum {ref variants,..} if variants.len()==1);
                Ok((
                    condition,
                    unconditional && single,
                    unconditional.then_some(index),
                ))
            }
            a::PatternKind::Tuple(patterns) | a::PatternKind::Array(patterns) => {
                let (length, tuple) = match (&pattern.kind, &value.ty) {
                    (a::PatternKind::Tuple(_), Type::Tuple(fields)) => (fields.len(), true),
                    (a::PatternKind::Tuple(_), Type::Unit) => (0, true),
                    (a::PatternKind::Array(_), Type::Array(_, count)) => (*count as usize, false),
                    _ => {
                        return Err(error(
                            span,
                            "E0342",
                            "pattern shape does not match value type",
                        ))
                    }
                };
                let rest = patterns.iter().position(|p| p.kind == a::PatternKind::Rest);
                let explicit = patterns.len() - usize::from(rest.is_some());
                if explicit > length || rest.is_none() && explicit != length {
                    return Err(error(span, "E0342", "wrong number of pattern elements"));
                }
                let mut condition = yes();
                let mut irrefutable = true;
                for (position, pattern) in patterns.iter().enumerate() {
                    if pattern.kind == a::PatternKind::Rest {
                        continue;
                    }
                    let index = if rest.is_some_and(|r| position > r) {
                        length - (patterns.len() - position)
                    } else {
                        position
                    };
                    let ty = match &value.ty {
                        Type::Tuple(fields) => fields[index].clone(),
                        Type::Array(element, _) => (**element).clone(),
                        _ => unreachable!(),
                    };
                    let kind = if tuple {
                        h::ExprKind::Field(Box::new(value.clone()), index)
                    } else {
                        h::ExprKind::Index(
                            Box::new(value.clone()),
                            Box::new(h::Expr {
                                kind: h::ExprKind::Value(Value::Integer(Integer::from_bits(
                                    I::Usize,
                                    index as u128,
                                ))),
                                ty: Type::Integer(I::Usize),
                                span,
                            }),
                        )
                    };
                    let (test, unconditional, _) =
                        self.pattern(pattern, &h::Expr { kind, ty, span }, bindings)?;
                    irrefutable &= unconditional;
                    condition = h::Expr {
                        kind: h::ExprKind::Binary("&&", Box::new(condition), Box::new(test)),
                        ty: Type::Bool,
                        span,
                    };
                }
                Ok((condition, irrefutable, None))
            }
            _ => Err(error(
                span,
                "E0900",
                "this pattern requires nominal types or ownership checking",
            )),
        }
    }
}
