use super::*;

#[test]
fn default_methods_require_unambiguous_implementations_and_only_interface_members() {
    for source in [
        "pub interface A {read(self)->i32 {return 1;}} pub interface B {read(self)->i32 {return 2;}} pub class C:A,B {new() {}} pub fn main()->i32 {return 0;}",
        "pub interface Read {read(self)->i32 {return self.value;}} pub class C:Read {pub value:i32;new() {self.value=7;}} pub fn main()->i32 {let c=C.new();return c.read();}",
        "pub interface Read {read(self)->i32 {return self.hidden();}} pub class C:Read {new() {} pub hidden(self)->i32 {return 7;}} pub fn main()->i32 {let c=C.new();return c.read();}",
        "pub interface Broken {read(self)->i32 {return true;}} pub fn main()->i32 {return 0;}",
    ] {
        assert!(checked(source).is_err(), "accepted {source}");
    }
}

#[test]
fn nominal_interfaces_dispatch_statically_and_preserve_borrowed_contracts() {
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
fn rejects_missing_or_incompatible_nominal_implementations() {
    for source in [
        "pub interface I { get<T=T>(self,x:T) -> T} pub fn main() -> () {}",
        "pub interface I { get<T=U,U=i32>(self,x:T) -> T} pub fn main() -> () {}",
        "pub interface I<T=Missing> {} pub fn main() -> () {}",
        "pub interface I<T=T> {} pub fn main() -> () {}",
        "pub interface I<T=U,U=i32> {} pub fn main() -> () {}",
        "pub interface I { read(self) -> Missing} pub fn main() -> () {}",
        "pub interface I<T:Missing> {} pub fn main() -> () {}",
        "pub interface I { read<const N:usize>(self,xs:{i32;N}) -> i32} use compiler_fixtures.record_62.{S}; pub fn main() -> () {}",
        "pub interface I { read<T=i32>(self,x:T) -> T} use compiler_fixtures.record_63.{S}; pub fn main() -> () {}",
        "pub interface I { read(self) -> i32} use compiler_fixtures.record_64.{S}; pub fn main() -> () {}",
        "pub interface I { read(self) -> i32} use compiler_fixtures.record_65.{S}; pub fn main() -> () {}",
        "pub interface I { read(self) -> i32} use compiler_fixtures.record_66.{S}; pub fn main() -> () {}",
        "pub interface I { read(var self) -> i32} use compiler_fixtures.record_67.{S}; pub fn main() -> () {}",
        "pub interface I { read(self) -> i32} use compiler_fixtures.record_68.{S}; pub fn main() -> () {}",
        "pub interface I { read(self) -> i32} use compiler_fixtures.record_69.{S}; pub fn main() -> () {}",
        "pub interface I { read(self,var x:i32) -> i32} use compiler_fixtures.record_70.{S}; pub fn main() -> () {}",
        "pub interface I { read(self) -> i32} use compiler_fixtures.record_71.{S}; pub fn read<T:I>(x:T) -> i32 {return x.read();} pub fn main() -> i32 {return read(S {});}",
        "pub interface I { read(self) -> i32} use compiler_fixtures.record_67.{S}; pub fn read<T:I & Copy>(x:T) -> i32 {return x.read();} pub fn main() -> i32 {return read(S {});}",
        "pub interface I { read(self) -> i32} pub interface J { other(self) -> i32} use compiler_fixtures.record_67.{S}; pub fn read<T:I & J>(x:T) -> i32 {return x.read();} pub fn main() -> i32 {return read(S {});}",
        "pub interface I<T> { get(self) -> T} use compiler_fixtures.record_72.{S}; pub fn main() -> () {}",
        "pub interface I { read<T:Copy>(self,x:T) -> T} use compiler_fixtures.record_73.{S}; pub fn main() -> () {}",
        "pub interface I { read(self) -> i32; read(self) -> i32} pub fn main() -> () {}",
        "pub interface I {} use compiler_fixtures.record_74.{I}; pub fn main() -> () {}",
        "pub interface I {} use compiler_fixtures.record_75.{S}; pub fn main() -> () {}",
        "pub interface I {} use compiler_fixtures.record_76.{S}; pub fn main() -> () {}",
        "pub interface I { bump(var self) -> ()} use compiler_fixtures.record_77.{S}; pub fn bump<T:I>(x:T) -> () {x.bump();} pub fn main() -> () {bump(S {});}",
        "pub interface I { read(self) -> i32} use compiler_fixtures.record_78.{S}; pub fn read<T:I>(x:T) -> i32 {return x.extra();} pub fn main() -> i32 {return read(S {});}",
        "use compiler_fixtures.record_71.{S}; pub fn read<T>(x:T) -> i32 {return x.read();} pub fn main() -> i32 {return read(S {});}",
        "pub interface I { read(self) -> i32} use compiler_fixtures.record_79.{S}; pub fn needs_copy<T:Copy>(x:T) -> T {return x;} pub fn copy<T:I>(x:T) -> T {return needs_copy(x);} pub fn main() -> i32 {return copy(S {}).read();}",
    ] {
        assert!(checked(source).is_err(),"accepted {source}");
    }
}

#[test]
fn native_interface_calls_match_reference_results_and_drop_order() {
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
    let dir = std::env::temp_dir().join(format!("clust-interfaces-{}", std::process::id()));
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
