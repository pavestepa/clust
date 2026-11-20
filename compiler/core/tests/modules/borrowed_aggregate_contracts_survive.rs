use super::*;

#[test]
fn borrowed_aggregate_contracts_survive_generic_imports_and_reexports() {
    let files = Fixture::new(&[
        ("api.cl", "pub use crate.storage.{Holder, wrap, get};"),
        ("storage.cl", "use compiler_fixtures.record_121.{Holder}; pub fn wrap<T>(_borrowed_x:T) -> Holder<T> {let x=_borrowed_x;return Holder<T> {value: x};} pub fn get<T>(x:Holder<T>) -> T {return x.value;}"),
        ("main.cl", "mod storage;mod api;use crate.api.{Holder, wrap, get};\nuse compiler_fixtures.record_4.{R}; pub fn main() -> i32 {let a=R {value: 7};let ref va=a;let h=wrap(va);let result=get(h);return a.value+result.value;}"),
    ]);
    assert_eq!(files.code(), 14);
    fs::write(files.0.join("main.cl"), "mod storage;mod api;use crate.api.{Holder, wrap, get};\nuse compiler_fixtures.record_4.{R}; pub fn main() -> i32 {var a=R {value: 7};let ref v=a;let h=wrap(v);let result=get(h);a=R {value: 2};return result.value;}").unwrap();
    let loaded = load(&files.0.join("main.cl"), "app");
    assert!(loaded.diagnostics.is_empty());
    let diagnostics = check(&loaded.module.unwrap()).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code == "E0381" && !d.notes.is_empty()),
        "{diagnostics:?}"
    );
}

#[test]
fn imports_and_reexports_resolve_before_type_checking() {
    let files = Fixture::new(&[
        (
            "main.cl",
            "mod math;mod api;use crate.api.{add, VALUE};\npub fn main() -> i32{return add(VALUE,1);}",
        ),
        ("api.cl", "pub use crate.math.{add, VALUE};"),
        (
            "math.cl",
            "pub const VALUE:i32=6;\npub fn add(a:i32,b:i32) -> i32{return a+b;}",
        ),
    ]);
    assert_eq!(files.code(), 7);
}

#[test]
fn cyclic_declarations_and_mutual_calls_are_allowed() {
    let files=Fixture::new(&[
        ("main.cl","mod odd;mod even;use crate.even.{even};\npub fn main() -> i32{return if even(10) {1} else {0};}"),
        ("even.cl","use crate.odd.{odd};\npub fn even(n:i32) -> bool {if n==0 {return true;};return odd(n-1);}"),
        ("odd.cl","use crate.even.{even};\npub fn odd(n:i32) -> bool {if n==0 {return false;};return even(n-1);}"),
    ]);
    assert_eq!(files.code(), 1);
}

#[test]
fn lexical_shadowing_and_source_ids_are_preserved() {
    let files = Fixture::new(&[
        (
            "main.cl",
            "mod value;use crate.value.{VALUE};\npub fn main() -> i32 {let VALUE=VALUE+1;return VALUE;}",
        ),
        ("value.cl", "pub const VALUE:i32=5;"),
    ]);
    assert_eq!(files.code(), 6);
    fs::write(files.0.join("value.cl"), "pub const VALUE:i32=missing").unwrap();
    let loaded = load(&files.0.join("main.cl"), "app");
    let errors = check(&loaded.module.unwrap()).unwrap_err();
    let source = &loaded.sources[errors[0].span.source.0];
    assert!(source.name.ends_with("value.cl"));
}

#[test]
fn private_missing_conflicting_and_unfounded_cyclic_exports_are_rejected() {
    for (main, other) in [
        ("use crate.other.{hidden};", " fn hidden() -> (){}"),
        ("use crate.other.{missing};", "pub fn present() -> (){}"),
        (
            "use crate.other.{f};\npub fn f() -> (){}",
            "pub fn f() -> (){}",
        ),
        ("pub use crate.other.{x};", "pub use crate.main.{x};"),
    ] {
        let files = Fixture::new(&[("main.cl", main), ("other.cl", other)]);
        let loaded = load(&files.0.join("main.cl"), "app");
        assert!(loaded.module.is_none());
        assert!(!loaded.diagnostics.is_empty());
    }
}

#[test]
fn package_root_escape_is_rejected_before_reading_the_target() {
    let files = Fixture::new(&[("main.cl", "use super.outside.{x};")]);
    let loaded = load(&files.0.join("main.cl"), "app");
    assert_eq!(loaded.diagnostics[0].code, "E0501");
}

