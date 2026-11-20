use super::*;
impl<F: Fn(&str) -> a::Type> DispatchRoot<'_, F> {
    pub(super) fn emit(
        &self,
        function: a::Function,
        field_access: Option<(String, &'static str)>,
    ) -> a::Item {
        let Self {
            span,
            poly,
            cases,
            root_generics,
            roots,
            concrete,
            instance,
        } = *self;
        let mut dispatch = function.clone();
        rewrite_function(&mut dispatch, roots);
        dispatch.name = format!("{poly}.{}", function.name);
        let mut arguments = Vec::new();
        for (index, parameter) in dispatch.parameters.iter_mut().enumerate() {
            let name = if parameter.receiver {
                "this".to_owned()
            } else {
                format!("$arg{index}")
            };
            parameter.pattern = a::Pattern {
                kind: a::PatternKind::Binding {
                    name: name.clone(),
                    by_ref: parameter.receiver,
                    mutable: parameter.mutable,
                },
                span,
            };
            if parameter.receiver {
                parameter.ty = Some(instance(poly));
            } else {
                arguments.push(expr(&name, span));
            }
        }
        let mutable = dispatch.parameters[0].mutable;
        let types = dispatch
            .generics
            .iter()
            .map(|g| match g.kind {
                a::GenericKind::Type { .. } => a::TypeArgument::Type(ty(&g.name, g.span)),
                a::GenericKind::Const { .. } => a::TypeArgument::Const(expr(&g.name, g.span)),
            })
            .collect::<Vec<_>>();
        let arms = concrete
            .iter()
            .map(|name| a::Arm {
                pattern: a::Pattern {
                    kind: a::PatternKind::Variant {
                        path: vec![cases.to_owned(), name.clone()],
                        fields: a::PatternFields::Tuple(vec![a::Pattern {
                            kind: a::PatternKind::Binding {
                                name: "$object".into(),
                                by_ref: true,
                                mutable,
                            },
                            span,
                        }]),
                    },
                    span,
                },
                guard: None,
                value: if let Some((field, operation)) = &field_access {
                    let place = a::Expr {
                        kind: a::ExprKind::Member {
                            value: Box::new(expr("$object", span)),
                            name: field.clone(),
                        },
                        span: site(span, 12),
                    };
                    if *operation == "get" {
                        place
                    } else {
                        a::Expr {
                            kind: a::ExprKind::Assign {
                                operator: operation,
                                place: Box::new(place),
                                value: Box::new(arguments[0].clone()),
                            },
                            span: site(span, 13),
                        }
                    }
                } else {
                    a::Expr {
                        kind: a::ExprKind::Call {
                            callee: Box::new(a::Expr {
                                kind: a::ExprKind::Member {
                                    value: Box::new(expr("$object", span)),
                                    name: function.name.clone(),
                                },
                                span: site(span, 14),
                            }),
                            types: types.clone(),
                            arguments: arguments.clone(),
                        },
                        span: site(span, 15),
                    }
                },
                span,
            })
            .collect();
        let mut dispatch_generics = root_generics.to_vec();
        dispatch_generics.append(&mut dispatch.generics);
        dispatch.generics = dispatch_generics;
        dispatch.body = Some(a::Block {
            statements: vec![a::Statement {
                kind: a::StatementKind::Return(Some(a::Expr {
                    kind: a::ExprKind::Match {
                        value: Box::new(a::Expr {
                            kind: a::ExprKind::Member {
                                value: Box::new(expr("this", span)),
                                name: "$value".into(),
                            },
                            span,
                        }),
                        arms,
                    },
                    span,
                })),
                span,
            }],
            span,
        });
        a::Item {
            span,
            private: false,
            attributes: vec![],
            kind: a::ItemKind::Function(dispatch),
        }
    }
}
