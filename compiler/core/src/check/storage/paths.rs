use super::*;

pub(in super::super) fn key(span: Span) -> (usize, usize, usize) {
    (span.source.0, span.start, span.end)
}

pub(in super::super) fn union(a: &Type, b: &Type, span: Span) -> Result<Type> {
    same(a, b, span)?;
    if a == b {
        return Ok(a.clone());
    }
    Ok(match (a, b) {
        (
            Type::Function { contract: None, .. },
            b @ Type::Function {
                contract: Some(_), ..
            },
        ) => b.clone(),
        (
            a @ Type::Function {
                contract: Some(_), ..
            },
            Type::Function { contract: None, .. },
        ) => a.clone(),
        (
            Type::Function {
                unsafe_: a_unsafe,
                parameters: a,
                result: ar,
                contract: Some(ac),
            },
            Type::Function {
                unsafe_: b_unsafe,
                parameters: b,
                result: br,
                contract: Some(bc),
            },
        ) => {
            debug_assert_eq!(a_unsafe, b_unsafe);
            let mut c = (**ac).clone();
            for source in &bc.sources {
                if !c.sources.contains(source) {
                    c.sources.push(source.clone())
                }
            }
            c.sources.sort();
            for (mode, other) in c.modes.iter_mut().zip(&bc.modes) {
                if mode.is_none() {
                    *mode = *other
                }
            }
            match (&mut c.view, &bc.view) {
                (Some(a), Some(b)) => {
                    if a.access != b.access {
                        return Err(error(
                            span,
                            "E0396",
                            "incompatible function result capabilities",
                        ));
                    }
                    a.origins.extend(b.origins.clone());
                }
                (None, Some(b)) => c.view = Some(b.clone()),
                _ => {}
            }
            for item in &bc.storage {
                if let Some(old) = c.storage.iter_mut().find(|old| old.path == item.path) {
                    old.origins.extend(item.origins.clone())
                } else {
                    c.storage.push(item.clone())
                }
            }
            for item in &bc.writes {
                if let Some(old) = c.writes.iter_mut().find(|old| old.target == item.target) {
                    old.origins.extend(item.origins.clone())
                } else {
                    c.writes.push(item.clone())
                }
            }
            Type::Function {
                unsafe_: *a_unsafe,
                parameters: a
                    .iter()
                    .zip(b)
                    .map(|((m, a), (_, b))| Ok((*m, union(a, b, span)?)))
                    .collect::<Result<_>>()?,
                result: Box::new(union(ar, br, span)?),
                contract: Some(Box::new(c)),
            }
        }
        (Type::StoredView(a), Type::StoredView(b)) => {
            Type::StoredView(Box::new(union(a, b, span)?))
        }
        (Type::StoredView(a), b) | (b, Type::StoredView(a)) => {
            Type::StoredView(Box::new(union(a, b, span)?))
        }
        (
            Type::Class { storage: None, .. },
            right @ Type::Class {
                storage: Some(_), ..
            },
        ) => right.clone(),
        (
            left @ Type::Class {
                storage: Some(_), ..
            },
            Type::Class { storage: None, .. },
        ) => left.clone(),
        (
            left @ Type::Class {
                storage: Some(a), ..
            },
            Type::Class {
                storage: Some(b), ..
            },
        ) => {
            let mut result = left.clone();
            let Type::Class {
                storage: Some(fields),
                ..
            } = &mut result
            else {
                unreachable!()
            };
            for ((field, a), b) in fields.iter_mut().zip(a).zip(b) {
                field.ty = union(&a.ty, &b.ty, span)?;
            }
            result
        }
        (left @ Type::Record { fields: a, .. }, Type::Record { fields: b, .. }) => {
            let mut ty = a
                .iter()
                .zip(b)
                .map(|(a, b)| {
                    let mut field = a.clone();
                    field.ty = union(&a.ty, &b.ty, span)?;
                    Ok(field)
                })
                .collect::<Result<Vec<_>>>()?;
            let mut result = left.clone();
            let Type::Record { fields, .. } = &mut result else {
                unreachable!()
            };
            std::mem::swap(fields, &mut ty);
            result
        }
        (Type::Tuple(a), Type::Tuple(b)) => Type::Tuple(
            a.iter()
                .zip(b)
                .map(|(a, b)| union(a, b, span))
                .collect::<Result<_>>()?,
        ),
        (Type::Array(a, n), Type::Array(b, _)) => Type::Array(Box::new(union(a, b, span)?), *n),
        (left @ Type::Enum { variants: a, .. }, Type::Enum { variants: b, .. }) => {
            let mut result = left.clone();
            let Type::Enum { variants, .. } = &mut result else {
                unreachable!()
            };
            for ((variant, a), b) in variants.iter_mut().zip(a).zip(b) {
                variant.fields = a
                    .fields
                    .iter()
                    .zip(&b.fields)
                    .map(|(a, b)| union(a, b, span))
                    .collect::<Result<_>>()?;
            }
            result
        }
        _ => {
            return Err(error(
                span,
                "E0396",
                "incompatible static storage representations",
            ))
        }
    })
}

pub(super) fn place_paths(
    checker: &Checker<'_>,
    value: &h::Expr,
    seen: &mut Vec<usize>,
) -> Vec<(usize, Vec<Step>)> {
    match &value.kind {
        h::ExprKind::Local(local) | h::ExprKind::View(local) => {
            if checker.storage_roots.contains(local) {
                return vec![(*local, Vec::new())];
            }
            if seen.contains(local) {
                return Vec::new();
            }
            seen.push(*local);
            let paths = checker
                .storage_sources
                .get(local)
                .into_iter()
                .flatten()
                .flat_map(|source| place_paths(checker, source, seen))
                .collect();
            seen.pop();
            paths
        }
        h::ExprKind::Field(base, i) | h::ExprKind::Payload(base, i) => {
            let step = if matches!(value.kind, h::ExprKind::Field(..)) {
                Step::Field(*i)
            } else {
                Step::Variant(*i)
            };
            place_paths(checker, base, seen)
                .into_iter()
                .map(|(root, mut path)| {
                    path.push(step.clone());
                    (root, path)
                })
                .collect()
        }
        h::ExprKind::Index(base, _) => place_paths(checker, base, seen)
            .into_iter()
            .map(|(root, mut path)| {
                path.push(Step::Index);
                (root, path)
            })
            .collect(),
        _ => Vec::new(),
    }
}
