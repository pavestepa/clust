use clust_core::{check::check, hir, interpret, mir};
use clust_frontend::{
    source::{Diagnostic, SourceId},
    testing::parse,
};
use clust_semantics::OverflowChecks;
fn checked(source: &str) -> Result<hir::Program, Vec<Diagnostic>> {
    let parsed = parse(SourceId(0), source);
    if !parsed.diagnostics.is_empty() {
        return Err(parsed.diagnostics);
    }
    check(&parsed.module.unwrap())
}
#[test]
fn constructor_initialization_follows_typed_paths_and_aliases() {
    for body in [
        "if flag {value=1} else {value=2}",
        "match flag {true=>{value=1},false=>{value=2}}",
        "let ref var alias=self;alias.value=3;",
        "if flag {self.value=1;} else {self.value=2;}",
        "value=1;while flag {value+=1;break;}",
        "value=1;let h=(self,);let read=h.0.value;",
    ] {
        let source=format!("pub class C {{pub value:i32;new(flag:bool) {{{body};}}}} pub fn main() -> i32 {{let c=C.new(true);return c.value;}}");
        let p = checked(&source).unwrap_or_else(|e| panic!("{source}: {e:?}"));
        let m = mir::lower(&p);
        assert_eq!(
            interpret::execute(&p, p.main.unwrap(), vec![], OverflowChecks::Checked, 10_000),
            mir::interpret::execute(&m, m.main.unwrap(), vec![], OverflowChecks::Checked, 10_000),
            "{source}"
        );
    }
    for body in [
        "let read=value;value=1;",
        "value+=1;",
        "if flag {value=1}",
        "while flag {value=1;break;}",
        "if flag {self.value=1;}",
        "let value=1;",
        "let ref var alias=self;let read=alias.value;value=1;",
        "self.read();value=1;",
        "let ref alias=self;consume(alias);value=1;",
        "let h=(self,);let read=h.0.value;value=1;",
        "let h=(self,);h.0.read();value=1;",
        "let h=(self,);consume(h.0);value=1;",
        "let h=(self,);consume_wrapper(h);value=1;",
    ] {
        let source=format!("pub class C {{pub value:i32;new(flag:bool) {{{body};}};pub read(self) -> i32 {{return value;}}}} pub fn consume(x:C) -> i32 {{return x.value;}} pub fn consume_wrapper(x:(C,)) -> i32 {{return x.0.value;}} pub fn main() -> i32 {{let c=C.new(true);return c.value;}}");
        let e = checked(&source).unwrap_err();
        assert!(e.iter().any(|d| d.code == "E0393"), "{source}: {e:?}");
    }
}
#[test]
fn class_copy_and_view_contracts_are_checked() {
    for source in [
        "#[Copy] pub class C {new() {} drop() {}}",
        "pub class C {new() {};new() {}}",
        "pub class C {new() {} pub value:i32} pub fn main() -> () {let c=C.new();}",
        "pub class C {pub value:i32;new(n:i32) {value=n;}} pub fn main() -> () {var a=C.new(1);let ref var v=a;*v=C.new(2);}",
        "pub class C {pub value:i32;new(n:i32) {value=n;};pub bad(self) -> () {value=2;}}",
        "pub class C {pub value:i32;new(n:i32) {value=n;}} pub fn main() -> i32 {var a=C.new(1);let ref v=a;a=C.new(2);return v.value;}",
        "use compiler_fixtures.record_31.{R}; use compiler_fixtures.record_118.{W}; pub class C {pub r:R;new() {r=R {value: 1};};drop() {}} pub fn main() -> i32 {let c=C.new();let w=W {r: c.r};return w.r.value;}",
        "pub class R {new() {} } #[Copy] pub class C {new() {} pub r:R}",
        "use compiler_fixtures.record_31.{R}; pub fn main() -> () {let r=R.new();}",
        "pub class C {new() {} } pub fn main() -> () {let a=C.new();let x:i32=a==a;}",
        "pub class C {new() {} } pub fn main() -> () {let a=C.new();let x:i32=same_identity(a,a);}",
        "pub class C {new() {} } pub class D {new() {} } pub fn main() -> () {let a=C.new();let b=D.new();let x=a==b;}",
        "pub class C {new() {} } pub fn main() -> () {let a=C.new();let x=a<a;}",
        "pub class C {pub value:i32;new() {let same=self==self;value=1;}}",
        "pub class C {pub value:i32;new() {let same=same_identity(self,self);value=1;}}",
        "pub class C {pub value:i32;new() {value=1;}} pub fn main() -> () {let c=C {value:1};}",
    ] {
        assert!(checked(source).is_err(),"accepted {source}");
    }
}
#[test]
fn generic_class_arguments_infer_and_const_payloads_are_separate() {
    let source="pub class Cell<T> {pub value:T;new(v:T) {value=v;}} pub fn read<T:Copy>(x:Cell<T>) -> T {return x.value;} pub class Array<const N:usize> {pub values:{i32;N};new(v:{i32;N}) {values=v;}} pub fn main() -> i32 {let c=Cell<i32>.new(3);let a=Array<2>.new({4,5});return read(c)+a.values[1];}";
    let p = checked(source).unwrap();
    let m = mir::lower(&p);
    let result = interpret::execute(&p, p.main.unwrap(), vec![], OverflowChecks::Checked, 10_000);
    assert_eq!(
        result,
        mir::interpret::execute(&m, m.main.unwrap(), vec![], OverflowChecks::Checked, 10_000)
    );
    assert!(matches!(result,Ok(hir::Value::Integer(n)) if n.bits()==8));
}
