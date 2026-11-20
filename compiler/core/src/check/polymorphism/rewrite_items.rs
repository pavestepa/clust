use super::*;

pub(super) fn rewrite_function(function: &mut a::Function, roots: &BTreeSet<String>) {
    let roots = scoped(roots, &function.generics);
    for parameter in &mut function.parameters {
        if !parameter.receiver {
            if let Some(ty) = &mut parameter.ty {
                rewrite_type(ty, &roots);
            }
        }
    }
    rewrite_type(&mut function.result, &roots);
    if let Some(body) = &mut function.body {
        rewrite_block(body, &roots);
    }
}

pub(super) fn rewrite_item(item: &mut a::Item, roots: &BTreeSet<String>) {
    match &mut item.kind {
        a::ItemKind::Function(function) => rewrite_function(function, roots),
        a::ItemKind::Aggregate {
            generics, members, ..
        } => {
            let roots = scoped(roots, generics);
            for member in members {
                match member {
                    a::Member::Field { name, ty, .. } if name != class_bases::BASE => {
                        rewrite_type(ty, &roots)
                    }
                    a::Member::Method { function, .. } => rewrite_function(function, &roots),
                    a::Member::Constructor {
                        parameters, body, ..
                    } => {
                        for parameter in parameters {
                            if let Some(ty) = &mut parameter.ty {
                                rewrite_type(ty, &roots);
                            }
                        }
                        rewrite_block(body, &roots);
                    }
                    a::Member::Destructor(body) => rewrite_block(body, &roots),
                    a::Member::Const { ty, value, .. } => {
                        rewrite_type(ty, &roots);
                        rewrite_expr(value, &roots);
                    }
                    _ => {}
                }
            }
        }
        a::ItemKind::Enum {
            generics, variants, ..
        } => {
            let roots = scoped(roots, generics);
            for variant in variants {
                match &mut variant.fields {
                    a::VariantFields::Tuple(types) => {
                        for ty in types {
                            rewrite_type(ty, &roots);
                        }
                    }
                    a::VariantFields::Named(fields) => {
                        for (_, ty) in fields {
                            rewrite_type(ty, &roots);
                        }
                    }
                    _ => {}
                }
            }
        }
        a::ItemKind::Interface {
            generics, methods, ..
        } => {
            let roots = scoped(roots, generics);
            for function in methods {
                rewrite_function(function, &roots);
            }
        }
        a::ItemKind::Const { ty, value, .. } => {
            rewrite_type(ty, roots);
            rewrite_expr(value, roots);
        }
        _ => {}
    }
}
