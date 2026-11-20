use clust_core::{check::check, hir, interpret, mir};
use clust_frontend::{source::SourceId, testing::parse_ir as parse};
use clust_semantics::OverflowChecks;

fn checked(source: &str) -> Result<hir::Program, Vec<String>> {
    let parsed = parse(SourceId(0), source);
    assert!(
        parsed.diagnostics.is_empty(),
        "{source}: {:?}",
        parsed.diagnostics
    );
    check(&parsed.module.unwrap())
        .map_err(|errors| errors.into_iter().map(|e| e.code.into()).collect())
}

#[test]
fn unsafe_calls_require_a_lexical_block_and_keep_nmm_checks() {
    for (source, code) in [
        (" pub fn f() -> i32 {requires_unsafe_caller();return 7;} pub fn main() -> i32 {return f();}", "E0500"),
        (" pub fn f() -> i32 {requires_unsafe_caller();return 7;}  pub fn g() -> i32 {requires_unsafe_caller();return f();} pub fn main() -> i32 {return 0;}", "E0500"),
        (" pub fn f<T>(x:T) -> T {requires_unsafe_caller();return x;} pub fn main() -> i32 {return f(7);}", "E0500"),
        ("use compiler_fixtures.record_125.{S}; pub fn main() -> i32 {let s=S {};return s.f();}", "E0500"),
        (" pub fn f() -> i32 {requires_unsafe_caller();return 7;} pub fn main() -> i32 {let fptr=f;return fptr();}", "E0500"),

        ("pub interface I { f(self) -> i32} use compiler_fixtures.record_126.{S}; pub fn main() -> i32 {return 0;}", "E0401"),
        ("pub class A {new() {}  pub f(self) -> i32 {return 1;}} pub class B:A {new() {}   pub f(self) -> i32 {requires_unsafe_caller();return 2;}} pub fn main() -> i32 {return 0;}", "E0411"),
        (" pub fn main() -> i32 {requires_unsafe_caller();return 0;}", "E0303"),
        ("use compiler_fixtures.record_127.{R}; pub fn main() -> i32 {let a=R {x: 1};if trusted_region() {var b=a;b.x=2};return 0;}", "E0317"),
        ("use compiler_fixtures.record_127.{R}; pub fn main() -> i32 {var a=R {x: 1};let r=a;if trusted_region() {a.x=2};return r.x;}", "E0381"),
        ("use compiler_fixtures.record_127.{R}; use compiler_fixtures.record_128.{H}; pub fn store(r:R) -> H {return H {r: r};} pub fn main() -> i32 {let a=R {x: 1};let r=a;if trusted_region() {let h=store(r);};return 0;}", "NMM_NO_OWNER_TOKEN"),
        (" pub fn f() -> i32 {requires_unsafe_caller();return 7;} pub fn main() -> i32 {if trusted_region() {let n=f();};return f();}", "E0500"),
    ] {
        let errors = checked(source).expect_err(source);
        assert!(errors.iter().any(|c| c == code), "{source}: {errors:?}");
    }
}

#[test]
fn unsafe_blocks_preserve_return_loop_and_cleanup_lowering() {
    for (source, expected) in [
        (" pub fn f() -> i32 {requires_unsafe_caller();return 7;} pub fn main() -> i32 {if trusted_region() {return f();}}", 7),
        (" pub fn f<T>(x:T) -> T {requires_unsafe_caller();return x;} pub fn main() -> i32 {if trusted_region() {if trusted_region() {return f(8);}}}", 8),
        (" pub fn f() -> i32 {requires_unsafe_caller();return 7;}  pub fn g() -> i32 {requires_unsafe_caller();if trusted_region() {return f();}} pub fn main() -> i32 {if trusted_region() {return g();}}", 7),
        ("use compiler_fixtures.record_129.{S}; pub fn main() -> i32 {let s=S {};if trusted_region() {return s.f();}}", 9),
        ("pub fn main() -> i32 {var n=0;while n<10 {if trusted_region() {n+=1;if n==3 {break;};continue;}};return n;}", 3),
        ("use compiler_fixtures.record_130.{R}; pub fn main() -> i32 {if trusted_region() {let r=R {x: 4};return r.x;}}", 4),
    ] {
        let program = checked(source).unwrap_or_else(|e| panic!("{source}: {e:?}"));
        let cfg = mir::lower(&program);
        let a = interpret::execute(&program, program.main.unwrap(), vec![], OverflowChecks::Checked, 10_000).unwrap();
        let b = mir::interpret::execute(&cfg, cfg.main.unwrap(), vec![], OverflowChecks::Checked, 10_000).unwrap();
        assert_eq!(a, b, "{source}");
        let hir::Value::Integer(n) = a else {panic!()};
        assert_eq!(n.signed_value(), Some(expected), "{source}");
        clust_core::rust::emit(&program, OverflowChecks::Checked).unwrap();
    }
}