#[test]
fn static_methods_preserve_module_lookup_and_private_member_visibility() {
    let fixture = Fixture::new(&[
        (
            "main.cl",
            "mod lib;use crate.lib.{Box};\npub fn main() -> i32 {return Box<i32>.create(8).value;}",
        ),
        (
            "lib.cl",
            "pub fn value() -> i32 {return 1;} use compiler_fixtures.record_122.{Box};",
        ),
    ]);
    assert_eq!(fixture.code(), 8);
    fs::write(
        fixture.0.join("main.cl"),
        "mod lib;use crate.lib.{Box};\npub fn main() -> i32 {return Box<i32>.secret();}",
    )
    .unwrap();
    let loaded = load(&fixture.0.join("main.cl"), "app");
    assert!(loaded.diagnostics.is_empty());
    let errors = check(&loaded.module.unwrap()).unwrap_err();
    assert_eq!(errors[0].code, "E0352");
}

#[test]
fn imported_generic_destructor_uses_private_definition_context() {
    let files=Fixture::new(&[
        ("main.cl","mod resource;use crate.resource.{R}; pub fn main() -> i32 {let a=R<i32>.make(7);return a.read();}"),
        ("resource.cl","use compiler_fixtures.record_123.{R};"),
    ]);
    assert_eq!(files.code(), 7);
    let loaded = load(&files.0.join("main.cl"), "app");
    let program = check(&loaded.module.unwrap()).unwrap();
    let mir = clust_core::mir::lower(&program);
    let (a, ae) = clust_core::interpret::execute_traced(
        &program,
        program.main.unwrap(),
        vec![],
        OverflowChecks::Checked,
        10000,
    );
    let (b, be) = clust_core::mir::interpret::execute_traced(
        &mir,
        mir.main.unwrap(),
        vec![],
        OverflowChecks::Checked,
        10000,
    );
    assert_eq!(a, b);
    assert_eq!(ae, be);
    assert_eq!(ae.len(), 1);
    clust_core::rust::emit(&program, OverflowChecks::Checked).unwrap();
}

#[test]
fn imported_class_constructor_methods_and_destructor_keep_definition_context() {
    let files=Fixture::new(&[
        ("main.cl","mod cell;use crate.cell.{Cell}; pub fn main() -> i32 {var c=Cell<i32>.new(4);c.bump();return c.read();}"),
        ("cell.cl"," fn delta() -> i32 {return 3;} pub class Cell<T:Copy> { value:T; n:i32;new(v:T) {value=v;n=delta();};pub bump(var self) -> () {n+=1;};pub read(self) -> i32 {return n;};drop() {let x=value;}}"),
    ]);
    assert_eq!(files.code(), 4);
    fs::write(
        files.0.join("main.cl"),
        "mod cell;use crate.cell.{Cell}; pub fn main() -> i32 {let c=Cell<i32>.new(4);return c.n;}",
    )
    .unwrap();
    let loaded = load(&files.0.join("main.cl"), "app");
    assert!(check(&loaded.module.unwrap())
        .unwrap_err()
        .iter()
        .any(|e| e.code == "E0352"));
}

#[test]
fn imported_generic_borrowed_results_preserve_lifetime_contracts() {
    let files=Fixture::new(&[
        ("main.cl","mod implementation;mod api;use crate.api.{Cell, relay};\npub fn main() -> i32 {let a=Cell {value: 7};let b=relay(a);return a.value+b.value;}"),
        ("api.cl","pub use crate.implementation.{Cell, relay};"),
        ("implementation.cl","use compiler_fixtures.record_5.{Cell}; pub fn id<T>(x:T) -> T {return x;} pub fn relay<T>(x:T) -> T {return id(x);}"),
    ]);
    assert_eq!(files.code(), 14);
    fs::write(files.0.join("main.cl"),"mod implementation;mod api;use crate.api.{Cell, relay};\npub fn main() -> i32 {var a=Cell {value: 7};let ref x=a;let b=relay(x);a=Cell {value: 8};return b.value;}").unwrap();
    let loaded = load(&files.0.join("main.cl"), "app");
    assert!(loaded.diagnostics.is_empty());
    let errors = check(&loaded.module.unwrap()).unwrap_err();
    assert!(errors.iter().any(|e| e.code == "E0381"), "{errors:?}");
}
