//! Anonymous functions lower to checked named functions or uniquely owned
//! capture classes. No runtime reference counting or hidden cloning is added.
use super::*;
use std::collections::BTreeSet;
#[derive(Clone)]
struct Binding {
    ty: Option<a::Type>,
    mutable: bool,
    parameter: bool,
    scoped: Option<Scoped>,
}
#[derive(Clone)]
struct Scoped {
    function: String,
    captures: Vec<String>,
    types: Vec<a::TypeArgument>,
    mutable: bool,
}
type Scope = BTreeMap<String, Binding>;
struct Lower<'a> {
    module: &'a a::Module,
    generated: Vec<a::Item>,
}
fn ty(name: &str, span: Span) -> a::Type {
    a::Type {
        kind: a::TypeKind::Named {
            path: vec![name.into()],
            arguments: vec![],
        },
        span,
    }
}
fn name(name: &str, span: Span) -> a::Expr {
    a::Expr {
        kind: a::ExprKind::Name(name.into()),
        span,
    }
}
fn pattern(name: &str, span: Span) -> a::Pattern {
    a::Pattern {
        kind: a::PatternKind::Binding {
            name: name.into(),
            by_ref: false,
            mutable: false,
        },
        span,
    }
}
fn bind(pattern: &a::Pattern, value: Binding, scope: &mut Scope) {
    match &pattern.kind {
        a::PatternKind::Binding { name, .. } => {
            scope.insert(name.clone(), value);
        }
        a::PatternKind::Tuple(fields) | a::PatternKind::Array(fields) => {
            for field in fields {
                bind(
                    field,
                    Binding {
                        ty: None,
                        ..value.clone()
                    },
                    scope,
                );
            }
        }
        a::PatternKind::Variant { fields, .. } => match fields {
            a::PatternFields::Tuple(fields) => {
                for field in fields {
                    bind(
                        field,
                        Binding {
                            ty: None,
                            ..value.clone()
                        },
                        scope,
                    );
                }
            }
            a::PatternFields::Named(fields, _) => {
                for (_, field) in fields {
                    bind(
                        field,
                        Binding {
                            ty: None,
                            ..value.clone()
                        },
                        scope,
                    );
                }
            }
            _ => {}
        },
        _ => {}
    }
}
fn params(parameters: &[a::Parameter], scope: &mut Scope) {
    for parameter in parameters {
        bind(
            &parameter.pattern,
            Binding {
                ty: parameter.ty.clone(),
                mutable: parameter.mutable,
                parameter: true,
                scoped: None,
            },
            scope,
        );
    }
}
enum ResolvedMember<'a> {
    Method(&'a a::Function),
    Field(&'a a::Type),
}

mod captures;
mod compatibility;
mod conversion;
mod mutation;
mod preparation;
mod receivers;
mod rewrite;
pub(super) use self::captures::generic_arguments;
use self::compatibility::compatible;
use self::compatibility::crate_name;
use self::mutation::mutations;
pub(super) use self::preparation::prepare;
use self::receivers::find_member;
use self::receivers::receiver_type;
use self::rewrite::rewrite_block;
