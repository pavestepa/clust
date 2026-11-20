use super::*;

pub(in super::super) fn compatible(
    required: &a::Function,
    actual: &a::Function,
    constants: &crate::constants::Constants,
    nominals: &records::Nominals,
) -> Result<()> {
    if required.generics.len() != actual.generics.len() {
        return Err(error(
            actual.span,
            "E0401",
            "interface method generic parameter count differs",
        ));
    }
    if symbolic_constants(required) != symbolic_constants(actual) {
        return Err(error(
            actual.span,
            "E0401",
            "interface method symbolic const signature differs",
        ));
    }
    let mut left = required.clone();
    let mut right = actual.clone();
    let mut a = Substitution::default();
    let mut b = Substitution::default();
    for (i, (x, y)) in required.generics.iter().zip(&actual.generics).enumerate() {
        match (&x.kind, &y.kind) {
            (
                a::GenericKind::Type {
                    constraints: xc, ..
                },
                a::GenericKind::Type { .. },
            ) => {
                let marker = nominals.marker(
                    format!(
                        "<interface method {:?}:{:?}:{i}>",
                        required.span, actual.span
                    ),
                    xc.iter().any(copy),
                );
                a.types.insert(x.name.clone(), marker.clone());
                b.types.insert(y.name.clone(), marker);
            }
            (a::GenericKind::Const { ty: xt, .. }, a::GenericKind::Const { ty: yt, .. }) => {
                if resolve_type_with(xt, constants, nominals)? != Type::Integer(I::Usize)
                    || resolve_type_with(yt, constants, nominals)? != Type::Integer(I::Usize)
                {
                    return Err(error(
                        actual.span,
                        "E0401",
                        "const method parameter must be usize",
                    ));
                }
                a.constants.insert(x.name.clone(), i as u64 + 1);
                b.constants.insert(y.name.clone(), i as u64 + 1);
            }
            _ => {
                return Err(error(
                    actual.span,
                    "E0401",
                    "interface method generic kinds differ",
                ))
            }
        }
    }
    nominals
        .interfaces
        .prepare_markers(&required.generics, &a, constants, nominals)?;
    if defaults(required, &a, constants, nominals)? != defaults(actual, &b, constants, nominals)? {
        return Err(error(
            actual.span,
            "E0401",
            "interface method generic defaults differ",
        ));
    }
    // Compare generic constraints after alpha-renaming, without source spans.
    for (x, y) in required.generics.iter().zip(&actual.generics) {
        if let (
            a::GenericKind::Type {
                constraints: xc, ..
            },
            a::GenericKind::Type {
                constraints: yc, ..
            },
        ) = (&x.kind, &y.kind)
        {
            let keys = |constraints: &[a::Type],
                        substitution: &Substitution|
             -> Result<BTreeSet<String>> {
                constraints
                    .iter()
                    .map(|constraint| {
                        if copy(constraint) {
                            Ok("Copy".into())
                        } else {
                            let (bound, _) = nominals.interfaces.instance(
                                constraint,
                                substitution,
                                constants,
                                nominals,
                            )?;
                            Ok(format!("{bound:?}"))
                        }
                    })
                    .collect()
            };
            if keys(xc, &a)? != keys(yc, &b)? {
                return Err(error(
                    actual.span,
                    "E0401",
                    "interface method constraints differ",
                ));
            }
        }
    }
    a.function(&mut left);
    b.function(&mut right);
    if signature(&left, constants, nominals)? != signature(&right, constants, nominals)? {
        return Err(error(
            actual.span,
            "E0401",
            "interface method signature or receiver mode differs",
        )
        .with_note(
            required.span,
            "required interface signature is declared here",
        ));
    }
    Ok(())
}
