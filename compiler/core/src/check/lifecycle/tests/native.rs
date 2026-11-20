use super::cleanup::execute_cleanup;
use super::*;
#[test]
fn generated_rust_executes_resource_cleanup() {
    use std::{fs, process::Command};
    let cases = [
            "use compiler_fixtures.record_4.{R}; pub fn id<T>(x:T) -> T {return x;} pub fn apply<T>(f:(T)=>T,x:T) -> T {return f(x);} pub fn main() -> i32 {let f:(R)=>R=id;let t=R {value: 9};let r=apply(f,t);return r.value;}",
            "pub class C {pub value:i32;new(n:i32) {value=n;};drop() {}} pub fn relay(x:C) -> C {return x;} pub fn main() -> i32 {let f:(C)=>C=relay;let a=C.new(7);let b=f(a);return b.value;}",
            "use compiler_fixtures.record_4.{R}; pub fn fail(a:R,b:R) -> R {let n=1/0;return a;} pub fn main() -> i32 {let f=fail;let t1=R {value: 1};let t2=R {value: 2};let a=f(t1,t2);return a.value;}",
            "use compiler_fixtures.record_4.{R}; pub fn take(a:R,b:R) -> i32 {return a.value+b.value;} pub fn bad() -> R {let n=1/0;return R {value: 8};} pub fn main() -> i32 {let f=take;return f(R {value: 1},bad());}",

            "pub class C {pub value:i32;new(n:i32) {value=n;};drop() {}} pub fn id(x:C) -> C {return x;} pub fn relay(x:C) -> C {return id(x);} pub fn main() -> i32 {let a=C.new(7);let b=relay(a);return a.value+b.value;}",
            "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_147.{P}; pub fn main() -> i32 {let p=P {a: R {value: 1}, b: R {value: 2}};let r=p.get();return p.b.value+r.value;}",
            "use compiler_fixtures.record_4.{R}; pub fn id(var x:R) -> R {return x;} pub fn main() -> i32 {var a=R {value: 1};let other=R {value: 9};let ref var v=id(a);*v=R {value: 2};return a.value;}",
            "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_107.{F}; pub fn get(x:R) -> R {let f=F {value: 0};return x;} pub fn main() -> i32 {let a=R {value: 7};let b=get(a);return a.value+b.value;}",
            "pub class C {pub value:i32;new(n:i32) {value=n;};drop() {}} pub fn pick(flag:bool,x:C,y:C) -> C {return if flag {x} else {y};} pub fn main() -> i32 {let a=C.new(3);let b=C.new(7);let c=pick(false,a,b);return a.value+b.value+c.value;}",
            "pub class C {pub value:i32;new(n:i32) {value=n;};drop() {}} pub fn main() -> i32 {let a=C.new(1);let b=C.new(1);let ref v=a;if a!=b && a==v && same_identity(v,a) {return 7;};return 0;}",
            "#[Copy] pub class C {pub value:i32;new(n:i32) {value=n;}} pub fn main() -> i32 {let identity_probe=C.new(1);let ref v=identity_probe;if identity_probe==v && same_identity(identity_probe,v) {return 3;};return 0;}",
            "#[Copy] pub class C {pub value:i32;new(n:i32) {value=n;}} pub fn main() -> i32 {let a=C.new(1);let b=a;if a==b {return 1;};return 0;}",
            "pub class C {pub value:i32;new(n:i32) {value=n;};drop() {}} pub fn main() -> i32 {return if same_identity(C.new(1),C.new(1)) {1} else {0};}",
            "#[Copy] pub class C {pub value:i32;new(n:i32) {value=n;}} use compiler_fixtures.record_148.{S}; pub fn main() -> i32 {let a=S {c: C.new(1)};var b=a;b.c.value=7;return a.c.value*10+b.c.value;}",

            "pub class C {pub value:i32;new(n:i32) {value=n;};drop() {}} pub fn relay(x:C) -> C {return x;} pub fn main() -> i32 {let a=C.new(7);let b=relay(a);return b.value;}",
            "pub class Empty {new() {} } pub fn main() -> i32 {let e=Empty.new();return 0;}",
            "pub class C {pub value:i32;new(flag:bool) {match flag {true=>{value=1},false=>{value=2}}}} pub fn main() -> i32 {let c=C.new(false);return c.value;}",
            "pub class C {pub value:i32;new(n:i32) {let ref var alias=self;alias.value=n;}} pub class Pair {pub left:C;pub right:C;new() {right=C.new(2);left=C.new(1);}} pub fn main() -> i32 {var p=Pair.new();p.left=C.new(3);return p.left.value+p.right.value;}",
            "pub class C {pub value:i32;new(n:i32) {value=n;}} pub class Pair {pub left:C;pub right:C;new() {left=C.new(1);let fail=1/0;right=C.new(2);}} pub fn main() -> i32 {let p=Pair.new();return 0;}",
            "pub class Cell<T> {pub value:T;new(v:T) {value=v;}} pub fn read<T:Copy>(x:Cell<T>) -> T {return x.value;} pub class Array<const N:usize> {pub values:{i32;N};new(v:{i32;N}) {values=v;}} pub fn main() -> i32 {let c=Cell<i32>.new(3);let a=Array<2>.new({4,5});return read(c)+a.values[1];}",

            "#[Copy] pub class C {pub value:i32;new(n:i32) {value=n;}} pub fn main() -> i32 {let a=C.new(1);var b=a;b.value=7;return a.value*10+b.value;}",
            "#[Copy] pub class C {pub value:i32;new(n:i32) {value=n;}} #[Copy] pub class Pair {pub left:C;pub right:C;new(a:C,b:C) {left=a;right=b;}} pub fn main() -> i32 {let a=Pair.new(C.new(1),C.new(2));var b=a;b.left.value=7;return a.left.value*10+b.left.value;}",
            "#[Copy] pub class C {pub value:i32;new(n:i32) {value=n;}} #[Copy] pub enum E {Some(C),None} pub fn main() -> i32 {let a=E.Some(C.new(1));var b=a;match b {E.Some(ref var x)=>{x.value=7},E.None=>{}};return match a {E.Some(x)=>x.value,E.None=>0};}",
            "#[Copy] pub class C {pub value:i32;new(n:i32) {value=n;}} pub fn id(x:C) -> C {return x;} pub fn main() -> i32 {let a={C.new(1),C.new(2)};var b=a;b[0].value=7;let f:(C)=>C=id;var c=f(b[0]);c.value=9;return a[0].value*10+b[0].value+c.value;}",

            "pub class C {pub value:i32;new(n:i32) {value=n;};pub bump(var self) -> () {value+=1;};drop() {value+=10;}} pub fn main() -> i32 {var x=C.new(3);x.bump();return x.value;}",
            "pub class C {pub value:i32;new(n:i32) {value=n;};drop() {}} pub fn make(n:i32) -> C {return C.new(n);} pub fn id(x:C) -> C {return x;} pub fn main() -> i32 {var x=make(1);let y=id(x);let n=y.value;x=make(3);return x.value+n;}",
            "use compiler_fixtures.record_4.{R}; pub class C {pub value:i32;pub a:R;pub b:R;new() {value=9;b=R {value: 2};a=R {value: 1};let fail=1/0;};drop() {}} pub fn main() -> i32 {let outer=R {value: 8};let x=C.new();return 0;}",
            "use compiler_fixtures.record_4.{R}; pub class C {pub value:i32;pub a:R;pub b:R;new() {value=9;a=R {value: 1};b=R {value: 2};};drop() {a.value+=10;}} pub fn main() -> i32 {let x=C.new();return 0;}",
            "pub class C {pub value:i32;new(n:i32) {value=n;};drop() {}} pub fn main() -> i32 {var a=C.new(1);let b=C.new(2);a=C.new(3);return a.value+b.value;}",
            "pub class C {pub value:i32;new(n:i32) {value=n;};drop() {let fail=1/value;}} pub fn main() -> i32 {var a=C.new(0);a=C.new(2);return 0;}",
            "pub class C {pub value:i32;new(n:i32) {value=n;};drop() {let fail=1/value;}} pub fn main() -> i32 {let a=C.new(0);let b=C.new(0);return 0;}",
            "pub class C<T:Copy> {pub value:T;new(n:T) {value=n;};pub read(self) -> T {return value;};drop() {let x=value;}} pub fn main() -> i32 {let a=C<i32>.new(4);let b=C<i64>.new(5);return a.read()+b.read() as i32;}",
            "pub class Node {pub value:i32;pub next:Link;new(n:i32,tail:Link) {value=n;next=tail;};drop() {}} pub enum Link {End,More(Node)} pub fn main() -> i32 {let a=Node.new(1,Link.End);let b=Node.new(2,Link.More(a));return b.value;}",
            "pub class C {pub value:i32;new(n:i32) {value=n;};drop() {}} pub fn main() -> i32 {var a=C.new(1);var b=C.new(2);let ref var v=a;v.value=4;v=b;v.value=5;return a.value+b.value;}",
            "pub class C {pub value:i32;new(n:i32) {value=n;};drop() {}} pub fn main() -> i32 {let x={C.new(1),C.new(2)};return x[0].value+x[1].value;}",
            "use compiler_fixtures.record_4.{R}; pub class C {pub value:i32;pub a:R;new(n:i32,x:R) {value=n;a=x;};drop() {}} pub fn main() -> i32 {let x=C.new(9,R {value: 1});return x.a.value;}",

            "use compiler_fixtures.record_4.{R}; pub enum E {A(R,R),B} pub fn main() -> i32 {let e=E.A(R {value: 1},R {value: 2});match e {E.A(a,b)=>{let x=a;let y=b;},E.B=>{}};return 0;}",
            "use compiler_fixtures.record_4.{R}; pub enum E {A(R,R),B} pub fn main() -> i32 {let e=E.A(R {value: 1},R {value: 2});match e {E.A(a,b) if false=>{},E.A(a,b)=>{let x=a;let y=b;},E.B=>{}};return 0;}",
            "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_149.{P}; pub fn main() -> i32 {let p=P {value: 9, a: R {value: 1}, b: R {value: 2}};return 0;}",
            "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_150.{P}; pub fn main() -> i32 {let p=P {value: 9, a: R {value: 1}, b: R {value: 2}};return 0;}",
            "use compiler_fixtures.record_151.{R}; pub fn main() -> i32 {var a=R {value: 0};a=R {value: 2};return 0;}",
            "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_152.{P}; pub fn main() -> i32 {var p=P {a: R {value: 1}, b: R {value: 2}};let other=R {value: 9};let ref var a=p.a;*a=R {value: 3};return 0;}",
            "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_107.{F}; pub fn main() -> i32 {let live=R {value: 3};let a=F {value: 0};let b=F {value: 0};return 0;}",
            "use compiler_fixtures.record_4.{R}; pub fn main() -> i32 {let values={R {value: 1},R {value: 2}};let i:usize=3;let x=values[i];return 0;}",
            "use compiler_fixtures.record_145.{R}; pub fn main() -> i32 {let a=R<i32> {value: 1};let b=R<i64> {value: 2};return 0;}",
            "use compiler_fixtures.record_153.{R}; pub fn main() -> i32 {var a=R {value: 1};let b=R {value: 2};a=R {value: 3};return 0;}",
            "use compiler_fixtures.record_154.{R}; pub fn main() -> i32 {var a=R {value: 1};let b=R {value: 2};a.replace();return 0;}",
            "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_155.{P}; pub fn main() -> i32 {let p=P {b:R {value: 2},a:R {value: 1},c:1/0};return 0;}",
            "use compiler_fixtures.record_4.{R}; pub fn main() -> i32 {var n=0;while n<3 {let x=R {value: n};n+=1;if n==2 {continue;};if n==3 {break;}};return 0;}",
            "use compiler_fixtures.record_4.{R}; pub fn id(x:R) -> R {return x;} pub fn main() -> i32 {let t=R {value: 7};let x=id(t);return 0;}",
            "use compiler_fixtures.record_4.{R}; pub enum E {A(R,R),B} pub fn main() -> i32 {let x=E.A(R {value: 1},R {value: 2});let y={R {value: 3},R {value: 4}};return 0;}",
            "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_156.{F}; pub fn make() -> R {let fail=F {value: 0};return R {value: 7};} pub fn main() -> i32 {let x=make();return 0;}",
            "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_152.{P}; pub fn main() -> i32 {let p=P {a: R {value: 1}, b: R {value: 2}};let x=p.a;return 0;}",
        ];
    let mut rust = String::new();
    let mut expected = Vec::new();
    for (i, source) in cases.iter().enumerate() {
        let p = analyzed(source).unwrap();
        let (result, events) = execute_cleanup(source);
        expected.push((result.map_err(|e| e.kind), events));
        let emitted = crate::rust::emit(&p, clust_semantics::OverflowChecks::Checked).unwrap();
        let emitted = emitted.replace("fn main()", "pub fn run()");
        let emitted = emitted
            .lines()
            .map(|line| {
                if line == "// class-allocation" {
                    "eprintln!(\"ALLOC:{:p}\",heap);"
                } else if line == "// class-deallocation" {
                    "eprintln!(\"FREE:{:p}\",heap);"
                } else if line.starts_with("// class-destructor-entry ") {
                    "eprintln!(\"DROP:{}\",(*(*heap).data.as_ptr()).f0);"
                } else if line.starts_with("// destructor-entry ") {
                    "eprintln!(\"DROP:{}\",(*p).f0);"
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        rust.push_str(&format!("mod case{i} {{\n{emitted}\n}}\n"));
    }
    rust.push_str(
        "fn main() {match std::env::args().nth(1).unwrap().parse::<usize>().unwrap() {\n",
    );
    for i in 0..cases.len() {
        rust.push_str(&format!("{i}=>case{i}::run(),\n"));
    }
    rust.push_str("_=>panic!(),}}\n");
    let directory =
        std::env::temp_dir().join(format!("clust-resource-native-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let source = directory.join("cases.rs");
    fs::write(&source, rust).unwrap();
    for optimization in ["0", "3"] {
        let binary = directory.join(format!("cases-{optimization}"));
        let output = Command::new(std::env::var("CLUST_RUSTC").unwrap_or_else(|_| "rustc".into()))
            .arg("--edition=2021")
            .args(["-C", &format!("opt-level={optimization}")])
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        for (i, (expected_result, events)) in expected.iter().enumerate() {
            let output = Command::new(&binary).arg(i.to_string()).output().unwrap();
            if let Ok(value) = expected_result {
                let code = match value {
                    Value::Integer(n) => n.bits() as i32 & 255,
                    _ => 0,
                };
                assert_eq!(
                    output.status.code(),
                    Some(code),
                    "case {i}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            if let Err(kind) = expected_result {
                if *kind == crate::interpret::TrapKind::DoublePanic {
                    assert!(output.status.code().is_none(), "double panic must abort");
                } else {
                    assert_eq!(output.status.code(), Some(101));
                }
            }
            let mut allocations = 0;
            let mut live = std::collections::BTreeSet::new();
            for line in String::from_utf8_lossy(&output.stderr).lines() {
                if let Some(address) = line.strip_prefix("ALLOC:") {
                    allocations += 1;
                    assert!(
                        live.insert(address.to_owned()),
                        "allocation identity reused while live: case {i}"
                    );
                } else if let Some(address) = line.strip_prefix("FREE:") {
                    assert!(
                        live.remove(address),
                        "double free or unknown identity: case {i}"
                    );
                }
            }
            if cases[i].contains("let identity_probe=") {
                assert_eq!(
                    allocations, 1,
                    "identity comparison must not copy its operands"
                );
            }
            if cases[i].contains("pub fn relay(") {
                assert_eq!(allocations, 1, "move must preserve allocation identity");
            }
            if cases[i].contains("let a=C.new(1);var b=a;") {
                assert_eq!(allocations, 2, "Copy must create exactly one new identity");
            }
            if !matches!(
                expected_result,
                Err(crate::interpret::TrapKind::DoublePanic)
            ) {
                assert!(
                    live.is_empty(),
                    "leaked class allocation: case {i}: {live:?}"
                );
            }
            let trace = String::from_utf8_lossy(&output.stderr)
                .lines()
                .filter_map(|line| line.strip_prefix("DROP:"))
                .map(|n| n.parse::<i128>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(&trace, events, "case {i}, O{optimization}");
        }
    }
    fs::remove_dir_all(directory).unwrap();
}
