use clust_core::{check::check, hir, interpret, mir};
use clust_frontend::{source::SourceId, testing::parse};
use clust_semantics::OverflowChecks;

fn checked(source: &str) -> Result<hir::Program, Vec<clust_frontend::source::Diagnostic>> {
    let parsed = parse(SourceId(0), source);
    assert!(
        parsed.diagnostics.is_empty(),
        "{source}: {:?}",
        parsed.diagnostics
    );
    check(&parsed.module.unwrap())
}
const CASES:&[(&str,i32)]=&[
    ("pub class Counter {pub value:i32;new(n:i32) {self.value=n;};pub read(self)->i32 {return self.value;};pub bump(var self) {self.value+=1;};drop() {self.value+=10;}} pub fn main()->i32 {var c=Counter.new(6);c.bump();return c.read();}",7),
    ("pub class Cell<T:Copy> {pub value:T;new(n:T) {self.value=n;};pub read(self)->T {return self.value;};drop() {let n=self.value;}} pub fn main()->i32 {let c=Cell<i32>.new(7);return c.read();}",7),
    ("pub class C {new() {};hidden(self)->i32 {return 7;};pub read(self)->i32 {return self.hidden();};drop() {}} pub fn main()->i32 {let c=C.new();return c.read();}",7),
    ("pub class A {pub value:i32;new(n:i32) {value=n;}; pub read(self) -> i32 {return value;}} pub fn main() -> i32 {let a=A.new(7);return a.read();}",7),
    ("pub class A<T:Copy> {pub value:T;new(n:T) {value=n;}; pub read(self) -> T {return value;}} pub fn main() -> i32 {let a=A<i32>.new(7);let f:(A<i32>)=>i32=A<i32>.read;return f(a)+a.read();}",14),
    ("pub class A {pub value:i32;new(n:i32) {value=n;}; pub bump(var self) -> () {value+=1;};pub read(self) -> i32 {return value;}} pub fn main() -> i32 {var a=A.new(7);a.bump();return a.read();}",8),
    ("pub class A {new() {}  pub identity<T:Copy>(self,value:T) -> T {return value;}} pub fn main() -> i32 {let a=A.new();let f:(A,i32)=>i32=A.identity<i32>;return f(a,7);}",7),
    ("pub interface Read { read(self) -> i32} pub class A:Read {new() {}  pub read(self) -> i32 {return 7;}} pub fn read<T:Read>(x:T) -> i32 {return x.read();} pub fn main() -> i32 {return read(A.new());}",7),
    ("pub class A {pub value:i32;new(n:i32) {value=n;}; pub bump(var self) -> () {value+=1;};drop() {bump();}} pub fn main() -> i32 {let a=A.new(7);return a.value;}",7),
    ("use compiler_fixtures.record_4.{R}; pub class A {pub value:R;new(n:R) {value=n;}; pub get(self) -> R {return value;}} pub fn main() -> i32 {let a=A.new(R {value: 7});return a.get().value;}",7),
];
#[test]
fn clust_private_methods_and_destructor_contracts_are_enforced() {
    for (source, code) in [
        ("pub class C {new() {};hidden(self)->i32 {return 1;}} pub fn main()->i32 {let c=C.new();return c.hidden();}", "E0352"),
        ("pub class C {new() {};drop() {};drop() {}} pub fn main()->i32 {return 0;}", "E0390"),
    ] {
        let errors = checked(source).unwrap_err();
        assert_eq!(errors[0].code, code, "{source}: {errors:?}");
    }
}

