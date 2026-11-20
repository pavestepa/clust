use super::*;
impl<F: Fn(&str) -> a::Type> DispatchRoot<'_, F> {
    pub(super) fn payload(&self, result: &mut a::Module) {
        let Self {
            span,
            poly,
            cases,
            root_generics,
            concrete,
            instance,
            ..
        } = *self;
        result.items.push(a::Item {
            span,
            private: false,
            attributes: vec![],
            kind: a::ItemKind::Enum {
                name: cases.to_owned(),
                generics: root_generics.to_vec(),
                variants: concrete
                    .iter()
                    .map(|name| a::Variant {
                        name: name.clone(),
                        fields: a::VariantFields::Tuple(vec![instance(name)]),
                        span,
                    })
                    .collect(),
            },
        });
        result.items.push(a::Item {
            span,
            private: false,
            attributes: vec![],
            kind: a::ItemKind::Aggregate {
                class: true,
                name: poly.to_owned(),
                generics: root_generics.to_vec(),
                interfaces: vec![],
                members: vec![
                    a::Member::Field {
                        name: "$value".into(),
                        ty: instance(cases),
                        weak: false,
                        private: false,
                        span,
                    },
                    a::Member::Constructor {
                        parameters: vec![a::Parameter {
                            pattern: a::Pattern {
                                kind: a::PatternKind::Binding {
                                    name: "$input".into(),
                                    by_ref: false,
                                    mutable: false,
                                },
                                span: site(span, 1),
                            },
                            mutable: false,
                            ty: Some(instance(cases)),
                            receiver: false,
                            span: site(span, 1),
                        }],
                        body: a::Block {
                            statements: vec![a::Statement {
                                kind: a::StatementKind::Expression(a::Expr {
                                    kind: a::ExprKind::Assign {
                                        operator: "=",
                                        place: Box::new(a::Expr {
                                            kind: a::ExprKind::Member {
                                                value: Box::new(expr("this", span)),
                                                name: "$value".into(),
                                            },
                                            span: site(span, 3),
                                        }),
                                        value: Box::new(expr("$input", span)),
                                    },
                                    span: site(span, 5),
                                }),
                                span,
                            }],
                            span,
                        },
                        span,
                    },
                ],
            },
        });
    }
}
