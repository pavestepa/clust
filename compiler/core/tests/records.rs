use clust_core::{check::check, hir::Value, interpret::execute};
use clust_frontend::{source::SourceId, testing::parse};
use clust_semantics::OverflowChecks;
fn checked(
    source: &str,
) -> Result<clust_core::hir::Program, Vec<clust_frontend::source::Diagnostic>> {
    let parsed = parse(SourceId(0), source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    check(&parsed.module.unwrap())
}
#[test]
fn copy_structs_have_nominal_types_nested_places_and_ordered_initialization() {
    let program=checked("use compiler_fixtures.record_17.{Pair}; use compiler_fixtures.record_18.{Boxed}; pub fn swap(p:Pair) -> Pair {return Pair {right:p.left,left:p.right};} pub fn main() -> i32 {var n=0;var box=Boxed {pair:Pair {right:if true {n+=1;n} else {0},left:if true {n+=1;n} else {0}}};let copy=box;box.pair.left+=10;return swap(copy.pair).left+box.pair.left;}").unwrap();
    let Value::Integer(result) = execute(
        &program,
        program.main.unwrap(),
        vec![],
        OverflowChecks::Checked,
        10000,
    )
    .unwrap() else {
        panic!()
    };
    assert_eq!(result.signed_value(), Some(13));
}
#[test]
fn rejects_recursive_incomplete_duplicate_and_structurally_similar_nominals() {
    for (source,code) in [
        ("use compiler_fixtures.record_19.{A};","E0350"),
        ("use compiler_fixtures.record_20.{A}; use compiler_fixtures.record_21.{B};","E0350"),
        ("use compiler_fixtures.record_22.{A};","E0301"),
        ("use compiler_fixtures.record_23.{A}; pub fn main() -> () {let a=A {};}","E0351"),
        ("use compiler_fixtures.record_23.{A}; pub fn main() -> () {let a=A {x:1,x:2};}","E0351"),
        ("use compiler_fixtures.record_23.{A}; pub fn main() -> () {let a=A {y:1};}","E0351"),
        ("use compiler_fixtures.record_23.{A}; use compiler_fixtures.record_24.{B}; pub fn f(a:A) -> () {} pub fn main() -> () {f(B {x: 1});}","E0320"),
        ("use compiler_fixtures.record_23.{A}; pub fn main() -> () {let a=A {x: 1};a.x=2;}","E0317"),
    ] { let errors=checked(source).unwrap_err(); assert_eq!(errors[0].code,code,"{source}: {errors:?}"); }
}
