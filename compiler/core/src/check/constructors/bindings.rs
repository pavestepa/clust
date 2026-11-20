use super::*;

pub(super) fn is_this(value: &a::Expr) -> bool {
    match &value.kind {
        a::ExprKind::Name(name) => name == "this",
        a::ExprKind::Group(v) => is_this(v),
        _ => false,
    }
}

pub(super) fn join(target: &mut State, a: State, b: State) {
    match (a.reachable, b.reachable) {
        (true, true) => {
            target.initialized = a
                .initialized
                .intersection(&b.initialized)
                .cloned()
                .collect();
            target.reachable = true;
        }
        (true, false) => *target = a,
        (false, true) => *target = b,
        (false, false) => target.reachable = false,
    }
}

pub(super) fn bindings(pattern: &a::Pattern, names: &mut BTreeSet<String>) {
    match &pattern.kind {
        a::PatternKind::Binding { name, .. } => {
            names.insert(name.clone());
        }
        a::PatternKind::Tuple(fields) | a::PatternKind::Array(fields) => {
            for p in fields {
                bindings(p, names);
            }
        }
        a::PatternKind::Variant { fields, .. } => match fields {
            a::PatternFields::Tuple(fields) => {
                for p in fields {
                    bindings(p, names);
                }
            }
            a::PatternFields::Named(fields, _) => {
                for (_, p) in fields {
                    bindings(p, names);
                }
            }
            _ => {}
        },
        _ => {}
    }
}
