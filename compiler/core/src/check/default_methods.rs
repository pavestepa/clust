//! Instantiate interface defaults before ordinary method collection. Explicit
//! implementations win; unrelated defaults with the same name are ambiguous.
use super::*;

pub(super) fn prepare(module: &a::Module) -> Result<a::Module> {
    let definitions: BTreeMap<_, _> = module
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            a::ItemKind::Interface {
                name,
                generics,
                methods,
            } => Some((name.clone(), (generics, methods))),
            _ => None,
        })
        .collect();
    if !definitions
        .values()
        .any(|(_, methods)| methods.iter().any(|method| method.body.is_some()))
    {
        return Ok(module.clone());
    }
    let constants = crate::constants::evaluate(module)?;
    let mut prepared = module.clone();
    for item in &mut prepared.items {
        let a::ItemKind::Aggregate {
            class,
            interfaces,
            members,
            ..
        } = &mut item.kind
        else {
            continue;
        };
        let explicit: std::collections::BTreeSet<_> = members
            .iter()
            .filter_map(|member| match member {
                a::Member::Method { function, .. } => Some(function.name.clone()),
                _ => None,
            })
            .collect();
        let mut defaults: BTreeMap<String, (a::Function, String)> = BTreeMap::new();
        for interface in interfaces {
            let a::TypeKind::Named { path, arguments } = &interface.kind else {
                continue;
            };
            let name = path.join("::");
            let Some((generics, methods)) = definitions.get(&name) else {
                continue;
            };
            for method in methods
                .iter()
                .filter(|method| method.body.is_some() && !explicit.contains(&method.name))
            {
                let instantiated = inheritance::instantiate_method(
                    generics,
                    arguments,
                    method,
                    &constants,
                    interface.span,
                )?;
                if let Some((previous, origin)) = defaults.get(&method.name) {
                    if origin == &name && previous == &instantiated {
                        continue;
                    }
                    return Err(error(
                        interface.span,
                        "E0402",
                        format!(
                            "ambiguous default method '{}'; declare an explicit implementation",
                            method.name
                        ),
                    )
                    .with_note(
                        previous.span,
                        "another default implementation is declared here",
                    ));
                }
                defaults.insert(method.name.clone(), (instantiated, name.clone()));
            }
        }
        members.extend(defaults.into_values().map(|(function, _)| {
            let instance = function
                .parameters
                .first()
                .is_some_and(|parameter| parameter.receiver);
            a::Member::Method {
                function,
                private: false,
                dispatch: if *class && instance {
                    a::MethodDispatch::Inferred
                } else {
                    a::MethodDispatch::Ordinary
                },
            }
        }));
    }
    // Audit defaults against the interface contract, independently of any
    // concrete implementer. This prevents access to accidental class fields.
    for (name, (generics, methods)) in &definitions {
        let arguments: Vec<_> = generics
            .iter()
            .map(|generic| match generic.kind {
                a::GenericKind::Type { .. } => {
                    a::TypeArgument::Type(named(&generic.name, generic.span))
                }
                a::GenericKind::Const { .. } => a::TypeArgument::Const(a::Expr {
                    kind: a::ExprKind::Name(generic.name.clone()),
                    span: generic.span,
                }),
            })
            .collect();
        for method in methods.iter().filter(|method| method.body.is_some()) {
            let mut audit = inheritance::instantiate_method(
                generics,
                &arguments,
                method,
                &constants,
                method.span,
            )?;
            let receiver_name = format!("$default-self-{name}");
            for parameter in &mut audit.parameters {
                if parameter.receiver {
                    parameter.ty = Some(named(&receiver_name, parameter.span));
                }
            }
            let mut parameters = (*generics).clone();
            parameters.push(a::Generic {
                name: receiver_name,
                kind: a::GenericKind::Type {
                    constraints: vec![a::Type {
                        kind: a::TypeKind::Named {
                            path: vec![name.clone()],
                            arguments: arguments.clone(),
                        },
                        span: method.span,
                    }],
                    default: None,
                },
                span: method.span,
            });
            parameters.append(&mut audit.generics);
            audit.generics = parameters;
            audit.name = format!("$default${name}${}", method.name);
            prepared.items.push(a::Item {
                span: audit.span,
                kind: a::ItemKind::Function(audit),
                private: true,
                attributes: vec![],
            });
        }
    }
    for item in &mut prepared.items {
        if let a::ItemKind::Interface { methods, .. } = &mut item.kind {
            for method in methods {
                method.body = None;
            }
        }
    }
    Ok(prepared)
}
fn named(name: &str, span: Span) -> a::Type {
    a::Type {
        kind: a::TypeKind::Named {
            path: vec![name.to_owned()],
            arguments: vec![],
        },
        span,
    }
}

mod audit;
pub(super) use self::audit::enqueue_audits;
