//! Visit type occurrences without confusing declarations, expressions or binders.
use clust_frontend::ast as a;
pub(super) fn module(module: &a::Module, visit: &mut impl FnMut(&a::Type)) {
    for item in &module.items {
        match &item.kind {
            a::ItemKind::Function(f) => function(f, visit),
            a::ItemKind::Aggregate {
                generics: gs,
                interfaces,
                members,
                ..
            } => {
                generics(gs, visit);
                for value in interfaces {
                    ty(value, visit);
                }
                for member in members {
                    match member {
                        a::Member::Field { ty: value, .. } => ty(value, visit),
                        a::Member::Method { function: f, .. } => function(f, visit),
                        a::Member::Constructor {
                            parameters, body, ..
                        } => {
                            params(parameters, visit);
                            block(body, visit);
                        }
                        a::Member::Destructor(body) => block(body, visit),
                        a::Member::Const {
                            ty: value,
                            value: expr,
                            ..
                        } => {
                            ty(value, visit);
                            expression(expr, visit);
                        }
                    }
                }
            }
            a::ItemKind::Interface {
                generics: gs,
                methods,
                ..
            } => {
                generics(gs, visit);
                for f in methods {
                    function(f, visit);
                }
            }
            a::ItemKind::Enum {
                generics: gs,
                variants,
                ..
            } => {
                generics(gs, visit);
                for variant in variants {
                    match &variant.fields {
                        a::VariantFields::Tuple(types) => {
                            for value in types {
                                ty(value, visit);
                            }
                        }
                        a::VariantFields::Named(fields) => {
                            for (_, value) in fields {
                                ty(value, visit);
                            }
                        }
                        _ => {}
                    }
                }
            }
            a::ItemKind::Const {
                ty: value,
                value: expr,
                ..
            } => {
                ty(value, visit);
                expression(expr, visit);
            }
            a::ItemKind::Extern(functions) => {
                for f in functions {
                    function(f, visit);
                }
            }
            _ => {}
        }
    }
}
fn generics(gs: &[a::Generic], visit: &mut impl FnMut(&a::Type)) {
    for g in gs {
        match &g.kind {
            a::GenericKind::Type {
                constraints,
                default,
            } => {
                for value in constraints {
                    ty(value, visit);
                }
                if let Some(value) = default {
                    ty(value, visit);
                }
            }
            a::GenericKind::Const { ty: value, default } => {
                ty(value, visit);
                if let Some(value) = default {
                    expression(value, visit);
                }
            }
        }
    }
}
fn params(ps: &[a::Parameter], visit: &mut impl FnMut(&a::Type)) {
    for p in ps {
        if let Some(value) = &p.ty {
            ty(value, visit);
        }
    }
}
fn function(f: &a::Function, visit: &mut impl FnMut(&a::Type)) {
    generics(&f.generics, visit);
    params(&f.parameters, visit);
    ty(&f.result, visit);
    if let Some(body) = &f.body {
        block(body, visit);
    }
}

mod expressions;
mod types;
use self::expressions::block;
use self::expressions::expression;
use self::types::arg;
use self::types::ty;
