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
    ("pub interface Read {read(self)->i32;} pub class A:Read {new() {} pub read(self)->i32 {return 3;}} pub class B:Read {new() {} pub read(self)->i32 {return 4;}} pub fn sum(a:Read,b:Read)->i32 {return a.read()+b.read();} pub fn main()->i32 {let a=A.new();let b=B.new();return sum(a,b);}",7),
    ("pub interface Count {add(var self);read(self)->i32;} pub class C:Count {pub value:i32;new() {self.value=6;} pub add(var self) {self.value+=1;} pub read(self)->i32 {return self.value;}} pub fn advance(var c:Count)->i32 {c.add();return c.read();} pub fn main()->i32 {var c=C.new();return advance(c);}",7),
    ("pub interface Get<T> {get(self)->T;} pub class C:Get<i32> {new() {} pub get(self)->i32 {return 7;}} pub fn read(c:Get<i32>)->i32 {return c.get();} pub fn main()->i32 {let c=C.new();return read(c);}",7),
    ("pub interface Read {read(self)->i32;twice(self)->i32 {return self.read()+self.read();}} pub class C:Read {pub value:i32;new(n:i32) {self.value=n;} pub read(self)->i32 {return self.value;}} pub fn main()->i32 {let c=C.new(4);return c.twice();}",8),
    ("pub interface Read {read(self)->i32 {return 1;}} pub class C:Read {new() {} pub read(self)->i32 {return 7;}} pub fn main()->i32 {let c=C.new();return c.read();}",7),
    ("pub interface Identity<T> {identity(self,x:T)->T {let value:T=x;return value;}} pub class C:Identity<i32> {new() {}} pub fn main()->i32 {let c=C.new();return c.identity(7);}",7),
    ("pub interface First<const N:usize> {first(self,x:{i32;N})->i32 {let values:{i32;N}=x;return values[0];}} pub class C:First<2> {new() {}} pub fn main()->i32 {let c=C.new();return c.first({7,8});}",7),
    ("pub interface Identity {identity<T:Copy>(self,x:T)->T {let value:T=x;return value;}} pub class C:Identity {new() {}} pub fn main()->i32 {let c=C.new();return c.identity<i32>(7);}",7),
    ("pub interface Counter {add(var self,n:i32);read(self)->i32;advance(var self)->i32 {self.add(2);return self.read();}} pub class C:Counter {pub value:i32;new() {self.value=5;} pub add(var self,n:i32) {self.value+=n;} pub read(self)->i32 {return self.value;}} pub fn main()->i32 {var c=C.new();return c.advance();}",7),
    ("pub interface Read { read(self) -> i32} use compiler_fixtures.record_42.{S}; pub fn read<T:Read>(x:T) -> i32 {return x.read();} pub fn main() -> i32 {return read(S {value: 7});}",7),
    ("pub interface Read { read(self) -> i32} pub interface Other { other(self) -> i32} use compiler_fixtures.record_43.{S}; pub fn sum<T:Read & Other>(x:T) -> i32 {return x.read()+x.other();} pub fn main() -> i32 {return sum(S {});}",7),
    ("pub interface Read { read(self) -> i32} use compiler_fixtures.record_44.{S}; pub fn sum<T:Read & Copy>(x:T) -> i32 {let y=x;return x.read()+y.read();} pub fn main() -> i32 {return sum(S {value: 4});}",8),
    ("pub interface Read { read(self) -> i32} pub interface Bump { bump(var self) -> ()} use compiler_fixtures.record_45.{S}; pub fn bump<T:Read & Bump>(var x:T) -> i32 {x.bump();return x.read();} pub fn main() -> i32 {var s=S {value: 3};let n=bump(s);return n+s.read();}",8),
    ("pub interface Map<T,R> { map(self,x:T) -> R} use compiler_fixtures.record_46.{S}; pub fn map<T:Map<i32,bool>>(x:T) -> bool {return x.map(7);} pub fn main() -> i32 {return if map(S {}) {7} else {0};}",7),
    ("pub interface Read { read(self) -> i32} use compiler_fixtures.record_42.{S}; use compiler_fixtures.record_47.{Holder}; pub fn main() -> i32 {let h=Holder<S> {value: S {value: 7}};return h.read();}",7),
    ("pub interface Read { read(self) -> i32} pub class S:Read {pub value:i32;new(n:i32) {value=n;};pub read(self) -> i32 {return value;}} pub fn read<T:Read>(x:T) -> i32 {return x.read();} pub fn main() -> i32 {let s=S.new(7);return read(s)+s.read();}",14),
    ("pub interface Read { read(self) -> i32} use compiler_fixtures.record_42.{S}; pub fn read<T:Read>(x:T) -> i32 {return x.read();} pub fn relay<U:Read>(x:U) -> i32 {return read(x);} pub fn main() -> i32 {return relay(S {value: 7});}",7),
    ("pub interface Read { read(self) -> i32} use compiler_fixtures.record_48.{S}; pub fn read<T:Read>(x:T,n:i32) -> i32 {if n==0 {return x.read();};return read(x,n-1);} pub fn main() -> i32 {return read(S {},3);}",7),
    ("pub interface Identity { id<T:Copy>(self,x:T) -> T} use compiler_fixtures.record_49.{S}; pub fn read<T:Identity>(x:T) -> i32 {return x.id(7);} pub fn main() -> i32 {return read(S {});}",7),
    ("pub interface First<const N:usize> { first(self,xs:{i32;N}) -> i32} use compiler_fixtures.record_50.{S}; pub fn read<T:First<2>>(x:T) -> i32 {return x.first({7,8});} pub fn main() -> i32 {return read(S {});}",7),
    ("pub interface Factory { make() -> i32} use compiler_fixtures.record_51.{S}; pub fn make<T:Factory>() -> i32 {return T.make();} pub fn main() -> i32 {return make<S>();}",7),
    ("pub interface Get<T> { get(self) -> T} use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_52.{S}; pub fn get<T:Get<R>>(x:T) -> i32 {return x.get().value;} pub fn main() -> i32 {let s=S {value: R {value: 7}};return s.value.value+get(s);}",14),
    ("pub interface Read { read(self) -> i32} use compiler_fixtures.record_53.{S}; pub fn read<T:Read>(x:T) -> i32 {return x.read();} pub fn main() -> i32 {return read(S<i32> {value: 3});}",7),
    ("pub interface Read { read(self) -> i32} use compiler_fixtures.record_48.{S}; pub fn read<T:Read=S>(x:T) -> i32 {return x.read();} pub fn main() -> i32 {return read(S {});}",7),
    ("pub interface Read { read(self) -> i32} use compiler_fixtures.record_48.{S}; pub fn read<T:Read & Read>(x:T) -> i32 {return x.read();} pub fn main() -> i32 {return read(S {});}",7),
    ("pub interface Get<T=i32> { get(self) -> T} use compiler_fixtures.record_54.{S}; pub fn get<T:Get>(x:T) -> i32 {return x.get();} pub fn main() -> i32 {return get(S {});}",7),
    ("pub interface Identity { id<const N:usize>(self,x:{i32;N}) -> i32} use compiler_fixtures.record_55.{S}; pub fn read<T:Identity>(x:T) -> i32 {return x.id<2>({7,8});} pub fn main() -> i32 {return read(S {});}",7),
    ("pub interface Factory { make() -> i32} use compiler_fixtures.record_56.{S}; pub fn make<T:Factory>() -> i32 {let f:()=>i32=T.make;return f();} pub fn main() -> i32 {return make<S<i32>>();}",7),
    ("pub interface I { read(self) -> i32} pub interface J { read(self) -> i32} use compiler_fixtures.record_57.{S}; pub fn read<T:I & J>(x:T) -> i32 {return x.read();} pub fn main() -> i32 {return read(S {});}",7),
    ("pub interface Get<T> { get(self) -> T} use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_52.{S}; use compiler_fixtures.record_58.{Adapter}; pub fn main() -> i32 {let s=Adapter<S> {value: S {value: R {value: 7}}};return s.get().value+s.value.value.value;}",14),
    ("pub interface Read { read(self) -> i32} use compiler_fixtures.record_48.{S}; use compiler_fixtures.record_59.{H}; pub interface Use<T> { apply(self,value:T) -> i32} use compiler_fixtures.record_60.{U}; pub fn read<T:Use<H<V>>,V:Read>(x:T,value:V) -> i32 {return x.apply(H<V> {value: value});} pub fn main() -> i32 {return read(U {},S {});}",7),
    ("pub interface Read { read(self) -> i32} use compiler_fixtures.record_48.{S}; use compiler_fixtures.record_59.{H}; pub interface Use<T> { apply(self,value:T) -> i32} use compiler_fixtures.record_60.{U}; pub interface Run { run<T:Use<H<V>>,V:Read>(self,x:T,value:V) -> i32} use compiler_fixtures.record_61.{Runner}; pub fn run<T:Run>(x:T) -> i32 {return x.run(U {},S {});} pub fn main() -> i32 {return run(Runner {});}",7),
];

#[path = "interfaces/default_methods_require_unambiguous.rs"]
mod default_methods_require_unambiguous;
