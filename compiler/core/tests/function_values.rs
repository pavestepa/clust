use clust_core::{check::check, hir::Value, interpret, mir};
use clust_frontend::{source::SourceId, testing::parse};
use clust_semantics::OverflowChecks;

#[test]
fn function_values_use_fixed_entries_and_propagate_panics() {
    for (source, expected) in [
        ("use compiler_fixtures.record_4.{R}; pub fn take(r:R) -> i32 {return r.value;} pub fn main() -> i32 {let f=take;return f(R {value: 8});}",Some(8)),
        ("use compiler_fixtures.record_4.{R}; pub fn id<T>(x:T) -> T {return x;} pub fn apply<T>(f:(T)=>T,x:T) -> T {return f(x);} pub fn main() -> i32 {let f:(R)=>R=id;let t=R {value: 9};let r=apply(f,t);return r.value;}",Some(9)),
        ("pub class C {pub value:i32;new(n:i32) {value=n;}} pub fn id(x:C) -> C {return x;} pub fn main() -> i32 {let f:(C)=>C=id;let a=C.new(7);let b=f(a);return b.value;}",Some(7)),

        ("pub fn identity<T>(x:T) -> T {return x;} pub fn main() -> i32 {let f:(i32)=>i32=identity;return f(9);}",Some(9)),
        ("pub fn identity<T>(x:T) -> T {return x;} pub fn main() -> i32 {let f:(i32)=>i32=identity<i32>;return f(9);}",Some(9)),
        ("pub fn first<T:Copy,const N:usize>(xs:{T;N}) -> T {return xs[0];} pub fn main() -> i32 {let f:({i32;2})=>i32=first;return f({8,9});}",Some(8)),
        ("pub fn identity<T>(x:T) -> T {return x;} pub fn select() -> (i32)=>i32 {return identity;} pub fn main() -> i32 {return select()(6);}",Some(6)),
        ("use compiler_fixtures.record_7.{Math}; pub fn main() -> i32 {let f=Math.add;return f(3);}",Some(5)),
        ("use compiler_fixtures.record_8.{Handler}; pub fn add(n:i32) -> i32 {return n+2;} pub fn main() -> i32 {let h=Handler {run: add};return h.invoke();}",Some(6)),
        ("pub fn add(n:i32) -> i32 {return n+2;} pub fn main() -> i32 {let f:(i32)=>i32=add;return f(3);}",Some(5)),
        ("pub fn one() -> i32 {return 1;} pub fn two() -> i32 {return 2;} pub fn choose(b:bool) -> ()=>i32 {return if b {one} else {two};} pub fn main() -> i32 {return choose(false)();}",Some(2)),
        ("pub fn bump(var n:i32) -> i32 {n+=1;return n;} pub fn apply(f:(var i32)=>i32,n:i32) -> i32 {return f(n);} pub fn main() -> i32 {let n=3;return apply(bump,n)+n;}",Some(7)),
        ("pub fn one() -> i32 {return 1;} pub fn two() -> i32 {return 2;} pub fn main() -> i32 {var f=one;let copy=f;f=two;return f()*10+copy();}",Some(21)),
        ("use compiler_fixtures.record_9.{Handler}; pub fn add(n:i32) -> i32 {return n+2;} pub fn main() -> i32 {let h=Handler {run: add};return h.run(3);}",Some(5)),
        ("pub fn add(n:i32) -> i32 {return n+2;} pub fn apply<T:Copy>(f:(T)=>T,n:T) -> T {return f(n);} pub fn main() -> i32 {return apply(add,3);}",Some(5)),
        ("pub fn add(n:i32) -> i32 {return n+2;} pub fn invoke<T:Copy>(f:()=>T) -> T {return f();} pub fn five() -> i32 {return 5;} pub fn main() -> i32 {return invoke(five);}",Some(5)),
        ("pub fn yes(n:i32) -> bool {return n>0;} pub fn main() -> i32 {let predicate=yes;return match 7 {x if predicate(x)=>x,_=>0};}",Some(7)),
        ("pub fn fail() -> i32 {return 1/0;} pub fn main() -> i32 {let f=fail;return f();}",None),
    ] {
        let parsed=parse(SourceId(0),source);
        assert!(parsed.diagnostics.is_empty(),"{source}: {:?}",parsed.diagnostics);
        let p=check(&parsed.module.unwrap()).unwrap_or_else(|e|panic!("{source}: {e:?}"));
        let cfg=mir::lower(&p);
        let a=interpret::execute(&p,p.main.unwrap(),vec![],OverflowChecks::Checked,10000).map_err(|e|e.kind);
        let b=mir::interpret::execute(&cfg,cfg.main.unwrap(),vec![],OverflowChecks::Checked,10000).map_err(|e|e.kind);
        assert_eq!(a,b,"{source}");
        match expected {
            Some(expected) => {let Value::Integer(value)=a.unwrap() else {panic!()};assert_eq!(value.signed_value(),Some(expected));}
            None=>assert!(a.is_err()),
        }
        clust_core::rust::emit(&p,OverflowChecks::Checked).unwrap();
    }
}

#[test]
fn function_values_reject_signature_and_entry_mode_mismatches() {
    for source in [
        "pub fn bad<T>(x:T) -> T {return x+1;} pub fn main() -> () {let f:(i32)=>i32=bad;}",
        "pub fn identity<T>(x:T) -> T {return x;} pub fn main() -> () {let f:(i32)=>i32=identity<bool>;}",
        "pub fn f(n:i32) -> i32 {return n;} pub fn main() -> () {let pointer:(bool)=>bool=f;}",
        "pub fn f(var n:i32) -> i32 {return n;} pub fn main() -> () {let pointer:(i32)=>i32=f;}",
        "pub fn f(n:i32) -> i32 {return n;} pub fn main() -> () {let pointer=f;pointer(true);}",
        "pub fn f(n:i32) -> i32 {return n;} pub fn main() -> () {let pointer=f;pointer();}",
        "pub fn main() -> () {let f=1;f();}",
        "pub fn identity<T>(x:T) -> T {return x;} pub fn main() -> () {let f=identity;}",
    ] {
        let parsed = parse(SourceId(0), source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert!(check(&parsed.module.unwrap()).is_err(), "accepted {source}");
    }
}
