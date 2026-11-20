use super::*;
pub(super) fn execute_cleanup(
    source: &str,
) -> (
    std::result::Result<Value, crate::interpret::Trap>,
    Vec<i128>,
) {
    let program = analyzed(source).unwrap_or_else(|errors| panic!("{source}: {errors:?}"));
    let mir = crate::mir::lower(&program);
    let (result, events) = crate::mir::interpret::execute_traced(
        &mir,
        mir.main.unwrap(),
        vec![],
        clust_semantics::OverflowChecks::Checked,
        100_000,
    );
    let (hir_result, hir_events) = crate::interpret::execute_traced(
        &program,
        program.main.unwrap(),
        vec![],
        clust_semantics::OverflowChecks::Checked,
        100_000,
    );
    assert_eq!(hir_result, result, "HIR/MIR result: {source}");
    assert_eq!(hir_events, events, "HIR/MIR cleanup: {source}");
    let events = events
        .into_iter()
        .map(|event| {
            let Value::Aggregate(fields) = event.value else {
                panic!()
            };
            let Value::Integer(value) = &fields[0] else {
                panic!()
            };
            value.signed_value().unwrap()
        })
        .collect();
    (result, events)
}
#[test]
fn mir_cleanup_executes_source_bodies_in_ownership_order() {
    let resource = "use compiler_fixtures.record_141.{R}; ";
    for (body,expected) in [
            ("pub fn main() -> i32 {let a=R {value: 1};let b=R {value: 2};return 0;}",vec![2,1]),
            ("pub fn main() -> i32 {var a=R {value: 1};let b=R {value: 2};a=R {value: 3};return 0;}",vec![1,3,2]),
            ("pub fn identity(x:R) -> R {return x;} pub fn main() -> i32 {let a=R {value: 1};let b=identity(a);return b.value;}",vec![1]),
            ("pub fn make() -> R {return R {value: 1};} pub fn main() -> i32 {let a=make();return a.value;}",vec![1]),
            ("pub fn main() -> i32 {let xs={R {value: 1},R {value: 2}};return 0;}",vec![1,2]),
            ("use compiler_fixtures.record_142.{Pair}; pub fn main() -> i32 {let pair=Pair {a: R {value: 1}, b: R {value: 2}};return 0;}",vec![1,2]),
            ("pub fn main() -> i32 {var a=R {value: 1};if true {let b=R {value: 2};a=R {value: 3}};return 0;}",vec![1,2,3]),
            ("pub fn main() -> i32 {var a=R {value: 1};let b=R {value: 2};a.replace();return a.value;}",vec![1,3,2]),
            ("pub fn main() -> i32 {R {value: 9};let x=R {value: 2};return 0;}",vec![9,2]),
            ("pub fn main() -> i32 {var n=0;while n<3 {let x=R {value: n};n+=1;if n==2 {continue;};if n==3 {break;}};return n;}",vec![0,1,2]),
            ("use compiler_fixtures.record_142.{Pair}; pub fn main() -> i32 {let p=Pair {a: R {value: 1}, b: R {value: 2}};let x=p.a;return x.value;}",vec![1,2]),
            ("pub enum E {A(R,R),B} pub fn main() -> i32 {let x=E.A(R {value: 1},R {value: 2});return 0;}",vec![1,2]),
            ("pub fn main() -> i32 {return R {value: 7}.value;}",vec![7]),
        ] {
            let source=format!("{resource}{body}");
            let (result,events)=execute_cleanup(&source);
            assert!(result.is_ok(),"{source}: {result:?}");
            assert_eq!(events,expected,"{source}");
        }
}
#[test]
fn mir_failed_aggregate_construction_cleans_completed_fields_in_order() {
    let types = "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_143.{Parent}; ";
    for (expression, expected) in [
        (
            "Parent {a:R {value: 1},b:R {value: 2},c:1/0}",
            vec![2, 1, 9],
        ),
        (
            "Parent {b:R {value: 2},a:R {value: 1},c:1/0}",
            vec![1, 2, 9],
        ),
        ("(R {value: 1},R {value: 2},1/0)", vec![2, 1, 9]),
    ] {
        let (result, events) = execute_cleanup(&format!(
                "{types} pub fn main() -> i32 {{let outer=R {{value: 9}};let x={expression};return 0;}}"
            ));
        assert!(matches!(
            result.unwrap_err().kind,
            crate::interpret::TrapKind::Arithmetic(_)
        ));
        assert_eq!(events, expected);
    }
    let (result, events) = execute_cleanup("use compiler_fixtures.record_4.{R}; pub fn fail() -> R {let x=1/0;return R {value: 3};} pub fn main() -> i32 {let x={R {value: 1},R {value: 2},fail()};return 0;}");
    assert!(result.is_err());
    assert_eq!(events, vec![2, 1]);
}
#[test]
fn resource_call_arguments_clean_in_reverse_on_failure_before_entry() {
    let source="use compiler_fixtures.record_4.{R}; pub fn take(a:R,b:R,n:i32) -> i32 {let x=a;let y=b;return n;} pub fn main() -> i32 {return take(R {value: 1},R {value: 2},1/0);}";
    let (result, events) = execute_cleanup(source);
    assert!(result.is_err());
    assert_eq!(events, vec![2, 1]);
}
#[test]
fn mir_unwind_cleans_return_staging_and_stops_on_double_panic() {
    let types = "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_144.{Fail}; ";
    let (result,events)=execute_cleanup(&format!("{types} pub fn make() -> R {{let fail=Fail {{value: 0}};return R {{value: 7}};}} pub fn main() -> i32 {{let result=make();return result.value;}}"));
    assert!(result.is_err());
    assert_eq!(events, vec![0, 7]);
    let (result,events)=execute_cleanup(&format!("{types} pub fn main() -> i32 {{let live=R {{value: 3}};let a=Fail {{value: 0}};let b=Fail {{value: 0}};return 0;}}"));
    assert_eq!(
        result.unwrap_err().kind,
        crate::interpret::TrapKind::DoublePanic
    );
    assert_eq!(events, vec![0, 0]);
}
#[test]
fn generic_destructors_are_specialized_and_audited() {
    let source="use compiler_fixtures.record_145.{R}; pub fn main() -> i32 {let a=R<i32> {value: 1};let b=R<i64> {value: 2};return 0;}";
    let (result, events) = execute_cleanup(source);
    assert!(result.is_ok());
    assert_eq!(events, vec![2, 1]);
    let invalid="use compiler_fixtures.record_146.{R}; pub fn main() -> i32 {let a=R<i32> {value: 1};return 0;}";
    assert!(analyzed(invalid)
        .unwrap_err()
        .iter()
        .any(|e| e.code == "E0373"));
}
