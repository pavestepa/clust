mod interfaces;
mod methods;
mod payload;
struct Interfaces {
    dynamic: BTreeMap<String, BTreeSet<String>>,
    dynamic_types: BTreeMap<(String, String), a::Type>,
    obligations: BTreeMap<String, a::Item>,
}
struct DispatchRoot<'a, F: Fn(&str) -> a::Type> {
    span: Span,
    poly: &'a str,
    cases: &'a str,
    root_generics: &'a [a::Generic],
    roots: &'a BTreeSet<String>,
    concrete: &'a [String],
    instance: &'a F,
}
use super::*;

pub(in super::super) fn prepare(module: &a::Module) -> Result<a::Module> {
    let mut classes: BTreeMap<_, _> = module
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            a::ItemKind::Aggregate {
                class: true,
                name,
                members,
                ..
            } => Some((name.clone(), (item.span, members.clone()))),
            _ => None,
        })
        .collect();
    let constants = crate::constants::evaluate(module)?;
    let parameters: BTreeMap<_, _> = module
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            a::ItemKind::Aggregate { name, generics, .. } => Some((name.clone(), generics.clone())),
            _ => None,
        })
        .collect();
    let Interfaces {
        dynamic,
        dynamic_types,
        obligations,
    } = interfaces::discover(module, &constants, &mut classes)?;
    let bases: BTreeMap<_, _> = classes
        .iter()
        .filter_map(|(name, (_, members))| {
            members.iter().find_map(|member| match member {
                a::Member::Field {
                    name: field,
                    ty:
                        a::Type {
                            kind: a::TypeKind::Named { path, .. },
                            ..
                        },
                    ..
                } if field == class_bases::BASE => Some((name.clone(), path.join("::"))),
                _ => None,
            })
        })
        .collect();
    let roots: BTreeSet<_> = bases.values().chain(dynamic.keys()).cloned().collect();
    if roots.is_empty() {
        return Ok(module.clone());
    }
    let mut result = module.clone();
    result.items.extend(obligations.into_values());
    for item in &mut result.items {
        rewrite_item(item, &roots);
    }
    for root in &roots {
        let (span, members) = &classes[root];
        let span = *span;
        let poly = format!("{PREFIX}{root}");
        let cases = format!("$cases${root}");
        let root_generics = parameters.get(root).cloned().unwrap_or_default();
        let root_arguments = closures::generic_arguments(&root_generics);
        let instance = |name: &str| {
            if let Some(value) = dynamic_types.get(&(
                root.clone(),
                name.strip_prefix(PREFIX).unwrap_or(name).to_owned(),
            )) {
                let mut value = value.clone();
                if let a::TypeKind::Named { path, .. } = &mut value.kind {
                    *path = vec![name.to_owned()];
                }
                return value;
            }
            let mut value = ty(name, span);
            if let a::TypeKind::Named { arguments, .. } = &mut value.kind {
                *arguments = root_arguments.clone();
            }
            value
        };
        let mut concrete: Vec<_> = classes
            .keys()
            .filter(|candidate| {
                if let Some(implementations) = dynamic.get(root) {
                    return implementations.contains(*candidate);
                }
                let mut cursor = *candidate;
                loop {
                    if cursor == root {
                        return true;
                    }
                    match bases.get(cursor) {
                        Some(base) => cursor = base,
                        None => return false,
                    }
                }
            })
            .cloned()
            .collect();
        let nested = concrete
            .iter()
            .filter(|name| *name != root && roots.contains(*name))
            .map(|name| format!("{PREFIX}{name}"))
            .collect::<Vec<_>>();
        concrete.extend(nested);
        let descriptor = DispatchRoot {
            span,
            poly: &poly,
            cases: &cases,
            root_generics: &root_generics,
            roots: &roots,
            concrete: &concrete,
            instance: &instance,
        };
        descriptor.payload(&mut result);
        let mut exposed: Vec<(a::Function, Option<(String, &'static str)>)> = members
            .iter()
            .filter_map(|member| match member {
                a::Member::Method {
                    function,
                    private: false,
                    ..
                } if function.parameters.first().is_some_and(|p| p.receiver) => {
                    Some((function.clone(), None))
                }
                _ => None,
            })
            .collect();
        let mut declaring = Some(root.clone());
        while let Some(owner) = declaring {
            for member in &classes[&owner].1 {
                let a::Member::Field {
                    name,
                    ty: field_type,
                    private: false,
                    ..
                } = member
                else {
                    continue;
                };
                let field_type = inheritance::instantiate_type(
                    parameters
                        .get(&owner)
                        .map(Vec::as_slice)
                        .unwrap_or_default(),
                    &root_arguments,
                    field_type,
                    &constants,
                )?;
                if name.starts_with('$') {
                    continue;
                }
                let mut operations = vec!["get", "="];
                if matches!(&field_type.kind,a::TypeKind::Named {path,..} if path.len()==1 && matches!(path[0].as_str(),"i8"|"i16"|"i32"|"i64"|"i128"|"isize"|"u8"|"u16"|"u32"|"u64"|"u128"|"usize"|"f32"|"f64"))
                {
                    operations.extend(["+=", "-=", "*=", "/=", "%="]);
                }
                for operation in operations {
                    let mut parameters = vec![a::Parameter {
                        pattern: a::Pattern {
                            kind: a::PatternKind::Binding {
                                name: "this".into(),
                                by_ref: true,
                                mutable: operation != "get",
                            },
                            span: site(span, 30),
                        },
                        mutable: operation != "get",
                        ty: Some(instance(&poly)),
                        receiver: true,
                        span: site(span, 30),
                    }];
                    if operation != "get" {
                        parameters.push(a::Parameter {
                            pattern: a::Pattern {
                                kind: a::PatternKind::Binding {
                                    name: "$argument".into(),
                                    by_ref: false,
                                    mutable: false,
                                },
                                span: site(span, 31),
                            },
                            mutable: false,
                            ty: Some(field_type.clone()),
                            receiver: false,
                            span: site(span, 31),
                        });
                    }
                    exposed.push((
                        a::Function {
                            name: accessor(name, operation),
                            generics: vec![],
                            parameters,
                            result: if operation == "get" {
                                field_type.clone()
                            } else {
                                a::Type {
                                    kind: a::TypeKind::Tuple(vec![]),
                                    span,
                                }
                            },
                            body: None,
                            c_abi: false,
                            unsafe_: false,
                            span,
                        },
                        Some((name.clone(), operation)),
                    ));
                }
            }
            declaring = bases.get(&owner).cloned();
        }
        for (function, field_access) in exposed {
            result.items.push(descriptor.emit(function, field_access));
        }
    }
    Ok(result)
}
