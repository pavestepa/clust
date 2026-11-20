use super::*;
use clust_frontend::{source::SourceId, testing::parse};
fn module(source: &str) -> a::Module {
    let parsed = parse(SourceId(0), source);
    assert!(
        parsed.diagnostics.is_empty(),
        "{source}: {:?}",
        parsed.diagnostics
    );
    parsed.module.unwrap()
}
#[test]
fn clust_methods_infer_overrides_across_generic_base_chains() {
    for source in [
        "pub class Animal {new() {} pub speak(self)->i32 {return 1;}} pub class Cat:Animal {new() {} pub speak(self)->i32 {return 2;}} pub class Kitten:Cat {new() {} pub speak(self)->i32 {return 3;}}",
        "pub class Base<T> {new() {} pub read(self,x:T)->T {return x;}} pub class Middle<U>:Base<U> {new() {} } pub class Leaf:Middle<i32> {new() {} pub read(self,x:i32)->i32 {return x;}}",
        "pub class Base {new() {} pub read<T:Copy>(self,x:T)->T {return x;}} pub class Leaf:Base {new() {} pub read<U:Copy>(self,x:U)->U {return x;}}",
        "pub class Base {new() {} read(self)->i32 {return 1;}} pub class Leaf:Base {new() {} pub read(self)->bool {return true;}}",
        "pub class Base {new() {} pub bump(var self) {}} pub class Leaf:Base {new() {} pub bump(var self) {}}",
    ] {
        validate(&module(source)).unwrap_or_else(|e| panic!("{source}: {e:?}"));
    }
}

