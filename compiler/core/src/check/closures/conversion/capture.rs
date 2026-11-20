use super::*;
use a::ExprKind as E;
impl Lower<'_> {
    pub(super) fn convert_closure(
        &mut self,
        value: &mut a::Expr,
        mut function: a::Function,
        scope: &Scope,
        owner: Option<&str>,
        generics: &[a::Generic],
        later: &BTreeSet<String>,
    ) -> Result<()> {
        let span = value.span;
        self.function(&mut function, scope, owner, generics)?;
        let mut bound = Scope::new();
        params(&function.parameters, &mut bound);
        let mut captures = BTreeMap::new();
        rewrite_block(
            function.body.as_mut().unwrap(),
            &mut bound,
            &mut |expression| {
                let E::Name(name) = &expression.kind else {
                    return;
                };
                if let Some(binding) = scope.get(name) {
                    captures
                        .entry(name.clone())
                        .or_insert((binding.clone(), expression.span));
                }
            },
        );
        let class = format!("$closure${}${}", span.source.0, span.start);
        if captures.is_empty() {
            function.name = owner.map_or_else(|| class.clone(), |owner| format!("{owner}.{class}"));
            function.generics = generics.to_vec();
            let target = name(&function.name, span);
            self.generated.push(a::Item {
                span,
                kind: a::ItemKind::Function(function),
                private: false,
                attributes: vec![],
            });
            value.kind = if generics.is_empty() {
                target.kind
            } else {
                E::Specialize {
                    value: Box::new(target),
                    arguments: generic_arguments(generics),
                }
            };
            return Ok(());
        }
        let mut mutated = BTreeSet::new();
        mutations(
            function.body.as_ref().unwrap(),
            self.module,
            scope,
            &mut mutated,
        );
        let mutable = captures.keys().any(|name| mutated.contains(name));
        for (name, (binding, site)) in &captures {
            if binding.ty.is_none() {
                return Err(error(
                    *site,
                    "E0405",
                    format!("capture '{name}' needs an explicit binding type"),
                ));
            }
            if (mutated.contains(name) && (!binding.mutable || binding.parameter))
                || (binding.mutable && later.contains(name))
            {
                return Err(error(*site,"E0405",format!("mutable capture '{name}' requires an exclusive local owner with no later outside use")));
            }
        }
        let mut bound = Scope::new();
        params(&function.parameters, &mut bound);
        rewrite_block(
            function.body.as_mut().unwrap(),
            &mut bound,
            &mut |expression| {
                if let E::Name(name) = &expression.kind {
                    if captures.contains_key(name) {
                        expression.kind = E::Member {
                            value: Box::new(a::Expr {
                                kind: E::Member {
                                    value: Box::new(crate_name(
                                        "this",
                                        polymorphism::site(expression.span, 2),
                                    )),
                                    name: "$captures".into(),
                                },
                                span: polymorphism::site(expression.span, 3),
                            }),
                            name: captures
                                .keys()
                                .position(|capture| capture == name)
                                .unwrap()
                                .to_string(),
                        };
                    }
                }
            },
        );
        value.kind = self.environment(
            function,
            owner,
            generics,
            super::environment::Captures {
                bindings: captures,
                mutable,
            },
            class,
            span,
        )?;
        Ok(())
    }
}
