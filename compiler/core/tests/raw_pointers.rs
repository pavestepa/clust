use clust_core::{check::check, hir, interpret, mir, rust};
use clust_frontend::{source::SourceId, testing::parse_ir as parse};
use clust_semantics::OverflowChecks;

fn checked(source: &str) -> Result<hir::Program, Vec<String>> {
    let parsed = parse(SourceId(0), source);
    assert!(
        parsed.diagnostics.is_empty(),
        "{source}: {:?}",
        parsed.diagnostics
    );
    check(&parsed.module.unwrap()).map_err(|e| e.into_iter().map(|e| e.code.to_owned()).collect())
}

#[path = "raw_pointers/raw_scalar_operations_match.rs"]
mod raw_scalar_operations_match;
#[path = "raw_pointers/reference_interpreters_diagnose_dangling.rs"]
mod reference_interpreters_diagnose_dangling;
