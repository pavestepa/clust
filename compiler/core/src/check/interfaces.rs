//! Nominal interface obligations and abstract generic method surfaces. Concrete
//! calls use the existing entry specializer; no interface value or vtable is made.
use super::*;
use generics::substitute::Substitution;
use std::{cell::RefCell, collections::BTreeSet};
mod signatures;
use signatures::{compatible, signature};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Bound {
    name: String,
    arguments: Vec<records::Argument>,
}
#[derive(Default)]
pub(super) struct Registry {
    definitions: BTreeMap<String, a::Item>,
    methods: BTreeMap<String, (a::Function, bool)>,
    markers: RefCell<BTreeMap<String, Vec<Bound>>>,
    active: RefCell<BTreeSet<String>>,
}
pub(super) fn copy(ty: &a::Type) -> bool {
    matches!(&ty.kind,a::TypeKind::Named {path,arguments} if path==&["Copy".to_owned()] && arguments.is_empty())
}
impl Registry {
    pub(super) fn collect(module: &a::Module) -> Result<Self> {
        let mut result = Self::default();
        let mut names = BTreeSet::new();
        for item in &module.items {
            match &item.kind {
                a::ItemKind::Interface { name, methods, .. } => {
                    if !names.insert(name.clone())
                        || result
                            .definitions
                            .insert(name.clone(), item.clone())
                            .is_some()
                    {
                        return Err(error(
                            item.span,
                            "E0400",
                            "duplicate interface/type declaration",
                        ));
                    }
                    let mut seen = BTreeSet::new();
                    for method in methods {
                        if !seen.insert(&method.name) {
                            return Err(error(method.span, "E0400", "duplicate interface method"));
                        }
                        if method.body.is_some() {
                            return Err(error(
                                method.span,
                                "E0400",
                                "interface methods must be declarations",
                            ));
                        }
                    }
                }
                a::ItemKind::Aggregate { name, .. } | a::ItemKind::Enum { name, .. } => {
                    if !names.insert(name.clone()) {
                        return Err(error(
                            item.span,
                            "E0400",
                            "duplicate interface/type declaration",
                        ));
                    }
                }
                a::ItemKind::Function(f) => {
                    result
                        .methods
                        .insert(f.name.clone(), (f.clone(), item.private));
                }
                _ => {}
            }
        }
        Ok(result)
    }
}

mod claims;
mod declarations;
mod implementation;
mod instances;
mod markers;
use self::markers::apply_generics;
