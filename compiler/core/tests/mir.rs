use clust_core::{check::check, hir, interpret, mir};
use clust_frontend::{
    source::{SourceId, Span},
    testing::parse,
};
use clust_semantics::OverflowChecks;
fn compile(source: &str) -> hir::Program {
    let parsed = parse(SourceId(0), source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    check(&parsed.module.unwrap()).unwrap()
}
#[path = "mir/cfg_execution_preserves_evaluation.rs"]
mod cfg_execution_preserves_evaluation;
#[path = "mir/explicit_moves_invalidate_only.rs"]
mod explicit_moves_invalidate_only;
