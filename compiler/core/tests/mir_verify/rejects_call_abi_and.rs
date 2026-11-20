use super::*;

#[test]
fn rejects_call_abi_and_storage_effect_tampering() {
    let p = program("use compiler_fixtures.record_31.{R}; pub fn id(x:R) -> R {return x;} pub fn main() -> i32 {let r=R {value: 3};let a=id(r);return a.value;}");
    let mut bad = p;
    let f = &mut bad.functions[bad.main.unwrap()];
    for b in &mut f.blocks {
        if let Terminator::Evaluate {
            operation: Operation::CallView { arguments, .. },
            ..
        } = &mut b.terminator
        {
            let Operand::Address(place) = &arguments[0] else {
                panic!()
            };
            arguments[0] = Operand::Local(place.local);
        }
    }
    rejected(&bad, "ABI");

    let mut bad = program("use compiler_fixtures.record_31.{R}; use compiler_fixtures.record_119.{H}; pub fn set(var h:H,r:R) -> () {let a=r;h.x=a;} pub fn main() -> i32 {let r=R {value: 3};let a=r;var h=H {x: a};set(h,r);return h.x.value;}");
    for f in &mut bad.functions {
        f.storage_writes = std::sync::Arc::new(vec![vec![]; f.storage_writes.len()]);
    }
    rejected(&bad, "effects");
}

#[test]
fn rejects_erased_indirect_result_contract() {
    let mut p = program("use compiler_fixtures.record_31.{R}; pub fn id(x:R) -> R {return x;} pub fn main() -> i32 {let r=R {value: 3};let f=id;let a=f(r);return a.value;}");
    let mut changed = false;
    for f in &mut p.functions {
        for b in &mut f.blocks {
            if let Terminator::Evaluate {
                operation: Operation::CallIndirect(_, _, contract),
                ..
            } = &mut b.terminator
            {
                contract.view = None;
                changed = true;
            }
        }
    }
    assert!(changed);
    rejected(&p, "provenance");
}

#[test]
fn rejects_forged_raw_capabilities_and_pointee_types() {
    let p = program(
        "pub fn main() -> i32 {var n=1;let p=&var n;if trusted_region() {*p=7;return *p;}}",
    );
    let mut bad = p.clone();
    let f = &mut bad.functions[bad.main.unwrap()];
    let n = f.locals.iter().position(|l| l.name == "n").unwrap();
    f.locals[n].mutable = false;
    rejected(&bad, "raw pointer");
    let mut bad = p.clone();
    let f = &mut bad.functions[bad.main.unwrap()];
    for b in &f.blocks {
        if let Terminator::Evaluate {
            operation: Operation::Binary("raw_write", Operand::Local(id), _),
            ..
        } = &b.terminator
        {
            let clust_core::hir::Type::Pointer { mutable, .. } = &mut f.locals[*id].ty else {
                panic!()
            };
            *mutable = false;
        }
    }
    rejected(&bad, "raw pointer");
    let mut bad = p;
    let f = &mut bad.functions[bad.main.unwrap()];
    for b in &f.blocks {
        if let Terminator::Evaluate {
            operation: Operation::Unary("raw_read", _),
            destination,
            ..
        } = &b.terminator
        {
            f.locals[destination.local].ty = clust_core::hir::Type::Bool;
        }
    }
    rejected(&bad, "raw pointer");
}
