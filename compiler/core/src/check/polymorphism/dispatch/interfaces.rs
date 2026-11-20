use super::*;
pub(super) fn discover(
    module: &a::Module,
    constants: &crate::constants::Constants,
    classes: &mut BTreeMap<String, (Span, Vec<a::Member>)>,
) -> Result<Interfaces> {
    let mut requests = BTreeMap::new();
    type_walk::module(module, &mut |ty| {
        if let a::TypeKind::Named { path, arguments } = &ty.kind {
            if path == &["$dyn"] {
                if let [a::TypeArgument::Type(interface)] = arguments.as_slice() {
                    requests.insert(format!("{:?}", interface.kind), interface.clone());
                }
            }
        }
    });
    let mut dynamic = BTreeMap::new();
    let mut dynamic_types = BTreeMap::new();
    let mut obligations = BTreeMap::new();
    for interface in requests.values() {
        let a::TypeKind::Named { path, arguments } = &interface.kind else {
            return Err(error(
                interface.span,
                "E0403",
                "dyn requires an interface type",
            ));
        };
        let name = path.join("::");
        let declaration = module
            .items
            .iter()
            .find(|item| matches!(&item.kind,a::ItemKind::Interface {name:n,..} if *n==name))
            .ok_or_else(|| error(interface.span, "E0403", "dyn requires a declared interface"))?;
        let a::ItemKind::Interface {
            generics, methods, ..
        } = &declaration.kind
        else {
            unreachable!()
        };
        if methods
            .iter()
            .any(|m| !m.generics.is_empty() || !m.parameters.first().is_some_and(|p| p.receiver))
        {
            return Err(error(
                interface.span,
                "E0403",
                "dynamic interface requires nongeneric instance methods",
            ));
        }
        let methods = methods
            .iter()
            .map(|method| {
                inheritance::instantiate_method(
                    generics,
                    arguments,
                    method,
                    constants,
                    interface.span,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let root = format!("$dyn${}", name_of(interface));
        if !generics.is_empty() {
            let span = interface.span;
            let unit = a::Type {
                kind: a::TypeKind::Tuple(vec![]),
                span,
            };
            let check_name = format!("{root}$bounds");
            let requirement = a::Function {
                name: check_name.clone(),
                generics: generics.clone(),
                parameters: vec![],
                result: unit.clone(),
                body: Some(a::Block {
                    statements: vec![],
                    span,
                }),
                c_abi: false,
                unsafe_: false,
                span,
            };
            let verify = a::Function {
                name: format!("{root}$verify"),
                generics: vec![],
                parameters: vec![],
                result: unit,
                body: Some(a::Block {
                    statements: vec![a::Statement {
                        kind: a::StatementKind::Expression(a::Expr {
                            kind: a::ExprKind::Call {
                                callee: Box::new(expr(&check_name, span)),
                                types: arguments.clone(),
                                arguments: vec![],
                            },
                            span,
                        }),
                        span,
                    }],
                    span,
                }),
                c_abi: false,
                unsafe_: false,
                span,
            };
            for function in [requirement, verify] {
                obligations.insert(
                    function.name.clone(),
                    a::Item {
                        kind: a::ItemKind::Function(function),
                        span,
                        private: false,
                        attributes: vec![],
                    },
                );
            }
        }

        let mut implementations = BTreeSet::new();
        for item in &module.items {
            let a::ItemKind::Aggregate {
                class: true,
                name: implementation,
                generics: parameters,
                interfaces,
                ..
            } = &item.kind
            else {
                continue;
            };
            for candidate in interfaces {
                let a::TypeKind::Named {
                    path: candidate_path,
                    arguments: candidate_arguments,
                } = &candidate.kind
                else {
                    continue;
                };
                if *candidate_path != *path {
                    continue;
                }
                let forwarded = parameters.len() == candidate_arguments.len()
                    && parameters
                        .iter()
                        .zip(candidate_arguments)
                        .all(|(parameter, argument)| match argument {
                            a::TypeArgument::Type(a::Type {
                                kind: a::TypeKind::Named { path, arguments },
                                ..
                            }) => {
                                path == std::slice::from_ref(&parameter.name)
                                    && arguments.is_empty()
                            }
                            a::TypeArgument::Const(a::Expr {
                                kind: a::ExprKind::Name(name),
                                ..
                            }) => name == &parameter.name,
                            _ => false,
                        });
                let instance_arguments = if parameters.is_empty()
                    && name_of(candidate) == name_of(interface)
                {
                    vec![]
                } else if !parameters.is_empty() && forwarded && parameters.len() == arguments.len()
                {
                    arguments.clone()
                } else {
                    continue;
                };
                implementations.insert(implementation.clone());
                dynamic_types.insert(
                    (root.clone(), implementation.clone()),
                    a::Type {
                        kind: a::TypeKind::Named {
                            path: vec![implementation.clone()],
                            arguments: instance_arguments,
                        },
                        span: interface.span,
                    },
                );
            }
        }
        dynamic.insert(root.clone(), implementations);
        classes.insert(
            root,
            (
                declaration.span,
                methods
                    .iter()
                    .map(|function| a::Member::Method {
                        function: function.clone(),
                        private: false,
                        dispatch: a::MethodDispatch::Ordinary,
                    })
                    .collect(),
            ),
        );
    }
    Ok(Interfaces {
        dynamic,
        dynamic_types,
        obligations,
    })
}
