use super::*;

#[test]
fn imported_virtual_override_diagnostics_preserve_base_declaration() {
    let files=Fixture::new(&[
        ("base.cl","pub class Base<T> {new() {}  pub read(self,value:T) -> T {return value;}}"),
        ("api.cl","pub use crate.base.{Base};"),
        ("main.cl","mod base;mod api;use crate.api.{Base};\npub class Child:Base<i32> {new() {}  pub read(self,value:i32) -> bool {return true;}} pub fn main() -> i32 {return 0;}"),
    ]);
    let loaded = load(&files.0.join("main.cl"), "app");
    assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
    let errors = check(&loaded.module.unwrap()).unwrap_err();
    let diagnostic = &errors[0];
    assert_eq!(diagnostic.code, "E0411", "{errors:?}");
    assert!(diagnostic
        .notes
        .iter()
        .any(|note| note.span.source != diagnostic.span.source));
    fs::write(files.0.join("main.cl"),"mod base;mod api;use crate.api.{Base};\npub class Child:Base<i32> {new() {}  pub read(self,value:i32) -> i32 {return value;}} pub fn main() -> i32 {return Child.new().read(7);}").unwrap();
    let loaded = load(&files.0.join("main.cl"), "app");
    assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
    let errors = check(&loaded.module.unwrap()).unwrap_err();
    assert_eq!(errors[0].code, "E0412", "{errors:?}");
}

#[test]
fn private_member_context_survives_imports_reexports_and_code_pointers() {
    let files = Fixture::new(&[
        ("value.cl", " fn delta() -> i32 {return 1;} pub class C<T:Copy> { value:T;new(n:T) {value=n;};  secret<U:Copy>(x:U) -> U {return x;};pub expose() -> (i32)=>i32 {return C<T>.secret<i32>;};pub read(self) -> T {return value;};drop() {let n=value;let d=delta();}}"),
        ("api.cl", "pub use crate.value.{C};"),
        ("main.cl", "mod value;mod api;use crate.api.{C};\npub fn main() -> i32 {let c=C<i32>.new(3);let f=C<i32>.expose();return c.read()+f(4);}"),
    ]);
    assert_eq!(files.code(), 7);
    for source in [
        "mod api;mod value;use crate.api.{C};\npub fn main() -> i32 {return C<i32>.new(7).value;}",
        "mod api;mod value;use crate.api.{C};\npub fn main() -> i32 {let f:(i32)=>i32=C<i32>.secret<i32>;return f(7);}",
        "mod api;mod value;use crate.api.{C};\npub class Other {new() {} pub read(c:C<i32>) -> i32 {return c.value;}} pub fn main() -> i32 {return Other.read(C<i32>.new(7));}",
    ] {
        fs::write(files.0.join("main.cl"), source).unwrap();
        let loaded = load(&files.0.join("main.cl"), "app");
        assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
        let errors = check(&loaded.module.unwrap()).unwrap_err();
        let diagnostic = errors.iter().find(|d| d.code == "E0352").unwrap();
        assert!(diagnostic.notes.iter().any(|note| note.span.source != diagnostic.span.source), "{errors:?}");
    }
}

#[test]
fn unsafe_contract_survives_imports_and_reexports() {
    let files = Fixture::new(&[
        ("values.cl", " pub fn trusted() -> i32 {requires_unsafe_caller();return 7;}"),
        ("api.cl", "pub use crate.values.{trusted};"),
        (
            "main.cl",
            "mod values;mod api;use crate.api.{trusted};\npub fn main() -> i32 {if trusted_region() {return trusted();}}",
        ),
    ]);
    assert_eq!(files.code(), 7);
    fs::write(
        files.0.join("main.cl"),
        "mod values;mod api;use crate.api.{trusted};\npub fn main() -> i32 {return trusted();}",
    )
    .unwrap();
    let loaded = load(&files.0.join("main.cl"), "app");
    assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
    let errors = check(&loaded.module.unwrap()).unwrap_err();
    assert!(errors.iter().any(|e| e.code == "E0500"), "{errors:?}");
}

#[test]
fn nominal_interface_constraints_survive_imports_and_reexports() {
    let files=Fixture::new(&[
        ("contracts.cl","pub interface Read { read(self) -> i32} pub interface Get<T> { get(self) -> T} use compiler_fixtures.record_120.{U}; pub fn get<T:Get<U>,U:Copy>(x:T) -> U {return x.get();} pub fn read<T:Read>(x:T) -> i32 {return x.read();}"),
        ("values.cl","use crate.contracts.{Read, Get};\npub class S:Read,Get<i32> {pub value:i32;new(n:i32) {value=n;};pub read(self) -> i32 {return value;};pub get(self) -> i32 {return value;}}"),
        ("api.cl","pub use crate.contracts.{Read, read, get};\npub use crate.values.{S};"),
        ("main.cl","mod contracts;mod values;mod api;use crate.api.{S, read, get};\npub fn main() -> i32 {let s=S.new(7);return read(s)+get(s)+s.read();}"),
    ]);
    assert_eq!(files.code(), 21);
    let loaded = load(&files.0.join("main.cl"), "app");
    let p = check(&loaded.module.unwrap()).unwrap();
    let m = clust_core::mir::lower(&p);
    let value = clust_core::mir::interpret::execute(
        &m,
        m.main.unwrap(),
        Vec::new(),
        OverflowChecks::Checked,
        100_000,
    )
    .unwrap();
    let Value::Integer(n) = value else { panic!() };
    assert_eq!(n.signed_value(), Some(21));
    clust_core::rust::emit(&p, OverflowChecks::Checked).unwrap();
    fs::write(
        files.0.join("values.cl"),
        "pub class S {pub value:i32;new(n:i32) {value=n;};pub read(self) -> i32 {return value;}}",
    )
    .unwrap();
    let loaded = load(&files.0.join("main.cl"), "app");
    assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
    let errors = check(&loaded.module.unwrap()).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.code == "E0402" && !e.notes.is_empty()),
        "{errors:?}"
    );
}

