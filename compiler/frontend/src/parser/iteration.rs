//! Collection iteration evaluates its source once. Advance before the body so
//! `continue` cannot repeat an element; each element binding has iteration scope.
use super::*;

impl Parser {
    pub(super) fn for_loop(&mut self, start: Span) -> Result<Expr> {
        let pattern = self.pattern()?;
        self.expect_word("in")?;
        let source = self.expr(0, false, true)?;
        let mut body = self.block()?;
        let span = start.join(body.span);
        let collection = format!("$iteration${}${}", start.source.0, start.start);
        let index = format!("{collection}$index");
        let current = format!("{collection}$current");
        let expr = |kind, role| Expr {
            kind,
            span: start.synthetic(role),
        };
        let name = |name: &str, role| expr(ExprKind::Name(name.into()), role);
        let binding = |name: &str, mutable, value, ty, role| Statement {
            kind: StatementKind::Binding {
                mutable,
                pattern: Pattern {
                    kind: PatternKind::Binding {
                        name: name.into(),
                        by_ref: false,
                        mutable,
                    },
                    span: start.synthetic(role),
                },
                ty,
                value,
            },
            span: start.synthetic(role),
        };
        let call = |method: &str, arguments, role| {
            expr(
                ExprKind::Call {
                    callee: Box::new(expr(
                        ExprKind::Member {
                            value: Box::new(name(&collection, role + 1)),
                            name: method.into(),
                        },
                        role + 2,
                    )),
                    types: vec![],
                    arguments,
                },
                role,
            )
        };
        let condition = expr(
            ExprKind::Binary {
                operator: "<",
                left: Box::new(name(&index, 4)),
                right: Box::new(call("len", vec![], 5)),
            },
            3,
        );
        let mut statements = vec![
            binding(&current, false, name(&index, 10), None, 11),
            Statement {
                kind: StatementKind::Expression(expr(
                    ExprKind::Assign {
                        operator: "+=",
                        place: Box::new(name(&index, 12)),
                        value: Box::new(expr(ExprKind::Literal(Literal::Integer("1".into())), 13)),
                    },
                    14,
                )),
                span: start.synthetic(14),
            },
            Statement {
                kind: StatementKind::Binding {
                    mutable: false,
                    pattern,
                    ty: None,
                    value: call("at", vec![name(&current, 15)], 16),
                },
                span: start.synthetic(19),
            },
        ];
        statements.append(&mut body.statements);
        body.statements = statements;
        Ok(Expr {
            kind: ExprKind::Branch(Block {
                statements: vec![
                    binding(&collection, false, source, None, 0),
                    binding(
                        &index,
                        true,
                        expr(ExprKind::Literal(Literal::Integer("0".into())), 1),
                        Some(Type {
                            kind: TypeKind::Named {
                                path: vec!["usize".into()],
                                arguments: vec![],
                            },
                            span: start.synthetic(2),
                        }),
                        2,
                    ),
                    Statement {
                        kind: StatementKind::Expression(expr(
                            ExprKind::While {
                                condition: Box::new(condition),
                                body,
                            },
                            20,
                        )),
                        span,
                    },
                ],
                span,
            }),
            span,
        })
    }
}
