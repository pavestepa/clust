use super::*;

pub(super) fn nominal_name(ty: &a::Type) -> Option<String> {
    match &ty.kind {
        a::TypeKind::Named { path, arguments } if path == &["$dyn"] => match arguments.first()? {
            a::TypeArgument::Type(interface) => nominal_name(interface),
            _ => None,
        },
        a::TypeKind::Named { path, .. } => Some(path.join("::")),
        _ => None,
    }
}

pub(super) fn find_member<'a>(
    module: &'a a::Module,
    owner: &str,
    name: &str,
) -> Option<ResolvedMember<'a>> {
    let mut pending = vec![owner.to_owned()];
    let mut seen = BTreeSet::new();
    while let Some(owner) = pending.pop() {
        if !seen.insert(owner.clone()) {
            continue;
        }
        for item in &module.items {
            match &item.kind {
                a::ItemKind::Aggregate {
                    name: class,
                    interfaces,
                    members,
                    ..
                } if class == &owner => {
                    for member in members {
                        match member {
                            a::Member::Method { function, .. } if function.name == name => {
                                return Some(ResolvedMember::Method(function))
                            }
                            a::Member::Field {
                                name: field, ty, ..
                            } if field == name => return Some(ResolvedMember::Field(ty)),
                            _ => {}
                        }
                    }
                    pending.extend(interfaces.iter().rev().filter_map(nominal_name));
                }
                a::ItemKind::Interface {
                    name: interface,
                    methods,
                    ..
                } if interface == &owner => {
                    if let Some(method) = methods.iter().find(|f| f.name == name) {
                        return Some(ResolvedMember::Method(method));
                    }
                }
                _ => {}
            }
        }
    }
    None
}

pub(super) fn receiver_type(value: &a::Expr, scope: &Scope, module: &a::Module) -> Option<String> {
    match &value.kind {
        a::ExprKind::Name(name) => {
            nominal_name(scope.get(name.strip_prefix("$free$")?)?.ty.as_ref()?)
        }
        a::ExprKind::Group(value) => receiver_type(value, scope, module),
        a::ExprKind::Member { value, name } => {
            match find_member(module, &receiver_type(value, scope, module)?, name)? {
                ResolvedMember::Field(ty) => nominal_name(ty),
                _ => None,
            }
        }
        _ => None,
    }
}