#[test]
fn borrowed_classes_destructors_and_callable_contracts_survive_reexports() {
    let files=Fixture::new(&[
        ("api.cl","pub use crate.values.{R, Cell, id, set};"),
        ("values.cl","use compiler_fixtures.record_4.{R}; pub class Cell<T> {pub value:T;new(_borrowed_x:T) {let x=_borrowed_x;value=x;};pub get(self) -> T {return value;};drop() {}} pub fn id<T>(x:T) -> T {return x;} pub fn set<T>(var h:Cell<T>,_borrowed_x:T) -> () {let x=_borrowed_x;h.value=x;}"),
        ("main.cl","mod values;mod api;use crate.api.{R, Cell, id, set};\npub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var c=Cell<R>.new(v);let f:(var Cell<R>,R)=>()=set;f(c,w);let g:(R)=>R=id;let result=g(c.get());return result.value;}"),
    ]);
    assert_eq!(files.code(), 3);
    fs::write(files.0.join("main.cl"),"mod values;mod api;use crate.api.{R, Cell, id, set};\npub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref v=a;let ref w=b;var c=Cell<R>.new(v);let f:(var Cell<R>,R)=>()=set;f(c,w);b=R {value: 2};return c.value.value;}").unwrap();
    let loaded = load(&files.0.join("main.cl"), "app");
    assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
    let diagnostics = check(&loaded.module.unwrap()).unwrap_err();
    assert!(
        diagnostics.iter().any(|d| d.code == "E0381"),
        "{diagnostics:?}"
    );
}

#[test]
fn local_storage_modes_are_inferred_before_imported_generic_calls() {
    let files = Fixture::new(&[
        ("api.cl", "pub use crate.storage.{Holder, relay};"),
        ("storage.cl", "use compiler_fixtures.record_121.{Holder}; pub fn relay<T>(value:T) -> T {return value;}"),
        ("main.cl", "mod storage;mod api;use crate.api.{Holder, relay};\nuse compiler_fixtures.record_4.{R}; pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref w=b;var h=Holder<R> {value:a};h.value=w;let result=relay(h);return result.value.value;}"),
    ]);
    assert_eq!(files.code(), 3);
    fs::write(files.0.join("main.cl"), "mod storage;mod api;use crate.api.{Holder, relay};\nuse compiler_fixtures.record_4.{R}; pub fn main() -> i32 {let b=R {value: 3};let ref w=b;var h=Holder<R> {value:R {value: 7}};h.value=w;return h.value.value;}").unwrap();
    let loaded = load(&files.0.join("main.cl"), "app");
    assert!(loaded.diagnostics.is_empty());
    let diagnostics = check(&loaded.module.unwrap()).unwrap_err();
    assert!(
        diagnostics.iter().any(|d| d.code == "E0396"),
        "{diagnostics:?}"
    );
}

#[test]
fn enum_refinement_survives_imported_storage_specialization() {
    let files = Fixture::new(&[
        ("api.cl", "pub use crate.storage.{R, E, clear};"),
        ("storage.cl", "use compiler_fixtures.record_4.{R}; pub enum E {Item(R),Empty} pub fn clear() -> i32 {let a=R {value: 7};let ref v=a;var current=E.Item(v);current=E.Empty;return match current {E.Item(x)=>x.value,E.Empty=>3};}"),
        ("main.cl", "mod storage;mod api;use crate.api.{R, E, clear};\npub fn main() -> i32 {return clear();}"),
    ]);
    assert_eq!(files.code(), 3);
}

#[test]
fn mutable_storage_contracts_survive_imports_and_generic_reexports() {
    let files = Fixture::new(&[
        ("api.cl", "pub use crate.storage.{Holder, set};"),
        ("storage.cl", "use compiler_fixtures.record_121.{Holder}; pub fn set<T>(var h:Holder<T>,_borrowed_x:T) -> () {let x=_borrowed_x;h.value=x;}"),
        ("main.cl", "mod storage;mod api;use crate.api.{Holder, set};\nuse compiler_fixtures.record_4.{R}; pub fn main() -> i32 {let a=R {value: 7};let b=R {value: 3};let ref v=a;let ref w=b;var h=Holder<R> {value:v};set(h,w);return h.value.value;}"),
    ]);
    assert_eq!(files.code(), 3);
    fs::write(files.0.join("main.cl"), "mod storage;mod api;use crate.api.{Holder, set};\nuse compiler_fixtures.record_4.{R}; pub fn main() -> i32 {let a=R {value: 7};var b=R {value: 3};let ref v=a;let ref w=b;var h=Holder<R> {value:v};set(h,w);b=R {value: 2};return h.value.value;}").unwrap();
    let loaded = load(&files.0.join("main.cl"), "app");
    assert!(loaded.diagnostics.is_empty());
    let diagnostics = check(&loaded.module.unwrap()).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code == "E0381" && d.notes.len() >= 2),
        "{diagnostics:?}"
    );
}
