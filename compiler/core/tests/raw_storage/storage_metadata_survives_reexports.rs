use super::*;

#[test]
fn storage_metadata_survives_reexports_and_verifier_rejects_corruption() {
    let files=Fixture::new("mod api;use crate.api.{allocate,initialize,take,release};\npub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);let n=take(s,0);release(s);return n;}}");
    fs::write(
        files.0.join("api.cl"),
        "pub use core.intrinsic.{allocate,initialize,take,release};",
    )
    .unwrap();
    let p = files.checked().unwrap();
    let original = mir::lower(&p);
    mir::verify::verify(&original).unwrap();
    let hir::Value::Integer(n) =
        interpret::execute(&p, p.main.unwrap(), vec![], OverflowChecks::Checked, 100000).unwrap()
    else {
        panic!()
    };
    assert_eq!(n.signed_value(), Some(7));
    let mut bad = original.clone();
    for f in &mut bad.functions {
        for b in &f.blocks {
            if let mir::Terminator::Evaluate {
                operation: mir::Operation::Unary("storage_allocate", _),
                destination,
                ..
            } = &b.terminator
            {
                let hir::Type::Pointer { mutable, .. } = &mut f.locals[destination.local].ty else {
                    panic!()
                };
                *mutable = false;
            }
        }
    }
    assert!(mir::verify::verify(&bad).is_err());
    assert!(rust::emit_mir(&bad, OverflowChecks::Checked).is_err());
    let mut bad = original;
    for f in &mut bad.functions {
        for b in &f.blocks {
            if let mir::Terminator::Evaluate {
                operation: mir::Operation::Unary("storage_take", _),
                destination,
                ..
            } = &b.terminator
            {
                f.locals[destination.local].ty = hir::Type::Bool;
            }
        }
    }
    assert!(mir::verify::verify(&bad).is_err());
    assert!(rust::emit_mir(&bad, OverflowChecks::Checked).is_err());
}

#[test]
fn owned_slots_run_destructors_once_in_transfer_order() {
    let files = Fixture::new(&source("use compiler_fixtures.record_2.{R}; pub fn main() -> i32 {if trusted_region() {var s=allocate<R>(3);initialize(s,0,R {v: 1});initialize(s,1,R {v: 2});move_within(s,1,0,2);drop_slot(s,1);let old=replace(s,2,R {v: 3});drop_slot(s,2);release(s);return old.v;}}"));
    let p = files.checked().unwrap();
    let m = mir::lower(&p);
    for mode in [OverflowChecks::Checked, OverflowChecks::Wrapping] {
        let a = interpret::execute_traced(&p, p.main.unwrap(), vec![], mode, 100000);
        let b = mir::interpret::execute_traced(&m, m.main.unwrap(), vec![], mode, 100000);
        assert!(a.0.is_ok());
        assert!(b.0.is_ok());
        for events in [a.1, b.1] {
            let values: Vec<_> = events
                .iter()
                .filter_map(|event| {
                    let hir::Value::Aggregate(fields) = &event.value else {
                        return None;
                    };
                    let [hir::Value::Integer(n)] = fields.as_slice() else {
                        return None;
                    };
                    n.signed_value()
                })
                .collect();
            assert_eq!(values, vec![1, 3, 2]);
        }
    }
}

