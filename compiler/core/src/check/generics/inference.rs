use super::*;

pub(super) fn infer(
    pattern: &a::Type,
    actual: &Type,
    type_names: &std::collections::BTreeSet<String>,
    const_names: &std::collections::BTreeSet<String>,
    output: &mut Substitution,
    nominals: &records::Nominals,
    span: Span,
) -> Result<()> {
    if let a::TypeKind::Named { path, .. } = &pattern.kind {
        if let Some(root) = path
            .first()
            .filter(|_| path.len() == 1)
            .and_then(|name| name.strip_prefix("$poly$"))
        {
            let mut current = actual.clone();
            while let Type::Class { name, .. } = &current {
                if nominals.origin(name) == root {
                    let mut concrete = pattern.clone();
                    if let a::TypeKind::Named { path, .. } = &mut concrete.kind {
                        *path = vec![root.to_owned()];
                    }
                    return infer(
                        &concrete,
                        &current,
                        type_names,
                        const_names,
                        output,
                        nominals,
                        span,
                    );
                }
                let Some(base) = nominals.fields(&current).and_then(|fields| {
                    fields
                        .into_iter()
                        .find(|field| field.name == class_bases::BASE)
                }) else {
                    break;
                };
                current = base.ty;
            }
        }
    }
    match (&pattern.kind, actual) {
        (
            a::TypeKind::Pointer { mutable, pointee },
            Type::Pointer {
                mutable: actual_mut,
                pointee: actual_pointee,
            },
        ) => {
            if mutable != actual_mut {
                return Err(error(span, "E0371", "pointer access mismatch"));
            }
            infer(
                pointee,
                actual_pointee,
                type_names,
                const_names,
                output,
                nominals,
                span,
            )?;
        }
        (
            a::TypeKind::Function {
                unsafe_,
                parameters,
                result,
            },
            Type::Function {
                unsafe_: actual_unsafe,
                parameters: actual_parameters,
                result: actual_result,
                ..
            },
        ) => {
            if unsafe_ != actual_unsafe || parameters.len() != actual_parameters.len() {
                return Err(error(
                    span,
                    "E0371",
                    "function pointer parameter count mismatch",
                ));
            }
            for ((mutable, parameter), (actual_mutable, actual)) in
                parameters.iter().zip(actual_parameters)
            {
                if mutable != actual_mutable {
                    return Err(error(
                        span,
                        "E0371",
                        "function pointer parameter modes mismatch",
                    ));
                }
                infer(
                    parameter,
                    actual,
                    type_names,
                    const_names,
                    output,
                    nominals,
                    span,
                )?;
            }
            infer(
                result,
                actual_result,
                type_names,
                const_names,
                output,
                nominals,
                span,
            )?;
        }
        (a::TypeKind::Named { path, arguments }, ty)
            if path.len() == 1 && arguments.is_empty() && type_names.contains(&path[0]) =>
        {
            if let Some(old) = output.types.get(&path[0]) {
                same(old, ty, span)?;
            } else {
                output.types.insert(path[0].clone(), ty.clone());
            }
        }
        (
            a::TypeKind::Named { path, arguments },
            Type::Record { name, .. } | Type::Enum { name, .. } | Type::Class { name, .. },
        ) if path == &[nominals.origin(name)] => {
            let actuals = nominals.arguments(name);
            for (pattern, actual) in arguments.iter().zip(actuals) {
                match (pattern, actual) {
                    (a::TypeArgument::Type(pattern), records::Argument::Type(actual)) => infer(
                        pattern,
                        &actual,
                        type_names,
                        const_names,
                        output,
                        nominals,
                        span,
                    )?,
                    (pattern, records::Argument::Const(count)) => {
                        let name = match pattern {
                            a::TypeArgument::Const(a::Expr {
                                kind: a::ExprKind::Name(name),
                                ..
                            }) => Some(name),
                            a::TypeArgument::Type(a::Type {
                                kind: a::TypeKind::Named { path, arguments },
                                ..
                            }) if path.len() == 1 && arguments.is_empty() => Some(&path[0]),
                            _ => None,
                        };
                        if let Some(name) = name.filter(|n| const_names.contains(*n)) {
                            if output.constants.get(name).is_some_and(|old| *old != count) {
                                return Err(error(
                                    span,
                                    "E0371",
                                    "conflicting const generic arguments",
                                ));
                            }
                            output.constants.insert(name.clone(), count);
                        }
                    }
                    _ => (),
                }
            }
        }
        (a::TypeKind::Tuple(patterns), Type::Tuple(fields)) if patterns.len() == fields.len() => {
            for (pattern, actual) in patterns.iter().zip(fields) {
                infer(
                    pattern,
                    actual,
                    type_names,
                    const_names,
                    output,
                    nominals,
                    span,
                )?;
            }
        }
        (a::TypeKind::Array { element, length }, Type::Array(ty, count)) => {
            infer(element, ty, type_names, const_names, output, nominals, span)?;
            if let a::ExprKind::Name(name) = &length.kind {
                if const_names.contains(name) {
                    if output.constants.get(name).is_some_and(|old| old != count) {
                        return Err(error(span, "E0371", "conflicting const generic lengths"));
                    }
                    output.constants.insert(name.clone(), *count);
                }
            }
        }
        _ => (),
    }
    Ok(())
}
