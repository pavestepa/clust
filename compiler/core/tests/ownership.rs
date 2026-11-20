use clust_core::{check::check, hir::Value, interpret, mir};
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
fn inline_noncopy_moves_views_and_reinitialization() {
    for (source, expected) in [
        ("use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn main() -> i32 {var x=Cell {value: 1};let y=Hold {cell: x};if true {x=Cell {value: 2}} else {x=Cell {value: 3}};return y.cell.value+x.value;}",3),
        ("use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn main() -> i32 {var x=(Cell {value: 1},Cell {value: 2});let y=Hold {cell: x.0};let z=Hold {cell: x.1};x.0=Cell {value: 3};return y.cell.value+z.cell.value+x.0.value;}",6),

        ("use compiler_fixtures.record_5.{Cell}; pub fn identity(x:Cell) -> Cell {return x;} pub fn main() -> i32 {let x=Cell {value: 7};let y=identity(x);return y.value;}",7),
        ("use compiler_fixtures.record_5.{Cell}; pub fn main() -> i32 {let x=Cell {value: 7};let y=x;return y.value+x.value;}",14),
        ("use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; use compiler_fixtures.record_15.{Pair}; pub fn main() -> i32 {var x=Pair {a: Cell {value: 1}, b: Cell {value: 2}};let y=Hold {cell: x.a};x.a=Cell {value: 4};let z=x;return y.cell.value+z.a.value+z.b.value;}",7),
        ("pub enum E {A(i32),B} pub fn main() -> i32 {let e=E.A(7);return match e {E.A(x) if x>2=>x,E.A(_)=>1,E.B=>0};}",7),
        ("use compiler_fixtures.record_5.{Cell}; pub fn identity<T>(x:T) -> T {return x;} pub fn main() -> i32 {let x=Cell {value: 5};let y=identity(x);return y.value;}",5),
        ("use compiler_fixtures.record_5.{Cell}; pub fn take(x:Cell) -> i32 {return x.value;} pub fn main() -> i32 {var x=Cell {value: 1};var n=0;while n<3 {n+=take(x);x=Cell {value: 1};};return n;}",3),
    ] {
        let hir=checked(source).unwrap_or_else(|e|panic!("{source}: {e:?}"));
        let cfg=mir::lower(&hir);
        // Ownership moves only in owning contexts such as aggregate construction.
        if source.contains("Hold {") {
            assert!(cfg.functions.iter().flat_map(|f| &f.blocks).any(|b| matches!(&b.terminator, mir::Terminator::Evaluate { operation: mir::Operation::Move(_), .. })), "missing source move: {source}");
        }
        let a=interpret::execute(&hir,hir.main.unwrap(),vec![],OverflowChecks::Checked,10000).unwrap();
        let b=mir::interpret::execute(&cfg,cfg.main.unwrap(),vec![],OverflowChecks::Checked,10000).unwrap();
        assert_eq!(a,b,"{source}");
        let Value::Integer(a)=a else {panic!()};
        assert_eq!(a.signed_value(),Some(expected),"{source}");
        clust_core::rust::emit(&hir,OverflowChecks::Checked).unwrap();
    }
}
#[test]
fn rejects_use_after_move_readonly_promotion_and_invalid_copy_claims() {
    for source in [
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {let x=Cell {value: 1};if true {take(x)};return x.value;}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_15.{Pair}; pub fn main() -> i32 {var x=Pair {a: Cell {value: 1}, b: Cell {value: 2}};let y=x.a;if true {x.a=Cell {value: 4}};let z=x;return y.value+z.a.value;}",

        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {let x=Cell {value: 1};let a=take(x);return a+x.value;}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {let x=Cell {value: 1};return take(x)+take(x);}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {let x=Cell {value: 1};var n=0;while n<2 {n+=take(x);};return n;}",
        "use compiler_fixtures.record_5.{Cell}; pub fn main() -> i32 {let x=Cell {value: 1};let ref y=x;var z=y;z.value=2;return z.value;}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn bad(var x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value;} pub fn main() -> i32 {var x=Cell {value: 1};return bad(x);}",
        "use compiler_fixtures.record_5.{Cell}; pub fn main() -> i32 {let x=Cell {value: 7};var y=x;y.value=8;return y.value;}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_80.{Invalid}; pub fn main() -> () {}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {let x=Cell {value: 1};let ref y=x;let z=take(x);return y.value+z;}",
    ] { assert!(checked(source).is_err(),"accepted {source}"); }
}
