use clust_core::{check::check_clust, hir::Value, interpret};
use clust_frontend::modules::load;
use clust_semantics::OverflowChecks;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Project(PathBuf);
impl Project {
    fn new(files: &[(&str, &str)]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "clust-module-paths-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        for (name, text) in files {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        Self(root)
    }
    fn value(&self) -> i128 {
        let loaded = load(&self.0.join("main.cl"), "app");
        assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
        let program = check_clust(&loaded.module.unwrap()).unwrap();
        let Value::Integer(n) = interpret::execute(
            &program,
            program.main.unwrap(),
            vec![],
            OverflowChecks::Checked,
            100_000,
        )
        .unwrap() else {
            panic!()
        };
        n.signed_value().unwrap()
    }
    fn error(&self) -> String {
        let loaded = load(&self.0.join("main.cl"), "app");
        assert!(!loaded.diagnostics.is_empty());
        loaded.diagnostics[0].code.into()
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn inline_modules_paths_aliases_nested_groups_and_reexports() {
    let project=Project::new(&[("main.cl", "mod animals {pub class Animal {new() {} pub read(self)->i32 {return 4;}} pub mod nested {pub fn n()->i32 {return 3;}}} mod api {pub use crate.animals.{Animal, nested.{n as number}};} use self.api.{Animal as A, number}; fn main()->i32 {let a=A.new();return a.read()+number();}")]);
    assert_eq!(project.value(), 7);
}
#[test]
fn files_mod_files_and_parent_relative_paths() {
    let project = Project::new(&[
        (
            "main.cl",
            "mod math;mod api; fn main()->i32 {return api.answer();}",
        ),
        (
            "math/mod.cl",
            "pub mod constants;pub fn add(n:i32)->i32 {return n+constants.VALUE;}",
        ),
        ("math/constants.cl", "pub const VALUE:i32=3;"),
        (
            "api.cl",
            "use super.math.{self,add};pub fn answer()->i32 {return add(math.constants.VALUE);}",
        ),
    ]);
    assert_eq!(project.value(), 6);
}
#[test]
fn glob_use_and_local_shadowing() {
    let project=Project::new(&[("main.cl","mod numbers {pub const N:i32=7;} use numbers.*;fn main()->i32 {let numbers=2;return numbers+N;}")]);
    assert_eq!(project.value(), 9);
}
#[test]
fn module_visibility_and_missing_declarations_are_checked() {
    for source in [
        "mod a {mod hidden {pub fn n()->i32 {return 7;}}} use a.hidden.n;",
        "mod a {fn n()->i32 {return 7;}} use a.n;",
    ] {
        assert_eq!(Project::new(&[("main.cl", source)]).error(), "E0504");
    }
    assert_eq!(
        Project::new(&[
            ("main.cl", "use undeclared.answer;"),
            ("undeclared.cl", "pub fn answer()->i32 {return 7;}")
        ])
        .error(),
        "E0503"
    );
    assert_eq!(
        Project::new(&[("main.cl", "use super.value;")]).error(),
        "E0501"
    );
}
#[test]
fn ambiguous_file_modules_and_duplicate_aliases_are_rejected() {
    assert_eq!(
        Project::new(&[("main.cl", "mod a;"), ("a.cl", ""), ("a/mod.cl", "")]).error(),
        "E0501"
    );
    assert_eq!(
        Project::new(&[(
            "main.cl",
            "mod a {pub fn n() {}} mod b {pub fn n() {}} use a.n;use b.n;"
        )])
        .error(),
        "E0502"
    );
}

#[test]
fn scoped_visibility_is_checked_on_uses_and_reexports() {
    let project=Project::new(&[("main.cl","mod a {pub(crate) fn n()->i32 {return 3;} mod inner {pub(super) fn n()->i32 {return 4;}} pub fn answer()->i32 {return n()+inner.n();}} fn main()->i32 {return a.answer();}")]);
    assert_eq!(project.value(), 7);
    for source in [
        "mod a {pub mod inner {pub(super) fn n() {}}} use a.inner.n;",
        "mod a {pub(crate) fn n() {}} pub use a.n;",
        "mod a {pub(in crate.unknown) fn n() {}}",
    ] {
        assert_eq!(Project::new(&[("main.cl", source)]).error(), "E0504");
    }
}
#[test]
fn qualified_paths_work_for_values_types_patterns_and_self_modules() {
    let project=Project::new(&[("main.cl","mod a {pub class C {new() {} pub n(self)->i32 {return 4;}} pub enum E {V(i32)} pub fn answer()->i32 {let value:self.C=self.C.new();return match self.E.V(3) {self.E.V(n)=>value.n()+n};}} fn main()->i32 {return a.answer();}")]);
    assert_eq!(project.value(), 7);
}

#[test]
fn explicit_names_shadow_globs_and_ambiguous_globs_fail_when_used() {
    for uses in ["use a.*;use b.N;", "use b.N;use a.*;"] {
        let source=format!("mod a {{pub const N:i32=1;}} mod b {{pub const N:i32=7;}} {uses} fn main()->i32 {{return N;}}");
        assert_eq!(Project::new(&[("main.cl", &source)]).value(), 7);
    }
    assert_eq!(Project::new(&[("main.cl","mod a {pub const N:i32=1;} mod b {pub const N:i32=2;} use a.*;use b.*;fn main()->i32 {return N;}")]).error(),"E0502");
    assert_eq!(Project::new(&[("main.cl","mod a {pub const N:i32=1;} mod b {pub const N:i32=2;} mod api {pub use crate.a.*;pub use crate.b.*;} use api.*;fn main()->i32 {return N;}")]).error(),"E0502");
}
#[test]
fn core_paths_can_be_used_without_imports() {
    assert_eq!(
        Project::new(&[(
            "main.cl",
            "fn main()->i32 {return core.option.unwrap_or(core.option.Option.None,7);}"
        )])
        .value(),
        7
    );
}

#[test]
fn file_children_can_access_private_parent_functions() {
    let project = Project::new(&[
        (
            "main.cl",
            "mod parent;fn main()->i32 {return parent.child.answer();}",
        ),
        ("parent.cl", "pub mod child;fn hidden()->i32 {return 7;}"),
        (
            "parent/child.cl",
            "use super.hidden;pub fn answer()->i32 {return hidden();}",
        ),
    ]);
    assert_eq!(project.value(), 7);
}

#[test]
fn private_names_can_be_aliased_without_widening_their_scope() {
    let project=Project::new(&[("main.cl","mod a {fn hidden()->i32 {return 7;} pub(self) use self.hidden as local;pub fn n()->i32 {return local();}} fn main()->i32 {return a.n();}")]);
    assert_eq!(project.value(), 7);
    assert_eq!(Project::new(&[("main.cl","mod a {pub(crate) fn n() {} use self.n as local;pub(crate) use self.local as exposed;}")]).error(),"E0504");
}

#[test]
fn declaring_core_does_not_grant_compiler_intrinsics() {
    let project=Project::new(&[("main.cl","mod core {pub mod intrinsic {pub fn attempt()->usize {return layout_size<i32>();}}} fn main()->i32 {return 0;}")]);
    let loaded = load(&project.0.join("main.cl"), "app");
    assert!(loaded.diagnostics.is_empty());
    let errors = check_clust(&loaded.module.unwrap()).unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(e.code, "E0310" | "E0370")),
        "{errors:?}"
    );
}
