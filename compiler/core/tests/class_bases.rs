use clust_core::{check::check_clust, hir, interpret, mir};
use clust_frontend::{parser::parse_clust, source::SourceId};
use clust_semantics::OverflowChecks;

fn checked(source: &str) -> Result<hir::Program, Vec<clust_frontend::source::Diagnostic>> {
    let parsed = parse_clust(SourceId(0), source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    check_clust(&parsed.module.unwrap())
}

const CASES: &[(&str, i128)] = &[
    ("class A<T:Copy> {pub value:T;new(value:T) {self.value=value;} pub read(self)->T {return self.value;}} class B<U:Copy>:A<U> {new(value:U) {super.new(value);}} class C<V:Copy>:B<V> {new(value:V) {super.new(value);} pub read(self)->V {return super.read();}} fn make()->A<i32> {return C<i32>.new(3);} fn main()->i32 {var a=make();a.value=6;let b=C<u8>.new(2);return a.read()+b.read() as i32;}",8),
    ("class A<const N:usize> {pub values:{i32;N};new(values:{i32;N}) {self.values=values;} pub first(self)->i32 {return self.values[0];}} class B<const M:usize>:A<M> {new(values:{i32;M}) {super.new(values);}} fn make()->A<2> {return B<2>.new({3,4});} fn main()->i32 {let value=make();return value.first();}",3),
    ("class Animal {pub age:i32;new() {self.age=3;} pub read(self)->i32 {return self.age;} pub twice(self)->i32 {return self.read()+self.read();} pub add(var self) {self.age+=1;}} class Cat:Animal {pub color:i32;new() {super.new();self.color=7;} pub read(self)->i32 {return super.read()+self.color;}} fn main()->i32 {var cat=Cat.new();cat.add();return cat.twice();}",22),
    ("class A {value:i32;new() {self.value=3;} hidden(self)->i32 {return self.value;} pub read(self)->i32 {return self.hidden();}} class B:A {value:i32;new() {super.new();self.value=9;} hidden(self)->i32 {return self.value;}} fn main()->i32 {let b=B.new();return b.read();}",3),
    ("class A {pub value:i32;new(n:i32) {self.value=n;} pub read(self)->i32 {return self.value;}} class B:A {new() {super.new(4);} pub read(self)->i32 {return super.read()+2;}} class C:B {new() {super.new();} pub read(self)->i32 {return super.read()+1;}} fn main()->i32 {let c=C.new();return c.read()+c.value;}",11),
    ("class Animal {\n    pub age:i32\n    new() {self.age=3;}\n    pub read(self)->i32 {return self.age;}\n    pub twice(self)->i32 {return self.read()+self.read();}\n    pub add(var self) {self.age+=1;}\n}\nclass Cat:Animal {\n    pub color:i32\n    new() {super.new();self.color=7;}\n    pub read(self)->i32 {return super.read()+self.color;}\n}\nfn listen(animal:Animal)->i32 {return animal.twice();}\nfn main()->i32 {\n    var cat=Cat.new();\n    cat.add();\n    let n=listen(cat);return n+cat.read();\n}\n",33),
    ("class Animal {\n    pub age:i32\n    new() {self.age=3;}\n    pub read(self)->i32 {return self.age;}\n    pub twice(self)->i32 {return self.read()+self.read();}\n    pub add(var self) {self.age+=1;}\n}\nclass Cat:Animal {\n    pub color:i32\n    new() {super.new();self.color=7;}\n    pub read(self)->i32 {return super.read()+self.color;}\n}\nfn listen(var animal:Animal)->i32 {animal.add();return animal.twice();}\nfn main()->i32 {\n    var cat=Cat.new();\n    cat.add();\n    let n=listen(cat);return n+cat.read();\n}\n",36),
    ("class Animal {\n    pub age:i32\n    new() {self.age=3;}\n    pub read(self)->i32 {return self.age;}\n    pub twice(self)->i32 {return self.read()+self.read();}\n    pub add(var self) {self.age+=1;}\n}\nclass Cat:Animal {\n    pub color:i32\n    new() {super.new();self.color=7;}\n    pub read(self)->i32 {return super.read()+self.color;}\n}\nfn listen(animal:Animal)->i32 {return animal.twice();}\nfn make()->Animal {return Cat.new();}\nfn main()->i32 {\n    var cat=make();\n    cat.add();\n    return listen(cat);\n}\n",22),
    ("class Animal {\n    pub age:i32\n    new() {self.age=3;}\n    pub read(self)->i32 {return self.age;}\n    pub twice(self)->i32 {return self.read()+self.read();}\n    pub add(var self) {self.age+=1;}\n}\nclass Cat:Animal {\n    pub color:i32\n    new() {super.new();self.color=7;}\n    pub read(self)->i32 {return super.read()+self.color;}\n}\nfn listen(animal:Animal)->i32 {return animal.twice();}\nclass Home {pub animal:Animal;new(a:Animal) {self.animal=a;}}\nfn main()->i32 {\n    let h=Home.new(Cat.new());return h.animal.twice();\n}\n",20),
    ("class Animal {\n    pub age:i32\n    new() {self.age=3;}\n    pub read(self)->i32 {return self.age;}\n    pub twice(self)->i32 {return self.read()+self.read();}\n    pub add(var self) {self.age+=1;}\n}\nclass Cat:Animal {\n    pub color:i32\n    new() {super.new();self.color=7;}\n    pub read(self)->i32 {return super.read()+self.color;}\n}\nfn listen(animal:Animal)->i32 {return animal.twice();}\nfn make()->Animal {return Cat.new();}\nfn main()->i32 {\n    var cat=make();\n    cat.age += 2;\n    return cat.age+listen(cat);\n}\n",29),
    ("interface Read {read(self)->i32;twice(self)->i32 {return self.read()+self.read();}} class A:Read {new() {} pub read(self)->i32 {return 3;}} class B:Read {new() {} pub read(self)->i32 {return 4;}} fn make(first:bool)->dyn Read {return if first {A.new()} else {B.new()};} fn main()->i32 {let a=make(true);let b=make(false);return a.twice()+b.twice();}",14),

];

#[test]
fn concrete_inheritance_preserves_fields_lexical_access_and_dispatch() {
    for (source, expected) in CASES {
        let p = checked(source).unwrap_or_else(|errors| panic!("{source}: {errors:?}"));
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
            matches!(a.0,Ok(hir::Value::Integer(n)) if n.signed_value()==Some(*expected)),
            "{a:?}"
        );
        clust_core::rust::emit(&p, OverflowChecks::Checked).unwrap();
    }
}

