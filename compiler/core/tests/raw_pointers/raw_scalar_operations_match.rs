use super::*;

#[test]
fn raw_scalar_operations_match_in_both_interpreters() {
    for (source, expected) in [
        ("#[Copy] pub enum E {A,B} pub fn main() -> i32 {var n=E.A;if trusted_region() {let p=(&var n) as *var u32;*p=1};return match n {E.A=>0,E.B=>7};}",7),
        ("#[Copy] pub enum E {A(u32),B(u32)} pub fn main() -> i32 {var n=E.A(2);if trusted_region() {let p=(&var n) as *var {u32;2};*p={1,7}};return match n {E.A(n)=>0,E.B(n)=>n as i32};}",7),

        ("pub fn main() -> i32 {let n:u8=1;if trusted_region() {let p=(&n) as *bool;if *p {return 7;};return 0;}}",7),
        ("pub fn main() -> i32 {var n=true;if trusted_region() {let p=(&var n) as *var u8;*p=0;if !n {return 7;};return 0;}}",7),
        ("pub fn main() -> i32 {let n:u32=65;if trusted_region() {let p=(&n) as *char;if *p=='A' {return 7;};return 0;}}",7),
        ("pub fn main() -> i32 {var n='A';if trusted_region() {let p=(&var n) as *var u32;*p=66;if n=='B' {return 7;};return 0;}}",7),
        ("pub fn main() -> i32 {var n:(bool,u8)=(false,7);if trusted_region() {let p=(&var n) as *var u8;*p=1;if n.0 {return n.1 as i32;};return 0;}}",7),

        ("pub fn main() -> i32 {let a={2,7};if trusted_region() {let all=(&a) as usize;let first=(&a[0]) as usize;let second=(&a[1]) as usize;if all==first && second-first==4 {return 7;};return 0;}}",7),
        ("use compiler_fixtures.record_82.{Pair}; pub fn main() -> i32 {let a=Pair {a: 1, b: 7};if trusted_region() {let all=(&a) as usize;let field=(&a.b) as usize;let p=field as *u32;if field-all==4 {return *p as i32;};return 0;}}",7),

        ("pub fn main() -> i32 {var n=2;if trusted_region() {*(&var n)+=5};return n;}",7),
        ("pub fn main() -> i32 {var n:f32=14.0;if trusted_region() {*(&var n)/=2.0};return n as i32;}",7),
        ("pub fn main() -> i32 {var n:u8=1;if trusted_region() {*(&var n)<<=3;*(&var n)^=15};return n as i32;}",7),
        ("pub fn address(p:*var i32,count:*var i32) -> *var i32 {if trusted_region() {*count+=1};return p;} pub fn rhs(p:*var i32) -> i32 {if trusted_region() {*p=99};return 5;} pub fn main() -> i32 {var n=2;var count=0;if trusted_region() {*address(&var n,&var count)+=rhs(&var n)};if count==1 {return n;};return 0;}",7),

        ("pub fn main() -> i32 {var n=2;if trusted_region() {let a=(&var n) as isize;let p=a as *var i32;*p=7;return n;}}",7),
        ("pub fn main() -> i32 {if trusted_region() {let a:isize=-1;let p=a as *i32;let b=p as isize;if b==a {return 7;};return 0;}}",7),
        ("pub fn main() -> i32 {let a={2,7};if trusted_region() {let first=(&a[0]) as usize;let end=(&a[0]+2) as usize;if end-first==8 {return 7;};return 0;}}",7),
        ("pub fn main() -> i32 {let n:f32=7.0;if trusted_region() {let a=(&n) as usize;let p=a as *u32;if *p==1088421888 {return 7;};return 0;}}",7),

        ("pub fn main() -> i32 {var n=2;if trusted_region() {let a=(&var n) as usize;let p=a as *var i32;*p=7;return n;}}",7),
        ("pub fn main() -> i32 {let a={2,7};if trusted_region() {let address=(&a[0]) as usize;let p=(address+4) as *i32;return *p;}}",7),
        ("pub fn main() -> i32 {if trusted_region() {let zero:usize=0;let p=zero as *i32;let a=p as usize;if a==0 {return 7;};return 0;}}",7),
        ("pub fn main() -> i32 {var n=1;let p=&var n;if trusted_region() {*p=7};if n==7 {return 7;};return 0;}",7),
        ("pub fn main() -> i32 {var n:u32=16843009;if trusted_region() {let p=(&var n) as *var u8;*(p+1)=7;return (*(p+1)) as i32;}}",7),
        ("pub fn main() -> i32 {let a:{u32;2}={16843009,117901063};if trusted_region() {let p=(&a[0]) as *u8;let end=p+8;let q=(end-4) as *u32;if *q==117901063 {return 7;};return 0;}}",7),
        ("pub fn main() -> i32 {let a:u64=0;if trusted_region() {let p=(&a) as *u8;let end=p+8;return (end-p) as i32;}}",8),
        ("pub fn main() -> i32 {var a:u64=0;if trusted_region() {let p=(&var a) as *var u8;let q=p as *var u64;*q=7;return a as i32;}}",7),
        ("pub fn pass<T:Copy>(p:*T) -> *T {return p;} pub fn main() -> i32 {let n:f32=7.0;if trusted_region() {let p=pass<u32>((&n) as *u32);if *p==1088421888 {return 7;};return 0;}}",7),
        ("pub fn main() -> i32 {var bits:u32=1065353216;let p=&var bits;if trusted_region() {let q=p as *var f32;*q=7.0;return (*q as i32);}}",7),
        ("pub fn main() -> i32 {let bits:u64=4619567317775286272;if trusted_region() {let q=(&bits) as *f64;return *q as i32;}}",7),
        ("pub fn main() -> i32 {var bits:i32=-1;if trusted_region() {let p=(&var bits) as *var u32;*p=7;return bits;}}",7),
        ("pub fn main() -> i32 {let a:{u32;2}={1065353216,1088421888};if trusted_region() {let p=(&a[0]) as *f32;return *(p+1) as i32;}}",7),
        ("pub fn main() -> i32 {let a={2,5,7};let p=&a[0];if trusted_region() {let end=p+3;let last=end-1;let distance:isize=last-p;return *last+(distance as i32);}}",9),
        ("pub fn main() -> i32 {var a=(2,5);let p=&var a;if trusted_region() {*p=(3,4);let b=*p;return b.0+b.1;}}",7),
        ("pub fn main() -> i32 {var a={2,5};let p=&var a;if trusted_region() {*p={3,4};let b=*p;return b[0]+b[1];}}",7),
        ("pub fn main() -> i32 {let a={1,7};let n:isize=1;if trusted_region() {return *(&a[0]+n);}}",7),
        ("pub fn main() -> i32 {let n=7;let p=&n;let ref v=p;let q:*i32=*v;if trusted_region() {return *q;}}",7),
        (" pub fn read(p:*i32) -> i32 {requires_unsafe_caller();if trusted_region() {return *p;}} pub fn main() -> i32 {let n=7;let f:unsafe (*i32)=>i32=read;if trusted_region() {return f(&n);}}",7),

        (include_str!("../fixtures/raw_pointers.fixture"),8),
        ("pub fn main() -> i32 {var n=2;let p=&var n;let q=p;if trusted_region() {*q=7;return *p;}}",7),
        ("pub fn main() -> i32 {let n=7;let p:*i32=&n;if trusted_region() {return *p;}}",7),
        ("pub fn identity<T:Copy>(p:*T) -> *T {return p;} pub fn main() -> i32 {let n=7;let p=identity(&n);if trusted_region() {return *p;}}",7),
        ("use compiler_fixtures.record_0.{S}; pub fn main() -> i32 {let n=7;let s=S {p: &n};if trusted_region() {return *s.p;}}",7),
        ("pub fn main() -> i32 {let a={7,3};let p=&a[0];if trusted_region() {let end=p+2;let first=end+(-2);return *first;}}",7),
        ("use compiler_fixtures.record_1.{S}; pub fn main() -> i32 {var s=S {n: 1};let p=&var s.n;if trusted_region() {*p=7};return s.n;}",7),
        ("pub class C {pub n:i32;new() {n=1;}} pub fn main() -> i32 {var c=C.new();let p=&var c.n;if trusted_region() {*p=7};return c.n;}",7),
        ("pub fn main() -> i32 {var n=1;let ref var v=n;let p=&var v;if trusted_region() {*p=7};return n;}",7),
        ("pub fn main() -> i32 {var b=false;let p=&var b;if trusted_region() {*p=true;if *p {return 7;}};return 0;}",7),
    ] {
        let p=checked(source).unwrap_or_else(|e|panic!("{source}: {e:?}"));
        let m=mir::lower(&p);
        for mode in [OverflowChecks::Checked,OverflowChecks::Wrapping] {
            let a=interpret::execute(&p,p.main.unwrap(),vec![],mode,10000).unwrap_or_else(|e|panic!("HIR {source}: {e:?}"));
            let b=mir::interpret::execute(&m,m.main.unwrap(),vec![],mode,10000).unwrap_or_else(|e|panic!("MIR {source}: {e:?}"));
            assert_eq!(a,b,"{source}");
            let hir::Value::Integer(n)=a else {panic!()};
            assert_eq!(n.signed_value(),Some(expected),"{source}");
            rust::emit(&p,mode).unwrap();
        }
    }
}