#[test]
fn buffer_cleanup_continues_after_first_panic_and_stops_after_second() {
    for (condition, expected_kind, expected_values) in [
        (
            "v==1",
            interpret::TrapKind::Arithmetic(clust_semantics::ArithmeticError::DivisionByZero),
            vec![1, 2, 3],
        ),
        ("v<3", interpret::TrapKind::DoublePanic, vec![1, 2]),
    ] {
        let code="use core.storage.{OwnedBuffer}; fn main()->i32 {var buffer=OwnedBuffer<Item>.make(3);buffer.push(Item {v:1});buffer.push(Item {v:2});buffer.push(Item {v:3});return 0;}";
        let files = Fixture::new(code);
        let mut loaded = clust_frontend::modules::load(&files.0.join("main.cl"), "app");
        assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
        let body = format!("if {condition} {{let bad=1/0;}}");
        let record = clust_frontend::testing::record::inline(
            "Item",
            &[("v", "i32")],
            &[],
            &[&body],
            clust_frontend::source::Span {
                source: clust_frontend::source::SourceId(0),
                start: 0,
                end: 0,
            },
        );
        let module = loaded.module.as_mut().unwrap();
        module.items.push(record);
        let p = check(module).unwrap();
        let m = mir::lower(&p);
        for mode in [OverflowChecks::Checked, OverflowChecks::Wrapping] {
            for (result, events) in [
                interpret::execute_traced(&p, p.main.unwrap(), vec![], mode, 100000),
                mir::interpret::execute_traced(&m, m.main.unwrap(), vec![], mode, 100000),
            ] {
                assert_eq!(result.unwrap_err().kind, expected_kind);
                let values: Vec<_> = events
                    .iter()
                    .filter_map(|event| {
                        let hir::Value::Aggregate(fields) = &event.value else {
                            return None;
                        };
                        let [hir::Value::Integer(n)] = fields.as_slice() else {
                            return None;
                        };
                        n.signed_value()
                    })
                    .collect();
                assert_eq!(values, expected_values);
            }
        }
    }
}

#[test]
fn safe_buffer_preserves_privacy_and_requires_owners() {
    for body in [
        "use compiler_fixtures.record_91.{Item}; pub fn main() -> i32 {var b=OwnedBuffer<Item>.make(1);let owner=Item {v: 7};let alias=owner;b.push(alias);return 0;}",
        "pub fn main() -> i32 {var b=OwnedBuffer<i32>.make(1);let hidden=b.storage;return 0;}",
        "pub fn main() -> i32 {let b=OwnedBuffer<i32>.make(1);b.push(7);return 0;}",
    ] {
        let files=Fixture::new(&format!("use core.storage.{{OwnedBuffer}};\n{body}"));
        assert!(files.checked().is_err(),"accepted {body}");
    }
}

#[test]
fn storage_views_keep_capability_and_owner_lifetime() {
    let prefix = "pub class Item {pub value:i32;new(n:i32) {value=n;}} ";
    for body in [
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(1);initialize(s,0,Item.new(2));let v=view(s,0);drop_slot(s,0);return v.value;}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(1);initialize(s,0,Item.new(2));let v=view(s,0);release(s);return v.value;}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(1);initialize(s,0,Item.new(2));let v=view(s,0);v.value=7;return 0;}}",
        "pub fn escape() -> Item {if trusted_region() {var s=allocate<Item>(1);initialize(s,0,Item.new(2));return view(s,0);}} pub fn main() -> i32 {return escape().value;}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(1);initialize(s,0,Item.new(2));var a=view_mut(s,0);var b=view_mut(s,0);b.value=3;return a.value;}}",
    ] {
        let files=Fixture::new(&source(&format!("{prefix}{body}")));
        assert!(files.checked().is_err(),"accepted {body}");
    }
    let files=Fixture::new(&source(&format!("{prefix} pub fn main() -> i32 {{if trusted_region() {{var s=allocate<Item>(1);initialize(s,0,Item.new(7));return view(s,0).value;}}}}")));
    let p = files.checked().unwrap();
    let mut m = mir::lower(&p);
    mir::verify::verify(&m).unwrap();
    for f in &mut m.functions {
        for b in &mut f.blocks {
            if let mir::Terminator::Evaluate {
                operation: mir::Operation::StorageView { contract, .. },
                ..
            } = &mut b.terminator
            {
                contract.origins.clear();
            }
        }
    }
    assert!(mir::verify::verify(&m).is_err());
    assert!(rust::emit_mir(&m, OverflowChecks::Checked).is_err());
}
