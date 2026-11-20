use super::*;
/// Methods become named functions; receiver flags force borrowed entries.
pub(super) fn collect(module: &a::Module) -> a::Module {
    let mut module = module.clone();
    let mut functions = Vec::new();
    for item in &mut module.items {
        let a::ItemKind::Aggregate {
            name,
            generics,
            members,
            ..
        } = &mut item.kind
        else {
            continue;
        };
        members.retain(|member| {
            let a::Member::Method {
                function, private, ..
            } = member
            else {
                return true;
            };
            let mut function = function.clone();
            for parameter in &mut function.parameters {
                if parameter.receiver {
                    parameter.ty = Some(a::Type {
                        kind: a::TypeKind::Named {
                            path: vec![name.clone()],
                            arguments: generics
                                .iter()
                                .map(|generic| match generic.kind {
                                    a::GenericKind::Type { .. } => a::TypeArgument::Type(a::Type {
                                        kind: a::TypeKind::Named {
                                            path: vec![generic.name.clone()],
                                            arguments: vec![],
                                        },
                                        span: generic.span,
                                    }),
                                    a::GenericKind::Const { .. } => {
                                        a::TypeArgument::Const(a::Expr {
                                            kind: a::ExprKind::Name(generic.name.clone()),
                                            span: generic.span,
                                        })
                                    }
                                })
                                .collect(),
                        },
                        span: parameter.span,
                    });
                }
            }
            function.name = format!("{name}.{}", function.name);
            let mut parameters = generics.clone();
            parameters.append(&mut function.generics);
            function.generics = parameters;
            functions.push(a::Item {
                span: function.span,
                kind: a::ItemKind::Function(function),
                private: *private,
                attributes: vec![],
            });
            false
        });
    }
    module.items.extend(functions);
    module
}
pub(super) fn static_call(callee: &a::Expr) -> Option<(String, Vec<a::TypeArgument>)> {
    let a::ExprKind::Member { value, name } = &callee.kind else {
        return None;
    };
    match &value.kind {
        a::ExprKind::Name(owner) => Some((format!("{owner}.{name}"), vec![])),
        a::ExprKind::Specialize { value, arguments } => {
            let a::ExprKind::Name(owner) = &value.kind else {
                return None;
            };
            Some((format!("{owner}.{name}"), arguments.clone()))
        }
        _ => None,
    }
}

pub(super) fn resolved_static_call(
    callee: &a::Expr,
    nominals: &records::Nominals,
) -> Option<(String, Vec<a::TypeArgument>)> {
    let (name, mut arguments) = static_call(callee)?;
    let (owner, method) = name.rsplit_once('.')?;
    if nominals.get(owner).is_some() {
        let mut supplied = nominals
            .arguments(owner)
            .into_iter()
            .map(|arg| match arg {
                records::Argument::Type(ty) => {
                    a::TypeArgument::Type(generics::substitute::ast_type(&ty, callee.span))
                }
                records::Argument::Const(n) => a::TypeArgument::Const(a::Expr {
                    kind: a::ExprKind::Literal(a::Literal::Integer(n.to_string())),
                    span: callee.span,
                }),
            })
            .collect::<Vec<_>>();
        supplied.append(&mut arguments);
        Some((format!("{}.{method}", nominals.origin(owner)), supplied))
    } else {
        Some((name, arguments))
    }
}

/// NMM §11: a non-Copy parameter is a borrow of the caller's value (R, or W for
/// `var`); receivers always borrow. Copy values are passed by value. Owned
/// entries of consuming parameters are selected per call (see `entries`).
pub(super) fn parameter_modes(
    function: &a::Function,
    types: &[Type],
) -> Vec<Option<crate::mir::loans::Access>> {
    function
        .parameters
        .iter()
        .enumerate()
        .map(|(index, p)| {
            let borrowed = p.receiver || types.get(index).is_some_and(|ty| !ty.copyable());
            borrowed.then_some(if p.mutable {
                crate::mir::loans::Access::Mutable
            } else {
                crate::mir::loans::Access::Readonly
            })
        })
        .collect()
}

mod instances;
