use super::Project;
use std::{env, fs, path::PathBuf, process::Command};
impl Project {
    pub fn build(&self, source: &str, release: bool) -> Result<PathBuf, String> {
        self.compile_source(source, release, None, None)
            .map(|(path, _)| path)
    }
    pub fn compile_mixed(
        &self,
        program: &crate::interop::Mixed,
        release: bool,
        command: &str,
    ) -> Result<(PathBuf, String, fs::File), String> {
        let lock = super::lock::acquire(&self.root.join("target/clust/mixed"))
            .map_err(|e| e.to_string())?;
        let (artifact, source) = self.compile_source(
            &program.rust,
            release,
            Some(command),
            program.native_items.clone(),
        )?;
        Ok((artifact, source, lock))
    }
    fn compile_source(
        &self,
        source: &str,
        release: bool,
        mixed_command: Option<&str>,
        native_items: Option<std::ops::Range<usize>>,
    ) -> Result<(PathBuf, String), String> {
        let mixed = mixed_command.is_some();
        let directory = self.root.join(if mixed {
            "target/clust/mixed"
        } else {
            "target/clust/generated"
        });
        fs::create_dir_all(directory.join("src")).map_err(|e| e.to_string())?;
        if mixed {
            super::modules::stage(&self.entry, &directory.join("src"))
                .map_err(|e| e.to_string())?;
        }
        let mut package = toml::Table::new();
        for (key, value) in [
            ("name", self.name.as_str()),
            ("version", "0.1.0"),
            ("edition", "2021"),
        ] {
            package.insert(key.into(), toml::Value::String(value.into()));
        }
        let mut manifest = toml::Table::new();
        if mixed {
            package.insert("autobins".into(), false.into());
            let mut bin = toml::Table::new();
            bin.insert("name".into(), self.name.clone().into());
            bin.insert("path".into(), "src/main.rs".into());
            manifest.insert("bin".into(), toml::Value::Array(vec![bin.into()]));
        }
        manifest.insert("package".into(), toml::Value::Table(package));
        manifest.insert("workspace".into(), toml::Value::Table(toml::Table::new()));
        manifest.insert(
            "dependencies".into(),
            toml::Value::Table(self.dependencies.clone()),
        );
        if let Some(checked) = self.async_overflow {
            let mut profiles = toml::Table::new();
            for name in ["dev", "release"] {
                let mut profile = toml::Table::new();
                profile.insert("overflow-checks".into(), checked.into());
                profiles.insert(name.into(), toml::Value::Table(profile));
            }
            manifest.insert("profile".into(), toml::Value::Table(profiles));
        }
        fs::write(directory.join("Cargo.toml"), manifest.to_string()).map_err(|e| e.to_string())?;
        fs::write(directory.join("src/main.rs"), source).map_err(|e| e.to_string())?;
        let target = if mixed {
            directory.join("artifacts")
        } else {
            self.root.join("target/clust/artifacts")
        };
        let mut command =
            Command::new(env::var_os("CLUST_CARGO").unwrap_or_else(|| "cargo".into()));
        command
            .args([
                if matches!(mixed_command, Some("check" | "emit-rust")) {
                    "check"
                } else {
                    "build"
                },
                "--offline",
                "--manifest-path",
            ])
            .arg(directory.join("Cargo.toml"))
            .arg("--target-dir")
            .arg(&target);
        if release {
            command.arg("--release");
        }
        if let Some(compiler) = env::var_os("CLUST_RUSTC") {
            command.env("RUSTC", compiler);
        }
        let mut code = source.to_owned();
        if mixed {
            if let Some(range) = native_items {
                let items = crate::interop::metadata::lower(
                    &directory.join("Cargo.toml"),
                    &target,
                    &source[range.clone()],
                    release,
                )?;
                code.replace_range(range, &items);
                fs::write(directory.join("src/main.rs"), &code).map_err(|e| e.to_string())?;
            }
            command.arg("--message-format=json");
            crate::interop::adapt::build(&mut command, &directory.join("src/main.rs"), &mut code)?;
        } else {
            let output = command
                .output()
                .map_err(|e| format!("cannot start Cargo: {e}"))?;
            if !output.status.success() {
                return Err(format!(
                    "Cargo build failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
        Ok((
            target
                .join(if release { "release" } else { "debug" })
                .join(format!("{}{}", self.name, env::consts::EXE_SUFFIX)),
            code,
        ))
    }
}
