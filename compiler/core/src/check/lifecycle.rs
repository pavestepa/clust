//! Destructor declarations become private borrowed entries with checked effects.
use super::*;
use crate::mir::{drop_plan::Destructors, effects};

pub(super) fn prepare(module: &a::Module) -> Result<a::Module> {
    let mut module = module.clone();
    for item in &mut module.items {
        let a::ItemKind::Aggregate { members, .. } = &mut item.kind else {
            continue;
        };
        let mut seen = false;
        for member in members {
            let a::Member::Destructor(body) = member else {
                continue;
            };
            if seen {
                return Err(error(body.span, "E0390", "duplicate destructor"));
            }
            seen = true;
            if item.attributes.contains(&a::Attribute::Copy) {
                return Err(error(
                    body.span,
                    "E0390",
                    "Copy type cannot declare a destructor",
                ));
            }
            let span = body.span;
            *member = a::Member::Method {
                dispatch: a::MethodDispatch::Ordinary,
                private: true,
                function: a::Function {
                    name: "$drop".into(),
                    generics: vec![],
                    parameters: vec![a::Parameter {
                        pattern: a::Pattern {
                            kind: a::PatternKind::Binding {
                                name: "this".into(),
                                by_ref: false,
                                mutable: true,
                            },
                            span,
                        },
                        mutable: true,
                        ty: None,
                        receiver: true,
                        span,
                    }],
                    result: a::Type {
                        kind: a::TypeKind::Tuple(vec![]),
                        span,
                    },
                    body: Some(body.clone()),
                    c_abi: false,
                    unsafe_: false,
                    span,
                },
            };
        }
    }
    Ok(module)
}

#[cfg(test)]
mod tests;

mod destructors;
mod scheduling;
mod validation;
pub(super) use self::destructors::is_destructor;
pub(super) use self::validation::validate;
