mod capture;
mod environment;
use super::*;

impl Lower<'_> {
    pub(super) fn expression(
        &mut self,
        value: &mut a::Expr,
        scope: &Scope,
        owner: Option<&str>,
        generics: &[a::Generic],
        later: &BTreeSet<String>,
    ) -> Result<()> {
        use a::ExprKind as E;
        if let E::Call {
            callee,
            types,
            arguments,
        } = &mut value.kind
        {
            if let E::Name(binding) = &callee.kind {
                if let Some(Binding {
                    scoped: Some(scoped),
                    mutable,
                    ..
                }) = scope.get(binding)
                {
                    if scoped.mutable && !mutable {
                        return Err(error(
                            value.span,
                            "E0317",
                            "a mutating closure requires a var binding",
                        ));
                    }
                    if !types.is_empty() {
                        return Err(error(
                            value.span,
                            "E0405",
                            "closure calls do not take type arguments",
                        ));
                    }
                    **callee = name(&scoped.function, callee.span);
                    *types = scoped.types.clone();
                    let mut captures = scoped
                        .captures
                        .iter()
                        .enumerate()
                        .map(|(index, capture)| {
                            name(capture, polymorphism::site(value.span, 32 + index))
                        })
                        .collect::<Vec<_>>();
                    captures.append(arguments);
                    *arguments = captures;
                }
            }
        } else if let E::Name(binding) = &value.kind {
            if scope.get(binding).is_some_and(|b| b.scoped.is_some()) {
                return Err(error(
                    value.span,
                    "E0405",
                    "a closure borrowing mutable locals cannot escape its scope or be stored",
                ));
            }
        }
        if let E::Closure(function) = &value.kind {
            let function = (**function).clone();
            return self.convert_closure(value, function, scope, owner, generics, later);
        }
        match &mut value.kind {
            E::Call {
                callee, arguments, ..
            } => {
                self.expression(callee, scope, owner, generics, later)?;
                for value in arguments {
                    self.expression(value, scope, owner, generics, later)?;
                }
            }
            E::New { arguments, .. } | E::Tuple(arguments) | E::Array(arguments) => {
                for value in arguments {
                    self.expression(value, scope, owner, generics, later)?;
                }
            }
            E::Construct { fields, .. } => {
                for (_, value) in fields {
                    self.expression(value, scope, owner, generics, later)?;
                }
            }
            E::Group(value)
            | E::Unary { value, .. }
            | E::Cast { value, .. }
            | E::Member { value, .. }
            | E::Specialize { value, .. } => {
                self.expression(value, scope, owner, generics, later)?
            }
            E::Index { value, index } => {
                self.expression(value, scope, owner, generics, later)?;
                self.expression(index, scope, owner, generics, later)?;
            }
            E::Binary { left, right, .. }
            | E::Assign {
                place: left,
                value: right,
                ..
            } => {
                self.expression(left, scope, owner, generics, later)?;
                self.expression(right, scope, owner, generics, later)?;
            }
            E::If {
                condition,
                then_block,
                else_branch,
            } => {
                self.expression(condition, scope, owner, generics, later)?;
                self.block(then_block, &mut scope.clone(), owner, generics, later)?;
                if let Some(value) = else_branch {
                    self.expression(value, scope, owner, generics, later)?;
                }
            }
            E::While { condition, body } => {
                self.expression(condition, scope, owner, generics, later)?;
                self.block(body, &mut scope.clone(), owner, generics, later)?;
            }
            E::Branch(body) => self.block(body, &mut scope.clone(), owner, generics, later)?,
            E::Match { value, arms } => {
                self.expression(value, scope, owner, generics, later)?;
                for arm in arms {
                    let mut local = scope.clone();
                    bind(
                        &arm.pattern,
                        Binding {
                            ty: None,
                            mutable: false,
                            parameter: false,
                            scoped: None,
                        },
                        &mut local,
                    );
                    if let Some(value) = &mut arm.guard {
                        self.expression(value, &local, owner, generics, later)?;
                    }
                    self.expression(&mut arm.value, &local, owner, generics, later)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
