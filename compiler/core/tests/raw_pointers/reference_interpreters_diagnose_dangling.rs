use super::*;

#[test]
fn reference_interpreters_diagnose_dangling_and_out_of_bounds_raw_access() {
    // Invalid unsafe programs: never execute these as native Rust (undefined behavior).
    for source in [
        "#[Copy] pub enum E {A,B} pub fn main() -> i32 {let n:u32=2;if trusted_region() {let p=(&n) as *E;let e=*p;return match e {E.A=>0,E.B=>7};}}",

        "pub fn main() -> i32 {let n:u8=2;if trusted_region() {let p=(&n) as *bool;if *p {return 7;};return 0;}}",
        "pub fn main() -> i32 {var n=true;if trusted_region() {let p=(&var n) as *var u8;*p=2;if n {return 7;};return 0;}}",
        "pub fn main() -> i32 {let n:u32=55296;if trusted_region() {let p=(&n) as *char;if *p=='A' {return 7;};return 0;}}",
        "pub fn main() -> i32 {let n:u32=1114112;if trusted_region() {let p=(&n) as *char;if *p=='A' {return 7;};return 0;}}",

        "pub fn main() -> i32 {let a=();if trusted_region() {let p=&a+1;};return 0;}",
        "pub fn dangling() -> usize {let n=7;if trusted_region() {return (&n) as usize;}} pub fn main() -> i32 {if trusted_region() {let p=dangling() as *i32;return *p;}}",
        "pub fn main() -> i32 {if trusted_region() {let a:usize=0;let p=a as *i32;return *p;}}",
        "pub fn main() -> i32 {let n=7;if trusted_region() {let a=(&n) as usize;let p=a as *var i32;*p=2;return n;}}",
        "pub fn main() -> i32 {let n=7;if trusted_region() {let a=(&n) as usize;let p=(a+1) as *i32;return *p;}}",
        "pub fn main() -> i32 {let n=7;if trusted_region() {let a=(&n) as usize;let p=(a+4) as *i32;return *p;}}",
        "pub fn main() -> i32 {let n=7;if trusted_region() {let a=(&n) as usize;let p=(a+64) as *i32;return *p;}}",

        "pub fn main() -> i32 {let n:(u32,u32)=(1,2);if trusted_region() {let p=(&n) as *{u32;3};let a=*p;return a[0] as i32;}}",
        "pub fn main() -> i32 {let n:(u32,u32)=(1,2);if trusted_region() {let p=(&n) as *u8;let q=(p+1) as *(u32,u32);let a=*q;return a.0 as i32;}}",
        "pub fn main() -> i32 {let a:u64=0;if trusted_region() {let p=(&a) as *u8;return *(p+8) as i32;}}",
        "pub fn main() -> i32 {let a:u64=0;if trusted_region() {let p=(&a) as *u8;let q=(p+1) as *u32;return *q as i32;}}",
        "pub fn main() -> i32 {let a:u32=0;if trusted_region() {let p=(&a) as *u64;return *p as i32;}}",
        "pub fn dangling() -> *f32 {let n:u32=1088421888;if trusted_region() {return (&n) as *f32;}} pub fn main() -> i32 {if trusted_region() {return *dangling() as i32;}}",
        "pub fn main() -> i32 {let a:{u32;2}={0,0};if trusted_region() {let p=(&a[0]) as *f32;return *(p+2) as i32;}}",
        "pub fn main() -> i32 {let a=1;let b=2;if trusted_region() {return (&a-&b) as i32;}}",
        "pub fn dangling() -> *i32 {let n=7;return &n;} pub fn main() -> i32 {let p=dangling();if trusted_region() {return *p;}}",
        "pub fn main() -> i32 {let a={1,2};let p=&a[0];if trusted_region() {return *(p+2);}}",
        "pub fn main() -> i32 {let a={1,2};let p=&a[0];if trusted_region() {let q=p+3;};return 0;}",
    ] {
        let p=checked(source).unwrap();let m=mir::lower(&p);
        let a=interpret::execute(&p,p.main.unwrap(),vec![],OverflowChecks::Checked,10000).unwrap_err();
        let b=mir::interpret::execute(&m,m.main.unwrap(),vec![],OverflowChecks::Checked,10000).unwrap_err();
        assert_eq!(a.kind,b.kind,"{source}");
    }
}

#[test]
fn verifier_rejects_reinterpret_capability_and_pointee_corruption() {
    let p = checked(
        "pub fn main() -> i32 {let n:u32=1088421888;if trusted_region() {let p=(&n) as *f32;return *p as i32;}}",
    )
    .unwrap();
    let original = mir::lower(&p);
    for target in [
        hir::Type::Pointer {
            mutable: false,
            pointee: Box::new(hir::Type::Tuple(vec![
                hir::Type::Integer(clust_semantics::IntegerType::U8),
                hir::Type::Integer(clust_semantics::IntegerType::U32),
            ])),
        },
        hir::Type::Pointer {
            mutable: true,
            pointee: Box::new(hir::Type::F32),
        },
        hir::Type::Pointer {
            mutable: false,
            pointee: Box::new(hir::Type::Str),
        },
    ] {
        let mut bad = original.clone();
        let mut changed = false;
        for f in &mut bad.functions {
            for block in &mut f.blocks {
                if let mir::Terminator::Evaluate {
                    operation: mir::Operation::Cast(_, ty),
                    destination,
                    ..
                } = &mut block.terminator
                {
                    if matches!(ty, hir::Type::Pointer { .. }) {
                        *ty = target.clone();
                        f.locals[destination.local].ty = target.clone();
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

#[test]
fn verifier_checks_address_cast_types_and_exposure_permissions() {
    let p =
        checked("pub fn main() -> i32 {let n=7;if trusted_region() {let a=(&n) as usize;let p=a as *i32;return *p;}}")
            .unwrap();
    let original = mir::lower(&p);
    for (opcode, replacement) in [
        ("raw_expose", "raw_expose_mut"),
        ("raw_recover", "raw_expose"),
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
