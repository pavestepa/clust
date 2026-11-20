use super::*;

impl Registry {
    pub(in super::super) fn contains(&self, name: &str) -> bool {
        self.definitions.contains_key(name)
    }
}

impl Registry {
    pub(in super::super) fn validate_declarations(
        &self,
        constants: &crate::constants::Constants,
        nominals: &records::Nominals,
    ) -> Result<()> {
        for (name, item) in &self.definitions {
            let a::ItemKind::Interface { generics, .. } = &item.kind else {
                unreachable!()
            };
            let mut substitution = Substitution::default();
            let mut seen = BTreeSet::new();
            for generic in generics {
                if !seen.insert(&generic.name) {
                    return Err(error(generic.span, "E0301", "duplicate generic parameter"));
                }
                match &generic.kind {
                    a::GenericKind::Type { constraints, .. } => {
                        substitution.types.insert(
                            generic.name.clone(),
                            nominals.marker(
                                format!("<interface {name}:{}>", generic.name),
                                constraints.iter().any(copy),
                            ),
                        );
                    }
                    a::GenericKind::Const { ty, .. } => {
                        if resolve_type_with(ty, constants, nominals)? != Type::Integer(I::Usize) {
                            return Err(error(
                                generic.span,
                                "E0370",
                                "const interface parameter must be usize",
                            ));
                        }
                        substitution.constants.insert(generic.name.clone(), 2);
                    }
                }
            }
            self.prepare_markers(generics, &substitution, constants, nominals)?;
            let receiver = nominals.marker(format!("<interface receiver {name}>"), false);
            let bound = Bound {
                name: name.clone(),
                arguments: generics
                    .iter()
                    .map(|g| match g.kind {
                        a::GenericKind::Type { .. } => {
                            records::Argument::Type(substitution.types[&g.name].clone())
                        }
                        a::GenericKind::Const { .. } => {
                            records::Argument::Const(substitution.constants[&g.name])
                        }
                    })
                    .collect(),
            };
            for method in self.surface(&bound, &receiver, constants, nominals)? {
                let mut seen = BTreeSet::new();
                for generic in &method.generics {
                    if !seen.insert(&generic.name) {
                        return Err(error(
                            generic.span,
                            "E0301",
                            "duplicate method generic parameter",
                        ));
                    }
                }
                compatible(&method, &method, constants, nominals)?;
            }
            // Defaults see preceding parameters, even when no instance is used.
            let mut prefix = Substitution::default();
            for generic in generics {
                match &generic.kind {
                    a::GenericKind::Type {
                        default: Some(ty),
                        constraints,
                    } => {
                        let mut ty = ty.clone();
                        prefix.ty(&mut ty);
                        let ty = resolve_type_with(&ty, constants, nominals)?;
                        let mut scope = substitution.clone();
                        scope.types.insert(generic.name.clone(), ty.clone());
                        for constraint in constraints {
                            self.require(&ty, constraint, &scope, constants, nominals)?;
                        }
                    }
                    a::GenericKind::Const {
                        default: Some(value),
                        ..
                    } => {
                        let mut value = value.clone();
                        prefix.expr(&mut value);
                        constant_value(&value, &Type::Integer(I::Usize), constants)?;
                    }
                    _ => {}
                }
                match generic.kind {
                    a::GenericKind::Type { .. } => {
                        prefix.types.insert(
                            generic.name.clone(),
                            substitution.types[&generic.name].clone(),
                        );
                    }
                    a::GenericKind::Const { .. } => {
                        prefix
                            .constants
                            .insert(generic.name.clone(), substitution.constants[&generic.name]);
                    }
                }
            }
        }
        Ok(())
    }
}
