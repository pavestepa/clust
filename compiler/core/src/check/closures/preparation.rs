use super::*;

pub(in super::super) fn prepare(module: &a::Module) -> Result<a::Module> {
    let mut output = module.clone();
    let mut lower = Lower {
        module,
        generated: vec![],
    };
    for item in &mut output.items {
        match &mut item.kind {
            a::ItemKind::Function(function) => {
                lower.function(function, &Scope::new(), None, &[])?
            }
            a::ItemKind::Aggregate {
                name: owner,
                generics,
                members,
                ..
            } => {
                let mut receiver = ty(owner, item.span);
                if let a::TypeKind::Named { arguments, .. } = &mut receiver.kind {
                    *arguments = generic_arguments(generics);
                }
                for member in members {
                    match member {
                        a::Member::Method { function, .. } => {
                            let mut outer = Scope::new();
                            outer.insert(
                                "this".into(),
                                Binding {
                                    ty: Some(receiver.clone()),
                                    mutable: function.parameters.first().is_some_and(|p| p.mutable),
                                    parameter: true,
                                    scoped: None,
                                },
                            );
                            lower.function(function, &outer, Some(owner), generics)?;
                        }
                        a::Member::Constructor {
                            parameters, body, ..
                        } => {
                            let mut scope = Scope::new();
                            params(parameters, &mut scope);
                            scope.insert(
                                "this".into(),
                                Binding {
                                    ty: Some(receiver.clone()),
                                    mutable: true,
                                    parameter: true,
                                    scoped: None,
                                },
                            );
                            lower.block(
                                body,
                                &mut scope,
                                Some(owner),
                                generics,
                                &BTreeSet::new(),
                            )?;
                        }
                        a::Member::Destructor(body) => {
                            let mut scope = Scope::new();
                            scope.insert(
                                "this".into(),
                                Binding {
                                    ty: Some(receiver.clone()),
                                    mutable: true,
                                    parameter: true,
                                    scoped: None,
                                },
                            );
                            lower.block(
                                body,
                                &mut scope,
                                Some(owner),
                                generics,
                                &BTreeSet::new(),
                            )?;
                        }
                        _ => {}
                    }
                }
            }
            a::ItemKind::Interface {
                generics, methods, ..
            } => {
                for function in methods {
                    lower.function(function, &Scope::new(), None, generics)?;
                }
            }
            _ => {}
        }
    }
    output.items.extend(lower.generated);
    Ok(output)
}
