use super::*;

pub(super) fn rewrite_type(value: &mut a::Type, roots: &BTreeSet<String>) {
    match &mut value.kind {
        a::TypeKind::Named { path, arguments } => {
            if path == &["$dyn"] {
                if let [a::TypeArgument::Type(interface)] = arguments.as_slice() {
                    *path = vec![format!("{PREFIX}$dyn${}", name_of(interface))];
                    arguments.clear();
                    return;
                }
            }
            if roots.contains(&path.join("::")) {
                *path = vec![format!("{PREFIX}{}", path.join("::"))];
            }
            for argument in arguments {
                rewrite_argument(argument, roots);
            }
        }
        a::TypeKind::Tuple(types) => {
            for ty in types {
                rewrite_type(ty, roots);
            }
        }
        a::TypeKind::Array { element, length } => {
            rewrite_type(element, roots);
            rewrite_expr(length, roots);
        }
        a::TypeKind::Pointer { pointee, .. } => rewrite_type(pointee, roots),
        a::TypeKind::Function {
            parameters, result, ..
        } => {
            for (_, ty) in parameters {
                rewrite_type(ty, roots);
            }
            rewrite_type(result, roots);
        }
    }
}

pub(super) fn rewrite_argument(arg: &mut a::TypeArgument, roots: &BTreeSet<String>) {
    match arg {
        a::TypeArgument::Type(ty) => rewrite_type(ty, roots),
        a::TypeArgument::Const(expr) => rewrite_expr(expr, roots),
    }
}

pub(super) fn scoped(roots: &BTreeSet<String>, generics: &[a::Generic]) -> BTreeSet<String> {
    roots
        .iter()
        .filter(|name| !generics.iter().any(|g| &g.name == *name))
        .cloned()
        .collect()
}
