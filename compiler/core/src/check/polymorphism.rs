//! Closed-world class values retain a complete concrete payload. Runtime tag
//! dispatch reuses verified enum borrowing and destruction, without slicing.
use super::*;
use std::collections::BTreeSet;
const PREFIX: &str = "$poly$";
pub(super) fn accessor(field: &str, operation: &str) -> String {
    format!("$field${operation}${field}")
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
// Generated nodes must not share source-span storage-inference keys with
// receiver parameters, user expressions or each other. Reserve a synthetic
// source namespace; these nodes never index source text. Offsets identify the
// fixed roles in each generated body, consistently across analysis passes.
pub(super) fn site(span: Span, offset: usize) -> Span {
    span.synthetic(offset)
}

fn expr(name: &str, span: Span) -> a::Expr {
    let span = site(
        span,
        match name {
            "this" => 2,
            "$input" => 4,
            "$object" => 8,
            _ => 10,
        },
    );
    a::Expr {
        kind: a::ExprKind::Name(name.into()),
        span,
    }
}

mod coercion;
mod dispatch;
mod rewrite_expressions;
mod rewrite_items;
mod rewrite_types;
pub(super) use self::coercion::is_polymorphic;
use self::coercion::name_of;
pub(super) use self::dispatch::prepare;
use self::rewrite_expressions::rewrite_block;
use self::rewrite_expressions::rewrite_expr;
use self::rewrite_items::rewrite_function;
use self::rewrite_items::rewrite_item;
use self::rewrite_types::rewrite_argument;
use self::rewrite_types::rewrite_type;
use self::rewrite_types::scoped;
