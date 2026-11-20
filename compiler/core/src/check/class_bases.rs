//! Concrete inheritance lowering. A hidden, uniquely owned base component is
//! destroyed after derived fields. Inherited bodies keep lexical privileges
//! while their receiver remains the complete, most-derived concrete class.
use super::*;
pub(super) const BASE: &str = "$base";

#[derive(Clone)]
struct Class {
    item: a::Item,
    base: Option<String>,
}
pub(super) fn helper(owner: &str, actual: &str, method: &str, private: bool) -> String {
    format!(
        "{owner}.${}body${actual}${method}",
        if private { "private$" } else { "" }
    )
}
fn name(value: &str, span: Span) -> a::Expr {
    a::Expr {
        kind: a::ExprKind::Name(value.into()),
        span,
    }
}

mod ancestry;
mod fields;
mod preparation;
mod rewrite;
use self::ancestry::ancestors;
pub(super) use self::preparation::prepare;
use self::rewrite::rewrite_block;
