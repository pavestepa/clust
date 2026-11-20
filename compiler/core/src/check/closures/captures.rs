use super::*;

pub(in super::super) fn generic_arguments(generics: &[a::Generic]) -> Vec<a::TypeArgument> {
    generics
        .iter()
        .map(|g| match g.kind {
            a::GenericKind::Type { .. } => a::TypeArgument::Type(ty(&g.name, g.span)),
            a::GenericKind::Const { .. } => a::TypeArgument::Const(name(&g.name, g.span)),
        })
        .collect()
}

impl Lower<'_> {
    pub(super) fn function(
        &mut self,
        function: &mut a::Function,
        outer: &Scope,
        owner: Option<&str>,
        generics: &[a::Generic],
    ) -> Result<()> {
        let mut scope = outer.clone();
        for parameter in &function.parameters {
            if !parameter.receiver {
                bind(
                    &parameter.pattern,
                    Binding {
                        ty: parameter.ty.clone(),
                        mutable: parameter.mutable,
                        parameter: true,
                        scoped: None,
                    },
                    &mut scope,
                );
            }
        }
        let mut generics = generics.to_vec();
        generics.extend(function.generics.clone());
        if let Some(body) = &mut function.body {
            self.block(body, &mut scope, owner, &generics, &BTreeSet::new())?;
        }
        Ok(())
    }
}

impl Lower<'_> {
    pub(super) fn scoped(
        &mut self,
        source: &a::Function,
        scope: &Scope,
        owner: Option<&str>,
        generics: &[a::Generic],
        later: &BTreeSet<String>,
    ) -> Result<Option<(Scoped, Vec<a::Statement>)>> {
        let mut function = source.clone();
        let span = function.span;
        let mut bound = Scope::new();
        params(&function.parameters, &mut bound);
        let mut captures = BTreeMap::new();
        rewrite_block(function.body.as_mut().unwrap(), &mut bound, &mut |expr| {
            if let a::ExprKind::Name(name) = &expr.kind {
                if let Some(binding) = scope.get(name) {
                    captures
                        .entry(name.clone())
                        .or_insert((binding.clone(), expr.span));
                }
            }
        });
        let mut mutated = BTreeSet::new();
        mutations(
            function.body.as_ref().unwrap(),
            self.module,
            scope,
            &mut mutated,
        );
        let needs_scope = captures.iter().any(|(name, (binding, _))| {
            (binding.mutable && later.contains(name))
                || (binding.parameter && mutated.contains(name))
        });
        if !needs_scope {
            return Ok(None);
        }
        self.function(&mut function, scope, owner, generics)?;
        let helper = format!("$scoped${}${}", span.source.0, span.start);
        function.name = owner.map_or_else(|| helper.clone(), |owner| format!("{owner}.{helper}"));
        function.generics = generics.to_vec();
        let mut parameters = Vec::new();
        let mut aliases = Vec::new();
        let mut names = Vec::new();
        for (index, (capture, (binding, location))) in captures.iter().enumerate() {
            let Some(ty) = &binding.ty else {
                return Err(error(
                    *location,
                    "E0405",
                    format!("capture '{capture}' needs an explicit binding type"),
                ));
            };
            let mutable = mutated.contains(capture);
            if mutable && !binding.mutable {
                return Err(error(
                    *location,
                    "E0317",
                    "mutable capture requires write access",
                ));
            }
            let alias = format!("{helper}$capture{index}");
            parameters.push(a::Parameter {
                pattern: a::Pattern {
                    kind: a::PatternKind::Binding {
                        name: capture.clone(),
                        by_ref: true,
                        mutable,
                    },
                    span: *location,
                },
                mutable,
                ty: Some(ty.clone()),
                receiver: true,
                span: *location,
            });
            aliases.push(a::Statement {
                kind: a::StatementKind::Binding {
                    mutable,
                    pattern: a::Pattern {
                        kind: a::PatternKind::Binding {
                            name: alias.clone(),
                            by_ref: true,
                            mutable,
                        },
                        span: polymorphism::site(*location, 32),
                    },
                    ty: None,
                    value: name(capture, *location),
                },
                span: *location,
            });
            names.push(alias);
        }
        let mut bound = Scope::new();
        params(&function.parameters, &mut bound);
        rewrite_block(
            function.body.as_mut().unwrap(),
            &mut bound,
            &mut |expression| {
                if matches!(&expression.kind,a::ExprKind::Name(name) if captures.contains_key(name))
                {
                    expression.kind = a::ExprKind::Unary {
                        operator: "*",
                        value: Box::new(expression.clone()),
                    };
                }
            },
        );
        parameters.append(&mut function.parameters);
        function.parameters = parameters;
        let scoped = Scoped {
            function: function.name.clone(),
            captures: names,
            types: generic_arguments(generics),
            mutable: captures.keys().any(|name| mutated.contains(name)),
        };
        self.generated.push(a::Item {
            span,
            kind: a::ItemKind::Function(function),
            private: false,
            attributes: vec![],
        });
        Ok(Some((scoped, aliases)))
    }
}