#[test]
fn raw_permissions_and_capabilities_cannot_be_erased() {
    for source in [
        "pub fn main() -> i32 {var n=7;let p=&var n;if trusted_region() {let q:i32=p as *i32;};return 0;}",
        "pub fn main() -> i32 {var n=7;let p=&var n;if trusted_region() {let q:i32=(*p=3);};return 0;}",
        "pub fn main() -> i32 {var n=7;let p=&var n;if trusted_region() {let q:i32=p+0;};return 0;}",
    ] {
        assert!(
            checked(source).is_err(),
            "accepted wrong annotation: {source}"
        );
    }

    for (source, code) in [
        ("pub fn main() -> i32 {let n=2;if trusted_region() {*(&n)+=5};return n;}","E0503"),
        ("pub fn main() -> i32 {var n=2;*(&var n)+=5;return n;}","E0500"),
        ("pub fn main() -> i32 {var n=false;if trusted_region() {*(&var n)&=true};return 0;}","E0311"),
        ("pub fn main() -> i32 {var n:f32=1.0;if trusted_region() {*(&var n)<<=2};return 0;}","E0311"),

        ("pub fn main() -> i32 {let n=7;let a=(&n) as usize;return 0;}","E0500"),
        ("pub fn main() -> i32 {let a:usize=0;let p=a as *i32;return 0;}","E0500"),
        ("pub fn main() -> i32 {let n=7;if trusted_region() {let a=(&n) as u32;};return 0;}","E0312"),
        ("pub fn main() -> i32 {let a:u32=0;if trusted_region() {let p=a as *i32;};return 0;}","E0312"),
        ("pub fn main() -> i32 {let n=();if trusted_region() {let a=(&n) as usize;};return 0;}","E0503"),

        ("use compiler_fixtures.record_92.{R}; pub fn main() -> i32 {let n=R {v: 0, w: 0};if trusted_region() {let p=(&n) as *u8;};return 0;}","E0503"),
        ("pub fn main() -> i32 {let n:(u8,u32)=(0,0);if trusted_region() {let p=(&n) as *u8;};return 0;}","E0503"),
        ("pub fn main() -> i32 {let n=7;let p=&n;if trusted_region() {let q=(&p) as *u64;};return 0;}","E0503"),
        ("#[Copy] pub enum E {None,Some(u32)} pub fn main() -> i32 {let n=E.None;if trusted_region() {let p=(&n) as *u32;};return 0;}","E0503"),
        ("pub fn main() -> i32 {let n:(u32,u32)=(0,0);if trusted_region() {let p=(&n) as *var {u32;2};};return 0;}","E0503"),
        (" pub fn bits<T>(p:*T) -> *u32 {requires_unsafe_caller();if trusted_region() {return p as *u32;}} pub fn main() -> i32 {let n:f32=7.0;if trusted_region() {let p=bits<f32>(&n);};return 0;}","E0373"),
        ("pub fn main() -> i32 {let n:u32=0;let p=(&n) as *f32;return 0;}","E0500"),
        ("pub fn main() -> i32 {let n:u32=0;if trusted_region() {let p=(&n) as *var f32;};return 0;}","E0503"),
        ("use compiler_fixtures.record_93.{R}; pub fn main() -> i32 {let n=R {v: 0, w: 0};if trusted_region() {let p=(&n) as *f32;};return 0;}","E0503"),
        ("pub fn main() -> i32 {let n=7;return *(&n);}", "E0500"),
        (
            " pub fn f(p:*i32) -> i32 {requires_unsafe_caller();return *p;} pub fn main() -> i32 {return 0;}",
            "E0500",
        ),
        ("pub fn main() -> i32 {let n=7;let p=&var n;return 0;}", "E0317"),
        (
            "pub fn main() -> i32 {var n=7;let p=&n;if trusted_region() {*p=3};return n;}",
            "E0503",
        ),
        (
            "pub fn main() -> i32 {let n=7;let p=&n;if trusted_region() {let q=p as *var i32;};return 0;}",
            "E0503",
        ),
        (
            "pub fn main() -> i32 {var n=7;let p=&var n;let q=p as *i32;return 0;}",
            "E0500",
        ),
        (
            "pub fn main() -> i32 {let n=7;let p=&n;let q=p+1;return 0;}",
            "E0500",
        ),
        ("pub fn main() -> i32 {let p=&(1+2);return 0;}", "E0502"),
        (
            "use compiler_fixtures.record_1.{S}; pub fn main() -> i32 {let s=S {n: 7};let p=&s;return 0;}",
            "E0502",
        ),
        (
            "pub fn main() -> i32 {var n=7;let ref v=n;let p=&var n;return v;}",
            "E0381",
        ),
        (
            "pub fn main() -> i32 {var n=7;let ref v=n;let p=&var v;return 0;}",
            "E0317",
        ),
    ] {
        let errors = checked(source).expect_err(source);
        assert!(errors.iter().any(|e| e == code), "{source}: {errors:?}");
    }
}
