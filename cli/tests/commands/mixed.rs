use super::*;
fn result(p: &Project, command: &str, target: &str) -> Output {
    p.run(&[command, target])
}
fn expect(out: Output, code: i32) {
    assert_eq!(
        out.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
fn rejects(out: Output, why: &str) {
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(why), "{stderr}");
}

#[test]
fn native_ownership_and_borrowed_results() {
    let p = Project::new(include_str!("../../../examples/mixed_rust/main.cl"));
    fs::write(
        p.0.join("device.rs"),
        include_str!("../../../examples/mixed_rust/device.rs"),
    )
    .unwrap();
    expect(result(&p, "run", "main.cl"), 7);
    fs::write(p.0.join("main.cl"),"mod device;fn size(h:device.Handle)->i32{return device.read(h).len() as i32;} fn main()->i32{let h=device.make();return size(h)+size(h);}").unwrap();
    expect(result(&p, "run", "main.cl"), 10);
    fs::write(p.0.join("main.cl"),"mod device; class Driver {new(){} pub append(self,var h:device.Handle){device.append(h);}} fn main()->i32{var h=device.make();Driver.new().append(h);return device.read(h).len() as i32;}").unwrap();
    expect(result(&p, "run", "main.cl"), 6);
    fs::write(p.0.join("helpers.cl"), "pub fn size(h:crate.device.Handle)->i32{return crate.device.read(h).len() as i32;} pub fn append(var h:crate.device.Handle){crate.device.append(h);}").unwrap();
    fs::write(p.0.join("main.cl"),"mod device;mod helpers;fn main()->i32{var h=device.make();helpers.append(h);return helpers.size(h)+helpers.size(h);}").unwrap();
    expect(result(&p, "run", "main.cl"), 12);
    for (body, error) in [
        ("device.close(h); device.read(h);", "moved value"),
        (
            "let s=device.read(h);device.append(h);return s.len() as i32;",
            "live",
        ),
        ("device.overlap(h,h);", "overlapping borrows"),
    ] {
        let rust = include_str!("../../../examples/mixed_rust/device.rs").to_owned()
            + "\npub fn overlap(_: &mut Handle,_: &Handle) {}";
        fs::write(p.0.join("device.rs"), rust).unwrap();
        fs::write(
            p.0.join("main.cl"),
            format!("mod device;fn main()->i32 {{var h=device.make();{body} return 0;}}"),
        )
        .unwrap();
        rejects(result(&p, "check", "main.cl"), error);
    }
}
#[path = "mixed/futures.rs"]
mod futures;
#[test]
fn dependencies_traits_generics_and_macro_expansion() {
    let p = Project::new(
        r#"
use api.Score;
#[derive(Clone, Debug)]
#[macros.add_method]
class Point {pub x:i32 new(x:i32){self.x=x;}}
fn main()->i32 {
    var values=Vec<i32>.new();
    values.push(api.identity(api.BASE));
    let p=Point.new(values[0]).clone();
    let n=api.score(p.x);
    let generated=api.Generated.new();
    return generated.value() + macros.number!() + n.unwrap_or(0) + p.generated() + p.x.score();
}"#,
    );
    fs::create_dir(p.0.join("api")).unwrap();
    fs::create_dir(p.0.join("macros")).unwrap();
    fs::write(p.0.join("Clust.toml"),"[package]\nname='mixed'\nentry='main.cl'\n[dependencies]\napi={package='native-api',path='api',features=['number']}\nmacros={path='macros'}").unwrap();
    fs::write(p.0.join("api/Cargo.toml"),"[package]\nname='native-api'\nversion='0.1.0'\nedition='2021'\n[features]\nnumber=[]\n[lib]\npath='lib.rs'").unwrap();
    fs::write(p.0.join("api/lib.rs"),r#"
macro_rules! api {()=>{#[cfg(feature="number")] pub const BASE:i32=2; pub struct Generated; impl Generated{pub fn new()->Self{Self}pub fn value(&self)->i32{7}} pub fn identity<T>(v:T)->T{v}}} api!();
pub trait Score {fn score(self)->i32;} impl Score for i32{fn score(self)->i32{self+1}}
pub fn score<T:Score>(v:T)->Result<i32,()>{Ok(v.score())}
"#).unwrap();
    fs::write(p.0.join("macros/Cargo.toml"),"[package]\nname='macros'\nversion='0.1.0'\nedition='2021'\n[lib]\nproc-macro=true\npath='lib.rs'").unwrap();
    fs::write(p.0.join("macros/lib.rs"),r#"
extern crate proc_macro;
use proc_macro::TokenStream;
#[proc_macro_attribute] pub fn add_method(_:TokenStream,item:TokenStream)->TokenStream{
 let mut out=item;out.extend("impl Point {pub fn generated(&self)->i32{4}}".parse::<TokenStream>().unwrap());out
}
#[proc_macro] pub fn number(_:TokenStream)->TokenStream{"5".parse().unwrap()}
"#).unwrap();
    expect(result(&p, "run", "."), 22);
}
#[test]
fn nested_mixed_modules_and_collisions() {
    let p = Project::new("mod layer; fn main()->i32{return layer.twice(21);}");
    fs::write(
        p.0.join("layer.cl"),
        "mod math;pub fn twice(n:i32)->i32{return math.double(n);}",
    )
    .unwrap();
    fs::create_dir(p.0.join("layer")).unwrap();
    fs::create_dir(p.0.join("layer/math")).unwrap();
    fs::write(
        p.0.join("layer/math.rs"),
        "mod inner; pub use inner::double;",
    )
    .unwrap();
    fs::write(
        p.0.join("layer/math/inner.rs"),
        "pub fn double(n:i32)->i32{n*2}",
    )
    .unwrap();
    expect(result(&p, "run", "main.cl"), 42);
    fs::write(p.0.join("layer/math.cl"), "").unwrap();
    rejects(result(&p, "check", "main.cl"), "ambiguous module");
}

#[test]
fn expanded_signatures_protect_lifetimes_and_drop_on_unwind() {
    let p = Project::new(
        "mod native; fn main()->i32{var h=native.make(); native.macro_pair(h,h);return 0;}",
    );
    fs::write(
        p.0.join("native.rs"),
        r#"
pub struct Handle(String);
impl Drop for Handle { fn drop(&mut self){println!("released-handle");} }
pub fn make()->Handle{Handle(String::new())}
macro_rules! declare {()=>{pub fn macro_pair(_: &Handle,_:&mut Handle){} }} declare!();
pub fn select<'a,'b>(_: &'a Handle,b:&'b Handle)->&'b str{&b.0}
pub fn finish(_:Handle){}
pub fn fail(){panic!("expected native panic")}
"#,
    )
    .unwrap();
    rejects(result(&p, "check", "main.cl"), "E0381");
    fs::write(p.0.join("main.cl"),"mod native; fn main()->i32{let a=native.make();let b=native.make();let s=native.select(a,b);native.finish(a);return s.len() as i32;}").unwrap();
    expect(result(&p, "run", "main.cl"), 0);
    fs::write(p.0.join("main.cl"),"mod native; fn main()->i32{let a=native.make();let b=native.make();let s=native.select(a,b);native.finish(b);return s.len() as i32;}").unwrap();
    rejects(result(&p, "check", "main.cl"), "borrow");
    fs::write(
        p.0.join("main.cl"),
        "mod native;fn main()->i32{let h=native.make();native.fail();return 0;}",
    )
    .unwrap();
    let output = result(&p, "run", "main.cl");
    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|s| *s == "released-handle")
            .count(),
        1
    );
}
#[test]
fn mixed_scalar_imports_preserve_clust_inheritance() {
    let p=Project::new("mod math; class Base {new(){} pub n(self)->i32{return 1;}} class Child:Base {new(){super.new();}pub n(self)->i32{return 3;}} fn count(x:Base)->i32{return math.twice(x.n());} fn main()->i32{let c=Child.new();return count(c);}");
    fs::write(p.0.join("math.rs"), "pub fn twice(n:i32)->i32{n*2}").unwrap();
    expect(result(&p, "run", "main.cl"), 6);
    expect(p.run(&["run", "main.cl", "--release"]), 6);
    rejects(p.run(&["build", "main.cl", "-o", "math.rs"]), "overwrite");
}

#[test]
fn generic_derived_class_runs_its_destructor() {
    let p = Project::new(
        r#"#[derive(Clone)] class Boxed<T> {
        pub value:T new(value:T){self.value=value;} drop(){println(9);}
    }
    fn main()->i32 {let a=Boxed<i32>.new(3);let b=a.clone();return b.value;}"#,
    );
    let out = result(&p, "run", "main.cl");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "9\n9\n",
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    expect(out, 3);
}

#[test]
fn clust_standard_types_cross_native_boundaries() {
    let p = Project::new(
        r#"mod api;use core.vec.{Vec};use core.string.{String};use core.option.{Option,is_some};
    fn main()->i32{var v=Vec<i32>.new();v.push(9);let s=String.from_native(api.label());let n=api.length(s);
       let address=api.address(v);let result=api.pop(v,address);assert(is_some(result));return n+result.unwrap();}"#,
    );
    fs::write(p.0.join("api.rs"),"pub fn label()->String{String::from(\"abc\")} pub fn length(s:&str)->i32{s.len() as i32}pub fn address(v:&Vec<i32>)->usize{v.as_ptr() as usize}pub fn pop(mut v:Vec<i32>,address:usize)->Option<i32>{assert_eq!(v.as_ptr() as usize,address);v.pop()}").unwrap();
    expect(result(&p, "run", "main.cl"), 12);
}
