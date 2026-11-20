use super::*;

#[test]
fn partial_borrowed_construction_cleans_only_completed_owners() {
    for (body, _) in PANICS {
        let p = checked(&format!("{PREFIX}{body}")).unwrap();
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
        assert_eq!(a, b, "{body}");
        assert!(a.0.is_err());
    }
}

#[test]
fn readonly_storage_moves_descriptors_and_preserves_cleanup() {
    for (body, expected) in CASES {
        let source = format!("{PREFIX}{body}");
        let p = checked(&source).unwrap_or_else(|e| panic!("{source}: {e:?}"));
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
            "{source}: {a:?}"
        );
        clust_core::rust::emit(&p, OverflowChecks::Checked).unwrap();
    }
}

#[test]
fn storage_cannot_escape_gain_mutability_or_mix_modes() {
    for body in [
 "pub fn main() -> i32 {let b=R {value: 3};let ref w=b;var h=Holder {value:R {value: 7}};h.value=w;return h.value.value;}",
 "pub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref w=b;var h=Holder {value:a};h.value=w;b=R {value: 2};return h.value.value;}",
 "pub fn main() -> i32 {let a=R {value: 7};var h=Holder {value:a};if true {let b=R {value: 3};let ref w=b;h.value=w};return h.value.value;}",
 "pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=Holder {value:a};h.value.value=4;h.value=w;return h.value.value;}",
 "pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;let xs={R {value: 2},w};return xs[0].value+xs[1].value;}",
 "pub fn main() -> i32 {var a=R {value: 7};let b=R {value: 3};let ref w=b;let xs={a,w};a=R {value: 2};return xs[0].value;}",

 "pub enum E {Item(R),Empty} pub fn set(var e:E,_borrowed_x:R) -> () {let x=_borrowed_x;let ref var target=e;*target=E.Item(x);} pub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref v=a;let ref w=b;var e=E.Item(v);e=E.Empty;set(e,w);b=R {value: 2};return match e {E.Item(x)=>x.value,E.Empty=>0};}",
 "pub enum E {Item(R),Empty} pub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref v=a;let ref w=b;var e=E.Item(v);e=E.Empty;let ref var target=e;*target=E.Item(w);b=R {value: 2};return match e {E.Item(x)=>x.value,E.Empty=>0};}",
 "pub enum E {Item(R),Empty} pub fn set(var e:E,_borrowed_x:R) -> () {let x=_borrowed_x;let ref var target=e;*target=E.Item(x);} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var e=E.Item(v);e=E.Empty;if true {let b=R {value: 3};let ref w=b;set(e,w)};return match e {E.Item(x)=>x.value,E.Empty=>0};}",

 "pub enum E {Item(R),Empty} pub fn unknown() -> bool {return true;} pub fn main() -> i32 {var a=R {value: 7};let ref v=a;var e=E.Item(v);if unknown() {e=E.Empty};a=R {value: 3};return match e {E.Item(x)=>x.value,E.Empty=>3};}",
 "pub enum E {Item(R),Empty} pub fn main() -> i32 {var a=R {value: 7};let ref v=a;var e=E.Item(v);var n=0;while n<2 {if n==1 {e=E.Empty};n+=1;};a=R {value: 3};return match e {E.Item(x)=>x.value,E.Empty=>3};}",
 "pub enum E {Item(R),Empty} pub fn main() -> i32 {var a=R {value: 7};let ref v=a;var e=E.Empty;e=E.Item(v);a=R {value: 3};return match e {E.Item(x)=>x.value,E.Empty=>3};}",
 "pub fn get(x:R,flag:bool) -> R {if flag {let local=R {value: 3};return local;};return x;} pub fn main() -> i32 {let a=R {value: 7};let result=get(a,true);return a.value+result.value;}",

 "pub class C {pub r:R;new() {r=R {value: 3};}} pub fn set(var h:Holder,x:C) -> () {h.value=x.r;} pub fn main() -> i32 {let a=R {value: 7};var b=C.new();let ref v=a;let ref w=b;var h=Holder {value:v};set(h,w);b=C.new();return h.value.value;}",

 "pub fn set(var h:Holder,xs:{R;2},i:usize) -> () {h.value=xs[i];} pub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};set(h,{v,w},1);b=R {value: 2};return h.value.value;}",

 "pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};set(h,w);b=R {value: 2};return h.value.value;}",
 "pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var h=Holder {value:v};if true {let b=R {value: 3};let ref w=b;set(h,w)};return h.value.value;}",
 "pub fn set(var h:Holder) -> () {let b=R {value: 3};h.value=b;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var h=Holder {value:v};set(h);return h.value.value;}",
 "pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var h=Holder {value:v};set(h,R {value: 3});return h.value.value;}",
 "pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn relay(var h:Holder,x:R) -> () {set(h,x);} pub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};relay(h,w);b=R {value: 2};return h.value.value;}",
 "pub fn set(var h:Holder,_borrowed_x:R) -> () {let x=_borrowed_x;h.value=x;} pub fn main() -> i32 {var a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};let old=h.value;set(h,w);a=R {value: 2};return old.value+h.value.value;}",
 "use compiler_fixtures.record_101.{A}; pub fn set(var a:A,_borrowed_x:R,i:usize) -> () {let x=_borrowed_x;a.xs[i]=x;} pub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref v=a;let ref w=b;var holder=A {xs: {v,v}};set(holder,w,1);b=R {value: 2};return holder.xs[1].value;}",
 "pub fn set(var h:Holder,_borrowed_x:R,flag:bool) -> () {let x=_borrowed_x;if flag {h.value=x}} pub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};set(h,w,true);b=R {value: 2};return h.value.value;}",
 "pub fn set(var h:Holder,_borrowed_x:R,flag:bool) -> () {let x=_borrowed_x;if flag {h.value=x} else {set(h,x,true)}} pub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref v=a;let ref w=b;var h=Holder {value:v};set(h,w,false);b=R {value: 2};return h.value.value;}",
 "pub fn set(var h:Holder) -> () {let b=R {value: 3};h.value=b;let fail=1/0;} pub fn main() -> i32 {let a=R {value: 7};let ref v=a;var h=Holder {value:v};set(h);return h.value.value;}",

 "pub fn id(h:Holder) -> Holder {return h;} pub fn main() -> i32 {var a=R {value: 7};let ref v=a;let h=Holder {value:v};let outer=id(h);let saved=h.value.value;a=R {value: 2};return outer.value.value;}",
 "pub fn set(var xs:{R;2},_borrowed_x:R,i:usize) -> {R;2} {let x=_borrowed_x;xs[i]=x;return xs;} pub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref x=a;let ref y=b;let xs=set({x,x},y,1);b=R {value: 2};return xs[1].value;}",
 "pub fn pair(_borrowed_x:R,_borrowed_y:R) -> (R,R) {let x=_borrowed_x;let y=_borrowed_y;return (x,y);} pub fn main() -> i32 {let a=R {value: 1};var b=R {value: 2};let ref x=a;let ref y=b;let h=pair(x,y);b=R {value: 3};return h.1.value;}",
 "pub fn wrap(_borrowed_x:R) -> Holder {let x=_borrowed_x;return Holder {value:x};} pub fn main() -> i32 {var a=R {value: 1};let ref v=a;let h=wrap(v);a=R {value: 2};return h.value.value;}",
 "pub fn wrap<T>(_borrowed_x:T) -> (T,) {let x=_borrowed_x;return (x,);} pub fn bad() -> (R,) {let a=R {value: 1};let ref v=a;return wrap(v);} pub fn main() -> i32 {return 0;}",
 "pub fn wrap(flag:bool,_borrowed_x:R) -> Holder {let x=_borrowed_x;return if flag {Holder {value:x}} else {Holder {value:R {value: 2}}};} pub fn main() -> i32 {let a=R {value: 1};let h=wrap(true,a);return a.value+h.value.value;}",
 "pub fn wrap(_borrowed_x:R) -> Holder {let x=_borrowed_x;return Holder {value:x};} pub fn main() -> i32 {let a=R {value: 1};let ref v=a;var h=wrap(v);h.value.value=2;return h.value.value;}",
 "use compiler_fixtures.record_98.{G}; pub fn get<T>(h:G<T>) -> T {return h.value;} pub fn main() -> i32 {var a=R {value: 1};let ref v=a;let h=G<R> {value: v};let result=get(h);a=R {value: 2};return result.value;}",
 "use compiler_fixtures.record_99.{Outer}; pub fn get(o:Outer) -> R {return o.h.value;} pub fn main() -> i32 {var a=R {value: 1};let ref v=a;let h=Holder {value:v};let ref inner=h;let o=Outer {h: inner};let result=get(o);a=R {value: 2};return result.value;}",
 "pub fn get(h:Holder) -> R {return h.value;} pub fn main() -> i32 {var a=R {value: 1};let ref v=a;let h=Holder {value:v};let result=get(h);a=R {value: 2};return result.value;}",
 "pub fn bad() -> R {let a=R {value: 1};let ref v=a;let h=Holder {value:v};return get(h);} pub fn get(h:Holder) -> R {return h.value;} pub fn main() -> i32 {return bad().value;}",
 "pub fn change(var h:Holder) -> () {h.value=R {value: 2};} pub fn main() -> i32 {let a=R {value: 1};let ref v=a;var h=Holder {value:v};change(h);return h.value.value;}",
 "pub enum E {Item(R)} pub fn main() -> i32 {var a=R {value: 1};let ref v=a;let e=E.Item(v);a=R {value: 2};return match e {E.Item(x)=>x.value};}",
 "pub enum E {Item {view:R}} pub fn bad() -> E {let a=R {value: 1};let ref v=a;return E.Item {view:v};} pub fn main() -> i32 {return 0;}",

 "pub fn main() -> i32 {var a=R {value: 1};let ref v=a;let h=Holder {value:v};let ref outer=h;a=R {value: 2};return outer.value.value;}",
 "pub fn main() -> i32 {var a=R {value: 1};let ref v=a;let h=Holder {value:v};let ref outer=h;let xs=(outer,);a=R {value: 2};return xs.0.value.value;}",
 "pub fn main() -> i32 {let a=R {value: 1};let ref v=a;var h=Holder {value:v};if true {let b=R {value: 2};h.value=b};return h.value.value;}",

 "pub fn main() -> i32 {var a=R {value: 1};let ref v=a;let h=Holder {value:v};a=R {value: 2};return h.value.value;}",
 "pub fn main() -> i32 {var a=R {value: 1};let ref v=a;var h=Holder {value:v};h.value.value=2;return h.value.value;}",
 "pub fn main() -> i32 {let a=R {value: 1};let ref v=a;let xs={v,R {value: 2}};return xs[0].value;}",
 "pub fn main() -> i32 {let a=R {value: 1};let ref v=a;var h=Holder {value:v};h=Holder {value:R {value: 2}};return h.value.value;}",
 "pub fn main() -> i32 {let a=R {value: 1};let ref v=a;let h=if true {Holder {value:v}} else {Holder {value:R {value: 2}}};return h.value.value;}",
 "pub fn bad() -> Holder {let a=R {value: 1};let ref v=a;return Holder {value:v};} pub fn main() -> i32 {return bad().value.value;}",

 ] {let source=format!("{PREFIX}{body}");assert!(checked(&source).is_err(),"accepted {source}");}
}
