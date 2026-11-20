use clust_core::{check::check, hir, interpret, mir};
use clust_frontend::{
    source::{Diagnostic, SourceId},
    testing::parse,
};
use clust_semantics::OverflowChecks;
fn checked(source: &str) -> Result<hir::Program, Vec<Diagnostic>> {
    let parsed = parse(SourceId(0), source);
    assert!(
        parsed.diagnostics.is_empty(),
        "{source}: {:?}",
        parsed.diagnostics
    );
    check(&parsed.module.unwrap())
}
const CELL: &str = "use compiler_fixtures.record_5.{Cell};";
const CASES:&[(&str,i128)]=&[
    ("pub enum Link {End,Next(Node)} pub class Node {pub value:i32;pub next:Link;new(n:i32,link:Link) {value=n;next=link;};pub tail(self) -> Node {return match next {Link.End=>self,Link.Next(n)=>n.tail()};}} pub fn main() -> i32 {let root=Node.new(1,Link.Next(Node.new(2,Link.End)));let last=root.tail();return root.value+last.value;}",3),
    ("pub fn pick(flag:bool,x:Cell,y:Cell) -> Cell {return match flag {true=>x,false=>y};} pub fn main() -> i32 {let a=Cell {value: 3};let b=Cell {value: 7};let c=pick(false,a,b);return a.value+b.value+c.value;}",17),
    ("use compiler_fixtures.record_26.{Pair}; pub fn main() -> i32 {var a=Pair {a: Cell {value: 3}, b: Cell {value: 7}};let ref var b=a.left();a.b.value+=1;b.value+=2;return a.a.value+a.b.value;}",13),
    ("pub class C {pub value:i32;new(n:i32) {value=n;}} pub class Pair {pub a:C;pub b:C;new() {a=C.new(3);b=C.new(7);};pub left(self) -> C {return a;}} pub fn main() -> i32 {let a=Pair.new();let b=a.left();return if same_identity(a.a,b) {b.value+a.b.value} else {0};}",10),
    ("pub enum E {Some(Cell),None} use compiler_fixtures.record_27.{Holder}; pub fn main() -> i32 {let h=Holder {value: E.Some(Cell {value: 6})};let b=h.get();return b.value;}",6),
    ("pub fn id<T>(x:T) -> T {return x;} use compiler_fixtures.record_28.{Box}; pub fn main() -> i32 {let b=Box<Cell> {value: Cell {value: 8}};let c=b.get();return b.value.value+c.value;}",16),
    ("use compiler_fixtures.record_29.{Pair}; pub fn main() -> i32 {var a=Pair {a: Cell {value: 3}, b: Cell {value: 7}};let b=a.get();return a.b.value+b.value;}",10),
    ("pub fn fresh(x:Cell) -> Cell {return Cell {value:x.value+1};} pub fn main() -> i32 {var a=Cell {value: 3};let b=fresh(a);a.value+=1;return a.value+b.value;}",8),
    ("pub fn id(x:Cell) -> Cell {return x;} pub fn main() -> i32 {let a=Cell {value: 7};let b=id(a);return a.value+b.value;}",14),
    ("pub fn id<T>(x:T) -> T {return x;} pub fn relay<T>(x:T) -> T {return id(x);} pub fn main() -> i32 {let a=Cell {value: 7};let b=relay(a);return a.value+b.value;}",14),
    ("pub fn id(x:Cell) -> Cell {return x;} pub fn relay(x:Cell) -> Cell {let y=id(x);let z=y;return z;} pub fn main() -> i32 {let a=Cell {value: 7};let b=relay(a);return a.value+b.value;}",14),
    ("pub fn id(var x:Cell) -> Cell {return x;} pub fn main() -> i32 {var a=Cell {value: 7};let ref var b=id(a);b.value+=2;return a.value;}",9),
    ("use compiler_fixtures.record_15.{Pair}; pub fn left(x:Pair) -> Cell {return x.a;} pub fn main() -> i32 {let a=Pair {a: Cell {value: 3}, b: Cell {value: 7}};let b=left(a);return a.b.value+b.value;}",10),
    ("pub fn pick(n:i32,x:Cell) -> Cell {if n==0 {return x;};return pick(n-1,x);} pub fn main() -> i32 {let a=Cell {value: 7};let b=pick(3,a);return a.value+b.value;}",14),
    ("pub fn a(n:i32,x:Cell) -> Cell {if n==0 {return x;};return b(n-1,x);} pub fn b(n:i32,x:Cell) -> Cell {return a(n,x);} pub fn main() -> i32 {let x=Cell {value: 7};let y=a(3,x);return x.value+y.value;}",14),
    ("pub fn pick(flag:bool,x:Cell,y:Cell) -> Cell {if flag {return x;};return y;} pub fn main() -> i32 {let a=Cell {value: 3};let b=Cell {value: 7};let c=pick(false,a,b);return a.value+b.value+c.value;}",17),
    ("pub fn pick(flag:bool,x:Cell,y:Cell) -> Cell {return if flag {x} else {y};} pub fn main() -> i32 {let a=Cell {value: 3};let b=Cell {value: 7};let c=pick(false,a,b);return a.value+b.value+c.value;}",17),
    ("pub fn get(xs:{Cell;2},i:usize) -> Cell {return xs[i];} pub fn main() -> i32 {let xs={Cell {value: 3},Cell {value: 7}};let c=get(xs,1);return xs[0].value+c.value;}",10),
    ("pub class C {pub value:i32;new(n:i32) {value=n;};pub get(self) -> C {return self;}} pub fn main() -> i32 {let a=C.new(7);let b=a.get();return if same_identity(a,b) {b.value} else {0};}",7),
];
#[test]
fn borrowed_results_preserve_origins_and_capabilities() {
    for (body, expected) in CASES {
        let source = format!("{CELL} {body}");
        let p = checked(&source).unwrap_or_else(|e| panic!("{source}: {e:?}"));
        let m = mir::lower(&p);
        let a = interpret::execute(
            &p,
            p.main.unwrap(),
            vec![],
            OverflowChecks::Checked,
            100_000,
        );
        let b = mir::interpret::execute(
            &m,
            m.main.unwrap(),
            vec![],
            OverflowChecks::Checked,
            100_000,
        );
        assert_eq!(a, b, "{source}");
        assert!(
            matches!(a,Ok(hir::Value::Integer(n)) if n.signed_value()==Some(*expected)),
            "{source}: {a:?}"
        );
    }
}
#[test]
fn borrowed_results_cannot_escape_or_gain_ownership() {
    for body in [
        "pub fn id(x:Cell) -> Cell {return x;} pub fn main() -> i32 {let a=Cell {value: 1};var b=Cell {value: 2};b=id(a);return a.value+b.value;}",
        "use compiler_fixtures.record_29.{Pair}; pub fn main() -> i32 {var a=Pair {a: Cell {value: 3}, b: Cell {value: 7}};let ref var b=a.get();b.value=4;return a.b.value;}",
        "use compiler_fixtures.record_30.{Holder}; pub fn main() -> i32 {let b=(Holder {a: Cell {value: 1}}).get();return b.value;}",
        "pub fn pick(flag:bool,x:Cell) -> Cell {if flag {return x;};return Cell {value: 2};} pub fn main() -> i32 {let a=Cell {value: 1};let b=pick(true,a);return a.value+b.value;}",
        "pub fn bad() -> Cell {let a=Cell {value: 1};let ref b=a;return b;} pub fn main() -> i32 {return bad().value;}",
        "pub fn id(x:Cell) -> Cell {return x;} pub fn main() -> i32 {var a=Cell {value: 1};let ref v=a;let b=id(v);a=Cell {value: 2};return b.value;}",
        "pub fn id(x:Cell) -> Cell {return x;} pub fn main() -> i32 {var a=Cell {value: 1};let b=id(a);let ref var c=b;c.value=2;return a.value;}",
        "pub fn id(var x:Cell) -> Cell {return x;} pub fn main() -> i32 {var a=Cell {value: 1};let ref var x=a;let b=id(x);x.value=2;return b.value;}",
        "pub fn id(x:Cell) -> Cell {return x;} pub fn bad(x:Cell) -> Cell {let ref b=x;return id(b);} pub fn main() -> i32 {return bad(Cell {value: 1}).value;}",
    ] {
        let source=format!("{CELL} {body}");
        assert!(checked(&source).is_err(),"accepted {source}");
    }
}
#[test]
fn generated_borrowed_results_run_at_both_optimization_levels() {
    use std::{fs, process::Command};
    let mut rust = String::new();
    for (i, (body, _)) in CASES.iter().enumerate() {
        let program = checked(&format!("{CELL} {body}")).unwrap();
        let emitted = clust_core::rust::emit(&program, OverflowChecks::Checked)
            .unwrap()
            .replace("fn main()", "pub fn run()");
        rust.push_str(&format!("mod case{i} {{\n{emitted}\n}}\n"));
    }
    rust.push_str(
        "fn main() {match std::env::args().nth(1).unwrap().parse::<usize>().unwrap() {\n",
    );
    for i in 0..CASES.len() {
        rust.push_str(&format!("{i}=>case{i}::run(),\n"));
    }
    rust.push_str("_=>panic!(),}}\n");
    let directory =
        std::env::temp_dir().join(format!("clust-borrowed-native-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let source = directory.join("cases.rs");
    fs::write(&source, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.join(format!("cases-{level}"));
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
        for (i, (_, expected)) in CASES.iter().enumerate() {
            let result = Command::new(&binary).arg(i.to_string()).output().unwrap();
            assert_eq!(
                result.status.code(),
                Some(*expected as i32),
                "case {i} O{level}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn escape_and_readonly_errors_name_the_origin() {
    let errors = checked(
        "use compiler_fixtures.record_31.{R}; pub fn bad() -> R {let x=R {value: 1};let ref v=x;return v;} pub fn main() -> () {}",
    )
    .unwrap_err();
    let error = errors.iter().find(|e| e.code == "E0395").unwrap();
    assert!(
        error
            .notes
            .iter()
            .any(|note| note.message.contains("owner ends its lifetime")),
        "{error:?}"
    );
    let errors=checked("use compiler_fixtures.record_32.{R}; pub fn main() -> i32 {var r=R {value: 1};let ref var v=r.get();v.value=2;return r.value;}").unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.notes.iter().any(|note| note.message.contains("readonly"))),
        "{errors:?}"
    );
}