impl Lower<'_> {
    pub(super) fn infer(&self, value: &a::Expr, scope: &Scope) -> Option<a::Type> {
        let span = value.span;
        match &value.kind {
            a::ExprKind::Name(name) => scope.get(name).and_then(|b| b.ty.clone()),
            a::ExprKind::Literal(value) => Some(ty(
                match value {
                    a::Literal::Integer(_) => "i32",
                    a::Literal::Float(_) => "f64",
                    a::Literal::String(_) => "str",
                    a::Literal::Char(_) => "char",
                    a::Literal::Bool(_) => "bool",
                },
                span,
            )),
            a::ExprKind::New { ty, .. }
            | a::ExprKind::Construct { ty, .. }
            | a::ExprKind::Cast { ty, .. } => Some(ty.clone()),
            a::ExprKind::Group(value) | a::ExprKind::Unary { value, .. } => {
                self.infer(value, scope)
            }
            a::ExprKind::Binary { operator, left, .. } => {
                if matches!(
                    *operator,
                    "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&" | "||"
                ) {
                    Some(ty("bool", span))
                } else {
                    self.infer(left, scope)
                }
            }
            a::ExprKind::Call { callee, .. } => {
                if let a::ExprKind::Name(name) = &callee.kind {
                    self.module
                        .items
                        .iter()
                        .chain(&self.generated)
                        .find_map(|item| match &item.kind {
                            a::ItemKind::Function(f) if f.name == *name => Some(f.result.clone()),
                            _ => None,
                        })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Lower<'_> {
    pub(super) fn block(
        &mut self,
        body: &mut a::Block,
        scope: &mut Scope,
        owner: Option<&str>,
        generics: &[a::Generic],
        later: &BTreeSet<String>,
    ) -> Result<()> {
        let mut additions = BTreeMap::new();
        for index in 0..body.statements.len() {
            let tail = a::Block {
                statements: body.statements[index + 1..].to_vec(),
                span: body.span,
            };
            let mut future = later.clone();
            future.extend(uses::names(&tail).into_keys());
            match &mut body.statements[index].kind {
                a::StatementKind::Binding {
                    pattern,
                    mutable,
                    ty,
                    value,
                } => {
                    if let (
                        a::PatternKind::Binding {
                            name: binding_name, ..
                        },
                        a::ExprKind::Closure(function),
                    ) = (&pattern.kind, &value.kind)
                    {
                        if let Some((scoped, aliases)) =
                            self.scoped(function, scope, owner, generics, &future)?
                        {
                            if let Some(annotation) = ty {
                                let compatible = matches!(&annotation.kind,a::TypeKind::Function {unsafe_:false,parameters,result} if parameters.len()==function.parameters.len() && result.kind==function.result.kind && parameters.iter().zip(&function.parameters).all(|((mutable,ty),p)|*mutable==p.mutable && p.ty.as_ref().is_some_and(|t|t.kind==ty.kind)));
                                if !compatible {
                                    return Err(error(annotation.span,"E0405","scoped closure annotation must match its function signature"));
                                }
                            }
                            for alias in &aliases {
                                if let a::StatementKind::Binding { pattern, value, .. } =
                                    &alias.kind
                                {
                                    bind(
                                        pattern,
                                        Binding {
                                            ty: self.infer(value, scope),
                                            mutable: matches!(
                                                pattern.kind,
                                                a::PatternKind::Binding { mutable: true, .. }
                                            ),
                                            parameter: false,
                                            scoped: None,
                                        },
                                        scope,
                                    );
                                }
                            }
                            additions.insert(index, aliases);
                            scope.insert(
                                binding_name.clone(),
                                Binding {
                                    ty: Some(a::Type {
                                        kind: a::TypeKind::Tuple(vec![]),
                                        span: value.span,
                                    }),
                                    mutable: *mutable,
                                    parameter: false,
                                    scoped: Some(scoped),
                                },
                            );
                            *ty = None;
                            value.kind = a::ExprKind::Tuple(vec![]);
                            continue;
                        }
                    }
                    self.expression(value, scope, owner, generics, &future)?;
                    let inferred = ty.clone().or_else(|| self.infer(value, scope));
                    bind(
                        pattern,
                        Binding {
                            ty: inferred,
                            mutable: *mutable,
                            parameter: false,
                            scoped: None,
                        },
                        scope,
                    );
                }
                a::StatementKind::Expression(value) | a::StatementKind::Return(Some(value)) => {
                    self.expression(value, scope, owner, generics, &future)?
                }
                a::StatementKind::Unsafe(body) => {
                    self.block(body, &mut scope.clone(), owner, generics, &future)?
                }
                _ => {}
            }
        }
        let original = std::mem::take(&mut body.statements);
        for (index, statement) in original.into_iter().enumerate() {
            if let Some(aliases) = additions.remove(&index) {
                body.statements.extend(aliases);
            }
            body.statements.push(statement);
        }
        Ok(())
    }
}
