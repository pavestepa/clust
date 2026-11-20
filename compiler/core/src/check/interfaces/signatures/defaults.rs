use super::*;

pub(super) fn defaults(
    method: &a::Function,
    full: &Substitution,
    constants: &crate::constants::Constants,
    nominals: &records::Nominals,
) -> Result<Vec<(Option<Type>, Option<Value>)>> {
    let mut prefix = Substitution::default();
    let mut result = Vec::new();
    for generic in &method.generics {
        match &generic.kind {
            a::GenericKind::Type {
                default,
                constraints,
            } => {
                let ty = default
                    .as_ref()
                    .map(|ty| {
                        let mut ty = ty.clone();
                        prefix.ty(&mut ty);
                        let ty = resolve_type_with(&ty, constants, nominals)?;
                        let mut scope = full.clone();
                        scope.types.insert(generic.name.clone(), ty.clone());
                        for constraint in constraints {
                            nominals
                                .interfaces
                                .require(&ty, constraint, &scope, constants, nominals)?;
                        }
                        Ok(ty)
                    })
                    .transpose()?;
                result.push((ty, None));
                prefix
                    .types
                    .insert(generic.name.clone(), full.types[&generic.name].clone());
            }
            a::GenericKind::Const { default, .. } => {
                let value = default
                    .as_ref()
                    .map(|value| {
                        let mut value = value.clone();
                        prefix.expr(&mut value);
                        constant_value(&value, &Type::Integer(I::Usize), constants)
                    })
                    .transpose()?;
                result.push((None, value));
                prefix
                    .constants
                    .insert(generic.name.clone(), full.constants[&generic.name]);
            }
        }
    }
    Ok(result)
}
