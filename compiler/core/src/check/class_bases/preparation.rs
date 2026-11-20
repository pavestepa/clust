use super::*;

pub(in super::super) fn prepare(module: &a::Module) -> Result<a::Module> {
    let mut classes: BTreeMap<String, Class> = module
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            a::ItemKind::Aggregate {
                class: true, name, ..
            } => Some((
                name.clone(),
                Class {
                    item: item.clone(),
                    base: None,
                },
            )),
            _ => None,
        })
        .collect();
    let names: std::collections::BTreeSet<_> = classes.keys().cloned().collect();
    for class in classes.values_mut() {
        let a::ItemKind::Aggregate { interfaces, .. } = &class.item.kind else {
            unreachable!()
        };
        class.base = interfaces.iter().find_map(|ty| match &ty.kind {
            a::TypeKind::Named { path, .. } if names.contains(&path.join("::")) => {
                Some(path.join("::"))
            }
            _ => None,
        });
    }
    if classes.values().all(|class| class.base.is_none()) {
        return Ok(module.clone());
    }
    let constants = crate::constants::evaluate(module)?;
    let mut output = module.clone();
    let mut helpers = Vec::new();
    for item in &mut output.items {
        let a::ItemKind::Aggregate {
            class: true,
            name: actual,
            generics,
            interfaces,
            members,
        } = &mut item.kind
        else {
            continue;
        };
        let Some(base) = &classes[actual].base else {
            continue;
        };
        if !members
            .iter()
            .any(|member| matches!(member, a::Member::Constructor { .. }))
        {
            return Err(error(
                item.span,
                "E0900",
                "derived construction requires an explicit constructor",
            ));
        }
        let base_type = interfaces
            .iter()
            .find(|ty| matches!(&ty.kind,a::TypeKind::Named {path,..} if path.join("::")==*base))
            .unwrap()
            .clone();
        let actual_arguments = closures::generic_arguments(generics);
        let a::TypeKind::Named {
            arguments: base_arguments,
            ..
        } = &base_type.kind
        else {
            unreachable!()
        };
        let a::ItemKind::Aggregate {
            generics: base_generics,
            ..
        } = &classes[base].item.kind
        else {
            unreachable!()
        };
        let uniform = generics.len() == base_generics.len()
            && base_arguments.len() == generics.len()
            && generics
                .iter()
                .zip(base_arguments)
                .all(|(generic, argument)| match (&generic.kind, argument) {
                    (
                        a::GenericKind::Type { .. },
                        a::TypeArgument::Type(a::Type {
                            kind: a::TypeKind::Named { path, arguments },
                            ..
                        }),
                    ) => path == std::slice::from_ref(&generic.name) && arguments.is_empty(),
                    (
                        a::GenericKind::Const { .. },
                        a::TypeArgument::Const(a::Expr {
                            kind: a::ExprKind::Name(name),
                            ..
                        }),
                    ) => name == &generic.name,
                    (
                        a::GenericKind::Const { .. },
                        a::TypeArgument::Type(a::Type {
                            kind: a::TypeKind::Named { path, arguments },
                            ..
                        }),
                    ) => path == std::slice::from_ref(&generic.name) && arguments.is_empty(),
                    _ => false,
                });
        if !uniform {
            return Err(error(item.span, "E0412", "generic inheritance currently requires forwarding every class parameter in order to the base"));
        }
        interfaces.retain(|ty|!matches!(&ty.kind,a::TypeKind::Named {path,..} if names.contains(&path.join("::"))));
        // A direct base is the last owned field, so complete destruction runs
        // the derived destructor and fields before recursively destroying it.
        members.push(a::Member::Field {
            name: BASE.into(),
            ty: base_type.clone(),
            weak: false,
            private: false,
            span: item.span,
        });
        for member in members.iter_mut() {
            match member {
                a::Member::Constructor { body, .. } => {
                    rewrite_block(body, actual, actual, &classes, true)?
                }
                a::Member::Method { function, .. } => {
                    if let Some(body) = &mut function.body {
                        rewrite_block(body, actual, actual, &classes, false)?;
                    }
                }
                a::Member::Destructor(body) => {
                    rewrite_block(body, actual, actual, &classes, false)?
                }
                _ => {}
            }
        }
        let own: std::collections::BTreeSet<_> = members
            .iter()
            .filter_map(|member| match member {
                a::Member::Method { function, .. } => Some(function.name.clone()),
                _ => None,
            })
            .collect();
        let mut inherited = BTreeMap::new();
        for owner in ancestors(&classes, actual).into_iter().rev() {
            let a::ItemKind::Aggregate {
                members: base_members,
                interfaces: base_interfaces,
                generics: owner_generics,
                ..
            } = &classes[&owner].item.kind
            else {
                unreachable!()
            };
            for interface in base_interfaces {
                let interface = inheritance::instantiate_type(
                    owner_generics,
                    &actual_arguments,
                    interface,
                    &constants,
                )?;
                if !matches!(&interface.kind,a::TypeKind::Named {path,..} if names.contains(&path.join("::")))
                    && !interfaces
                        .iter()
                        .any(|existing| existing.kind == interface.kind)
                {
                    interfaces.push(interface.clone());
                }
            }
            for member in base_members {
                let a::Member::Method {
                    function,
                    private,
                    dispatch,
                } = member
                else {
                    continue;
                };
                let function = inheritance::instantiate_method(
                    owner_generics,
                    &actual_arguments,
                    function,
                    &constants,
                    function.span,
                )?;
                let mut body = function.clone();
                let mut helper_generics = generics.clone();
                helper_generics.append(&mut body.generics);
                body.generics = helper_generics;
                body.name = helper(&owner, actual, &function.name, *private);
                for parameter in &mut body.parameters {
                    if parameter.receiver {
                        parameter.ty = Some(a::Type {
                            kind: a::TypeKind::Named {
                                path: vec![actual.clone()],
                                arguments: actual_arguments.clone(),
                            },
                            span: parameter.span,
                        });
                    }
                }
                if let Some(block) = &mut body.body {
                    rewrite_block(block, &owner, actual, &classes, false)?;
                }
                helpers.push(a::Item {
                    span: body.span,
                    kind: a::ItemKind::Function(body),
                    private: false,
                    attributes: vec![],
                });
                if !private {
                    inherited.insert(
                        function.name.clone(),
                        (owner.clone(), function.clone(), *dispatch),
                    );
                }
            }
        }
        for (method, (owner, mut function, dispatch)) in inherited {
            if own.contains(&method) {
                continue;
            }
            let span = function.span;
            let mut arguments = Vec::new();
            for (index, parameter) in function.parameters.iter_mut().enumerate() {
                let binding = if parameter.receiver {
                    "this".into()
                } else {
                    format!("$argument{index}")
                };
                parameter.pattern = a::Pattern {
                    kind: a::PatternKind::Binding {
                        name: binding.clone(),
                        by_ref: parameter.receiver,
                        mutable: parameter.mutable,
                    },
                    span: parameter.span,
                };
                arguments.push(name(&binding, parameter.span));
            }
            let types = generics
                .iter()
                .chain(&function.generics)
                .map(|generic| match generic.kind {
                    a::GenericKind::Type { .. } => a::TypeArgument::Type(a::Type {
                        kind: a::TypeKind::Named {
                            path: vec![generic.name.clone()],
                            arguments: vec![],
                        },
                        span: generic.span,
                    }),
                    a::GenericKind::Const { .. } => {
                        a::TypeArgument::Const(name(&generic.name, generic.span))
                    }
                })
                .collect();
            let call = a::Expr {
                kind: a::ExprKind::Call {
                    callee: Box::new(name(&helper(&owner, actual, &method, false), span)),
                    types,
                    arguments,
                },
                span,
            };
            function.body = Some(a::Block {
                statements: vec![a::Statement {
                    kind: a::StatementKind::Return(Some(call)),
                    span,
                }],
                span,
            });
            members.push(a::Member::Method {
                function,
                private: false,
                dispatch,
            });
        }
    }
    output.items.extend(helpers);
    Ok(output)
}
