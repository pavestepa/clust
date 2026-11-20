use clust_core::{check::check, rust::emit};
use clust_frontend::{source::SourceId, testing::parse};
use clust_semantics::OverflowChecks;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static SEQUENCE: AtomicUsize = AtomicUsize::new(0);

fn verify(source: &str, checks: OverflowChecks) {
    let parsed = parse(SourceId(0), source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let program = check(&parsed.module.unwrap()).unwrap();
    let ir = emit(&program, checks).unwrap();
    let compiler = std::env::var("CLUST_RUSTC").unwrap_or_else(|_| "rustc".into());
    // A missing rustc is a failure: these tests compile emitted Rust.
    let dir = std::env::temp_dir().join(format!(
        "clust-rust-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    let input = dir.join("case.rs");
    let output = dir.join("case.o");
    fs::write(&input, &ir).unwrap();
    for optimization in ["opt-level=0", "opt-level=3"] {
        let result = Command::new(&compiler)
            .args([
                "--edition=2021",
                "--crate-type=lib",
                "--emit=obj",
                "-C",
                optimization,
            ])
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .output()
            .expect("Rust compiler required; set CLUST_RUSTC");
        assert!(
            result.status.success(),
            "{}\n{ir}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(fs::metadata(&output).unwrap().len() > 0);
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rust_accepts_recursive_control_flow_and_aggregate_values() {
    for code in [
        include_str!("../../../examples/arithmetic/main.cl"),
        "use compiler_fixtures.record_16.{Box}; pub fn main() -> i32 {let boxed=Box<i32>.create(17);return Box.extract(boxed);}",
        "pub fn predicate(var x:i32) -> bool {x+=1;return x==4;} pub fn main() -> i32 {let n=3;return match n {x if predicate(x)=>x,_=>0};}",

        "#[Copy] pub enum Option<T> {None,Some(T)} pub fn unwrap<T:Copy>(value:Option<T>,fallback:T) -> T {return match value {Option.Some(x)=>x,Option.None=>fallback};} pub fn main() -> i32 {let value=Option<i32>.Some(8);return unwrap(value,1);}",

        "pub fn identity<T>(x:T) -> T {return x;} pub fn main() -> i32 {let a:u8=identity(7);return identity<i32>(a as i32);}",
        "pub fn last<T,const N:usize>(values:{T;N}) -> T {return values[N-1];} pub fn main() -> i32 {return last({1,2,3});}",
        "pub fn pick<T>(n:i32,x:T) -> T {if n==0 {return x;};return pick<T>(n-1,x);} pub fn main() -> i32 {return pick(5,120);}",

        "#[Copy] pub enum E {None,Some(i32)} pub fn main() -> i32 {let a=E.None;return match a {E.Some(x)=>x,E.None=>7};}",
        "#[Copy] pub enum E {A {x:i32,y:i32},B(u128),C} pub fn main() -> i32 {let a=E.B(340282366920938463463374607431768211455);return match a {E.A {x,y}=>x+y,E.B(x)=> (x/3) as i32,E.C=>0};}",
        "#[Copy] pub enum Inner {A(i32),B} #[Copy] pub enum Outer {Some(Inner),None} pub fn main() -> i32 {let x=Outer.Some(Inner.B);return match x {Outer.Some(Inner.A(v))=>v,Outer.Some(_)=>2,Outer.None=>3};}",

        "pub fn main() -> i32 {return match (3,true) {(x,false)=>x,(x,true) if x>4=>9,(x,_)=>x+10};}",
        "pub fn main() -> i32 {match true {false=>{return 4;},true=>{return 7;}}}",
        "pub fn main() -> i32 {return match {1,2,3,4} {{x,..,y}=>x+y};}",
        "use compiler_fixtures.record_17.{Pair}; pub fn main() -> i32 {var n=0;var p=Pair {right:if true {n+=1;n} else {0},left:if true {n+=1;n} else {0}};let c=p;p.left+=10;return c.left+p.left;}",

        "pub fn main() -> i32 {return if true {return 1;} else {return 2;};}",
        "pub fn main() -> i32 {var i=0;while i<4 {i+=1;if i==2 {continue;};if i==3 {break;}};return i;}",
        "pub fn main() -> i32 {var pair=(1,true);pair.0+=2;return pair.0;}",
        "pub fn main() -> i32 {var a={1,2};var i:usize=0;a[if true {i+=1;i} else {0}]+=3;return a[1];}",
        "pub fn main() -> i32 {return if false && 1/0==0 {1} else {2};}",
        "pub fn main() -> i32 {let text=\"héllo\";return 0;}",
    ] {
        verify(code, OverflowChecks::Checked);
    }
}

#[test]
fn rust_accepts_integer_checks_shifts_and_saturating_float_casts() {
    for mode in [OverflowChecks::Checked, OverflowChecks::Wrapping] {
        for code in [
            "pub fn main() -> i32 {let x:i8=127;return (x+1) as i32;}",
            "pub fn main() -> i32 {let x:i128=-170141183460469231731687303715884105728;return (x % -1) as i32;}",
            "pub fn main() -> i32 {let x:i128=1;let count:i8=-1;return (x << count) as i32;}",
            "pub fn main() -> i32 {let x:u8=255;let count:u128=256;return (x >> count) as i32;}",
            "pub fn main() -> i32 {let x:f32=1.5;return x as i32;}",
            "pub fn main() -> i32 {let x=0.0/0.0;return x as i32;}",
            "pub fn main() -> i32 {let x:f32=1.5;let wide=x as f64;return wide as i32;}",
        ] {verify(code,mode);}
    }
}

#[test]
fn rust_accepts_fixed_entry_function_values() {
    for source in [
        "use compiler_fixtures.record_7.{Math}; pub fn main() -> i32 {let f=Math.add;return f(3);}",
        "use compiler_fixtures.record_8.{Handler}; pub fn add(n:i32) -> i32 {return n+2;} pub fn main() -> i32 {let h=Handler {run: add};return h.invoke();}",
        "pub fn add(n:i32) -> i32 {return n+2;} pub fn main() -> i32 {let f:(i32)=>i32=add;return f(3);}",
        "pub fn one() -> i32 {return 1;} pub fn two() -> i32 {return 2;} pub fn choose(b:bool) -> ()=>i32 {return if b {one} else {two};} pub fn main() -> i32 {return choose(false)();}",
        "pub fn bump(var n:i32) -> i32 {n+=1;return n;} pub fn apply(f:(var i32)=>i32,n:i32) -> i32 {return f(n);} pub fn main() -> i32 {let n=3;return apply(bump,n)+n;}",
        "pub fn one() -> i32 {return 1;} pub fn two() -> i32 {return 2;} pub fn main() -> i32 {var f=one;let copy=f;f=two;return f()*10+copy();}",
        "use compiler_fixtures.record_9.{Handler}; pub fn add(n:i32) -> i32 {return n+2;} pub fn main() -> i32 {let h=Handler {run: add};return h.run(3);}",
        "pub fn add(n:i32) -> i32 {return n+2;} pub fn apply<T:Copy>(f:(T)=>T,n:T) -> T {return f(n);} pub fn main() -> i32 {return apply(add,3);}",
        "pub fn add(n:i32) -> i32 {return n+2;} pub fn invoke<T:Copy>(f:()=>T) -> T {return f();} pub fn five() -> i32 {return 5;} pub fn main() -> i32 {return invoke(five);}",
        "pub fn yes(n:i32) -> bool {return n>0;} pub fn main() -> i32 {let predicate=yes;return match 7 {x if predicate(x)=>x,_=>0};}",
        "pub fn fail() -> i32 {return 1/0;} pub fn main() -> i32 {let f=fail;return f();}",
    ] { verify(source, OverflowChecks::Checked); }
}

#[test]
fn rust_accepts_generic_function_values_and_consuming_reinitialization() {
    for source in [
        "pub fn identity<T>(x:T) -> T {return x;} pub fn main() -> i32 {let f:(i32)=>i32=identity;return f(9);}",
        "pub fn identity<T>(x:T) -> T {return x;} pub fn main() -> i32 {let f:(i32)=>i32=identity<i32>;return f(9);}",
        "pub fn first<T:Copy,const N:usize>(xs:{T;N}) -> T {return xs[0];} pub fn main() -> i32 {let f:({i32;2})=>i32=first;return f({8,9});}",
        "pub fn identity<T>(x:T) -> T {return x;} pub fn select() -> (i32)=>i32 {return identity;} pub fn main() -> i32 {return select()(6);}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {var x=Cell {value: 1};let n=take(x);x=Cell {value: 4};return n+x.value;}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {var x=Cell {value: 1};let n=take(x);if true {x=Cell {value: 4}} else {x=Cell {value: 5}};return n+x.value;}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {var x=Cell {value: 1};var n=0;while n<6 {n+=take(x);x=Cell {value: 1};};return n+x.value;}",
        "use compiler_fixtures.record_5.{Cell}; use compiler_fixtures.record_6.{Hold}; pub fn take(x:Cell) -> i32 {let held=Hold {cell: x};return held.cell.value+1;} pub fn main() -> i32 {var x=Cell {value: 1};var n=0;while n<6 {n+=take(x);x=Cell {value: 1};};return n+x.value;}",
    ] { verify(source, OverflowChecks::Checked); }
}
