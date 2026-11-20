use super::*;
use a::ExprKind as E;
pub(super) struct Captures {
    pub(super) bindings: BTreeMap<String, (Binding, Span)>,
    pub(super) mutable: bool,
}
impl Lower<'_> {
    pub(super) fn environment(
        &mut self,
        mut function: a::Function,
        owner: Option<&str>,
        generics: &[a::Generic],
        environment: Captures,
        class: String,
        span: Span,
    ) -> Result<a::ExprKind> {
        let Captures {
            bindings: captures,
            mutable,
        } = environment;
        let mut class_type = ty(&class, span);
        if let a::TypeKind::Named { arguments, .. } = &mut class_type.kind {
            *arguments = generic_arguments(generics);
        }
        let receiver = a::Parameter {
            pattern: a::Pattern {
                kind: a::PatternKind::Binding {
                    name: "this".into(),
                    by_ref: true,
                    mutable,
                },
                span,
            },
            mutable,
            ty: Some(class_type.clone()),
            receiver: true,
            span,
        };
        function.parameters.insert(0, receiver);
        function.name = "call".into();
        let mut interfaces = vec![];
        for item in &self.module.items {
            if let a::ItemKind::Interface {
                name,
                generics: gs,
                methods,
            } = &item.kind
            {
                if gs.is_empty()
                    && methods.len() == 1
                    && methods[0].name == "call"
                    && compatible(&function, &methods[0])
                {
                    interfaces.push(ty(name, span));
                }
            }
        }
        let environment = a::Type {
            kind: a::TypeKind::Tuple(
                captures
                    .values()
                    .map(|(binding, _)| binding.ty.clone().unwrap())
                    .collect(),
            ),
            span,
        };
        let parameter_span = polymorphism::site(span, 1);
        let mut fields = vec![a::Member::Field {
            name: "$captures".into(),
            ty: environment.clone(),
            weak: false,
            private: false,
            span,
        }];
        fields.push(a::Member::Constructor {
            parameters: vec![a::Parameter {
                pattern: pattern("$environment", parameter_span),
                mutable: false,
                ty: Some(environment),
                receiver: false,
                span: parameter_span,
            }],
            body: a::Block {
                statements: vec![a::Statement {
                    kind: a::StatementKind::Expression(a::Expr {
                        kind: E::Assign {
                            operator: "=",
                            place: Box::new(a::Expr {
                                kind: E::Member {
                                    value: Box::new(name("this", polymorphism::site(span, 2))),
                                    name: "$captures".into(),
                                },
                                span: polymorphism::site(span, 3),
                            }),
                            value: Box::new(name("$environment", polymorphism::site(span, 4))),
                        },
                        span: polymorphism::site(span, 5),
                    }),
                    span: polymorphism::site(span, 6),
                }],
                span,
            },
            span,
        });
        let arguments = vec![a::Expr {
            kind: E::Tuple(
                captures
                    .iter()
                    .map(|(capture, (_, location))| name(capture, *location))
                    .collect(),
            ),
            span: polymorphism::site(span, 7),
        }];
        if let Some(owner) = owner {
            let mut helper = function.clone();
            helper.name = format!("{owner}.{class}");
            helper.generics = generics.to_vec();
            let args = function
                .parameters
                .iter()
                .map(|parameter| match &parameter.pattern.kind {
                    a::PatternKind::Binding { name: n, .. } => Ok(name(n, parameter.span)),
                    _ => Err(error(
                        parameter.span,
                        "E0405",
                        "capturing method closures require named parameters",
                    )),
                })
                .collect::<Result<Vec<_>>>()?;
            function.body = Some(a::Block {
                statements: vec![a::Statement {
                    kind: a::StatementKind::Return(Some(a::Expr {
                        kind: E::Call {
                            callee: Box::new(name(&helper.name, span)),
                            types: generic_arguments(generics),
                            arguments: args,
                        },
                        span,
                    })),
                    span,
                }],
                span,
            });
            self.generated.push(a::Item {
                span,
                kind: a::ItemKind::Function(helper),
                private: false,
                attributes: vec![],
            });
        }
        fields.push(a::Member::Method {
            function,
            private: false,
            dispatch: a::MethodDispatch::Inferred,
        });
        self.generated.push(a::Item {
            span,
            kind: a::ItemKind::Aggregate {
                class: true,
                name: class,
                generics: generics.to_vec(),
                interfaces,
                members: fields,
            },
            private: false,
            attributes: vec![],
        });
        Ok(E::New {
            ty: class_type,
            arguments,
        })
    }
}