#[test]
fn clust_drop_runs_before_fields_in_declaration_order() {
    let source = "pub class First {new() {} drop() {}} pub class Second {new() {} drop() {}} pub class Owner {pub first:First;pub second:Second;new() {self.second=Second.new();self.first=First.new();} drop() {}} pub fn main()->i32 {let owner=Owner.new();return 0;}";
    let program = checked(source).unwrap();
    let (result, events) = interpret::execute_traced(
        &program,
        program.main.unwrap(),
        vec![],
        OverflowChecks::Checked,
        100_000,
    );
    assert!(result.is_ok(), "{result:?}");
    let owners: Vec<_> = events
        .iter()
        .map(|event| {
            program.functions[event.entry]
                .name
                .split(".$drop")
                .next()
                .unwrap()
        })
        .collect();
    assert_eq!(owners, ["Owner", "First", "Second"]);
    let lowered = mir::lower(&program);
    let other = mir::interpret::execute_traced(
        &lowered,
        lowered.main.unwrap(),
        vec![],
        OverflowChecks::Checked,
        100_000,
    );
    assert_eq!((result, events), other);
}
#[test]
fn known_concrete_virtual_methods_keep_static_entry_contracts() {
    for (source, expected) in CASES {
        let p = checked(source).unwrap_or_else(|e| panic!("{source}: {e:?}"));
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
        assert_eq!(a, b, "{source}");
        assert!(
            matches!(a.0,Ok(hir::Value::Integer(n)) if n.signed_value()==Some(*expected as i128)),
            "{source}: {a:?}"
        );
        let rust = clust_core::rust::emit(&p, OverflowChecks::Checked).unwrap();
        assert!(!rust.contains("dyn "));
    }
}
#[test]
fn invalid_inheritance_is_diagnosed_before_the_lowering_boundary() {
    for (source,code) in [
        ("pub class A:A {new() {} }", "E0410"),
        ("pub class A<T>:B<T> {new() {} } pub class B<U>:A<U> {new() {} }", "E0410"),
        ("pub class A {new() {} } pub class B {new() {} } pub class C:A,B {new() {} }", "E0410"),
        ("use compiler_fixtures.record_95.{A}; pub class B:A {new() {} }", "E0410"),
        ("pub class A {new() {} pub read(self) -> i32 {return 1;}} pub class B:A {new() {super.new();}  pub read(self) -> bool {return false;}}", "E0411"),
        ("pub class A {new() {}  pub read(self) -> i32 {return 1;}} pub class B:A {new() {}  pub read(var self) -> i32 {return 2;}}", "E0411"),
        ("pub class A {new() {}  pub read<T:Copy>(self,x:T) -> T {return x;}} pub class B:A {new() {}  pub read<U>(self,x:U) -> U {return x;}}", "E0411"),
    ] {
        let errors=checked(source).unwrap_err();
        assert_eq!(errors[0].code,code,"{source}: {errors:?}");
        if code=="E0411" {assert!(!errors[0].notes.is_empty(),"{errors:?}");}
    }
    // A derived constructor must initialize its base.
    let errors=checked("pub class A {new() {}  pub read(self) -> i32 {return 1;}} pub class B:A {new() {}  pub read(self) -> i32 {return 2;}} pub fn main() -> i32 {return B.new().read();}").unwrap_err();
    assert_eq!(errors[0].code, "E0393");
}
#[test]
fn native_static_virtual_entries_preserve_results_and_cleanup() {
    use std::{fs, process::Command};
    let mut rust = String::new();
    let mut traces = Vec::new();
    for (i, (source, _)) in CASES.iter().enumerate() {
        let p = checked(source).unwrap_or_else(|e| panic!("{source}: {e:?}"));
        let (_, events) = interpret::execute_traced(
            &p,
            p.main.unwrap(),
            vec![],
            OverflowChecks::Checked,
            100_000,
        );
        traces.push(
            events
                .into_iter()
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
    let dir = std::env::temp_dir().join(format!("clust-inheritance-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let source = dir.join("cases.rs");
    fs::write(&source, rust).unwrap();
    for level in ["0", "3"] {
        let binary = dir.join(format!("cases-{level}"));
        let output = Command::new(std::env::var("CLUST_RUSTC").unwrap_or_else(|_| "rustc".into()))
            .arg("--edition=2021")
            .args(["-C", &format!("opt-level={level}")])
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
        for (i, (_, expected)) in CASES.iter().enumerate() {
            let output = Command::new(&binary).arg(i.to_string()).output().unwrap();
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_eq!(
                output.status.code(),
                Some(*expected),
                "case {i} O{level}: {stderr}"
            );
            assert_eq!(
                stderr
                    .lines()
                    .filter(|s| s.starts_with("DROP:"))
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
                traces[i],
                "case {i} O{level}"
            );
        }
    }
    fs::remove_dir_all(dir).unwrap();
}
