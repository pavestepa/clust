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
const CASES: &[(&str, i32)] = &[
    ("pub class C { value:i32;new(n:i32) {value=n;};  secret(self) -> i32 {return value;};pub read(self) -> i32 {return secret();};drop() {value+=1;}} pub fn main() -> i32 {let c=C.new(7);return c.read();}", 7),
    ("pub class C { value:i32;new(n:i32) {value=n;};pub read_other(self,other:C) -> i32 {return other.value;}} pub fn main() -> i32 {let a=C.new(3);let b=C.new(7);return a.read_other(b);}", 7),
    ("use compiler_fixtures.record_109.{S}; pub fn main() -> i32 {return S.make(7).read();}", 7),
    ("use compiler_fixtures.record_110.{S}; pub fn main() -> i32 {return S.make(7).read();}", 7),
    ("use compiler_fixtures.record_111.{S}; pub fn main() -> i32 {let f=S.expose();return f();}", 7),
    ("pub class C<T:Copy> { value:T;new(n:T) {value=n;};  secret(self) -> T {return value;};pub read(self) -> T {return secret();};drop() {let n=value;}} pub fn main() -> i32 {let a=C<i32>.new(3);let b=C<i64>.new(4);return a.read()+b.read() as i32;}", 7),
    ("use compiler_fixtures.record_112.{S}; pub fn main() -> i32 {let s=S<i32>.make(3);return s.read()+S<i32>.pass<i32>(4);}", 7),
    ("pub class C { value:i32;new(n:i32) {value=n;};pub bump(var self) -> () {let ref var alias=self;alias.value+=1;};pub read(self) -> i32 {return value;}} pub fn main() -> i32 {var c=C.new(6);c.bump();return c.read();}", 7),
    (" fn helper() -> i32 {return 7;} use compiler_fixtures.record_113.{S}; pub fn main() -> i32 {return S.read();}", 7),
    ("use compiler_fixtures.record_4.{R}; pub class C { value:R;new(n:R) {value=n;};pub get(self) -> R {return value;};drop() {let n=value.value;}} pub fn main() -> i32 {let c=C.new(R {value: 7});return c.get().value;}", 7),
    ("use compiler_fixtures.record_114.{S}; pub fn main() -> i32 {let s=S.make();let S {visible:n,..}=s;return n;}", 7),
    ("pub class C<T:Copy> {new() {}   secret<U:Copy>(x:U) -> U {return x;};pub expose() -> (i32)=>i32 {return C<T>.secret<i32>;}} pub fn main() -> i32 {let f=C<i32>.expose();return f(7);}",7),
    ("pub class C<T:Copy> {new() {}   secret<U:Copy>(x:U) -> U {return x;};pub expose() -> (i32)=>i32 {return C<T>.secret<i32>;}} pub fn relay<T:Copy>(x:T) -> i32 {let f=C<T>.expose();return f(7);} pub fn main() -> i32 {return relay(1);}",7),
];
#[test]
fn declaring_types_keep_private_access_in_specialized_entries() {
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
fn private_members_are_not_module_visible_or_granted_by_callers() {
    for source in [
        "pub class C { value:i32;new() {value=7;}} pub fn main() -> i32 {let c=C.new();return c.value;}",
        "pub class C { value:i32;new() {value=7;}} pub class D {new() {} pub read(c:C) -> i32 {return c.value;}} pub fn main() -> i32 {return D.read(C.new());}",
        "pub class C {new() {}   secret() -> i32 {return 7;}} pub fn main() -> i32 {return C.secret();}",
        "pub class C {new() {}   secret(self) -> i32 {return 7;}} pub fn main() -> i32 {return C.new().secret();}",
        "pub class C {new() {}   secret(self) -> i32 {return 7;}} pub fn main() -> i32 {let f=C.secret;return f(C.new());}",
        "pub class C {new() {}   secret() -> i32 {return 7;}} pub fn main() -> i32 {let f=C.secret;return f();}",
        "pub class C<T:Copy> { value:T;new(n:T) {value=n;}} pub fn read<T:Copy>(c:C<T>) -> T {return c.value;} pub fn main() -> i32 {return read(C<i32>.new(7));}",
        "pub class C<T:Copy> {new() {}   secret<U:Copy>(x:U) -> U {return x;}} pub fn main() -> i32 {return C<i32>.secret<i32>(7);}",
        "pub class C<T:Copy> {new() {}   secret<U:Copy>(x:U) -> U {return x;}} pub fn main() -> i32 {let f:(i32)=>i32=C<i32>.secret<i32>;return f(7);}",
        "pub class C { value:i32;new() {value=7;};pub call(self) -> i32 {return helper(self);}} pub fn helper(c:C) -> i32 {return c.value;} pub fn main() -> i32 {return C.new().call();}",
        "use compiler_fixtures.record_115.{S}; pub fn main() -> i32 {let s=S {value: 7};return 7;}",
        "use compiler_fixtures.record_116.{S}; pub fn main() -> i32 {let s=S.make();let S {value:n}=s;return n;}",
        "pub class C { value:i32;new() {value=7;}} pub fn main() -> i32 {var c=C.new();let ref var alias=c;alias.value=3;return 7;}",
        "use compiler_fixtures.record_116.{S}; use compiler_fixtures.record_117.{D}; pub fn main() -> i32 {return D.read(S.make());}",
] {
        let errors = checked(source).expect_err(source);
        assert!(errors.iter().any(|d| d.code == "E0352" && !d.notes.is_empty()), "{source}: {errors:?}");
    }
}
#[test]
fn native_private_entries_preserve_results_and_drop_order() {
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
    let dir = std::env::temp_dir().join(format!("clust-privacy-{}", std::process::id()));
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
