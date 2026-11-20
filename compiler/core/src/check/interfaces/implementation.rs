use super::*;

impl Registry {
    pub(in super::super) fn validate_implementation(
        &self,
        ty: &Type,
        sources: &[a::Type],
        outer: &Substitution,
        constants: &crate::constants::Constants,
        nominals: &records::Nominals,
    ) -> Result<()> {
        let (Type::Record { name, .. } | Type::Class { name, .. }) = ty else {
            unreachable!()
        };
        let owner = nominals.origin(name);
        for source in sources {
            let (bound, _) = self.instance(source, outer, constants, nominals)?;
            self.require(ty, source, outer, constants, nominals)?;
            for required in self.surface(&bound, &ty.logical(), constants, nominals)? {
                let candidate = format!("{owner}.{}", required.name);
                let (actual, private) = self.methods.get(&candidate).ok_or_else(|| {
                    error(
                        source.span,
                        "E0401",
                        format!("missing interface method '{}'", required.name),
                    )
                    .with_note(required.span, "required by this interface declaration")
                })?;
                if *private {
                    return Err(error(
                        actual.span,
                        "E0401",
                        "interface implementation method must be public",
                    ));
                }
                let mut actual = actual.clone();
                outer.clone().function(&mut actual);
                let outer_count = nominals.arguments(name).len();
                actual.generics.drain(..outer_count);
                apply_generics(&mut outer.clone(), &mut actual.generics);
                compatible(&required, &actual, constants, nominals)?;
            }
        }
        Ok(())
    }
}
