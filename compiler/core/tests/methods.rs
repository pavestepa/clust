use clust_core::{check::check, hir::Value, interpret};
use clust_frontend::{source::SourceId, testing::parse};
use clust_semantics::OverflowChecks;
#[test]
fn static_methods_use_the_same_generic_call_contracts_as_free_functions() {
    let source="use compiler_fixtures.record_16.{Box}; pub fn main() -> i32 {let boxed=Box<i32>.create(17);return Box.extract(boxed);}";
    let parsed = parse(SourceId(0), source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let p = check(&parsed.module.unwrap()).unwrap();
    let Value::Integer(result) =
        interpret::execute(&p, p.main.unwrap(), vec![], OverflowChecks::Checked, 10000).unwrap()
    else {
        panic!()
    };
    assert_eq!(result.signed_value(), Some(17));
}

#[test]
fn instance_methods_borrow_receivers_including_copy_types() {
    for (source,expected) in [
        ("use compiler_fixtures.record_10.{Cell}; pub fn main() -> i32 {var c=Cell {value: 1};c.twice();return c.read();}",3),
        ("use compiler_fixtures.record_11.{Cell}; pub fn main() -> i32 {var c=Cell {value: 1};c.bump();return c.read()+c.value;}",4),
        ("use compiler_fixtures.record_12.{Box}; pub fn main() -> i32 {let b=Box<i32> {value: 3};return b.read();}",7),
        ("use compiler_fixtures.record_13.{Box}; pub fn main() -> i32 {return (Box<i32> {value: 9}).get();}",9),
        ("use compiler_fixtures.record_14.{Cell}; pub fn main() -> i32 {let c=Cell {value: 3};return c.read();}",7),
    ] {
        let parsed=clust_frontend::testing::parse(clust_frontend::source::SourceId(0),source);
        assert!(parsed.diagnostics.is_empty(),"{:?}",parsed.diagnostics);
        let p=clust_core::check::check(&parsed.module.unwrap()).unwrap_or_else(|e|panic!("{source}: {e:?}"));
        let cfg=clust_core::mir::lower(&p);
        let a=clust_core::interpret::execute(&p,p.main.unwrap(),vec![],clust_semantics::OverflowChecks::Checked,10000).unwrap();
        let b=clust_core::mir::interpret::execute(&cfg,cfg.main.unwrap(),vec![],clust_semantics::OverflowChecks::Checked,10000).unwrap();
        assert_eq!(a,b,"{source}");
        let clust_core::hir::Value::Integer(a)=a else {panic!()};assert_eq!(a.signed_value(),Some(expected),"{source}");
        clust_core::rust::emit(&p,clust_semantics::OverflowChecks::Checked).unwrap();
    }
    for source in [
        "use compiler_fixtures.record_33.{Cell}; pub fn main() -> () {}",
        "use compiler_fixtures.record_34.{Cell}; pub fn main() -> () {let c=Cell {value: 1};c.bump();}",

    ] {
        let parsed=clust_frontend::testing::parse(clust_frontend::source::SourceId(0),source);
        assert!(parsed.diagnostics.is_empty());
        assert!(clust_core::check::check(&parsed.module.unwrap()).is_err(),"accepted {source}");
    }
}
