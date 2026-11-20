//! Declaration hierarchy, independent of allocation and dispatch lowering.
//! Inherited declarations retain the lexical owner of their private members.
use super::*;
use std::collections::BTreeSet;
mod signatures;
use signatures::Bindings;

pub(super) fn instantiate_method(
    generics: &[a::Generic],
    arguments: &[a::TypeArgument],
    method: &a::Function,
    constants: &crate::constants::Constants,
    span: Span,
) -> Result<a::Function> {
    let mut method = method.clone();
    Bindings::parent(generics, arguments, span, constants)?.method(&mut method);
    Ok(method)
}

pub(super) fn instantiate_type(
    generics: &[a::Generic],
    arguments: &[a::TypeArgument],
    source: &a::Type,
    constants: &crate::constants::Constants,
) -> Result<a::Type> {
    let mut ty = source.clone();
    Bindings::parent(generics, arguments, source.span, constants)?.ty(&mut ty);
    Ok(ty)
}

#[derive(Clone)]
struct Method {
    function: a::Function,
    private: bool,
    dispatch: a::MethodDispatch,
    owner: String,
}
#[derive(Clone, Default)]
struct Node {
    fields: BTreeMap<String, (bool, Span)>,
    methods: BTreeMap<String, Method>,
}
struct Hierarchy<'a> {
    definitions: BTreeMap<String, &'a a::Item>,
    interfaces: BTreeSet<String>,
    active: BTreeSet<String>,
    nodes: BTreeMap<String, Node>,
    constants: crate::constants::Constants,
}

#[cfg(test)]
mod tests;

mod validation;
pub(super) use self::validation::validate;
