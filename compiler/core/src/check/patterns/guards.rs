use super::*;

pub(super) fn readonly_guard(expr: &h::Expr) -> bool {
    use h::ExprKind::*;
    fn block(block: &h::Block) -> bool {
        block
            .statements
            .iter()
            .all(|s| matches!(s, h::Statement::Evaluate(e) if readonly_guard(e)))
    }
    match &expr.kind {
        Value(_) | Local(_) | View(_) => true,
        ArgumentView { place, access } => {
            *access == crate::mir::loans::Access::Readonly && readonly_guard(place)
        }
        // The admitted value subset has Copy arguments and no foreign calls,
        // mutable captures, globals or resource operations. Thus these calls
        // cannot mutate a guard's input; ownership/FFI support must use summaries.
        Call(_, values) | Tuple(values) | Array(values) => values.iter().all(readonly_guard),
        CallIndirect(callee, values, _) => {
            readonly_guard(callee) && values.iter().all(readonly_guard)
        }
        StoredView(value)
        | StoreView(value)
        | Unary(_, value)
        | Cast(value)
        | Field(value, _)
        | Tag(value)
        | Payload(value, _) => readonly_guard(value),
        Identity(left, right) | Binary(_, left, right) | Index(left, right) => {
            readonly_guard(left) && readonly_guard(right)
        }
        Block(body) => block(body),
        If { condition, yes, no } => {
            readonly_guard(condition) && block(yes) && no.as_ref().is_none_or(block)
        }
        _ => false,
    }
}

pub(super) fn readonly_pattern(pattern: &mut a::Pattern) {
    match &mut pattern.kind {
        a::PatternKind::Binding { mutable, .. } => *mutable = false,
        a::PatternKind::Tuple(v) | a::PatternKind::Array(v) => {
            v.iter_mut().for_each(readonly_pattern)
        }
        a::PatternKind::Variant { fields, .. } => match fields {
            a::PatternFields::Tuple(v) => v.iter_mut().for_each(readonly_pattern),
            a::PatternFields::Named(v, _) => v.iter_mut().for_each(|(_, p)| readonly_pattern(p)),
            _ => (),
        },
        _ => (),
    }
}
