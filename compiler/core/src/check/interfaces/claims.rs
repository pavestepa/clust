use super::*;

impl Registry {
    pub(in super::super) fn claims(
        &self,
        sources: &[a::Type],
        substitution: &Substitution,
        constants: &crate::constants::Constants,
        nominals: &records::Nominals,
    ) -> Result<Vec<Bound>> {
        let mut result = Vec::new();
        for source in sources {
            let (bound, _) = self.instance(source, substitution, constants, nominals)?;
            if result.contains(&bound) {
                return Err(error(
                    source.span,
                    "E0401",
                    "duplicate interface implementation",
                ));
            }
            result.push(bound);
        }
        Ok(result)
    }
}

impl Registry {
    pub(super) fn surface(
        &self,
        bound: &Bound,
        receiver: &Type,
        constants: &crate::constants::Constants,
        nominals: &records::Nominals,
    ) -> Result<Vec<a::Function>> {
        let item = &self.definitions[&bound.name];
        let a::ItemKind::Interface {
            generics, methods, ..
        } = &item.kind
        else {
            unreachable!()
        };
        let mut substitution = Substitution::default();
        for (generic, arg) in generics.iter().zip(&bound.arguments) {
            match arg {
                records::Argument::Type(ty) => {
                    substitution.types.insert(generic.name.clone(), ty.clone());
                }
                records::Argument::Const(n) => {
                    substitution.constants.insert(generic.name.clone(), *n);
                }
            }
        }
        let mut result = Vec::new();
        for method in methods {
            let mut method = method.clone();
            // Method generics shadow interface generics of the same name.
            let mut scope = substitution.clone();
            for generic in &method.generics {
                scope.types.remove(&generic.name);
                scope.constants.remove(&generic.name);
            }
            scope.function(&mut method);
            apply_generics(&mut scope, &mut method.generics);
            for p in &mut method.parameters {
                if p.receiver {
                    p.ty = Some(generics::substitute::ast_type(receiver, p.span));
                }
            }
            // Resolve nongeneric signatures now; generic method signatures are
            // resolved against fresh markers during compatibility checking.
            if method.generics.is_empty() {
                signature(&method, constants, nominals)?;
            }
            result.push(method);
        }
        Ok(result)
    }
}
