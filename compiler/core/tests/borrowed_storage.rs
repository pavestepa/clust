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
const PREFIX: &str =
    "use compiler_fixtures.record_4.{R}; use compiler_fixtures.record_96.{Holder}; ";
const CASES:&[(&str,i128)]=&[
 ("pub fn flag() -> bool {return false;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;let h=if flag() {Holder {value:a}} else {Holder {value:w}};return h.value.value;}",3),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;let h=match 1 {0=>Holder {value:a},_=>Holder {value:w}};return h.value.value;}",3),
 ("pub fn wrap(_borrowed_x:R) -> Holder {let x=_borrowed_x;return Holder {value:x};} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=wrap(a);h.value=w;return h.value.value;}",3),

 ("pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=Holder {value:a};set(h,w);return h.value.value;}",3),
 ("pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;let ref var alias=h;alias.value=x;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=Holder {value:a};set(h,w);return h.value.value;}",3),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=Holder {value:a};let ref var alias=h;alias.value=w;return h.value.value;}",3),

 ("pub class C {pub value:R;new(x:R) {value=x;}} pub fn flag() -> bool {return true;} pub fn main() -> i32 {let x=R {value: 7};var c=C.new(R {value: 3});if flag() {c.value=x} else {return x.value;};return c.value.value;}",7),

 ("pub fn main() -> i32 {let a:u8=7;let xs={a,3};return xs[1] as i32;}",3),

 ("use compiler_fixtures.record_97.{Counter}; pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var count=Counter {value: 0};let xs={count.tap(a),w};return a.value+xs[0].value+xs[1].value+count.value;}",18),

 ("pub fn main() -> i32 {let a=R {value: 7};let xs={a,a};return xs[0].value+xs[1].value;}",14),
 ("use compiler_fixtures.record_98.{G}; pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;let xs={G<R> {value: a},G<R> {value: w}};return xs[0].value.value+xs[1].value.value;}",10),

 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=Holder {value:a};h.value=w;return h.value.value;}",3),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=(a,);h.0=w;return h.0.value;}",3),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var xs={a};xs[0]=w;return xs[0].value;}",3),
 ("use compiler_fixtures.record_99.{Outer}; pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var o=Outer {h: Holder {value:a}};o.h.value=w;return o.h.value.value;}",3),
 ("pub fn flag() -> bool {return false;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=Holder {value:a};if flag() {h.value=w};return h.value.value;}",7),
 ("use compiler_fixtures.record_98.{G}; pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=G<R> {value: a};h.value=w;return h.value.value;}",3),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=Holder {value:a};h=Holder {value:w};return h.value.value;}",3),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var first=Holder {value:a};var second=first;second.value=w;return second.value.value;}",3),
 ("pub fn id<T>(x:T) -> T {return x;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=Holder {value:a};h.value=w;let result=id(h);return result.value.value;}",3),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;let xs={a,w};return xs[0].value+xs[1].value;}",10),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;let xs={w,a};return xs[0].value+xs[1].value;}",10),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;let xs={Holder {value:a},Holder {value:w}};return xs[0].value.value+xs[1].value.value;}",10),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let c=R {value: 2};let ref w=c;var h=Holder {value:a};if true {var h=Holder {value:b};h.value=w};return h.value.value;}",7),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var xs={a,a};xs[1]=w;return xs[0].value+xs[1].value;}",10),

 ("pub enum E {Item(R),Empty} pub fn set(var e:E,_borrowed_x:R) -> () {let x=_borrowed_x;let ref var target=e;*target=E.Item(x);} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var e=E.Item(v);e=E.Empty;set(e,w);return match e {E.Item(x)=>x.value,E.Empty=>0};}",3),
 ("pub enum E {Item(R),Empty} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var e=E.Item(v);e=E.Empty;let ref var target=e;*target=E.Item(w);return match e {E.Item(x)=>x.value,E.Empty=>0};}",3),
 ("pub enum E {Item(R),Empty} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var e=E.Item(v);e=E.Empty;return match e {E.Item(x) if 1/0==0=>x.value,E.Item(x)=>x.value,E.Empty if false=>1/0,E.Empty=>3};}",3),
 ("pub enum E {Item(R),Empty} pub fn clear<T>(x:T) -> i32 {return 3;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var e=E.Item(v);e=E.Empty;let saved=match e {E.Item(x)=>x.value,E.Empty=>3};return saved+clear(0);}",6),

 ("pub enum E {Item(R),Empty} pub fn main() -> i32 {var a=R {value: 7};let ref v=a;var e=E.Item(v);e=E.Empty;a=R {value: 3};return match e {E.Item(x)=>x.value,E.Empty=>a.value};}",3),
 ("pub enum E {Item(R),Empty} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var e=E.Item(v);e=E.Empty;return match e {E.Item(x)=>x.value,E.Empty=>3};}",3),
 ("pub enum E {Item(R),Empty} pub fn unknown() -> bool {return true;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var e=E.Item(v);if unknown() {e=E.Empty} else {e=E.Empty};return match e {E.Item(x)=>x.value,E.Empty=>3};}",3),
 ("pub enum E {Item(R),Empty} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var e=E.Item(v);var n=0;while n<2 {e=E.Empty;n+=1;};e=E.Empty;return match e {E.Item(x)=>x.value,E.Empty=>3};}",3),
 ("pub enum E {Item(R),Empty} pub fn clear() -> i32 {let a=R {value: 7};let ref v=a;var current=E.Item(v);current=E.Empty;return match current {E.Item(x)=>x.value,E.Empty=>3};} pub fn main() -> i32 {return clear();}",3),
 ("pub fn get<T>(x:T) -> T {if false {return x;};return x;} pub fn main() -> i32 {let a=R {value: 7};let result=get(a);return a.value+result.value;}",14),
 ("pub fn get(_borrowed_x:R) -> R {let x=_borrowed_x;if false {let local=R {value: 3};return local;};return x;} pub fn main() -> i32 {let a=R {value: 7};let result=get(a);return a.value+result.value;}",14),
 ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var h=Holder {value:v};if false {let local=R {value: 3};h.value=local};return h.value.value;}",7),

 ("pub class C {pub r:R;new() {r=R {value: 3};}} pub fn set(var h:Holder,x:C) -> () {h.value=x.r;} pub fn main() -> i32 {let a=R {value: 7};let b=C.new();let ref v=a;let ref w=b;var h=Holder {value:v};set(h,w);return h.value.value;}",3),

 ("pub fn set(var h:Holder,xs:{R;2},i:usize) -> () {h.value=xs[i];} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};set(h,{v,w},1);return h.value.value;}",3),

 ("pub fn set(var h:Holder,_borrowed_x:R) -> Holder {let x=_borrowed_x;h.value=x;return Holder {value:h.value};} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};let result=set(h,w);return result.value.value+h.value.value;}",6),
 ("pub fn set(var h:Holder,s:Holder) -> () {h.value=s.value;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};set(h,Holder {value:w});return h.value.value;}",3),
 ("pub fn set(var h:Holder,_borrowed_x:R,n:i32) -> () {let x=_borrowed_x;if n==0 {h.value=x} else {relay(h,x,n-1)}} pub fn relay(var h:Holder,x:R,n:i32) -> () {set(h,x,n);} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};set(h,w,2);return h.value.value;}",3),
 ("pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};var n=0;while n<2 {set(h,w);n+=1;};return h.value.value;}",3),

 ("pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;var h=Holder {value:v};set(h,b);return b.value+h.value.value;}",6),
 ("use compiler_fixtures.record_98.{G}; pub fn set<T>(var h:G<T>,_borrowed_x:T) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=G<R> {value: v};set(h,w);return h.value.value;}",3),
 ("use compiler_fixtures.record_100.{S}; pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=S {value: v};h.set(w);return h.value.value;}",3),
 ("pub fn set(var h:Holder,_borrowed_x:R,flag:bool) -> () {let x=_borrowed_x;if flag {h.value=x} else {set(h,x,true)}} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};set(h,w,false);return h.value.value;}",3),
 ("pub fn set(var h:Holder,_borrowed_x:R,flag:bool) -> () {let x=_borrowed_x;if flag {h.value=x}} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};set(h,w,false);return h.value.value;}",7),
 ("pub fn set(var h:Holder,_borrowed_x:R) -> R {let x=_borrowed_x;h.value=x;return h.value;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};let result=set(h,w);return result.value+h.value.value;}",6),
 ("use compiler_fixtures.record_99.{Outer}; pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var o=Outer {h: Holder {value:v}};set(o.h,w);return o.h.value.value;}",3),
 ("pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;let ref var target=h;*target=Holder {value:x};} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};set(h,w);return h.value.value;}",3),

 ("pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};set(h,w);return h.value.value;}",3),
 ("pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};let old=h.value;set(h,w);return old.value+h.value.value;}",10),
 ("pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn relay(var h:Holder,x:R) -> () {set(h,x);} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};relay(h,w);return h.value.value;}",3),
 ("use compiler_fixtures.record_101.{A}; pub fn set(var a:A,_borrowed_x:R,i:usize) -> () {let x=_borrowed_x;a.xs[i]=x;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var holder=A {xs: {v,v}};set(holder,w,1);return holder.xs[0].value+holder.xs[1].value;}",10),

 ("pub fn id<T>(x:T) -> T {return x;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var h=(v,);let g=id(h);return g.0.value;}",7),
 ("pub fn id(h:Holder) -> Holder {return h;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=Holder {value:v};let outer=id(h);return h.value.value+outer.value.value;}",14),
 ("pub fn pair(_borrowed_x:R,_borrowed_y:R) -> (R,R) {let x=_borrowed_x;let y=_borrowed_y;return (x,y);} pub fn main() -> i32 {let a=R {value: 7};let ref va=a;let b=R {value: 3};let ref vb=b;let h=pair(va,vb);return a.value+b.value+h.0.value+h.1.value;}",20),
 ("pub class C {pub value:i32;new(n:i32) {value=n;};drop() {}} pub fn wrap(_borrowed_x:C) -> (C,) {let x=_borrowed_x;return (x,);} pub fn main() -> i32 {let a=C.new(7);let ref va=a;let h=wrap(va);return if same_identity(va,h.0) {h.0.value} else {0};}",7),
 ("pub fn set(var xs:{R;2},_borrowed_x:R,i:usize) -> {R;2} {let x=_borrowed_x;xs[i]=x;return xs;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref x=a;let ref y=b;var arr={x,x};let xs=set(arr,y,1);return xs[0].value+xs[1].value;}",10),
 ("pub fn wrap(_borrowed_x:R) -> Holder {let x=_borrowed_x;return Holder {value:x};} pub fn main() -> i32 {let a=R {value: 7};let ref va=a;let h=wrap(va);return a.value+h.value.value;}",14),
 ("pub fn wrap(_borrowed_x:R) -> Holder {let x=_borrowed_x;return Holder {value:x};} pub fn relay(x:R) -> Holder {return wrap(x);} pub fn main() -> i32 {let a=R {value: 7};let ref va=a;let h=relay(va);return a.value+h.value.value;}",14),
 ("pub fn id<T>(x:T) -> T {return x;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let t=(v,);let h=id(t);return h.0.value;}",7),
 ("pub fn wrap<T>(_borrowed_x:T) -> (T,) {let x=_borrowed_x;return (x,);} pub fn main() -> i32 {let a=R {value: 7};let ref va=a;let h=wrap(va);return a.value+h.0.value;}",14),
 ("pub fn wrap(_borrowed_x:R) -> {R;1} {let x=_borrowed_x;return {x};} pub fn main() -> i32 {let a=R {value: 7};let ref va=a;let h=wrap(va);return a.value+h[0].value;}",14),
 ("pub enum E {Item(R),Empty} pub fn wrap(_borrowed_x:R) -> E {let x=_borrowed_x;return E.Item(x);} pub fn main() -> i32 {let a=R {value: 7};let ref va=a;let e=wrap(va);return a.value+match e {E.Item(x)=>x.value,E.Empty=>0};}",14),
 ("pub fn wrap(flag:bool,_borrowed_x:R) -> Holder {let x=_borrowed_x;if flag {return Holder {value:x};};return wrap(true,x);} pub fn main() -> i32 {let a=R {value: 7};let ref va=a;let h=wrap(false,va);return a.value+h.value.value;}",14),
 ("use compiler_fixtures.record_102.{Mixed}; pub fn wrap(_borrowed_x:R) -> Mixed {let x=_borrowed_x;return Mixed {view: x, owned: R {value: 3}};} pub fn main() -> i32 {let a=R {value: 7};let ref va=a;let h=wrap(va);return a.value+h.view.value+h.owned.value;}",17),
 ("use compiler_fixtures.record_98.{G}; pub fn get<T>(h:G<T>) -> T {return h.value;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=G<R> {value: v};let result=get(h);return result.value;}",7),
 ("pub fn consume<T>(h:T) -> i32 {return 3;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=(v,);return consume(h)+h.0.value;}",10),
 ("use compiler_fixtures.record_103.{G}; pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=G<R> {value: v};let result=h.get();return result.value;}",7),
 ("pub fn get(h:Holder) -> R {return h.value;} pub fn relay(h:Holder) -> R {return get(h);} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=Holder {value:v};let result=relay(h);return result.value;}",7),
 ("use compiler_fixtures.record_99.{Outer}; pub fn get(o:Outer) -> R {return o.h.value;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=Holder {value:v};let ref inner=h;let o=Outer {h: inner};let result=get(o);return result.value;}",7),
 ("pub fn read(h:Holder) -> i32 {return h.value.value;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=Holder {value:v};return read(h);}",7),
 ("pub fn read(h:Holder) -> i32 {return h.value.value;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=Holder {value:v};return read(h)+h.value.value;}",14),
 ("pub fn get(h:Holder) -> R {return h.value;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=Holder {value:v};let result=get(h);return result.value;}",7),
 ("pub fn get(h:Holder) -> R {return h.value;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=Holder {value:v};let result=get(h);return h.value.value+result.value;}",14),
 ("use compiler_fixtures.record_104.{S}; pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=S {value: v};let result=h.get();return h.read()+result.value;}",14),
 ("pub fn get(xs:{R;2},i:usize) -> R {return xs[i];} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let arr={v,v};let result=get(arr,1);return result.value;}",7),
 ("pub enum E {Item(R),Empty} pub fn read(e:E) -> i32 {return match e {E.Item(x)=>x.value,E.Empty=>0};} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;return read(E.Item(v));}",7),
 ("pub enum E {Item(R),Empty} pub fn main() -> i32 {let a=R {value: 7};let e=E.Item(a);return a.value+match e {E.Item(x)=>x.value,E.Empty=>0};}",14),
 ("pub enum E {Item {view:R,owned:R},Empty} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let e=E.Item {owned:R {value: 3},view:v};return match e {E.Item {view:x,owned:y}=>x.value+y.value,E.Empty=>0};}",10),
 ("pub enum E {Item(Holder)} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let e=E.Item(Holder {value:v});let moved=e;return match moved {E.Item(h)=>h.value.value};}",7),
 ("pub enum E {Item(R),Empty} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let e=if true {E.Item(v)} else {E.Empty};return match e {E.Item(x)=>x.value,E.Empty=>3};}",7),
 ("pub fn get(_borrowed_x:R) -> R {let x=_borrowed_x;let h=(x,);return h.0;} pub fn main() -> i32 {let a=R {value: 7};let ref va=a;let result=get(va);return a.value+result.value;}",14),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;var h=Holder {value:v};let old=h.value;let ref var container=h;container.value=b;return old.value+container.value.value;}",10),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref x=a;let ref y=b;let h=if false {Holder {value:x}} else {Holder {value:y}};return h.value.value;}",3),
 ("use compiler_fixtures.record_105.{Plain}; pub fn main() -> i32 {let a=Plain {value: 7};let xs=(a,);return a.value+xs.0.value;}",14),
 ("pub fn id(x:R) -> R {return x;} pub fn main() -> i32 {let a=R {value: 7};let xs={id(a)};return a.value+xs[0].value;}",14),
 ("pub fn id(x:R) -> R {return x;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=Holder {value:v};let result=id(h.value);return result.value;}",7),
 ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=Holder {value:v};let ref outer=h;let xs=(outer,);return xs.0.value.value;}",7),

 ("pub fn main() -> i32 {let a=R {value: 7};let h=Holder {value:a};return a.value+h.value.value;}",14),
 ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=Holder {value:v};let moved=h;return moved.value.value;}",7),
 ("pub fn main() -> i32 {let a=R {value: 7};let xs={a,a};return a.value+xs[0].value+xs[1].value;}",21),
 ("pub fn main() -> i32 {let a=R {value: 7};let xs=(a,R {value: 3});let ys=xs;return a.value+ys.0.value+ys.1.value;}",17),
 ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let x=(v,);let y=x;return x.0.value+y.0.value;}",14),
 ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let xs={v,v};let i:usize=1;let y=xs[i];return y.value;}",7),
 ("pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;var h=Holder {value:v};h.value=b;return h.value.value+a.value;}",10),
 ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let (x,)= (v,);return x.value;}",7),
 ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let xs=({v,v},);return xs.0[1].value;}",7),
 ("pub fn id(x:R) -> R {return x;} pub fn main() -> i32 {let a=R {value: 7};let h=Holder {value:id(a)};return a.value+h.value.value;}",14),
 ("pub class C {pub value:i32;new(n:i32) {value=n;};drop() {}} use compiler_fixtures.record_106.{Box}; pub fn main() -> i32 {let a=C.new(7);let h=Box {value: a};return if same_identity(a,h.value) {h.value.value} else {0};}",7),
];
const PANICS:&[(&str,i128)]=&[
 ("pub fn fail(x:R) -> R {let fail=1/0;return x;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;let xs={a,w,fail(a)};return a.value+xs[0].value;}",101),

 ("pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;let fail=1/0;} pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};set(h,w);return h.value.value;}",101),

 ("use compiler_fixtures.record_107.{F}; use compiler_fixtures.record_102.{Mixed}; pub fn wrap(_borrowed_x:R) -> Mixed {let x=_borrowed_x;let fail=F {value: 0};return Mixed {view: x, owned: R {value: 3}};} pub fn main() -> i32 {let a=R {value: 7};let ref va=a;let h=wrap(va);return a.value+h.view.value;}",101),
 ("pub enum E {Item {view:R,owned:R,fail:i32}} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let e=E.Item {view:v,owned:R {value: 3},fail:1/0};return 0;}",101),
 ("pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=(v,R {value: 3},1/0);return 0;}",101),
 ("use compiler_fixtures.record_108.{Mix}; pub fn main() -> i32 {let a=R {value: 7};let ref v=a;let h=Mix {borrowed:v,owned:R {value: 3},fail:1/0};return 0;}",101),
 ("pub fn get(_borrowed_x:R) -> R {let x=_borrowed_x;let h=(x,);let fail=1/0;return h.0;} pub fn main() -> i32 {let a=R {value: 7};let ref va=a;let h=(get(va),);return a.value+h.0.value;}",101),
];
#[path = "borrowed_storage/native_storage_drops_owned.rs"]
mod native_storage_drops_owned;
#[path = "borrowed_storage/partial_borrowed_construction_cleans.rs"]
mod partial_borrowed_construction_cleans;
