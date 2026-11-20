use crate::hir::{self as h, Type, Value};
use clust_frontend::{
    ast as a,
    source::{Diagnostic, Span},
};
use clust_semantics::{Integer, IntegerType as I};
use std::collections::BTreeMap;
mod class_bases;
mod classes;
mod classes_init;
mod closures;
mod construction_contracts;
mod constructors;
mod consume;
mod default_methods;
mod entries;
mod enums;
mod function_values;
mod future;
mod generics;
mod inheritance;
mod interface_parameters;
mod interfaces;
mod intrinsics;
mod lifecycle;
mod methods;
mod patterns;
mod polymorphism;
mod privacy;
mod records;
mod rust_adapters;
mod storage;
mod text;
mod type_walk;
mod uses;
mod views;
pub(crate) mod weak;

type Result<T> = std::result::Result<T, Diagnostic>;
/// Parameters inferred as consuming, by declaration name (without entry suffix).
type Consuming = std::rc::Rc<std::cell::RefCell<BTreeMap<String, Vec<bool>>>>;
#[derive(Clone)]
struct Signature {
    modes: Vec<Option<crate::mir::loans::Access>>,
    id: usize,
    parameters: Vec<Type>,
    result: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReturnInfo {
    complete: bool,
    parameters: Vec<Type>,
    modes: Vec<Option<crate::mir::loans::Access>>,
    ty: Type,
    view: Option<crate::mir::returns::Contract>,
    storage: crate::mir::returns::StorageContract,
    writes: crate::mir::storage_writes::Contract,
}
type ReturnContracts = BTreeMap<String, ReturnInfo>;
enum Pass {
    Ready(h::Program),
    Retry(ReturnContracts),
}
pub fn check(module: &a::Module) -> std::result::Result<h::Program, Vec<Diagnostic>> {
    let closures = closures::prepare(module).map_err(|e| vec![e])?;
    let module = &closures;
    let (adapted, adapters) = rust_adapters::prepare(module).map_err(|error| vec![error])?;
    let module = &adapted;
    let static_parameters = interface_parameters::prepare(module);
    let module = &static_parameters;
    inheritance::validate(module).map_err(|e| vec![e])?;
    let prepared = default_methods::prepare(module).map_err(|e| vec![e])?;
    let inherited = class_bases::prepare(&prepared).map_err(|e| vec![e])?;
    let polymorphic = polymorphism::prepare(&inherited).map_err(|e| vec![e])?;
    let module = &polymorphic;
    let mut contracts = ReturnContracts::new();
    let consuming = Consuming::default();
    for _ in 0..64 {
        match check_pass(module, &contracts, &consuming)? {
            Pass::Ready(mut program) => {
                for function in &mut program.functions {
                    let name = function.name.split("$entry").next().unwrap();
                    function.rust_adapter = adapters.get(name).cloned();
                }
                return Ok(program);
            }
            Pass::Retry(next) => contracts = next,
        }
    }
    Err(vec![error(
        module.items.first().map(|i| i.span).unwrap_or(Span {
            source: clust_frontend::source::SourceId(0),
            start: 0,
            end: 0,
        }),
        "E0395",
        "result contracts did not converge",
    )])
}

pub fn check_clust(module: &a::Module) -> std::result::Result<h::Program, Vec<Diagnostic>> {
    let prepared = closures::prepare(module).map_err(|e| vec![e])?;
    let module = &prepared;
    inheritance::validate(module).map_err(|error| vec![error])?;
    construction_contracts::validate(module).map_err(|error| vec![error])?;
    check(module)
}

#[derive(Clone)]
struct Checker<'a> {
    member_owner: Option<String>,
    unsafe_depth: usize,
    callable_parameters: std::collections::BTreeSet<usize>,
    storage_hints: storage::Modes,
    storage_requests: storage::Modes,
    storage_roots: std::collections::BTreeSet<usize>,
    storage_sources: BTreeMap<usize, Vec<h::Expr>>,
    storage_parameters: std::collections::BTreeSet<usize>,
    signatures: &'a BTreeMap<String, Signature>,
    mono: &'a std::cell::RefCell<generics::Monomorphization>,
    constants: &'a BTreeMap<String, (Value, Type)>,
    nominals: &'a records::Nominals,
    private_functions: &'a BTreeMap<String, Span>,
    scopes: Vec<BTreeMap<String, usize>>,
    locals: Vec<h::Local>,
    result: Type,
    actual_result: Option<Type>,
    loops: usize,
    value_uses: BTreeMap<String, Vec<Span>>,
    pattern_views: bool,
    continuations: Vec<(a::Block, usize)>,
}

// A break belonging to a nested loop does not exit the current loop. Walk
// expression blocks too: a break can occur while evaluating an argument.

mod access;
mod blocks;
mod branches;
mod declarations;
mod divergence;
mod expressions;
mod literals;
mod pipeline;
mod scope;
mod type_resolution;
use self::declarations::check_declaration;
use self::divergence::error;
use self::divergence::expression_diverges;
use self::divergence::same;
pub(crate) use self::literals::constant_value;
use self::literals::literal;
use self::literals::magnitude;
use self::pipeline::check_pass;
pub use self::type_resolution::resolve_type;
pub(crate) use self::type_resolution::resolve_type_in;
use self::type_resolution::resolve_type_with;
