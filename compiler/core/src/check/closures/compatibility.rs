use super::*;

pub(super) fn crate_name(value: &str, span: Span) -> a::Expr {
    name(value, span)
}

pub(super) fn compatible(a: &a::Function, b: &a::Function) -> bool {
    a.generics.is_empty()
        && b.generics.is_empty()
        && a.parameters.len() == b.parameters.len()
        && a.result.kind == b.result.kind
        && a.parameters.iter().zip(&b.parameters).all(|(a, b)| {
            a.receiver == b.receiver
                && a.mutable == b.mutable
                && (a.receiver || a.ty.as_ref().map(|t| &t.kind) == b.ty.as_ref().map(|t| &t.kind))
        })
}
