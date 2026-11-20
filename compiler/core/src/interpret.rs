//! Bounded reference execution for the currently checked value subset.
//! This is an oracle, not the native runtime or a substitute for Rust source lowering.
use crate::hir::*;
mod drop;
pub use crate::mir::interpret::DropEvent;
use crate::reference_memory::{self as memory, Argument, Reference as Place, Slot};
use clust_frontend::source::Span;
use clust_semantics::{ArithmeticError, BinaryOp, Integer, OverflowChecks};

#[derive(Clone, Debug, PartialEq)]
pub enum TrapKind {
    Arithmetic(ArithmeticError),
    Bounds,
    StepLimit,
    CallDepth,
    InvalidIr,
    Allocation,
    Assertion,
    Output,
    ForeignFunction,
    DoublePanic,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Trap {
    pub kind: TrapKind,
    pub span: Span,
}
#[derive(Debug)]
enum Signal {
    Return,
    Break,
    Continue,
    Trap(Trap),
}
type Result<T> = std::result::Result<T, Signal>;

pub fn execute(
    program: &Program,
    entry: FunctionId,
    arguments: Vec<Value>,
    checks: OverflowChecks,
    fuel: u64,
) -> std::result::Result<Value, Trap> {
    execute_traced(program, entry, arguments, checks, fuel).0
}
pub fn execute_traced(
    program: &Program,
    entry: FunctionId,
    arguments: Vec<Value>,
    checks: OverflowChecks,
    fuel: u64,
) -> (std::result::Result<Value, Trap>, Vec<DropEvent>) {
    let mut engine = Engine {
        program,
        checks,
        fuel,
        depth: 0,
        clock: Default::default(),
        allocations: Default::default(),
        weak: Default::default(),
        destructors: crate::mir::drop_plan::Destructors::from_functions(&program.functions)
            .expect("checked destructors"),
        active_panic: false,
        events: Vec::new(),
    };
    let span = program
        .functions
        .get(entry)
        .map(|f| f.span)
        .unwrap_or(Span {
            source: clust_frontend::source::SourceId(0),
            start: 0,
            end: 0,
        });
    let result = match engine.call(
        entry,
        arguments.into_iter().map(Argument::Owned).collect(),
        span,
    ) {
        Ok(value) => Ok(value),
        Err(Signal::Trap(trap)) => Err(trap),
        _ => Err(Trap {
            kind: TrapKind::InvalidIr,
            span,
        }),
    };
    (result, engine.events)
}

struct Engine<'a> {
    program: &'a Program,
    checks: OverflowChecks,
    fuel: u64,
    depth: usize,
    clock: memory::Clock,
    allocations: memory::RawAllocations,
    weak: memory::WeakRegistry,
    destructors: crate::mir::drop_plan::Destructors,
    active_panic: bool,
    events: Vec<DropEvent>,
}
struct Frame {
    returned_view: bool,
    views: std::collections::BTreeMap<usize, Place>,
    locals: Vec<Slot>,
    return_staging: Option<(Value, Type)>,
    temporary_places: Vec<Slot>,
}

mod blocks;
mod expressions;
mod invocation;
mod places;
mod scalars;
pub use self::scalars::binary;
pub use self::scalars::cast;
use self::scalars::index_value;
use self::scalars::trap;
