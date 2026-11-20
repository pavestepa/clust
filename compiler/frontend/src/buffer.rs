//! Lower the private instruction markers in the embedded core.storage module.
//! Source modules cannot select this pass by path or declare trusted operations.
use crate::ast::*;
use crate::source::Span;

pub(crate) fn lower(module: &mut Module) {
    for item in &mut module.items {
        let ItemKind::Aggregate { class, members, .. } = &mut item.kind else {
            continue;
        };
        *class = false;
        members.retain(|member| !matches!(member, Member::Constructor { .. }));
        for member in members {
            match member {
                Member::Method {
                    function, dispatch, ..
                } => {
                    *dispatch = MethodDispatch::Ordinary;
                    if let Some(body) = &mut function.body {
                        lower_block(body);
                    }
                }
                Member::Destructor(body) => lower_block(body),
                _ => {}
            }
        }
    }
}

fn lower_block(body: &mut Block) {
    for statement in &mut body.statements {
        let StatementKind::Expression(expression) = &mut statement.kind else {
            continue;
        };
        if let ExprKind::If { then_block, .. } = &mut expression.kind {
            lower_block(then_block);
            continue;
        }
        let ExprKind::Call {
            callee, arguments, ..
        } = &expression.kind
        else {
            continue;
        };
        let ExprKind::Name(marker) = &callee.kind else {
            continue;
        };
        if !arguments.is_empty() {
            continue;
        }
        let span = statement.span;
        let ir = BufferIr(span);
        let name = |value: &str| ir.expr(ExprKind::Name(String::from(value)));
        let call = |function, operands: &[&str]| {
            ir.call(function, operands.iter().map(|s| name(s)).collect())
        };
        let count = name("count");
        let allocate = || ir.generic_call("allocate", vec![count.clone()]);
        let statements = match marker.as_str() {
            "make_empty" => vec![StatementKind::Return(Some(ir.expr(ExprKind::Construct {
                ty: ir.generic_type("OwnedBuffer"),
                fields: vec![("storage".into(), allocate()), ("length".into(), ir.zero())],
            })))],
            "push_slot" => vec![StatementKind::Expression(call(
                "initialize",
                &["storage", "length", "value"],
            ))],
            "pop_slot" => {
                let some = ir.expr(ExprKind::Member {
                    value: Box::new(ir.expr(ExprKind::Specialize {
                        value: Box::new(name("BufferResult")),
                        arguments: ir.arguments(),
                    })),
                    name: "Some".into(),
                });
                vec![StatementKind::Return(Some(ir.expr(ExprKind::Call {
                    callee: Box::new(some),
                    types: vec![],
                    arguments: vec![call("take", &["storage", "length"])],
                })))]
            }
            "read_slot" => vec![StatementKind::Return(Some(call(
                "view",
                &["storage", "valid"],
            )))],
            "write_view" => vec![StatementKind::Return(Some(call(
                "view_mut",
                &["storage", "valid"],
            )))],
            "replace_slot" => vec![StatementKind::Return(Some(call(
                "replace",
                &["storage", "valid", "value"],
            )))],
            "destroy_prefix" => vec![StatementKind::Expression(
                ir.call("drop_range", vec![name("storage"), ir.zero(), count]),
            )],
            // Allocate first. The native move validates both ranges before copying.
            // Replacing RawStorage releases only its allocation. The buffer's length
            // now describes exactly the same initialized prefix in the new allocation.
            "grow_initialized" => vec![
                StatementKind::Binding {
                    mutable: true,
                    pattern: Pattern {
                        kind: PatternKind::Binding {
                            name: "$grown".into(),
                            by_ref: false,
                            mutable: false,
                        },
                        span,
                    },
                    ty: None,
                    value: allocate(),
                },
                StatementKind::Expression(ir.call(
                    "move_slots",
                    vec![
                        name("$grown"),
                        ir.zero(),
                        name("storage"),
                        ir.zero(),
                        name("length"),
                    ],
                )),
                StatementKind::Expression(ir.expr(ExprKind::Assign {
                    operator: "=",
                    place: Box::new(name("storage")),
                    value: Box::new(name("$grown")),
                })),
            ],
            _ => continue,
        };
        statement.kind = StatementKind::Unsafe(Block {
            statements: statements
                .into_iter()
                .map(|kind| Statement { kind, span })
                .collect(),
            span,
        });
    }
}

struct BufferIr(Span);
impl BufferIr {
    fn expr(&self, kind: ExprKind) -> Expr {
        Expr { kind, span: self.0 }
    }
    fn zero(&self) -> Expr {
        self.expr(ExprKind::Literal(Literal::Integer("0".into())))
    }
    fn arguments(&self) -> Vec<TypeArgument> {
        vec![TypeArgument::Type(Type {
            kind: TypeKind::Named {
                path: vec!["T".into()],
                arguments: vec![],
            },
            span: self.0,
        })]
    }
    fn generic_type(&self, name: &str) -> Type {
        Type {
            kind: TypeKind::Named {
                path: vec![name.into()],
                arguments: self.arguments(),
            },
            span: self.0,
        }
    }
    fn call(&self, name: &str, arguments: Vec<Expr>) -> Expr {
        self.expr(ExprKind::Call {
            callee: Box::new(self.expr(ExprKind::Name(name.into()))),
            types: vec![],
            arguments,
        })
    }
    fn generic_call(&self, name: &str, arguments: Vec<Expr>) -> Expr {
        self.expr(ExprKind::Call {
            callee: Box::new(self.expr(ExprKind::Name(name.into()))),
            types: self.arguments(),
            arguments,
        })
    }
}
