use super::*;

#[test]
fn scalar_storage_reference_lifecycle_layout_and_unwind() {
    for (source, expected) in cases() {
        let files = Fixture::new(&source);
        let p = files
            .checked()
            .unwrap_or_else(|e| panic!("{source}: {e:?}"));
        let m = mir::lower(&p);
        for mode in [OverflowChecks::Checked, OverflowChecks::Wrapping] {
            let a = interpret::execute(&p, p.main.unwrap(), vec![], mode, 100000);
            let b = mir::interpret::execute(&m, m.main.unwrap(), vec![], mode, 100000);
            assert_eq!(
                a.as_ref().map_err(|e| &e.kind),
                b.as_ref().map_err(|e| &e.kind),
                "{source}"
            );
            match expected {
                Some(n) => assert!(
                    matches!(a,Ok(hir::Value::Integer(i)) if i.signed_value()==Some(n as i128)),
                    "{source}: {a:?}"
                ),
                None => assert!(a.is_err(), "{source}"),
            }
            rust::emit(&p, mode).unwrap();
        }
    }
}

#[test]
fn storage_symbols_ownership_privacy_and_safety_are_enforced() {
    for body in [
        "pub fn f(s:RawStorage<i32>) -> (*var i32,usize) {return into_raw_parts(s);} pub fn main() -> i32 {return 0;}",
        "pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(0);let alias=s;let parts=into_raw_parts(alias);};return 0;}",
        "pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(0);let parts=into_raw_parts(s);return capacity(s) as i32;}}",
        "pub class R {pub v:i32;new() {v=7;}} pub fn main() -> i32 {if trusted_region() {var s=allocate<R>(1);initialize(s,0,R.new());let v=view(s,0);let parts=into_raw_parts(s);return v.v;}}",
        "pub fn main() -> i32 {var n=7;drop_in_place(&var n);return 0;}",
        "pub fn main() -> i32 {var n=7;if trusted_region() {drop_in_place(&n)};return 0;}",
        "pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(1);let p=slot_address_mut(s,0);};return 0;}",
        "pub fn f(s:RawStorage<i32>) -> *i32 {return slot_address(s,0);} pub fn main() -> i32 {return 0;}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);let p=slot_address(s,0);*p=7};return 0;}",
        "use compiler_fixtures.record_88.{R}; use compiler_fixtures.record_89.{S}; pub fn main() -> i32 {let target=R {v: 7};let alias=target;let item=S {r: alias};if trusted_region() {var storage=allocate<S>(1);initialize(storage,0,item)};return 0;}",
        "use compiler_fixtures.record_3.{R}; pub fn main() -> i32 {if trusted_region() {var s=allocate<R>(1);let r=R {v: 7};let alias=r;initialize(s,0,alias)};return 0;}",
        "use compiler_fixtures.record_3.{R}; pub fn main() -> i32 {if trusted_region() {var s=allocate<R>(1);let r=R {v: 7};initialize(s,0,r);return r.v;}}",
        "pub fn main() -> i32 {let s=allocate<i32>(1);return 0;}",
        "pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(1);let alias=s;release(alias)};return 0;}",
        "pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(1);release(s);return capacity(s) as i32;}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);let alias=s;initialize(alias,0,7)};return 0;}",
        "pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(1);let p=s.data;};return 0;}",
        "pub fn main() -> i32 {let n=7;let fake=RawStorage<i32> {data:&var n,cap:1};return 0;}",




    ] {let files=Fixture::new(&source(body));assert!(files.checked().is_err(),"accepted: {body}");}
    let files =
        Fixture::new("use core.intrinsic.{heap_allocate};\npub fn main() -> i32 {return 0;}");
    assert!(files
        .checked()
        .unwrap_err()
        .iter()
        .any(|e| e.code == "E0504"));
    let files = Fixture::new(
        "pub fn heap_allocate<T>(n:usize) -> i32 {return 7;} pub fn main() -> i32 {return heap_allocate<i32>(1);}",
    );
    assert!(files.checked().is_ok());
}

#[test]
fn interpreter_rejects_invalid_unsafe_slot_protocols() {
    // Deliberately invalid unsafe programs are never executed natively.
    for body in [
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);let p=slot_address_mut(s,0);write_owned(p,7);write_owned(p,2);return 0;}}",

        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);let p=slot_address_mut(s,0);let a=read_owned(p);return read_owned(p);}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);return read_owned(slot_address_mut(s,1));}}",

        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);let p=slot_address_mut(s,0);let n=take(s,0);*p=2;return n;}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);let address=slot_address(s,0) as usize;drop_slot(s,0);release(s);let p=address as *i32;return *p;}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);let ref v=view(s,0);let p=&*v;drop_slot(s,0);release(s);return *p;}}",

        "pub fn main() -> i32 {if trusted_region() {let s=allocate<u32>(1);let parts=into_raw_parts(s);let p=parts.0 as *var f32;let bad=from_raw_parts(p,parts.1);return 0;}}",
        "pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(0);let parts=into_raw_parts(s);let a=from_raw_parts(parts.0,parts.1);let b=from_raw_parts(parts.0,parts.1);return 0;}}",
        "pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(1);let parts=into_raw_parts(s);let a=from_raw_parts(parts.0,0);return 0;}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);drop_in_place(slot_address_mut(s,0))};return 0;}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);let p=slot_address_mut(s,0);drop_in_place(p);drop_in_place(p)};return 0;}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);let p=slot_address_mut(s,1);drop_in_place(p)};return 0;}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);let p=slot_address(s,0);release(s);return *p;}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);var t=allocate<i32>(1);return (slot_address(s,0)-slot_address(t,0)) as i32;}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);return take(s,0);}}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);initialize(s,0,9)};return 0;}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,1,7)};return 0;}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);drop_slot(s,0);drop_slot(s,0)};return 0;}",
        "pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);release(s)};return 0;}",
    ] {
        let files=Fixture::new(&source(body));let p=files.checked().unwrap();let m=mir::lower(&p);
        let a=interpret::execute(&p,p.main.unwrap(),vec![],OverflowChecks::Checked,100000).unwrap_err();
        let b=mir::interpret::execute(&m,m.main.unwrap(),vec![],OverflowChecks::Checked,100000).unwrap_err();
        assert_eq!(a.kind,b.kind,"{body}");
    }
}
