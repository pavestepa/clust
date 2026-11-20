use super::*;

impl Registry {
    pub(in super::super) fn prepare_markers(
        &self,
        generics: &[a::Generic],
        outer: &Substitution,
        constants: &crate::constants::Constants,
        nominals: &records::Nominals,
    ) -> Result<()> {
        let mut pending = Vec::new();
        for generic in generics {
            if let a::GenericKind::Type { constraints, .. } = &generic.kind {
                let marker = &outer.types[&generic.name];
                let Type::Record { name, .. } = marker else {
                    unreachable!()
                };
                self.markers.borrow_mut().insert(name.clone(), Vec::new());
                for constraint in constraints.iter().filter(|ty| !copy(ty)) {
                    pending.push((name.clone(), constraint.clone()));
                }
            }
        }
        // A bound's type arguments can depend on a later parameter's bounds.
        // Publish independently resolved claims before retrying those obligations.
        while !pending.is_empty() {
            let previous = pending.len();
            let mut next = Vec::new();
            let mut diagnostic = None;
            for (name, constraint) in pending {
                match self.instance(&constraint, outer, constants, nominals) {
                    Ok((bound, _)) => {
                        let mut markers = self.markers.borrow_mut();
                        let claims = markers.get_mut(&name).unwrap();
                        if !claims.contains(&bound) {
                            claims.push(bound)
                        }
                    }
                    Err(error) => {
                        diagnostic = Some(error);
                        next.push((name, constraint));
                    }
                }
            }
            if next.len() == previous {
                return Err(diagnostic.unwrap());
            }
            pending = next;
        }
        for generic in generics {
            if let a::GenericKind::Type { constraints, .. } = &generic.kind {
                for constraint in constraints {
                    self.require(
                        &outer.types[&generic.name],
                        constraint,
                        outer,
                        constants,
                        nominals,
                    )?;
                }
            }
        }
        Ok(())
    }
}

impl Registry {
    pub(in super::super) fn marker_methods(
        &self,
        marker: &Type,
        constants: &crate::constants::Constants,
        nominals: &records::Nominals,
    ) -> Result<Vec<a::Function>> {
        let Type::Record { name, .. } = marker else {
            unreachable!()
        };
        let bounds = self.markers.borrow().get(name).cloned().unwrap_or_default();
        let mut functions = BTreeMap::<String, a::Function>::new();
        for bound in bounds {
            for mut method in self.surface(&bound, marker, constants, nominals)? {
                let short = method.name.clone();
                method.name = format!("{name}.{short}");
                if let Some(old) = functions.get(&short) {
                    compatible(old, &method, constants, nominals)?;
                } else {
                    functions.insert(short, method);
                }
            }
        }
        Ok(functions.into_values().collect())
    }
}

pub(super) fn apply_generics(substitution: &mut Substitution, generics: &mut [a::Generic]) {
    for generic in generics {
        match &mut generic.kind {
            a::GenericKind::Type {
                constraints,
                default,
            } => {
                for ty in constraints {
                    substitution.ty(ty)
                }
                if let Some(ty) = default {
                    substitution.ty(ty)
                }
            }
            a::GenericKind::Const { ty, default } => {
                substitution.ty(ty);
                if let Some(value) = default {
                    substitution.expr(value)
                }
            }
        }
    }
}
