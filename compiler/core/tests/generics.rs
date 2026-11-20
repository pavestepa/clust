use clust_core::{check::check, hir::Value, interpret, mir};
use clust_frontend::{source::SourceId, testing::parse};
use clust_semantics::OverflowChecks;
fn checked(
    source: &str,
) -> Result<clust_core::hir::Program, Vec<clust_frontend::source::Diagnostic>> {
    let parsed = parse(SourceId(0), source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    check(&parsed.module.unwrap())
}
#[test]
fn specializes_inferred_explicit_recursive_and_const_generic_functions() {
    for (source,expected) in [
  ("pub fn identity<T>(value:T) -> T {return value;} pub fn main() -> i32 {let a:u8=identity(7);return identity<i32>(a as i32);}",7),
  ("pub fn first<T:Copy>(a:T,b:T) -> T {return a;} pub fn main() -> i32 {let n:u8=4;return first(7,n) as i32;}",7),
  ("pub fn last<T,const N:usize>(values:{T;N}) -> T {return values[N-1];} pub fn main() -> i32 {return last({1,2,3});}",3),
  ("pub fn pick<T>(n:i32,x:T) -> T {if n==0 {return x;};return pick<T>(n-1,x);} pub fn main() -> i32 {return pick(5,120);}",120),
  ("pub fn value<T=i32>() -> i32 {return 5;} pub fn main() -> i32 {return value();}",5),
  ("pub fn length<const N:usize>(a:{i32;N}) -> usize {return N;} pub fn main() -> i32 {return length<3>({1,2,3}) as i32;}",3),
 ] {
  let hir=checked(source).unwrap();let cfg=mir::lower(&hir);
  let expected=Some(expected);
  for value in [interpret::execute(&hir,hir.main.unwrap(),vec![],OverflowChecks::Checked,100000).unwrap(),mir::interpret::execute(&cfg,cfg.main.unwrap(),vec![],OverflowChecks::Checked,100000).unwrap()]{let Value::Integer(v)=value else{panic!()};assert_eq!(v.signed_value(),expected,"{source}");}
 }
}
#[test]
fn rejects_inconsistent_or_missing_specializations() {
    for source in [
  "pub fn id<T>(x:T) -> T {return x;} pub fn main() -> () {let a=id<i32>(true);}",
  "pub fn first<T>(a:T,b:T) -> T {return a;} pub fn main() -> () {let a=first(1,true);}",
  "pub fn size<T>() -> i32 {return 1;} pub fn main() -> i32 {return size();}",
  "pub fn length<const N:usize>(a:{i32;N}) -> usize {return N;} pub fn main() -> usize {return length<2>({1,2,3});}",
 ] {assert!(checked(source).is_err(),"accepted {source}");}
}
#[test]
fn defaults_follow_inference_and_recursive_instances_are_reused() {
    let p=checked("pub fn id<T=i32>(x:T) -> T {return x;} pub fn main() -> i32 {let a=id(if true {1 as u8} else {2 as u8});return a as i32;}").unwrap();
    assert_eq!(p.functions.len(), 2);
    let p=checked("pub fn pick<T>(n:i32,x:T) -> T {if n==0 {return x;};return pick<T>(n-1,x);} pub fn main() -> i32 {return pick(3,6)+pick(4,24);}").unwrap();
    assert_eq!(p.functions.len(), 2);
    let error = checked(
        "pub fn grow<const N:usize>() -> i32 {return grow<N+1>();} pub fn main() -> i32 {return grow<0>();}",
    )
    .unwrap_err();
    assert_eq!(error[0].code, "E0372");
}
#[test]
fn generic_structs_and_enums_use_ordinary_layout_and_pattern_lowering() {
    let source="use compiler_fixtures.record_94.{Fixed}; #[Copy] pub enum Option<T> {None,Some(T)} pub fn main() -> i32 {let f=Fixed<i32,3> {values: {1,2,3}};let value:Option<i32>=Option.Some(f.values[2]);return match value {Option.Some(x)=>x,Option.None=>0};}";
    let hir = checked(source).unwrap();
    let cfg = mir::lower(&hir);
    let Value::Integer(value) = mir::interpret::execute(
        &cfg,
        cfg.main.unwrap(),
        vec![],
        OverflowChecks::Checked,
        10000,
    )
    .unwrap() else {
        panic!()
    };
    assert_eq!(value.signed_value(), Some(3));
}
#[test]
fn infers_function_parameters_through_generic_nominal_arguments() {
    let source="#[Copy] pub enum Option<T> {None,Some(T)} use compiler_fixtures.record_94.{Fixed}; pub fn unwrap<T:Copy>(value:Option<T>,fallback:T) -> T {return match value {Option.Some(x)=>x,Option.None=>fallback};} pub fn first<T:Copy,const N:usize>(value:Fixed<T,N>) -> T {return value.values[0];} pub fn main() -> i32 {let fixed:Fixed<i32,2>=Fixed {values: {8,9}};let value=Option<i32>.Some(first(fixed));return unwrap(value,1);}";
    let p = checked(source).unwrap();
    let Value::Integer(value) =
        interpret::execute(&p, p.main.unwrap(), vec![], OverflowChecks::Checked, 10000).unwrap()
    else {
        panic!()
    };
    assert_eq!(value.signed_value(), Some(8));
}
#[test]
fn concrete_specializations_do_not_grant_undeclared_capabilities() {
    for source in [
  "pub fn add<T>(x:T) -> T {return x+1;} pub fn main() -> i32 {return add(1);}",
  "pub fn cast_value<T>(x:T) -> i32 {return x as i32;} pub fn main() -> i32 {return cast_value(1);}",
  "pub fn convert<T,U>(x:T) -> U {return x;} pub fn main() -> i32 {return convert<i32,i32>(1);}",
  "pub fn needs_copy<T:Copy>(x:T) -> T {return x;} pub fn generic<T>(x:T) -> T {return needs_copy(x);} pub fn main() -> i32 {return generic(1);}",
  "pub fn concrete(x:i32) -> i32 {return x;} pub fn generic<T>(x:T) -> i32 {return concrete(x);} pub fn main() -> i32 {return generic(1);}",
 ] {let errors=checked(source).unwrap_err();assert_eq!(errors[0].code,"E0373","{source}: {errors:?}");}
}
