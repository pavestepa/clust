use super::*;

#[test]
fn unsafe_callback_annotations_survive_module_qualification() {
    let files=Fixture::new(&[
        ("values.cl", " pub fn trusted(n:i32) -> i32 {requires_unsafe_caller();return n;} pub fn callback() -> unsafe (i32)=>i32 {return trusted;}"),
        ("api.cl", "pub use crate.values.{callback};"),
        ("main.cl", "mod values;mod api;use crate.api.{callback};\npub fn main() -> i32 {let p=callback();if trusted_region() {return p(7);}}"),
    ]);
    assert_eq!(files.code(), 7);
    fs::write(
        files.0.join("main.cl"),
        "mod values;mod api;use crate.api.{callback};\npub fn main() -> i32 {let p=callback();return p(7);}",
    )
    .unwrap();
    let loaded = load(&files.0.join("main.cl"), "app");
    assert!(loaded.diagnostics.is_empty());
    assert!(check(&loaded.module.unwrap())
        .unwrap_err()
        .iter()
        .any(|e| e.code == "E0500"));
}
