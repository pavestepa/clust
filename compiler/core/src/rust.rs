//! Standalone Rust source backend. Clust's checked MIR owns lifetime semantics;
//! generated raw places do not introduce Rust reference aliasing assumptions.
use crate::{
    hir::{self, Type, Value},
    mir::{self, *},
};
use clust_frontend::source::{Diagnostic, Source};
use clust_semantics::OverflowChecks;
mod classes;
mod lifecycle;
mod operations;
mod stack;
mod structured;

pub fn emit(program: &hir::Program, checks: OverflowChecks) -> Result<String, Diagnostic> {
    emit_with_sources(program, checks, &[])
}
pub fn emit_with_sources(
    program: &hir::Program,
    checks: OverflowChecks,
    sources: &[Source],
) -> Result<String, Diagnostic> {
    let active = structured::candidates(program, checks);
    emit_internal(
        &mir::lower(program),
        checks,
        sources,
        Some((program, &active)),
    )
}
pub fn emit_mir(program: &Program, checks: OverflowChecks) -> Result<String, Diagnostic> {
    emit_mir_with_sources(program, checks, &[])
}
pub fn emit_mir_with_sources(
    program: &Program,
    checks: OverflowChecks,
    sources: &[Source],
) -> Result<String, Diagnostic> {
    emit_internal(program, checks, sources, None)
}
struct Emitter {
    types: Vec<Type>,
    checks: OverflowChecks,
    destructors: mir::drop_plan::Destructors,
    resources: bool,
    stack_slots: std::collections::BTreeSet<usize>,
    classes: hir::Classes,
}

mod constants;
mod operands;
mod program;
mod types;
use self::program::emit_internal;
