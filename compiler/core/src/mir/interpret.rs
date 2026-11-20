//! Reference execution of explicit CFG, independent of HIR control-flow traversal.
use super::*;
mod drop;
use crate::reference_memory::{self as memory, Argument, Reference, Slot};
type Views = std::collections::BTreeMap<usize, Reference>;
use crate::interpret::{binary, cast, Trap, TrapKind};
use clust_semantics::{Integer, IntegerType, OverflowChecks};
pub fn execute(
    program: &Program,
    entry: usize,
    arguments: Vec<Value>,
    checks: OverflowChecks,
    fuel: u64,
) -> Result<Value, Trap> {
    execute_traced(program, entry, arguments, checks, fuel).0
}
#[derive(Clone, Debug, PartialEq)]
pub struct DropEvent {
    pub entry: usize,
    pub value: Value,
}
pub fn execute_traced(
    program: &Program,
    entry: usize,
    arguments: Vec<Value>,
    checks: OverflowChecks,
    fuel: u64,
) -> (Result<Value, Trap>, Vec<DropEvent>) {
    let span = program
        .functions
        .get(entry)
        .map(|f| f.span)
        .unwrap_or(Span {
            source: clust_frontend::source::SourceId(0),
            start: 0,
            end: 0,
        });
    let mut engine = Engine {
        program,
        checks,
        fuel,
        depth: 0,
        active_panic: false,
        clock: Default::default(),
        allocations: Default::default(),
        weak: Default::default(),
        events: Vec::new(),
    };
    let result = engine.call(
        entry,
        arguments.into_iter().map(Argument::Owned).collect(),
        span,
    );
    (result, engine.events)
}
struct Engine<'a> {
    program: &'a Program,
    checks: OverflowChecks,
    fuel: u64,
    depth: usize,
    active_panic: bool,
    clock: memory::Clock,
    allocations: memory::RawAllocations,
    weak: memory::WeakRegistry,
    events: Vec<DropEvent>,
}

mod execution;
mod invocation;
mod operands;
mod operations;
use self::operands::operand;
use self::operands::resolve;