#[test]
fn clust_overrides_preserve_receiver_signature_and_visibility() {
    for source in [
        "pub class Base {new() {} pub read(self)->i32 {return 1;}} pub class Leaf:Base {new() {} pub read(var self)->i32 {return 2;}}",
        "pub class Base {new() {} pub read(self)->i32 {return 1;}} pub class Leaf:Base {new() {} pub read(self)->bool {return true;}}",
        "pub class Base {new() {} pub read(self)->i32 {return 1;}} pub class Leaf:Base {new() {} read(self)->i32 {return 2;}}",
        "pub class Base {new() {} pub read(self)->i32 {return 1;}} pub class Leaf:Base {new() {} pub read()->i32 {return 2;}}",
        "pub class Base {new() {} pub read<T:Copy>(self,x:T)->T {return x;}} pub class Leaf:Base {new() {} pub read<U>(self,x:U)->U {return x;}}",
        "pub class Base {new() {} pub read(self,var x:i32)->i32 {return x;}} pub class Leaf:Base {new() {} pub read(self,x:i32)->i32 {return x;}}",
    ] {
        let error = validate(&module(source)).unwrap_err();
        assert_eq!(error.code, "E0411", "{source}: {error:?}");
        assert!(!error.notes.is_empty(), "{source}: {error:?}");
    }
}
#[test]
fn valid_hierarchies_substitute_generics_and_keep_override_chains() {
    for source in [
        "const N:usize=2;pub class A<const M:usize=N> {new() {}  pub read(self,x:{i32;M}) -> i32 {return 1;}} pub class B:A {new() {}  pub read(self,x:{i32;2}) -> i32 {return 2;}}",
        "const N:usize=1;pub class A<const N:usize> {new() {}  pub read(self,x:{i32;N}) -> i32 {return 1;}} pub class B<const N:usize>:A<N> {new() {}  pub read(self,x:{i32;N}) -> i32 {return 2;}}",
        "pub class A<T> {new() {}  pub read<U:Copy>(self,x:T,y:U) -> T {return x;}} pub class B<U>:A<U> {new() {}  pub read<V:Copy>(self,x:U,y:V) -> U {return x;}}",
        "pub class A<const N:usize> {new() {}  pub read<const M:usize>(self,x:{i32;N},y:{i32;M}) -> i32 {return 1;}} pub class B<const M:usize>:A<M> {new() {}  pub read<const K:usize>(self,x:{i32;M},y:{i32;K}) -> i32 {return 2;}}",
        "pub class A {new() {}  pub read(self) -> i32 {return 1;}} pub class B:A {new() {}  pub read(self) -> i32 {return 2;}} pub class C:B {new() {}  pub read(self) -> i32 {return 3;}}",
        "pub class C:B {new() {} } pub class B:A {new() {} } pub class A {new() {} }",
        "pub interface I { read(self) -> i32} pub class A {new() {}  pub read(self) -> i32 {return 1;}} pub class B:A,I {new() {}  pub read(self) -> i32 {return 2;}}",
        "pub class A<T> {new() {}  pub read(self,x:T) -> T {return x;}} pub class B<U>:A<U> {new() {}  pub read(self,x:U) -> U {return x;}} pub class C:B<i32> {new() {}  pub read(self,x:i32) -> i32 {return x;}}",
        "pub class A<T,U=T> {new() {}  pub read(self,x:U) -> T {return x;}} pub class B:A<i32> {new() {}  pub read(self,x:i32) -> i32 {return x;}}",
        "pub class A<T,U> {new() {}  pub read(self,x:T,y:U) -> T {return x;}} pub class B<T,U>:A<U,T> {new() {}  pub read(self,x:U,y:T) -> U {return x;}}",
        "pub class A<const N:usize> {new() {}  pub read(self,x:{i32;N}) -> i32 {return 1;}} pub class B<const M:usize>:A<M> {new() {}  pub read(self,x:{i32;M}) -> i32 {return 2;}}",
        "const N:usize=2;pub class A<const M:usize> {new() {}  pub read(self,x:{i32;M}) -> i32 {return 1;}} pub class B:A<N> {new() {}  pub read(self,x:{i32;2}) -> i32 {return 2;}}",
        "pub class A<const N:usize=2> {new() {}  pub read(self,x:{i32;N}) -> i32 {return 1;}} pub class B:A {new() {}  pub read(self,x:{i32;1+1}) -> i32 {return 2;}}",
        "pub class A<T> {new() {}  pub read<U:Copy>(self,x:T,y:U) -> U {return y;}} pub class B:A<i32> {new() {}  pub read<V:Copy>(self,x:i32,y:V) -> V {return y;}}",
        "pub class A {new() {}  pub first<const N:usize>(self,x:{i32;N}) -> i32 {return 1;}} pub class B:A {new() {}  pub first<const M:usize>(self,x:{i32;M}) -> i32 {return 2;}}",
        "pub interface I {} pub interface J {} pub class A {new() {}  pub read<T:I & J>(self,x:T) -> i32 {return 1;}} pub class B:A {new() {}  pub read<U:J & I>(self,x:U) -> i32 {return 2;}}",
        "pub class A {new() {}  field:i32;  read(self) -> i32 {return 1;}} pub class B:A {new() {} pub field:i32;pub read(self) -> i32 {return 2;}}",
        "pub class A {new() {}  pub read(self) -> i32 {return 1;}} pub class B:A {new() {} } pub class C:B {new() {}  pub read(self) -> i32 {return 2;}}",
        "use compiler_fixtures.record_157.{A};",
        "pub class A {new() {}  pub read() -> i32 {return 1;}}",
        "pub class A {new() {}    read(self) -> i32 {return 1;}}",
        "pub class A {new() {}  pub read(self) -> i32 {return 1;}}",
        "pub class A {new() {} pub read(self) -> i32 {return 1;}} pub class B:A {new() {}  pub read(self) -> i32 {return 2;}}",
        "pub class A {new() {}  pub read(self) -> i32 {return 1;}} pub class B:A {new() {} pub read(self) -> i32 {return 2;}}",
        "pub class A {new() {}  pub read(self) -> i32 {return 1;}} pub class B:A {new() {}  pub read(self) -> i32 {return 2;}}",
        "pub class A {new() {}   read(self) -> i32 {return 1;}} pub class B:A {new() {}  pub read(self) -> i32 {return 2;}}",
    ] {validate(&module(source)).unwrap_or_else(|e|panic!("{source}: {e:?}"));}
}
#[test]
fn invalid_hierarchies_fail_before_layout_or_instantiation() {
    for source in [
        "const N:usize=2;pub class A<const M:usize=N> {new() {}  pub read(self,x:{i32;M}) -> i32 {return 1;}} pub class B<const N:usize>:A {new() {}  pub read(self,x:{i32;N}) -> i32 {return 2;}}",
        "const N:usize=2;pub class A<const M:usize> {new() {}  pub read(self,x:{i32;M}) -> i32 {return 1;}} pub class B:A<N> {new() {} } pub class C<const N:usize>:B {new() {}  pub read(self,x:{i32;N}) -> i32 {return 2;}}",
        "const N:usize=1;pub class A<const N:usize> {new() {}  pub read(self,x:{i32;N}) -> i32 {return 1;}} pub class B<const N:usize>:A<N> {new() {}  pub read(self,x:{i32;N*N}) -> i32 {return 2;}}",
        "const N:usize=2;pub class A {new() {}  pub read(self,x:{i32;N}) -> i32 {return 1;}} pub class B<const N:usize>:A {new() {}  pub read(self,x:{i32;N}) -> i32 {return 2;}}",
        "pub class A:A {new() {} }",
        "pub class A<T>:B<T> {new() {} } pub class B<U>:A<U> {new() {} }",
        "pub class A:B {new() {} } pub class B:C {new() {} } pub class C:A {new() {} }",
        "pub class A {new() {} } pub class B {new() {} } pub class C:A,B {new() {} }",
        "pub interface I {} pub class A {new() {} } pub class B:I,A {new() {} }",
        "use compiler_fixtures.record_95.{A}; pub class B:A {new() {} }",
        "pub class A {new() {} } use compiler_fixtures.record_158.{B};",
        "pub class B:Missing {new() {} }",
        "pub class A<T> {new() {} } pub class B:A {new() {} }",
        "pub class A {new() {} } pub class B:A<i32> {new() {} }",
        "pub class A<T> {new() {} } pub class B:A<2> {new() {} }",
        "pub class A<const N:usize> {new() {} } pub class B:A<{i32;2}> {new() {}}",
        "pub class A {new() {}  pub read(self) -> i32 {return 1;}} pub class B:A {new() {}  pub read(var self) -> i32 {return 2;}}",
        "pub class A {new() {}  pub read(self,var x:i32) -> i32 {return x;}} pub class B:A {new() {}  pub read(self,x:i32) -> i32 {return x;}}",
        "pub class A {new() {}  pub read(self) -> i32 {return 1;}} pub class B:A {new() {}  pub read(self) -> bool {return true;}}",
        "pub class A {new() {}  pub read(self) -> i32 {return 1;}} pub class B:A {new() {}  pub read() -> i32 {return 2;}}",
        "pub class A {new() {}  pub read<T:Copy>(self,x:T) -> T {return x;}} pub class B:A {new() {}  pub read<U>(self,x:U) -> U {return x;}}",
        "pub class A {new() {}  pub read<T=i32>(self,x:T) -> T {return x;}} pub class B:A {new() {}  pub read<U=bool>(self,x:U) -> U {return x;}}",
        "pub class A {new() {}  pub read<const N:usize>(self,x:{i32;N}) -> i32 {return 1;}} pub class B:A {new() {}  pub read<const M:usize>(self,x:{i32;M*M}) -> i32 {return 2;}}",
        "pub class A {new() {} pub value:i32} pub class B:A {new() {} pub value:i32}",
        "pub class A {new() {} pub value:i32} pub class B:A {new() {} pub value(self) -> i32 {return 1;}}",
        "pub class A {new() {} pub value(self) -> i32 {return 1;}} pub class B:A {new() {} pub value:i32}",
        "pub class A {new() {} pub read(self) -> i32 {return 1;};pub read(self) -> i32 {return 2;}}",
    ] {assert!(validate(&module(source)).is_err(),"accepted {source}");}
}
