use super::*;

pub(in super::super) fn enqueue_audits(
    nominals: &records::Nominals,
    constants: &crate::constants::Constants,
    mono: &std::cell::RefCell<generics::Monomorphization>,
) -> Result<()> {
    let defaults: Vec<_> = mono
        .borrow()
        .templates
        .values()
        .filter(|method| method.name.starts_with("$default$"))
        .cloned()
        .collect();
    for mut method in defaults {
        let mut substitution = generics::substitute::Substitution::default();
        for generic in &method.generics {
            match &generic.kind {
                a::GenericKind::Type { constraints, .. } => {
                    substitution.types.insert(
                        generic.name.clone(),
                        nominals.marker(
                            format!("<default {}:{}>", method.name, generic.name),
                            constraints.iter().any(interfaces::copy),
                        ),
                    );
                }
                a::GenericKind::Const { .. } => {
                    substitution.constants.insert(generic.name.clone(), 2);
                }
            }
        }
        nominals.interfaces.prepare_markers(
            &method.generics,
            &substitution,
            constants,
            nominals,
        )?;
        for marker in substitution.types.values() {
            for method in nominals
                .interfaces
                .marker_methods(marker, constants, nominals)?
            {
                mono.borrow_mut()
                    .templates
                    .insert(method.name.clone(), method);
            }
        }
        substitution.function(&mut method);
        method.generics.clear();
        let parameters = method
            .parameters
            .iter()
            .map(|parameter| resolve_type_with(parameter.ty.as_ref().unwrap(), constants, nominals))
            .collect::<Result<Vec<_>>>()?;
        let result = resolve_type_with(&method.result, constants, nominals)?;
        let modes = methods::parameter_modes(&method, &parameters);
        mono.borrow_mut().audits.push((
            Signature {
                modes,
                id: 0,
                parameters,
                result,
            },
            method,
        ));
    }
    Ok(())
}
