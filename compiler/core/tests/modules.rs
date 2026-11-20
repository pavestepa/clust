use clust_core::{check::check, hir::Value, interpret::execute};
use clust_frontend::testing::load_ir as load;
use clust_semantics::OverflowChecks;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(files: &[(&str, &str)]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "clust-modules-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        for (path, text) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        Self(root)
    }
    fn code(&self) -> i128 {
        let loaded = load(&self.0.join("main.cl"), "app");
        assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
        let program = check(&loaded.module.unwrap()).unwrap();
        let Value::Integer(value) = execute(
            &program,
            program.main.unwrap(),
            Vec::new(),
            OverflowChecks::Checked,
            10000,
        )
        .unwrap() else {
            panic!()
        };
        value.signed_value().unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[path = "modules/borrowed_aggregate_contracts_survive.rs"]
mod borrowed_aggregate_contracts_survive;
#[path = "modules/imported_virtual_override_diagnostics.rs"]
mod imported_virtual_override_diagnostics;
#[path = "modules/unsafe_callback_annotations_survive.rs"]
mod unsafe_callback_annotations_survive;
