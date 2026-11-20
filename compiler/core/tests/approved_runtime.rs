//! End-to-end checks of the approved syntax and bundled library in both engines.
use clust_core::{check::check_clust, hir, interpret, mir};
use clust_frontend::modules::load_clust;
use clust_semantics::OverflowChecks;
use std::path::PathBuf;

#[test]
fn approved_examples_agree_in_reference_engines() {
    // The recursive HIR oracle needs more host stack than Rust's 2 MiB test
    // threads for nested String/Vec/library calls. The native backend is tested
    // separately and does not execute this oracle.
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(run_examples)
        .unwrap()
        .join()
        .unwrap();
}
fn run_examples() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    for (example, expected) in [
        ("clust_counter", 5),
        ("clust_results", 7),
        ("clust_defaults", 7),
        ("clust_vec", 9),
        ("clust_animals", 5),
        ("clust_string", 0),
        ("clust_dynamic", 14),
        ("clust_weak", 7),
        ("clust_closures", 7),
        ("clust_captures", 7),
        ("clust_iteration", 16),
        ("clust_weak_identity", 14),
        ("clust_weak_backedge", 7),
        ("clust_generic_inheritance", 7),
        ("clust_generic_dynamic", 10),
    ] {
        let loaded = load_clust(&root.join(example).join("main.cl"), "app");
        assert!(
            loaded.diagnostics.is_empty(),
            "{example}: {:?}",
            loaded.diagnostics
        );
        let program = check_clust(&loaded.module.unwrap())
            .unwrap_or_else(|errors| panic!("{example}: {errors:?}"));
        let lowered = mir::lower(&program);
        mir::verify::verify(&lowered).unwrap_or_else(|error| panic!("{example}: {error:?}"));
        let a = interpret::execute_traced(
            &program,
            program.main.unwrap(),
            vec![],
            OverflowChecks::Checked,
            1_000_000,
        );
        let b = mir::interpret::execute_traced(
            &lowered,
            lowered.main.unwrap(),
            vec![],
            OverflowChecks::Checked,
            1_000_000,
        );
        assert_eq!(a.0, b.0, "{example}");
        // Allocation identities are private to each engine. Debug traces retain
        // payloads, slot offsets and lengths while omitting host addresses.
        assert_eq!(format!("{:?}", a.1), format!("{:?}", b.1), "{example}");
        assert!(
            matches!(a.0,Ok(hir::Value::Integer(value)) if value.signed_value()==Some(expected)),
            "{example}: {a:?}"
        );
    }
}
