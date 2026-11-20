//! Check generated Tokio code in an isolated Cargo project, without running user code.
use std::{fs, path::Path, process::Command};
pub fn check(root: &Path, source: &str) -> Result<(), String> {
    let src = root.join("src");
    fs::create_dir_all(&src).map_err(|e| e.to_string())?;
    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "clust-editor-async"
version = "0.0.0"
edition = "2021"
[dependencies]
clust_tokio = { package = "tokio", version = "=1.54.1", features = ["rt-multi-thread", "sync"] }
[workspace]
"#,
    )
    .map_err(|e| e.to_string())?;
    fs::write(src.join("main.rs"), source).map_err(|e| e.to_string())?;
    let output = Command::new("cargo")
        .current_dir(root)
        .args(["check", "--offline", "--message-format=json"])
        .output()
        .map_err(|e| format!("Async type checking requires Cargo: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    let errors = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter(|v| v["message"]["level"] == "error")
        .filter_map(|v| Some(v["message"]["message"].as_str()?.to_owned()))
        .collect::<Vec<_>>();
    Err(if errors.is_empty() {
        String::from_utf8_lossy(&output.stderr).trim().into()
    } else {
        errors.join("\n")
    })
}

pub struct Cache(pub std::path::PathBuf);
impl Cache {
    pub fn create() -> std::io::Result<Self> {
        for n in 0..1000 {
            let p = std::env::temp_dir().join(format!("clust-editor-{}-{n}", std::process::id()));
            match fs::create_dir(&p) {
                Ok(()) => return Ok(Self(p)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(std::io::Error::other("No available async cache directory"))
    }
}
impl Drop for Cache {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
