use super::*;

#[test]
fn scalar_output_and_assertions_preserve_partial_and_complete_cleanup() {
    let prefix =
        "class Item {label:i32;new(label:i32) {self.label=label;} drop() {println(self.label);}}";
    for (constructor, expected, status) in [
        (
            "self.first=Item.new(1);self.second=Item.new(2);",
            "3\n1\n2\n",
            0,
        ),
        (
            "self.first=Item.new(1);self.second=Item.new(2);assert(false);",
            "2\n1\n",
            101,
        ),
    ] {
        let source=format!("{prefix} class Pair {{first:Item;second:Item;new() {{{constructor}}} drop() {{println(3);}}}} fn main() {{let pair=Pair.new();}} ");
        let project = Project::new(&source);
        for release in [false, true] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_clust"));
            command.current_dir(&project.0).args(["run", "main.cl"]);
            if release {
                command.arg("--release");
            }
            let output = command.output().unwrap();
            assert_eq!(
                output.status.code(),
                Some(status),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
        }
    }
    let project=Project::new("fn main() {print(\"value=\");println(7);println(true);println('x');println(1.5);assert(true);}");
    let output = Command::new(env!("CARGO_BIN_EXE_clust"))
        .current_dir(&project.0)
        .args(["run", "main.cl"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "value=7\ntrue\nx\n1.5\n"
    );
}

#[test]
fn rust_adapters_allow_unsafe_calls_without_source_unsafe_and_unwind_through_clust() {
    let declarations="#[rust(\"support.increment\")] #[safety(\"value must be nonnegative\")] extern \"Rust\" unsafe fn increment(value:i32)->i32; #[rust(\"support.calls\")] extern \"Rust\" fn calls()->i32;";
    let project=Project::new(&format!("{declarations} fn main()->i32 {{let f=increment;let value=f(6);assert(calls()==1);return value;}}"));
    fs::create_dir(project.0.join("support")).unwrap();
    fs::write(
        project.0.join("support/Cargo.toml"),
        "[package]\nname='support'\nversion='0.1.0'\nedition='2021'\n[lib]\npath='lib.rs'\n",
    )
    .unwrap();
    fs::write(project.0.join("support/lib.rs"),"use std::sync::atomic::{AtomicI32,Ordering};static CALLS:AtomicI32=AtomicI32::new(0);pub unsafe fn increment(value:i32)->i32 {assert!(value>=0);CALLS.fetch_add(1,Ordering::SeqCst);value+1} pub fn calls()->i32 {CALLS.load(Ordering::SeqCst)} pub fn fail()->i32 {panic!(\"adapter failure\")} ").unwrap();
    fs::write(project.0.join("Clust.toml"),"[package]\nname='adapter-test'\nentry='main.cl'\n[dependencies]\nsupport={path='support'}\n").unwrap();
    for args in [
        vec!["run"],
        vec!["run", "--release"],
        vec!["run", "--release", "--overflow-checks=off"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_clust"))
            .current_dir(&project.0)
            .args(args)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(7),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = Command::new(env!("CARGO_BIN_EXE_clust"))
        .current_dir(&project.0)
        .args(["emit-rust"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let generated = String::from_utf8_lossy(&output.stdout);
    assert!(generated.contains("unsafe {support::increment(a0)}"));
    assert!(generated.contains("Safety contract: value must be nonnegative"));
    let output = Command::new(env!("CARGO_BIN_EXE_clust"))
        .current_dir(&project.0)
        .args(["eval"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ForeignFunction"));
    fs::write(project.0.join("main.cl"),"#[rust(\"support.fail\")] extern \"Rust\" fn fail()->i32;class Guard {new() {} drop() {println(99);}} fn main()->i32 {let guard=Guard.new();return fail();}").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_clust"))
        .current_dir(&project.0)
        .args(["run"])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(101),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "99\n");
}

#[test]
fn approved_runtime_features_execute_in_both_native_profiles() {
    for (source,expected,output) in [
        (include_str!("../../../examples/clust_vec/main.cl").to_owned(),9,String::new()),
        (include_str!("../../../examples/clust_animals/main.cl").to_owned(),5,"20\n10\n30\n10\n".to_owned()),
        (include_str!("../../../examples/clust_dynamic/main.cl").to_owned(),14,String::new()),
        (include_str!("../../../examples/clust_string/main.cl").to_owned(),0,"Hello, 🌍\n".to_owned()),
        (include_str!("../../../examples/clust_weak/main.cl").to_owned(),7,"9\n7\n".to_owned()),
        (include_str!("../../../examples/clust_closures/main.cl").to_owned(),7,String::new()),
        (include_str!("../../../examples/clust_generic_inheritance/main.cl").to_owned(),7,String::new()),
        (include_str!("../../../examples/clust_generic_dynamic/main.cl").to_owned(),10,String::new()),
        (include_str!("../../../examples/clust_captures/main.cl").to_owned(),7,"7\n4\n".to_owned()),
        (include_str!("../../../examples/clust_iteration/main.cl").to_owned(),16,"8\n9\n".to_owned()),
        (include_str!("../../../examples/clust_weak_identity/main.cl").to_owned(),14,"1\n9\n".to_owned()),
        (include_str!("../../../examples/clust_weak_backedge/main.cl").to_owned(),7,"1\n2\n".to_owned()),
        ("use core.vec.{Vec};use core.option.{Option};class Item {pub value:i32;new(n:i32) {self.value=n;} drop() {println(self.value);}} fn main()->i32 {var items=Vec<Item>.new();var n=0;while n<10 {items.push(Item.new(n));n+=1;} items.reserve(32);return match items.pop() {Option.Some(last)=>last.value,Option.None=>0};}".to_owned(),9,"9\n0\n1\n2\n3\n4\n5\n6\n7\n8\n".to_owned()),
    ] {
        let project=Project::new(&source);
        for release in [false,true] {
            let mut command=Command::new(env!("CARGO_BIN_EXE_clust"));
            command.current_dir(&project.0).args(["run","main.cl"]);
            if release {command.arg("--release");}
            let result=command.output().unwrap();
            assert_eq!(result.status.code(),Some(expected),"{}",String::from_utf8_lossy(&result.stderr));
            assert_eq!(String::from_utf8_lossy(&result.stdout),output);
        }
    }
    let project=Project::new("use core.vec.{Vec};class Item {pub value:i32;new(n:i32) {self.value=n;}} fn main()->i32 {var items=Vec<Item>.new();items.push(Item.new(7));let borrowed=items.at(0);items.push(Item.new(8));return borrowed.value;}");
    let result = Command::new(env!("CARGO_BIN_EXE_clust"))
        .current_dir(&project.0)
        .args(["check", "main.cl"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("E0381"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn borrow_diagnostics_show_origin_creation_and_later_use() {
    let project=Project::new("mod lib;use crate.lib.{Cell, id};\nfn main()->i32 {\nvar a=Cell.new(1);\nlet ref source=a;\nlet b=id(source);\na=Cell.new(2);\nreturn b.value;\n}");
    fs::write(
        project.0.join("lib.cl"),
        "pub class Cell {pub value:i32;new(n:i32) {self.value=n;}}\npub fn id<T>(x:T)->T {return x;}",
    )
    .unwrap();
    let output = project.run(&["check", "main.cl"]);
    assert!(!output.status.success());
    let message = String::from_utf8_lossy(&output.stderr);
    for expected in [
        "error[E0381]",
        "main.cl:3:",
        "main.cl:5:",
        "main.cl:7:",
        "borrowed target 'a' originates here",
        "this view is created here",
        "this later use keeps the view live",
    ] {
        assert!(message.contains(expected), "missing {expected}: {message}");
    }
}