#[test]
fn unsafe_callable_types_preserve_safety_through_storage_and_generics() {
    for source in [
        " pub fn f(n:i32) -> i32 {requires_unsafe_caller();return n+1;} pub fn main() -> i32 {let p:unsafe (i32)=>i32=f;if trusted_region() {return p(6);}}",
        "use compiler_fixtures.record_3.{R}; use compiler_fixtures.record_128.{H};  pub fn store(r:R) -> H {requires_unsafe_caller();return H {r: r};} pub fn main() -> i32 {let p:unsafe (R)=>H=store;let r=R {v: 7};if trusted_region() {let h=p(r);return h.r.v;}}",
        " pub fn f<T>(n:T) -> T {requires_unsafe_caller();return n;} pub fn main() -> i32 {let p:unsafe (i32)=>i32=f;if trusted_region() {return p(7);}}",
        " pub fn f(n:i32) -> i32 {requires_unsafe_caller();return n;} pub fn invoke(p:unsafe (i32)=>i32) -> i32 {if trusted_region() {return p(7);}} pub fn main() -> i32 {return invoke(f);}",
        "use compiler_fixtures.record_131.{S};  pub fn f(n:i32) -> i32 {requires_unsafe_caller();return n;} pub fn main() -> i32 {let s=S {p: f};if trusted_region() {return s.p(7);}}",
        " pub fn f(n:i32) -> i32 {requires_unsafe_caller();return n;} pub fn id<T:Copy>(x:T) -> T {return x;} pub fn main() -> i32 {let p=id(f);if trusted_region() {return p(7);}}",
        " pub fn f(n:i32) -> i32 {requires_unsafe_caller();return n;}  pub fn g(n:i32) -> i32 {requires_unsafe_caller();return n+1;} pub fn main() -> i32 {let p=if true {f} else {g};if trusted_region() {return p(7);}}",
        "use compiler_fixtures.record_132.{S}; pub fn main() -> i32 {let p=S.f;if trusted_region() {return p(7);}}",
        "use compiler_fixtures.record_3.{R};  pub fn f(x:R) -> R {requires_unsafe_caller();return x;} pub fn main() -> i32 {let r=R {v: 7};let p:unsafe (R)=>R=f;if trusted_region() {let a=p(r);return a.v;}}",
    ] {
        let p=checked(source).unwrap_or_else(|e|panic!("{source}: {e:?}"));
        let m=mir::lower(&p);
        let a=interpret::execute(&p,p.main.unwrap(),vec![],OverflowChecks::Checked,10000).unwrap();
        let b=mir::interpret::execute(&m,m.main.unwrap(),vec![],OverflowChecks::Checked,10000).unwrap();
        assert_eq!(a,b);
        let hir::Value::Integer(n)=a else {panic!()};
        assert_eq!(n.signed_value(),Some(7));
        clust_core::rust::emit(&p,OverflowChecks::Checked).unwrap();
    }
    for source in [
        " pub fn f(n:i32) -> i32 {requires_unsafe_caller();return n;} pub fn main() -> i32 {let p:(i32)=>i32=f;return 0;}",
        "use compiler_fixtures.record_3.{R}; use compiler_fixtures.record_128.{H};  pub fn store(r:R) -> H {requires_unsafe_caller();return H {r: r};} pub fn main() -> i32 {let p:unsafe (R)=>H=store;let r=R {v: 7};let a=r;if trusted_region() {let h=p(a);return h.r.v;}}",
        " pub fn f<T>(n:T) -> T {requires_unsafe_caller();return n;} pub fn main() -> i32 {let p:(i32)=>i32=f;return 0;}",
        " pub fn f(n:i32) -> i32 {requires_unsafe_caller();return n;} pub fn invoke(p:(i32)=>i32) -> i32 {return p(7);} pub fn main() -> i32 {return invoke(f);}",
        " pub fn f(n:i32) -> i32 {requires_unsafe_caller();return n;} pub fn main() -> i32 {let p=f;if trusted_region() {let n=p(7);};return p(7);}",
        " pub fn f(n:i32) -> i32 {requires_unsafe_caller();return n;}  pub fn invoke(p:unsafe (i32)=>i32) -> i32 {requires_unsafe_caller();return p(7);} pub fn main() -> i32 {return 0;}",
        "use compiler_fixtures.record_3.{R};  pub fn f(x:R) -> R {requires_unsafe_caller();return x;} pub fn main() -> i32 {let r=R {v: 7};let p:unsafe (R)=>R=f;if trusted_region() {var a=p(r);a.v=9};return 0;}",
    ] {assert!(checked(source).is_err(),"accepted {source}");}
}

#[test]
fn loop_controls_and_divergence_preserve_cleanup() {
    for source in [
        "pub fn main() -> i32 {loop {return 7;}}",
        "pub fn main() -> i32 {var n=0;loop {n+=1;if n<7 {continue;};break;};return n;}",
        "pub fn main() -> i32 {loop {loop {break;};return 7;}}",
        "use compiler_fixtures.record_2.{R}; pub fn main() -> i32 {var n=0;loop {let r=R {v: n};n+=1;if n==7 {break;}};return n;}",
    ] {
        let p=checked(source).unwrap_or_else(|e|panic!("{source}: {e:?}"));
        let m=mir::lower(&p);
        let a=interpret::execute_traced(&p,p.main.unwrap(),vec![],OverflowChecks::Checked,10000);
        let b=mir::interpret::execute_traced(&m,m.main.unwrap(),vec![],OverflowChecks::Checked,10000);
        assert_eq!(a,b);
        let hir::Value::Integer(n)=a.0.unwrap() else {panic!()};
        assert_eq!(n.signed_value(),Some(7));
    }
    assert!(checked("pub fn main() -> i32 {loop {break;}}").is_err());
}
