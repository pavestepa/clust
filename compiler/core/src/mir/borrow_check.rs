//! Flow-sensitive origins and last-use loan checking for local safe views.
//! View slots carry pointers, while their logical type remains the target type.
use super::{
    dataflow,
    loans::{overlaps, Access},
    *,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
type Bindings = BTreeMap<Place, BTreeSet<BlockId>>;
#[derive(Clone, Default, PartialEq, Eq)]
struct Origin {
    places: BTreeSet<Place>,
    parents: BTreeSet<BlockId>,
    ancestors: BTreeSet<BlockId>,
}
#[derive(Clone, Debug)]
pub struct Error {
    pub loan: Option<BlockId>,
    pub block: BlockId,
    pub message: &'static str,
}

/// Flow-sensitive provenance shared by loan checking and interprocedural effects.
pub(crate) struct Provenance {
    incoming: Vec<Option<Bindings>>,
    origins: Vec<Origin>,
    sites: Vec<Option<BlockId>>,
}

mod components;
mod conflicts;
mod diagnostics;
mod origins;
mod parameters;
mod provenance_analysis;
mod transfer;
pub use self::conflicts::check;
use self::diagnostics::compatible_prefix;
use self::diagnostics::covers;
pub use self::diagnostics::diagnostic;
use self::origins::resolve;
use self::parameters::merge;
use self::parameters::parameter_loans;
pub(crate) use self::provenance_analysis::provenance;
use self::transfer::call_origin;
use self::transfer::parameter_slot;
use self::transfer::transfer;
