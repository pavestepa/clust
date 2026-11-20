//! Build inline nominal IR directly. No parsed class or constructor is rewritten.
use crate::{ast::*, parser, source::Span};

pub struct RecordField {
    pub name: &'static str,
    pub ty: &'static str,
    pub private: bool,
}
pub struct RecordMethod {
    pub source: &'static str,
    pub private: bool,
}
pub struct Record {
    pub name: &'static str,
    pub generics: &'static str,
    pub interfaces: &'static str,
    pub fields: &'static [RecordField],
    pub methods: &'static [RecordMethod],
    pub drops: &'static [&'static str],
    pub constants: &'static [&'static str],
    pub copy: bool,
    pub private: bool,
}
fn parse(text: &str, span: Span) -> Module {
    let parsed = parser::parse(span.source, text);
    assert!(
        parsed.diagnostics.is_empty(),
        "invalid Rust AST fixture snippet {text}: {:?}",
        parsed.diagnostics
    );
    parsed.module.unwrap()
}
impl Record {
    pub fn build(&self, span: Span) -> Item {
        let ItemKind::Function(prototype) =
            parse(&format!("fn prototype{}() {{}}", self.generics), span)
                .items
                .remove(0)
                .kind
        else {
            unreachable!()
        };
        let mut members = vec![];
        for field in self.fields {
            let ItemKind::Function(mut f) = parse(
                &format!("fn field{}(value: {}) {{}}", self.generics, field.ty),
                span,
            )
            .items
            .remove(0)
            .kind
            else {
                unreachable!()
            };
            members.push(Member::Field {
                name: field.name.into(),
                ty: f.parameters.remove(0).ty.unwrap(),
                weak: false,
                private: field.private,
                span,
            });
        }
        for method in self.methods {
            let ItemKind::Interface { mut methods, .. } = parse(
                &format!("interface Methods{} {{ {} }}", self.generics, method.source),
                span,
            )
            .items
            .remove(0)
            .kind
            else {
                unreachable!()
            };
            members.push(Member::Method {
                function: methods.remove(0),
                private: method.private,
                dispatch: MethodDispatch::Ordinary,
            });
        }
        for body in self.drops {
            members.push(Member::Destructor(block(body, span)));
        }
        for constant in self.constants {
            let ItemKind::Const { name, ty, value } = parse(constant, span).items.remove(0).kind
            else {
                unreachable!()
            };
            members.push(Member::Const {
                name,
                ty,
                value,
                span,
            });
        }
        let mut interfaces = vec![];
        if !self.interfaces.is_empty() {
            let ItemKind::Aggregate {
                interfaces: parsed, ..
            } = parse(
                &format!(
                    "class Interfaces{}: {} {{new() {{}}}}",
                    self.generics, self.interfaces
                ),
                span,
            )
            .items
            .remove(0)
            .kind
            else {
                unreachable!()
            };
            interfaces = parsed;
        }
        Item {
            kind: ItemKind::Aggregate {
                class: false,
                name: self.name.into(),
                generics: prototype.generics,
                interfaces,
                members,
            },
            private: self.private,
            attributes: if self.copy {
                vec![Attribute::Copy]
            } else {
                vec![]
            },
            span,
        }
    }
}
pub fn block(text: &str, span: Span) -> Block {
    let ItemKind::Function(f) = parse(&format!("fn body() {{ {text} }}"), span)
        .items
        .remove(0)
        .kind
    else {
        unreachable!()
    };
    f.body.unwrap()
}

/// Construct an inline IR record for a test with a dynamically chosen body.
pub fn inline(
    name: &str,
    fields: &[(&str, &str)],
    methods: &[&str],
    drops: &[&str],
    span: Span,
) -> Item {
    let mut members = vec![];
    for (name, ty) in fields {
        let ItemKind::Function(mut f) = parse(&format!("fn field(value: {ty}) {{}}"), span)
            .items
            .remove(0)
            .kind
        else {
            unreachable!()
        };
        members.push(Member::Field {
            name: (*name).into(),
            ty: f.parameters.remove(0).ty.unwrap(),
            weak: false,
            private: false,
            span,
        });
    }
    for method in methods {
        let ItemKind::Interface { mut methods, .. } =
            parse(&format!("interface Methods {{ {method} }}"), span)
                .items
                .remove(0)
                .kind
        else {
            unreachable!()
        };
        members.push(Member::Method {
            function: methods.remove(0),
            private: false,
            dispatch: MethodDispatch::Ordinary,
        });
    }
    for body in drops {
        members.push(Member::Destructor(block(body, span)));
    }
    Item {
        kind: ItemKind::Aggregate {
            class: false,
            name: name.into(),
            generics: vec![],
            interfaces: vec![],
            members,
        },
        private: false,
        attributes: vec![],
        span,
    }
}
