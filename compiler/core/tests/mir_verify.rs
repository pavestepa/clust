//! Corrupt checked MIR to ensure the backend does not trust frontend invariants.
use clust_core::{
    check::check,
    mir::{self, Operand, Operation, Place, Terminator},
    rust,
};
use clust_frontend::{source::SourceId, testing::parse_ir as parse};
use clust_semantics::OverflowChecks;

fn program(source: &str) -> mir::Program {
    let parsed = parse(SourceId(0), source);
    assert!(parsed.diagnostics.is_empty());
    let p = mir::lower(&check(&parsed.module.unwrap()).unwrap());
    mir::verify::verify(&p).unwrap();
    p
}
fn rejected(p: &mir::Program, message: &str) {
    let error = mir::verify::verify(p).expect_err("invalid MIR accepted");
    assert!(error.message.contains(message), "{error:?}");
    assert_eq!(
        rust::emit_mir(p, OverflowChecks::Checked).unwrap_err().code,
        "E0401"
    );
}
const OWNER: &str = "use compiler_fixtures.record_31.{R}; use compiler_fixtures.record_119.{H}; pub fn main() -> i32 {let r=R {value: 3};let h=H {x: r};return h.x.value;}";

#[path = "mir_verify/rejects_call_abi_and.rs"]
mod rejects_call_abi_and;
#[path = "mir_verify/rejects_invalid_cfg_and.rs"]
mod rejects_invalid_cfg_and;
