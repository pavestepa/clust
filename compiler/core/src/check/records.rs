use super::generics::substitute::Substitution;
use super::*;
use std::{cell::RefCell, collections::BTreeSet};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Argument {
    Type(Type),
    Const(u64),
}
#[derive(Default)]
pub(super) struct Nominals {
    pub(super) interfaces: interfaces::Registry,
    definitions: BTreeMap<String, a::Item>,
    classes: RefCell<h::Classes>,
    instances: RefCell<BTreeMap<String, Type>>,
    active: RefCell<BTreeSet<String>>,
    pending_classes: RefCell<Vec<(a::Item, String, Substitution, usize)>>,
    class_depth: std::cell::Cell<usize>,
    opaque: RefCell<BTreeMap<String, bool>>,
    origins: RefCell<BTreeMap<String, String>>,
    arguments: RefCell<BTreeMap<String, Vec<Argument>>>,
}
impl Nominals {
    pub fn register_storage(&self, ty: &Type) {
        match ty {
            Type::Class {
                storage: Some(fields),
                copy,
                ..
            } => {
                self.classes.borrow_mut().insert(
                    ty.class_key(),
                    h::ClassDefinition {
                        fields: fields.clone(),
                        copy: *copy,
                    },
                );
                for field in fields {
                    self.register_storage(&field.ty)
                }
            }
            Type::Record { fields, .. } => {
                for field in fields {
                    self.register_storage(&field.ty)
                }
            }
            Type::Tuple(fields) => {
                for field in fields {
                    self.register_storage(field)
                }
            }
            Type::Array(ty, _) | Type::StoredView(ty) => self.register_storage(ty),
            Type::Enum { variants, .. } => {
                for field in variants.iter().flat_map(|v| &v.fields) {
                    self.register_storage(field)
                }
            }
            Type::Function {
                parameters, result, ..
            } => {
                for (_, ty) in parameters {
                    self.register_storage(ty)
                }
                self.register_storage(result)
            }
            _ => {}
        }
    }
    pub(super) fn interface_claims(
        &self,
        ty: &Type,
        constants: &crate::constants::Constants,
    ) -> Result<Vec<interfaces::Bound>> {
        let (Type::Record { name, .. } | Type::Class { name, .. }) = ty else {
            return Ok(Vec::new());
        };
        let Some(item) = self.definitions.get(&self.origin(name)) else {
            return Ok(Vec::new());
        };
        let a::ItemKind::Aggregate {
            generics,
            interfaces,
            ..
        } = &item.kind
        else {
            return Ok(Vec::new());
        };
        let mut outer = Substitution::default();
        for (generic, arg) in generics.iter().zip(self.arguments(name)) {
            match arg {
                Argument::Type(ty) => {
                    outer.types.insert(generic.name.clone(), ty);
                }
                Argument::Const(n) => {
                    outer.constants.insert(generic.name.clone(), n);
                }
            }
        }
        self.interfaces.claims(interfaces, &outer, constants, self)
    }
    pub(super) fn field_declaration(&self, owner: &str, name: &str) -> Option<Span> {
        let a::ItemKind::Aggregate { members, .. } = &self.definitions.get(owner)?.kind else {
            return None;
        };
        members.iter().find_map(|member| match member {
            a::Member::Field {
                name: field, span, ..
            } if field == name => Some(*span),
            _ => None,
        })
    }
}

#[cfg(test)]
mod class_tests {
    use super::*;
    use clust_frontend::{source::SourceId, testing::parse};
    fn registry(source: &str) -> Result<Nominals> {
        let parsed = parse(SourceId(0), source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        collect(&parsed.module.unwrap(), &BTreeMap::new())
    }
    #[test]
    fn class_indirection_breaks_recursive_payload_layout() {
        let nominals=registry("pub class Node {new(next: Link, value: i32) { self.next = next; self.value = value; } pub next:Link;pub value:i32} pub enum Link {End,More(Node)} pub class Left {new(right: Right) { self.right = right; } pub right:Right} pub class Right {new(left: Left) { self.left = left; } pub left:Left}").unwrap();
        let node = nominals.get("Node").unwrap();
        assert!(matches!(node, Type::Class { copy: false, .. }));
        assert_eq!(
            crate::layout::layout(&node).unwrap(),
            clust_semantics::Layout::POINTER
        );
        let classes = nominals.classes.borrow();
        assert_eq!(classes["Node"].fields.len(), 2);
        assert!(matches!(&classes["Left"].fields[0].ty,Type::Class {name,..} if name=="Right"));
        assert!(matches!(&classes["Right"].fields[0].ty,Type::Class {name,..} if name=="Left"));
        assert!(registry("use compiler_fixtures.record_133.{Node};").is_err());
        let reversed =
            registry("pub enum Link {End,More(Node)} pub class Node {new(next: Link, value: i32) { self.next = next; self.value = value; } pub next:Link;pub value:i32}").unwrap();
        assert_eq!(reversed.classes()["Node"].fields.len(), 2);
        let generic=registry("pub enum Link<T> {End,More(Node<T>)} pub class Node<T> {new(next: Link<T>, value: T) { self.next = next; self.value = value; } pub next:Link<T>;pub value:T} use compiler_fixtures.record_134.{Root};").unwrap();
        assert_eq!(generic.classes().len(), 1);
        assert!(registry("use compiler_fixtures.record_135.{Root}; pub class Node<T> {new(next: Node<{T;1}>) { self.next = next; } pub next:Node<{T;1}>}").is_err());
    }
    #[test]
    fn generic_class_instances_keep_distinct_payload_definitions() {
        let nominals =
            registry("pub class Cell<T> {new(value: T) { self.value = value; } pub value:T} use compiler_fixtures.record_136.{Holder};")
                .unwrap();
        let classes = nominals.classes.borrow();
        assert_eq!(classes.len(), 2);
        let types = classes
            .values()
            .map(|c| c.fields[0].ty.clone())
            .collect::<Vec<_>>();
        assert!(types.contains(&Type::Integer(I::I32)));
        assert!(types.contains(&Type::Integer(I::I64)));
    }
    #[test]
    fn copy_class_requires_copy_payload_and_still_has_pointer_layout() {
        assert!(registry("pub class Owned {new() {} } #[Copy] pub class Bad {new(value: Owned) { self.value = value; } pub value:Owned}").is_err());
        let nominals = registry(
            "#[Copy] pub class Number {new(value: i32) { self.value = value; } pub value:i32}",
        )
        .unwrap();
        let number = nominals.get("Number").unwrap();
        assert!(number.copyable());
        assert_eq!(
            crate::layout::layout(&number).unwrap(),
            clust_semantics::Layout::POINTER
        );
    }
}

mod arguments;
mod construction;
mod definitions;
mod discovery;
mod resolution;
pub(super) use self::discovery::collect;