#[test]
fn inherited_private_fields_remain_private() {
    for body in [
        "fn main()->i32 {let b=B.new();return b.value;}",
        "fn main()->i32 {let b=B.new();return b.bad();}",
    ] {
        let extra = if body.contains("bad") {
            "pub bad(self)->i32 {return self.value;}"
        } else {
            ""
        };
        let source=format!("class A {{value:i32;new() {{self.value=3;}}}} class B:A {{new() {{super.new();}} {extra}}} {body}");
        let errors = checked(&source).expect_err("private base field leaked");
        assert!(errors.iter().any(|e| e.code == "E0352"), "{errors:?}");
    }
}

#[test]
fn generic_dispatch_preserves_bounds_and_reports_unsupported_base_mappings() {
    for (source, code) in [
        ("interface Read<T:Copy> {read(self)->T;} class Item {new() {}} fn invalid(value:dyn Read<Item>) {} fn main()->i32 {return 0;}", "E0373"),
        ("class A<T,U> {new() {}} class B<T,U>:A<U,T> {new() {super.new();}} fn main()->i32 {return 0;}", "E0412"),
    ] {
        let errors = checked(source).expect_err("invalid generic declaration accepted");
        assert!(errors.iter().any(|error| error.code == code), "{errors:?}");
    }
}

#[test]
fn closure_receiver_effect_comes_from_its_class() {
    let program = "class A {new() {} pub read(self)->i32 {return 7;}} class Derived:A {new() {super.new();}} fn main()->i32 {let a=Derived.new();let f=fn()->i32 {return a.read();};return f();}";
    checked(program).unwrap();
    checked(&format!(
        "{program} class Unrelated {{new() {{}} pub read(var self)->i32 {{return 0;}}}}"
    ))
    .unwrap();
}

#[test]
fn mutable_method_capture_keeps_its_receiver_effect() {
    let program = "class Counter {pub n:i32;new() {self.n=0;} pub step(var self) {self.n+=1;}} class Other {new() {} pub step(self) {}} fn main()->i32 {var counter=Counter.new();var update=fn() {counter.step();};update();return counter.n;}";
    let program = checked(program).unwrap();
    let result = interpret::execute(
        &program,
        program.main.unwrap(),
        vec![],
        OverflowChecks::Checked,
        10000,
    )
    .unwrap();
    assert!(matches!(result, hir::Value::Integer(n) if n.signed_value() == Some(1)));
}
