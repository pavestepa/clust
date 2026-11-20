use super::*;

pub(super) fn bind(pattern: &a::Pattern, names: &mut BTreeSet<String>) {
    match &pattern.kind {
        a::PatternKind::Binding { name, .. } => {
            names.insert(name.clone());
        }
        a::PatternKind::Tuple(fields) | a::PatternKind::Array(fields) => {
            for field in fields {
                bind(field, names);
            }
        }
        a::PatternKind::Variant { fields, .. } => match fields {
            a::PatternFields::Tuple(fields) => {
                for field in fields {
                    bind(field, names);
                }
            }
            a::PatternFields::Named(fields, _) => {
                for (_, field) in fields {
                    bind(field, names);
                }
            }
            _ => {}
        },
        _ => {}
    }
}
