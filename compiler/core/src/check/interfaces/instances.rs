use super::*;

impl Registry {
    pub(super) fn instance(
        &self,
        source: &a::Type,
        outer: &Substitution,
        constants: &crate::constants::Constants,
        nominals: &records::Nominals,
    ) -> Result<(Bound, Substitution)> {
        let mut source = source.clone();
        outer.clone().ty(&mut source);
        let a::TypeKind::Named { path, arguments } = &source.kind else {
            return Err(error(
                source.span,
                "E0400",
                "constraint must name an interface or Copy",
            ));
        };
        let name = path.join("::");
        let Some(item) = self.definitions.get(&name) else {
            return Err(error(
                source.span,
                "E0400",
                format!("unknown interface '{name}'"),
            ));
        };
        let a::ItemKind::Interface { generics, .. } = &item.kind else {
            unreachable!()
        };
        if arguments.len() > generics.len() {
            return Err(error(source.span, "E0370", "too many interface arguments"));
        }
        let mut substitution = Substitution::default();
        let mut actual = Vec::new();
        let mut names = BTreeSet::new();
        for (index, generic) in generics.iter().enumerate() {
            if !names.insert(&generic.name) {
                return Err(error(generic.span, "E0301", "duplicate generic parameter"));
            }
            match &generic.kind {
                a::GenericKind::Type { default, .. } => {
                    let mut ty = match arguments.get(index) {
                        Some(a::TypeArgument::Type(ty)) => ty.clone(),
                        None => default.clone().ok_or_else(|| {
                            error(source.span, "E0371", "missing interface type argument")
                        })?,
                        _ => {
                            return Err(error(
                                source.span,
                                "E0370",
                                "expected interface type argument",
                            ))
                        }
                    };
                    substitution.ty(&mut ty);
                    let ty = resolve_type_with(&ty, constants, nominals)?;
                    actual.push(records::Argument::Type(ty.clone()));
                    substitution.types.insert(generic.name.clone(), ty);
                }
                a::GenericKind::Const { default, .. } => {
                    let mut value = match arguments.get(index) {
                        Some(a::TypeArgument::Const(value)) => value.clone(),
                        Some(a::TypeArgument::Type(a::Type {
                            kind: a::TypeKind::Named { path, arguments },
                            span,
                        })) if path.len() == 1 && arguments.is_empty() => a::Expr {
                            kind: a::ExprKind::Name(path[0].clone()),
                            span: *span,
                        },
                        None => default.clone().ok_or_else(|| {
                            error(source.span, "E0371", "missing interface const argument")
                        })?,
                        _ => {
                            return Err(error(
                                source.span,
                                "E0370",
                                "expected interface const argument",
                            ))
                        }
                    };
                    substitution.expr(&mut value);
                    let Value::Integer(value) =
                        constant_value(&value, &Type::Integer(I::Usize), constants)?
                    else {
                        unreachable!()
                    };
                    let value = value.bits() as u64;
                    actual.push(records::Argument::Const(value));
                    substitution.constants.insert(generic.name.clone(), value);
                }
            }
        }
        Ok((
            Bound {
                name,
                arguments: actual,
            },
            substitution,
        ))
    }
}

impl Registry {
    pub(in super::super) fn require(
        &self,
        ty: &Type,
        source: &a::Type,
        outer: &Substitution,
        constants: &crate::constants::Constants,
        nominals: &records::Nominals,
    ) -> Result<()> {
        if copy(source) {
            return if ty.copyable() {
                Ok(())
            } else {
                Err(error(
                    source.span,
                    "E0373",
                    "generic argument does not satisfy Copy",
                ))
            };
        }
        let (bound, substitution) = self.instance(source, outer, constants, nominals)?;
        let key = format!("{:?}:{bound:?}", ty.logical());
        if self.active.borrow().contains(&key) {
            return Ok(());
        }
        if self.active.borrow().len() >= 32 {
            return Err(error(
                source.span,
                "E0372",
                "interface obligation nesting limit exceeded",
            ));
        }
        self.active.borrow_mut().insert(key.clone());
        let result = (|| {
            let a::ItemKind::Interface { generics, .. } = &self.definitions[&bound.name].kind
            else {
                unreachable!()
            };
            for generic in generics {
                if let a::GenericKind::Type { constraints, .. } = &generic.kind {
                    for constraint in constraints {
                        self.require(
                            &substitution.types[&generic.name],
                            constraint,
                            &substitution,
                            constants,
                            nominals,
                        )?;
                    }
                }
            }
            let claims = match ty {
                Type::Record { name, .. } | Type::Class { name, .. } => {
                    if let Some(bounds) = self.markers.borrow().get(name) {
                        bounds.clone()
                    } else {
                        nominals.interface_claims(ty, constants)?
                    }
                }
                _ => Vec::new(),
            };
            if !claims.contains(&bound) {
                return Err(error(
                    source.span,
                    "E0402",
                    format!(
                        "type does not explicitly implement interface '{}'",
                        bound.name
                    ),
                ));
            }
            Ok(())
        })();
        self.active.borrow_mut().remove(&key);
        result
    }
}
