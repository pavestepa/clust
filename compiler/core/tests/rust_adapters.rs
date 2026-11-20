use clust_core::{check::check_clust, hir, interpret, mir};
use clust_frontend::{parser::parse_clust, source::SourceId};
use clust_semantics::OverflowChecks;

fn check(source: &str) -> Result<hir::Program, Vec<clust_frontend::source::Diagnostic>> {
    let parsed = parse_clust(SourceId(0), source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    check_clust(&parsed.module.unwrap())
}
#[test]
fn foreign_calls_are_never_silently_evaluated_as_their_analysis_body() {
    let program = check(include_str!("../../../examples/clust_rust/main.cl")).unwrap();
    let result = interpret::execute(
        &program,
        program.main.unwrap(),
        vec![],
        OverflowChecks::Checked,
        1000,
    );
    assert_eq!(
        result.unwrap_err().kind,
        interpret::TrapKind::ForeignFunction
    );
    let lowered = mir::lower(&program);
    let (result, _) = mir::interpret::execute_traced(
        &lowered,
        lowered.main.unwrap(),
        vec![],
        OverflowChecks::Checked,
        1000,
    );
    assert_eq!(
        result.unwrap_err().kind,
        interpret::TrapKind::ForeignFunction
    );
    let rust = clust_core::rust::emit(&program, OverflowChecks::Wrapping).unwrap();
    assert!(rust.contains("std::cmp::max(a0,a1)"));
}

#[test]
fn adapters_reject_ownership_crossings_and_code_injection() {
    for source in [
        "#[rust(\"support.f; panic!()\")] extern \"Rust\" fn f()->i32;fn main()->i32 {return f();}",
        "#[rust(\"support.f\")] extern \"Rust\" fn f(var value:i32);fn main() {}",
        "class C {new() {}} #[rust(\"support.f\")] extern \"Rust\" fn f(value:C);fn main() {}",
        "#[rust(\"support.f\")] extern \"Rust\" fn f(value:*i32);fn main() {}",
    ] {
        assert_eq!(check(source).unwrap_err()[0].code, "E0505", "{source}");
    }
    let program = check(include_str!("../../../examples/clust_rust/main.cl")).unwrap();
    let mut lowered = mir::lower(&program);
    let adapter = lowered
        .functions
        .iter_mut()
        .find_map(|function| function.rust_adapter.as_mut())
        .unwrap();
    adapter.path = "std::cmp::max);panic!();//".into();
    assert!(mir::verify::verify(&lowered).is_err());
    assert!(clust_core::rust::emit_mir(&lowered, OverflowChecks::Checked).is_err());
}
