//! A plain interface parameter is static sugar for an independently inferred
//! type parameter. It never introduces an erased value or shared ownership.
use super::*;
use std::collections::BTreeSet;

pub(super) fn prepare(module: &a::Module) -> a::Module {
    let interfaces: BTreeSet<_> = module
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            a::ItemKind::Interface { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let mut module = module.clone();
    for item in &mut module.items {
        match &mut item.kind {
            a::ItemKind::Function(function) if !function.c_abi => {
                function_parameters(function, &interfaces, &[])
            }
            a::ItemKind::Aggregate {
                generics, members, ..
            } => {
                for member in members {
                    if let a::Member::Method { function, .. } = member {
                        function_parameters(function, &interfaces, generics);
                    }
                }
            }
            a::ItemKind::Interface {
                generics, methods, ..
            } => {
                for method in methods {
                    function_parameters(method, &interfaces, generics);
                }
            }
            _ => {}
        }
    }
    module
}
fn function_parameters(
    function: &mut a::Function,
    interfaces: &BTreeSet<String>,
    outer: &[a::Generic],
) {
    let shadowed: BTreeSet<_> = function
        .generics
        .iter()
        .chain(outer)
        .map(|generic| generic.name.clone())
        .collect();
    for (index, parameter) in function.parameters.iter_mut().enumerate() {
        let Some(ty) = &mut parameter.ty else {
            continue;
        };
        let a::TypeKind::Named { path, .. } = &ty.kind else {
            continue;
        };
        let interface = path.join("::");
        if !interfaces.contains(&interface) || shadowed.contains(&interface) {
            continue;
        }
        let name = format!("$interface-parameter-{index}");
        function.generics.push(a::Generic {
            name: name.clone(),
            kind: a::GenericKind::Type {
                constraints: vec![ty.clone()],
                default: None,
            },
            span: parameter.span,
        });
        ty.kind = a::TypeKind::Named {
            path: vec![name],
            arguments: vec![],
        };
    }
}
