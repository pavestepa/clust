use clust_core::{check::check, hir, interpret, mir, rust};
use clust_frontend::{source::Diagnostic, testing::load_ir as load};
use clust_semantics::OverflowChecks;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "clust-storage-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("main.cl"), source).unwrap();
        Self(dir)
    }
    fn checked(&self) -> Result<hir::Program, Vec<Diagnostic>> {
        let loaded = load(&self.0.join("main.cl"), "app");
        if !loaded.diagnostics.is_empty() {
            return Err(loaded.diagnostics);
        }
        check(&loaded.module.unwrap())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
const IMPORT:&str="use core.intrinsic.{RawStorage,allocate,capacity,initialize,take,drop_slot,release,replace,size_of,align_of,slot_size,slot_align,move_within,move_slots,drop_range,view,view_mut,slot_address,slot_address_mut,drop_in_place,into_raw_parts,from_raw_parts,read_owned,write_owned};\n";
fn source(body: &str) -> String {
    format!("{IMPORT}{body}")
}
fn cases() -> Vec<(String, Option<i32>)> {
    let buffer = include_str!("fixtures/scalar_buffer.fixture");
    let mut result = vec![
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(4);initialize(s,0,2);initialize(s,1,5);move_within(s,1,0,2);move_within(s,0,1,2);let a=take(s,0);let b=take(s,1);release(s);return a+b;}}"),Some(7)),
        (source("pub class R {pub v:i32;new(n:i32) {v=n;};drop() {}} pub fn main() -> i32 {if trusted_region() {var a=allocate<R>(1);var b=allocate<R>(1);initialize(a,0,R.new(7));move_slots(b,0,a,0,1);let r=take(b,0);release(a);release(b);return r.v;}}"),Some(7)),
        (source("use compiler_fixtures.record_2.{R}; pub fn main() -> i32 {if trusted_region() {var s=allocate<R>(1);initialize(s,0,R {v: 7});let r=take(s,0);release(s);return r.v;}}"),Some(7)),
        (source("pub class R {pub v:i32;new(n:i32) {v=n;};drop() {}} pub fn main() -> i32 {if trusted_region() {var s=allocate<R>(1);initialize(s,0,R.new(7));drop_slot(s,0);release(s);return 7;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<(i32,i32)>(2);initialize(s,0,(2,5));let old=replace(s,0,(3,4));initialize(s,1,old);drop_slot(s,0);let pair=take(s,1);release(s);return pair.0+pair.1;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<{i32;3}>(1);initialize(s,0,{2,5,9});let a=take(s,0);release(s);return a[0]+a[1];}}"),Some(7)),
        (include_str!("fixtures/raw_storage.fixture").into(),Some(7)),
        (buffer.into(),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<({i32;2},())>(1);initialize(s,0,({2,5},()));let a=take(s,0);release(s);return a.0[0]+a.0[1];}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<{();2}>(1);initialize(s,0,{(),()});drop_slot(s,0);release(s);return 7;}}"),Some(7)),
        (buffer.replace("return buffer.pop();","buffer.push(1/0);return 0;"),None),
        (source("pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(0);let n=capacity(s);release(s);return n as i32;}}"),Some(0)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<()>(2);initialize(s,0,());initialize(s,1,());let a=take(s,0);drop_slot(s,1);release(s);return 7;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,1);let n=take(s,0);initialize(s,0,6);return take(s,0)+n;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);var alias=s;initialize(alias,0,7);return take(alias,0);}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(2);return (size_of<i32>()+align_of<i32>()+slot_size(s)+slot_align(s)+capacity(s)) as i32;}}"),Some(18)),
        (source("pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(18446744073709551615);return 0;}}"),None),
        (source("pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(1);return 1/0;}}"),None),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,2);let old=replace(s,0,5);return old+take(s,0);}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<u128>(1);initialize(s,0,340282366920938463463374607431768211455);let n=take(s,0);return (n%10) as i32;}}"),Some(5)),
    ];
    result.extend([
        (source("use compiler_fixtures.record_81.{Item}; pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(1);let p=slot_address_mut(s,0);write_owned(p,Item {v: 7});let item=read_owned(p);return item.v/(item.v-7);}}"),None),

        (source("use compiler_fixtures.record_81.{Item}; pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(2);let p=slot_address_mut(s,1);write_owned(p,Item {v: 7});let item=read_owned(p);release(s);return item.v;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<u32>(1);let p=slot_address_mut(s,0) as *var f32;write_owned(p,7.0);let n=read_owned(p);release(s);return n as i32;}}"),Some(7)),
        (source("pub fn main() -> i32 {var n=2;let p=&var n;if trusted_region() {let old=read_owned(p);write_owned(p,7)};return n;}"),Some(7)),
        (source("pub fn main() -> i32 {var n=(2,3);let p=&var n.1;if trusted_region() {let old=read_owned(p);write_owned(p,5)};return n.0+n.1;}"),Some(7)),

        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<u32>(1);initialize(s,0,1088421888);let p=slot_address_mut(s,0) as *var f32;let n=read_owned(p);release(s);return n as i32;}}"),Some(7)),

        (source("pub class Item {pub v:i32;new(n:i32) {v=n;};drop() {}} pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(1);initialize(s,0,Item.new(7));let item=read_owned(slot_address_mut(s,0));release(s);return item.v;}}"),Some(7)),
        (source("use compiler_fixtures.record_81.{Item}; pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(2);initialize(s,1,Item {v: 7});let p=slot_address_mut(s,0)+1;let item=read_owned(p);release(s);return item.v;}}"),Some(7)),
        (source("pub enum Item {None,Some(i32)} pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(1);initialize(s,0,Item.Some(7));let item=read_owned(slot_address_mut(s,0));release(s);return match item {Item.None=>0,Item.Some(n)=>n};}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<()>(1);initialize(s,0,());let n=read_owned(slot_address_mut(s,0));release(s);return 7;}}"),Some(7)),

        (source("#[Copy] pub enum E {A,B} pub fn main() -> i32 {var n=E.A;if trusted_region() {let p=(&var n) as *var u32;*p=1};return match n {E.A=>0,E.B=>7};}"),Some(7)),
        (source("#[Copy] pub enum E {A(u32),B(u32)} pub fn main() -> i32 {var n=E.A(2);if trusted_region() {let p=(&var n) as *var {u32;2};*p={1,7}};return match n {E.A(n)=>0,E.B(n)=>n as i32};}"),Some(7)),

        (source("pub fn main() -> i32 {let n:u8=1;if trusted_region() {let p=(&n) as *bool;if *p {return 7;};return 0;}}"),Some(7)),
        (source("pub fn main() -> i32 {var n='A';if trusted_region() {let p=(&var n) as *var u32;*p=66;if n=='B' {return 7;};return 0;}}"),Some(7)),
        (source("pub fn main() -> i32 {var n:(bool,u8)=(false,7);if trusted_region() {let p=(&var n) as *var u8;*p=1;if n.0 {return n.1 as i32;};return 0;}}"),Some(7)),

        (source("pub fn main() -> i32 {let a={2,7};if trusted_region() {let all=(&a) as usize;let first=(&a[0]) as usize;let second=(&a[1]) as usize;if all==first && second-first==4 {return 7;};return 0;}}"),Some(7)),
        (source("use compiler_fixtures.record_82.{Pair}; pub fn main() -> i32 {let a=Pair {a: 1, b: 7};if trusted_region() {let all=(&a) as usize;let field=(&a.b) as usize;let p=field as *u32;if field-all==4 {return *p as i32;};return 0;}}"),Some(7)),

        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(2);initialize(s,0,2);initialize(s,1,7);let ref v=view(s,0);let p=&*v;let address=p as usize;let direct=slot_address(s,0) as usize;let n=*(p+1);drop_range(s,0,2);if address==direct {return n;};return 0;}}"),Some(7)),

        (source(" pub fn item<T:Copy>(var s:RawStorage<T>) -> T {requires_unsafe_caller();if trusted_region() {return view_mut(s,0);}} pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,2);let ref var v=item<i32>(s);*v=7;return take(s,0);}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,2);let n=if true {view(s,0)} else {view(s,0)};drop_slot(s,0);return n+5;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,7);var copy=view_mut(s,0);copy=0;let n=take(s,0);return n+copy;}}"),Some(7)),
        (source(" pub fn item(var s:RawStorage<i32>) -> i32 {requires_unsafe_caller();if trusted_region() {return view_mut(s,0);}} pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,2);let f=item;let ref var v=f(s);*v=7;return take(s,0);}}"),Some(7)),

        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,2);let ref var v=view_mut(s,0);*v=7;let n=view(s,0);drop_slot(s,0);return n;}}"),Some(7)),
        (source("use compiler_fixtures.record_83.{Pair}; pub fn main() -> i32 {if trusted_region() {var s=allocate<Pair>(1);initialize(s,0,Pair {a: 2, b: 3});let ref var v=view_mut(s,0);v.b=5;let n=view(s,0);drop_slot(s,0);return n.a+n.b;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<{i32;2}>(1);initialize(s,0,{2,3});let ref var v=view_mut(s,0);v[1]=5;let n=view(s,0);drop_slot(s,0);return n[0]+n[1];}}"),Some(7)),
        (source("#[Copy] pub enum E {None,Some(i32)} pub fn main() -> i32 {if trusted_region() {var s=allocate<E>(1);initialize(s,0,E.None);let ref var v=view_mut(s,0);*v=E.Some(7);let n=view(s,0);drop_slot(s,0);return match n {E.None=>0,E.Some(n)=>n};}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<()>(2);initialize(s,0,());initialize(s,1,());let ref v=view(s,1);let n=*v;drop_range(s,0,2);return 7;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,2);let ref var v=view_mut(s,0);let p=&var *v;let old=take(s,0);initialize(s,0,7);let n=*p;drop_slot(s,0);return n;}}"),Some(7)),
        ("use core.storage.{OwnedBuffer};\npub fn main() -> i32 {var b=OwnedBuffer<i32>.make(1);b.push(2);let ref var v=b.at_mut(0);*v=7;let n=b.at(0);b.clear();return n;}".into(),Some(7)),

        (source("pub fn address(p:*var i32,count:*var i32) -> *var i32 {if trusted_region() {*count+=1};return p;} pub fn rhs(p:*var i32) -> i32 {if trusted_region() {*p=99};return 5;} pub fn main() -> i32 {var n=2;var count=0;if trusted_region() {*address(&var n,&var count)+=rhs(&var n)};if count==1 {return n;};return 0;}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(1);initialize(s,0,2);*slot_address_mut(s,0)+=5;return take(s,0);}}"),Some(7)),
        (source("pub fn main() -> i32 {var n=2;if trusted_region() {let a=(&var n) as isize;let p=a as *var i32;*p=7;return n;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {let a:isize=-1;let p=a as *i32;let b=p as isize;if b==a {return 7;};return 0;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {let s=allocate<i32>(1);let parts=into_raw_parts(s);let address=parts.0 as usize;let p=address as *var i32;var back=from_raw_parts(p,parts.1);initialize(back,0,7);return take(back,0);}}"),Some(7)),

        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<u32>(2);initialize(s,0,2);initialize(s,1,7);let addr=slot_address(s,0) as usize;let p=(addr+4) as *u32;let n=*p;drop_range(s,0,2);return n as i32;}}"),Some(7)),
        (source("pub fn main() -> i32 {var n=2;if trusted_region() {let a=(&var n) as usize;let p=a as *var i32;*p=7;return n;}}"),Some(7)),
        (source("use compiler_fixtures.record_84.{Pair}; pub fn main() -> i32 {var value=Pair {a: 1, b: 2};if trusted_region() {let p=(&var value) as *var u32;*(p+1)=6;return (value.a+value.b) as i32;}}"),Some(7)),
        (source("pub fn main() -> i32 {var value:(u32,u32)=(1,2);if trusted_region() {let p=(&var value) as *var {u32;2};*p={3,4};return (value.0+value.1) as i32;}}"),Some(7)),
        (source("pub fn main() -> i32 {var value:{u32;8}={1,2,3,4,5,6,7,8};if trusted_region() {let p=(&var value) as *var u32;*(p+7)=0;let q=p as *{u32;8};let copied=*q;return (copied[6]+copied[7]) as i32;}}"),Some(7)),
        (source("use compiler_fixtures.record_84.{Pair}; pub fn main() -> i32 {if trusted_region() {var s=allocate<Pair>(1);initialize(s,0,Pair {a: 2, b: 5});let p=slot_address(s,0) as *(u32,u32);let pair=*p;drop_slot(s,0);return (pair.0+pair.1) as i32;}}"),Some(7)),
        (source("use compiler_fixtures.record_85.{Set}; pub fn main() -> i32 {var n=1;if true {let guard=Set {p: &var n};};if n==7 {return 7;};return 0;}"),Some(7)),
        (source("pub fn main() -> i32 {var n=1;let p=&var n;if trusted_region() {*p=7};if n==7 {return 7;};return 0;}"),Some(7)),
        (source("pub fn main() -> i32 {var a:u64=506381209866536711;if trusted_region() {let p=(&var a) as *var u8;let q=(p+4) as *var u32;*q=0;if a!=0 && a!=506381209866536711 {return 7;};return 0;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<u32>(2);initialize(s,0,0);initialize(s,1,0);let p=slot_address_mut(s,0) as *var u8;*(p+4)=7;*(p+5)=7;*(p+6)=7;*(p+7)=7;let a=take(s,0);let b=take(s,1);if a==0 && b==117901063 {return 7;};return 0;}}"),Some(7)),
        (source("pub fn main() -> i32 {var a:u64=0;if trusted_region() {let p=(&var a) as *var u8;let q=(p+4) as *var u32;*q=117901063;return *(p+4) as i32;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {let s=allocate<u64>(1);let parts=into_raw_parts(s);let bytes=parts.0 as *var u8;let p=bytes as *var u64;var back=from_raw_parts(p,parts.1);initialize(back,0,7);return take(back,0) as i32;}}"),Some(7)),
        (source("pub fn main() -> i32 {var f:f32=0.0;if trusted_region() {let p=(&var f) as *var u32;*p=2147483648;let q=(&f) as *i32;if *q==(-2147483647-1) {return 7;};return 0;}}"),Some(7)),
        (source("pub fn main() -> i32 {let a:{u32;2}={0,1088421888};if trusted_region() {let p=(&a[0]) as *f32;let end=p+2;return *(end-1) as i32;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<u32>(1);initialize(s,0,1065353216);let p=slot_address_mut(s,0) as *var f32;*p=7.0;let bits=take(s,0);release(s);if bits==1088421888 {return 7;};return 0;}}"),Some(7)),
        (source("pub fn main() -> i32 {var bits:u64=4619567317775286272;if trusted_region() {let p=(&var bits) as *var f64;let q=p as *var u64;*q=4619567317775286272;return *p as i32;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(2);initialize(s,1,7);let parts=into_raw_parts(s);var adopted=from_raw_parts(parts.0,parts.1);let n=take(adopted,1);release(adopted);return n;}}"),Some(7)),
        (source("pub class Item {pub value:i32;new(n:i32) {value=n;}} pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(1);initialize(s,0,Item.new(7));let parts=into_raw_parts(s);var adopted=from_raw_parts(parts.0,parts.1);let n=view(adopted,0).value;drop_slot(adopted,0);return n;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {let s=allocate<()>(0);let parts=into_raw_parts(s);let adopted=from_raw_parts(parts.0,parts.1);return 7;}}"),Some(7)),
        (source("use compiler_fixtures.record_83.{Pair}; pub fn main() -> i32 {var value=Pair {a: 2, b: 3};let p=&var value;if trusted_region() {*p=Pair {a: 3, b: 4};let v=*p;return v.a+v.b;}}"),Some(7)),
        (source("#[Copy] pub enum E {None,Some(i32)} pub fn main() -> i32 {var value=E.None;let p=&var value;if trusted_region() {*p=E.Some(7);let v=*p;return match v {E.None=>0,E.Some(n)=>n};}}"),Some(7)),
        (source("pub fn main() -> i32 {var value=();let p=&var value;if trusted_region() {*p=();let v=*p;};return 7;}"),Some(7)),
        (source("pub class R {pub v:i32;new(n:i32) {v=n;};drop() {}} pub fn main() -> i32 {if trusted_region() {var s=allocate<R>(2);initialize(s,1,R.new(7));let first=slot_address_mut(s,0);let p=first+1;let n=p-first;let readonly=p as *R;drop_in_place(p);release(s);return 6+(n as i32);}}"),Some(7)),
        (source("use compiler_fixtures.record_86.{R}; pub fn main() -> i32 {if trusted_region() {var s=allocate<R>(2);initialize(s,1,R {v: 7});drop_in_place(slot_address_mut(s,1));return 7;}}"),None),
        (source("pub fn main() -> i32 {var n=2;let p=&var n;if trusted_region() {drop_in_place(p)};return 7;}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(2);initialize(s,0,2);initialize(s,1,5);let p=slot_address_mut(s,1);*p=7;let first=slot_address(s,0);let end=slot_address(s,2);let n=*(end-1);drop_range(s,0,2);return n+(end-first) as i32;}}"),Some(9)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<()>(0);let p=slot_address(s,0);return 7;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(0);let p=slot_address(s,1);return 7;}}"),None),
        ("use core.storage.{OwnedBuffer};\npub class Item {pub value:i32;new(n:i32) {value=n;}} pub fn main() -> i32 {var b=OwnedBuffer<Item>.make(2);b.push(Item.new(2));var item=b.at_mut(0);item.value=7;return b.at(0).value;}".into(),Some(7)),
        ("use core.storage.{OwnedBuffer};\npub class Item {pub value:i32;new(n:i32) {value=n;}} pub fn main() -> i32 {var b=OwnedBuffer<Item>.make(2);b.push(Item.new(2));return b.at(1).value;}".into(),None),
        (source("pub class Item {pub value:i32;new(n:i32) {value=n;}} pub fn main() -> i32 {if trusted_region() {var s=allocate<Item>(1);initialize(s,0,Item.new(2));var item=view_mut(s,0);item.value=7;let n=view(s,0).value;drop_slot(s,0);return n;}}"),Some(7)),
        (include_str!("fixtures/owned_buffer.fixture").into(),Some(7)),
        ("use core.storage.{OwnedBuffer,BufferResult};\npub fn main() -> i32 {var buffer=OwnedBuffer<i32>.make(1);buffer.push(3);let result=buffer.push(7);buffer.clear();return match result {BufferResult.Some(n)=>n,BufferResult.None=>0};}".into(),Some(7)),
        ("use core.storage.{OwnedBuffer};\npub class Item {pub v:i32;new(n:i32) {v=n;};drop() {if v==1 {let bad=1/0;}}} pub fn main() -> i32 {var buffer=OwnedBuffer<Item>.make(3);buffer.push(Item.new(1));buffer.push(Item.new(2));buffer.push(Item.new(3));return 7;}".into(),None),
        (source("pub class R {pub v:i32;new(n:i32) {v=n;};drop() {}} pub fn main() -> i32 {if trusted_region() {var s=allocate<R>(1);initialize(s,0,R.new(2));let old=replace(s,0,R.new(5));let current=take(s,0);release(s);return old.v+current.v;}}"),Some(7)),
        (source("use compiler_fixtures.record_86.{R}; pub fn main() -> i32 {if trusted_region() {var s=allocate<R>(1);initialize(s,0,R {v: 7});drop_slot(s,0);return 7;}}"),None),
        (source("use compiler_fixtures.record_87.{Z}; pub fn main() -> i32 {if trusted_region() {var s=allocate<Z>(2);initialize(s,0,Z {});move_within(s,1,0,1);drop_slot(s,1);release(s);return 7;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(0);move_within(s,0,0,0);release(s);return 7;}}"),Some(7)),
        (source("pub fn main() -> i32 {if trusted_region() {var s=allocate<i32>(0);move_within(s,0,0,18446744073709551615);return 7;}}"),None),
    ]);
    result
}

#[path = "raw_storage/native_storage_releases_allocations.rs"]
mod native_storage_releases_allocations;
#[path = "raw_storage/scalar_storage_reference_lifecycle.rs"]
mod scalar_storage_reference_lifecycle;
#[path = "raw_storage/storage_metadata_survives_reexports.rs"]
mod storage_metadata_survives_reexports;
#[path = "raw_storage/verifier_checks_raw_address.rs"]
mod verifier_checks_raw_address;
