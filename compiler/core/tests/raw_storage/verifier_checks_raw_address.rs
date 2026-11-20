use super::*;

#[test]
fn verifier_checks_raw_address_drop_and_difference_contracts() {
    let files=Fixture::new(&source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);let p=slot_address(s,0);let q=slot_address_mut(s,0);let d=q-p;drop_in_place(q);let parts=into_raw_parts(s);let adopted=from_raw_parts(parts.0,parts.1);return 7+(d as i32);}}"));
    let p = files.checked().unwrap();
    let original = mir::lower(&p);
    mir::verify::verify(&original).unwrap();
    for target in [
        "storage_address",
        "raw_drop",
        "raw_difference",
        "storage_detach",
        "storage_adopt",
    ] {
        let mut bad = original.clone();
        let mut changed = false;
        for f in &mut bad.functions {
            for block in &f.blocks {
                let mir::Terminator::Evaluate {
                    operation,
                    destination,
                    ..
                } = &block.terminator
                else {
                    continue;
                };
                match operation {
                    mir::Operation::Unary("storage_address", _) if target == "storage_address" => {
                        let hir::Type::Pointer { mutable, .. } =
                            &mut f.locals[destination.local].ty
                        else {
                            panic!()
                        };
                        *mutable = true;
                        changed = true;
                    }
                    mir::Operation::Unary(
                        op @ ("raw_drop" | "storage_adopt"),
                        mir::Operand::Local(args),
                    ) if target == *op => {
                        let hir::Type::Tuple(fields) = &mut f.locals[*args].ty else {
                            panic!()
                        };
                        let hir::Type::Pointer { mutable, .. } = &mut fields[0] else {
                            panic!()
                        };
                        *mutable = false;
                        changed = true;
                    }
                    mir::Operation::Unary("storage_detach", _) if target == "storage_detach" => {
                        f.locals[destination.local].ty = hir::Type::Bool;
                        changed = true;
                    }
                    mir::Operation::Binary("raw_difference", _, _)
                        if target == "raw_difference" =>
                    {
                        f.locals[destination.local].ty = hir::Type::Bool;
                        changed = true;
                    }
                    _ => {}
                }
            }
        }
        assert!(changed, "missing {target}");
        assert!(
            mir::verify::verify(&bad).is_err(),
            "accepted corrupted {target}"
        );
        assert!(rust::emit_mir(&bad, OverflowChecks::Checked).is_err());
    }
}

#[test]
fn copy_storage_views_preserve_loans_and_cannot_escape() {
    for body in [
        " pub fn item(s:RawStorage<i32>) -> i32 {requires_unsafe_caller();if trusted_region() {return view(s,0);}} pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,2);let f=item;let ref var v=f(s);*v=7;return take(s,0);}}",
        "pub fn number() -> i32 {return 7;} pub fn main() -> i32 {let f=number;let ref v=f();return *v;}",

        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);let ref v=view(s,0);drop_slot(s,0);return *v;}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);let ref v=view(s,0);*v=2;return 0;}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);let ref var a=view_mut(s,0);let ref var b=view_mut(s,0);*b=2;return *a;}}",
        "pub fn escape() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);return view(s,0);}} pub fn main() -> i32 {return escape();}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);let ref v=view(s,0);let n=take(s,0);return *v;}}",
    ] {
        let files=Fixture::new(&source(body));
        assert!(files.checked().is_err(),"accepted {body}");
    }
}

#[test]
fn raw_owned_transfers_require_unsafe_owners_and_writable_access() {
    for body in [
        "use compiler_fixtures.record_91.{Item}; pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(1);let item=Item {v: 7};let alias=item;write_owned(slot_address_mut(s,0),alias);return 0;}}",
        "pub fn main() -> i32 {var n=7;let p=&n;if trusted_region() {write_owned(p,2)};return 0;}",

        "pub fn main() -> i32 {var n=7;let p=&var n;return read_owned(p);}",
        "pub fn main() -> i32 {let n=7;let p=&n;if trusted_region() {return read_owned(p);}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);return read_owned(slot_address(s,0));}}",
    ] { let files=Fixture::new(&source(body)); assert!(files.checked().is_err(),"accepted {body}"); }
}

#[test]
fn owning_raw_transfers_keep_one_cleanup_and_verified_types() {
    let files=Fixture::new(&source("use compiler_fixtures.record_81.{Item}; pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(1);let p=slot_address_mut(s,0);write_owned(p,Item {v: 7});let item=read_owned(p);release(s);return item.v;}}"));
    let p = files.checked().unwrap();
    let original = mir::lower(&p);
    for (result, events) in [
        interpret::execute_traced(&p, p.main.unwrap(), vec![], OverflowChecks::Checked, 100000),
        mir::interpret::execute_traced(
            &original,
            original.main.unwrap(),
            vec![],
            OverflowChecks::Checked,
            100000,
        ),
    ] {
        assert!(matches!(result,Ok(hir::Value::Integer(n)) if n.signed_value()==Some(7)));
        assert_eq!(events.iter().filter(|e|matches!(&e.value,hir::Value::Aggregate(fields) if matches!(fields.as_slice(),[hir::Value::Integer(n)] if n.signed_value()==Some(7)))).count(),1);
    }
    for (opcode, replacement) in [
        ("raw_take", "storage_initialize"),
        ("raw_initialize", "storage_take"),
    ] {
        let mut bad = original.clone();
        let mut changed = false;
        for f in &mut bad.functions {
            for block in &mut f.blocks {
                if let mir::Terminator::Evaluate {
                    operation: mir::Operation::Unary(op, _),
                    ..
                } = &mut block.terminator
                {
                    if *op == opcode {
                        *op = replacement;
                        changed = true;
                    }
                }
            }
        }
        assert!(changed);
        assert!(mir::verify::verify(&bad).is_err());
        assert!(rust::emit_mir(&bad, OverflowChecks::Checked).is_err());
    }
}
