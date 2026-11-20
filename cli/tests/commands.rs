use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Project(PathBuf);
impl Project {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "clust-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("main.cl"), source).unwrap();
        Self(path)
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_clust"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[path = "commands/captures_and_weak_callbacks.rs"]
mod captures_and_weak_callbacks;
#[path = "commands/published_examples_and_benchmarks.rs"]
mod published_examples_and_benchmarks;
#[path = "commands/run_compiles_clust_syntax.rs"]
mod run_compiles_clust_syntax;
#[path = "commands/scalar_output_and_assertions.rs"]
mod scalar_output_and_assertions;

#[path = "commands/mixed.rs"]
mod mixed;
