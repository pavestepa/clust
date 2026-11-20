use clust_core::{check::check, hir, interpret, mir};
use clust_frontend::{source::SourceId, testing::parse};
use clust_semantics::OverflowChecks;

const PREFIX: &str = "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_35.{D}; ";
const CASES: &[(&str, i32)] = &[
    ("pub class C {pub value:R;new(_borrowed_x:R) {let x=_borrowed_x;value=x;}} pub fn wrap(_borrowed_x:R) -> C {let x=_borrowed_x;return C.new(x);} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var c=wrap(a);c.value=w;return c.value.value;}",3),

    ("pub fn first(x:R,y:R) -> R {return x;} pub fn second(x:R,y:R) -> R {return y;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let f=if false {first} else {second};let result=f(a,b);return a.value+b.value+result.value;}",13),

    ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let d=D {value:v};let r=d.get();return r.value;}",7),
    ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var d=D {value:v};let b=R {value: 7};let ref w=b;d=D {value:w};return 3;}",3),

    ("pub fn one(x:R) -> R {return x;} pub fn two(x:R) -> R {return x;} pub fn choose(b:bool) -> (R)=>R {return if b {one} else {two};} pub fn main() -> i32 {let a=R {value: 7};let f=choose(false);let b=f(a);return a.value+b.value;}",14),
    ("pub fn id(x:R) -> R {return x;} pub fn choose() -> (R)=>R {return id;} pub fn main() -> i32 {let a=R {value: 7};let b=choose()(a);return a.value+b.value;}",14),

    ("pub fn id(x:R) -> R {return x;} pub fn main() -> i32 {let a=R {value: 7};let f=id;let b=f(a);return a.value+b.value;}",14),
    ("pub fn id(x:R) -> R {return x;} pub fn apply(f:(R)=>R,x:R) -> R {return f(x);} pub fn main() -> i32 {let a=R {value: 7};let f=id;let b=apply(f,a);return a.value+b.value;}",14),
    ("use compiler_fixtures.record_36.{H}; pub fn wrap(_borrowed_x:R) -> H {let x=_borrowed_x;return H {value: x};} pub fn main() -> i32 {let a=R {value: 7};let f=wrap;let ref v=a;let h=f(v);return a.value+h.value.value;}",14),
    ("use compiler_fixtures.record_36.{H}; pub fn set(var h:H,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=H {value: a};let f=set;f(h,w);return h.value.value;}",3),
    ("use compiler_fixtures.record_37.{S}; pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let s=S {value: v};let f=S.get;let b=f(s);return b.value;}",7),
    ("pub fn one(x:R) -> R {return x;} pub fn two(x:R) -> R {return x;} pub fn main() -> i32 {let a=R {value: 7};let f=if false {one} else {two};let b=f(a);return a.value+b.value;}",14),
    ("pub fn id<T>(x:T) -> T {return x;} pub fn main() -> i32 {let a=R {value: 7};let f:(R)=>R=id;let b=f(a);return a.value+b.value;}",14),

    ("pub class C {pub value:R;new(_borrowed_x:R) {let x=_borrowed_x;value=x;};drop() {let valid=1/(value.value-6);}} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let c=C.new(v);return 3;}",3),
    ("pub class C {pub value:R;new(_borrowed_x:R) {let x=_borrowed_x;value=x;};pub get(self) -> R {return value;}} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let c=C.new(v);let r=c.get();return r.value;}",7),
    ("pub class C {pub value:R;new(_borrowed_x:R) {let x=_borrowed_x;value=x;}} pub fn wrap(_borrowed_x:R) -> C {let x=_borrowed_x;return C.new(x);} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let c=wrap(v);return a.value+c.value.value;}",14),
    ("pub class C {pub value:R;new(_borrowed_x:R) {let x=_borrowed_x;value=x;};drop() {let valid=1/(value.value-6);}} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let c=C.new(v);return 1/0;}",101),
    ("pub class C {pub value:R;new(_borrowed_x:R) {let x=_borrowed_x;value=x;};drop() {let valid=1/(value.value-6);}} pub fn main() -> i32 {let a=R {value: 6};let ref v=a;let c=C.new(v);return 3;}",101),
    ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let d=D {value:v};return d.value.value;}",7),
    ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let d=D {value:v};return 3;}",3),
    ("pub fn main() -> i32 {let a=R {value: 7};if true {let ref v=a;let d=D {value:v};};return a.value;}",7),
    ("pub fn consume(d:D) -> i32 {return d.value.value;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let d=D {value:v};return consume(d);}",7),
    ("pub fn wrap(_borrowed_x:R) -> D {let x=_borrowed_x;return D {value:x};} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let d=wrap(v);return a.value+d.value.value;}",14),
    ("pub fn id<T>(x:T) -> T {return x;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let d=D {value:v};let moved=id(d);return moved.value.value;}",7),
    ("use compiler_fixtures.record_38.{G}; pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let d=G<i32> {tag: 0, value: v};return 3;}",3),
    ("use compiler_fixtures.record_39.{Outer}; pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let outer=Outer {d: D {value:v}};return 3;}",3),
    ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var d=D {value:v};d=D {value:v};return 3;}",3),
    ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let d=D {value:v};return 1/0;}",101),
    ("pub fn main() -> i32 {let a=R {value: 6};let ref v=a;let d=D {value:v};return 3;}",101),
];
fn program(body: &str) -> hir::Program {
    let source = format!("{PREFIX}{body}");
    let parsed = parse(SourceId(0), &source);
    assert!(
        parsed.diagnostics.is_empty(),
        "{source}: {:?}",
        parsed.diagnostics
    );
    check(&parsed.module.unwrap()).unwrap_or_else(|e| panic!("{source}: {e:?}"))
}
#[test]
fn destructor_keeps_stored_views_alive_through_cleanup() {
    for (body, expected) in CASES {
        let p = program(body);
        let m = mir::lower(&p);
        let a = interpret::execute_traced(
            &p,
            p.main.unwrap(),
            vec![],
            OverflowChecks::Checked,
            100_000,
        );
        let b = mir::interpret::execute_traced(
            &m,
            m.main.unwrap(),
            vec![],
            OverflowChecks::Checked,
            100_000,
        );
        assert_eq!(a, b, "{body}");
        if *expected == 101 {
            assert!(a.0.is_err(), "{body}: {a:?}")
        } else {
            assert!(
                matches!(a.0,Ok(hir::Value::Integer(n)) if n.signed_value()==Some(*expected as i128)),
                "{body}: {a:?}"
            );
        }
    }
}
#[test]
fn destructor_dependencies_reject_early_target_invalidation() {
    for body in [
        "pub fn set(var d:D,_borrowed_x:R) -> () {let x=_borrowed_x;d.value=x;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var d=D {value:v};let b=R {value: 7};let ref w=b;let ref var alias=d;set(alias,w);return 0;}",
        "pub fn first(x:R,y:R) -> R {return x;} pub fn second(x:R,y:R) -> R {return y;} pub fn main() -> i32 {var a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;let f=if false {first} else {second};let result=f(v,w);a=R {value: 2};return result.value;}",
        "pub fn one(x:R) -> R {return x;} pub fn two(x:R) -> R {return R {value: 3};} pub fn main() -> i32 {let a=R {value: 7};let f=if false {one} else {two};let b=f(a);return a.value+b.value;}",
        "pub fn id(x:R) -> R {return x;} pub fn bad() -> R {let a=R {value: 7};let ref v=a;let f=id;return f(v);} pub fn main() -> i32 {return 0;}",
        "use compiler_fixtures.record_36.{H}; pub fn set(var h:H,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref v=a;let ref w=b;var h=H {value: v};let f=set;f(h,w);b=R {value: 2};return h.value.value;}",
        "pub class C {pub value:R;new(_borrowed_x:R) {let x=_borrowed_x;value=x;}} pub fn bad() -> C {let a=R {value: 7};let ref v=a;return C.new(v);} pub fn main() -> i32 {return 0;}",
        "pub class C {pub value:R;new(_borrowed_x:R) {let x=_borrowed_x;value=x;};drop() {let n=value.value;}} pub fn main() -> i32 {var a=R {value: 7};let ref v=a;let c=C.new(v);a=R {value: 3};return 0;}",
        "pub fn main() -> i32 {var a=R {value: 7};let ref v=a;let d=D {value:v};a=R {value: 3};return 0;}",
        "pub fn main() -> i32 {var a=R {value: 7};let ref v=a;let d=D {value:v};a.value=3;return 0;}",
        "pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var d=D {value:v};let b=R {value: 7};let ref w=b;d.value=w;return 0;}",
        "pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var d=D {value:v};if true {let b=R {value: 7};let ref w=b;d.value=w};return 0;}",
        "pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var d=D {value:v};var b=R {value: 7};let ref var alias=b;*alias=R {value: 7};let ref w=b;d.value=w;return 0;}",
        "pub fn bad() -> D {let a=R {value: 7};let ref v=a;return D {value:v};} pub fn main() -> i32 {return 0;}",
    ] {
        let parsed = parse(SourceId(0),&format!("{PREFIX}{body}"));
        assert!(parsed.diagnostics.is_empty(),"{:?}",parsed.diagnostics);
        assert!(check(&parsed.module.unwrap()).is_err(),"accepted {body}");
    }
}
#[test]
fn native_borrowed_destructors_observe_live_targets_and_exact_drop_order() {
    use std::{fs, process::Command};
    let mut rust = String::new();
    let mut traces = Vec::new();
    for (i, (body, _)) in CASES.iter().enumerate() {
        let p = program(body);
        let (_, events) = interpret::execute_traced(
            &p,
            p.main.unwrap(),
            vec![],
            OverflowChecks::Checked,
            100_000,
        );
        traces.push(
            events
                .iter()
                .map(|e| format!("DROP:{}", e.entry))
                .collect::<Vec<_>>(),
        );
        let emitted = clust_core::rust::emit(&p, OverflowChecks::Checked)
            .unwrap()
            .replace("fn main()", "pub fn run()");
        let emitted = emitted
            .lines()
            .map(|line| {
                if let Some(id) = line
                    .strip_prefix("// destructor-entry ")
                    .or_else(|| line.strip_prefix("// class-destructor-entry "))
                {
                    format!("eprintln!(\"DROP:{id}\");")
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        rust.push_str(&format!("mod case{i} {{\n{emitted}\n}}\n"));
    }
    rust.push_str(
        "fn main() {match std::env::args().nth(1).unwrap().parse::<usize>().unwrap() {\n",
    );
    for i in 0..CASES.len() {
        rust.push_str(&format!("{i}=>case{i}::run(),\n"));
    }
    rust.push_str("_=>panic!(),}}\n");
    let dir =
        std::env::temp_dir().join(format!("clust-borrowed-destructors-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let source = dir.join("cases.rs");
    fs::write(&source, rust).unwrap();
    for level in ["0", "3"] {
        let binary = dir.join(format!("cases-{level}"));
        let result = Command::new(std::env::var("CLUST_RUSTC").unwrap_or_else(|_| "rustc".into()))
            .arg("--edition=2021")
            .args(["-C", &format!("opt-level={level}")])
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        for (i, (body, expected)) in CASES.iter().enumerate() {
            let result = Command::new(&binary).arg(i.to_string()).output().unwrap();
            let stderr = String::from_utf8_lossy(&result.stderr);
            assert_eq!(
                result.status.code(),
                Some(*expected),
                "O{level}: {body}: {stderr}"
            );
            assert_eq!(
                stderr
                    .lines()
                    .filter(|s| s.starts_with("DROP:"))
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
                traces[i],
                "O{level}: {body}"
            );
        }
    }
    fs::remove_dir_all(dir).unwrap();
}
