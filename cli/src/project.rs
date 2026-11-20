//! Minimal application manifests and Cargo-backed native builds.
mod build;
mod lock;
mod modules;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Project {
    pub root: PathBuf,
    pub entry: PathBuf,
    pub name: String,
    dependencies: toml::Table,
    async_overflow: Option<bool>,
}
impl Project {
    pub fn resolve(path: &Path) -> Result<Option<Self>, String> {
        let manifest = if path.is_dir() {
            path.join("Clust.toml")
        } else if path.file_name().is_some_and(|name| name == "Clust.toml") {
            path.to_path_buf()
        } else {
            return Ok(None);
        };
        let manifest = manifest
            .canonicalize()
            .map_err(|e| format!("{}: {e}", manifest.display()))?;
        let text =
            fs::read_to_string(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
        let table: toml::Table = text
            .parse()
            .map_err(|e| format!("{}: {e}", manifest.display()))?;
        for key in table.keys() {
            if !["package", "dependencies"].contains(&key.as_str()) {
                return Err(format!("unsupported manifest section '{key}'"));
            }
        }
        let package = table
            .get("package")
            .and_then(toml::Value::as_table)
            .ok_or("manifest requires [package]")?;
        for key in package.keys() {
            if !["name", "entry"].contains(&key.as_str()) {
                return Err(format!("unsupported package key '{key}'"));
            }
        }
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .ok_or("package.name must be a string")?;
        if name.is_empty()
            || !name.as_bytes()[0].is_ascii_alphabetic()
            || !name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        {
            return Err("package.name must start with an ASCII letter and contain only letters, digits, '-' or '_'".into());
        }
        let entry = match package.get("entry") {
            None => "src/main.cl",
            Some(value) => value.as_str().ok_or("package.entry must be a string")?,
        };
        let root = manifest.parent().unwrap().to_path_buf();
        let entry = root
            .join(entry)
            .canonicalize()
            .map_err(|e| format!("package entry: {e}"))?;
        if entry.extension().is_none_or(|extension| extension != "cl") {
            return Err("package entry must be a .cl file".into());
        }
        let mut dependencies = match table.get("dependencies") {
            None => toml::Table::new(),
            Some(value) => value
                .as_table()
                .ok_or("dependencies must be a table")?
                .clone(),
        };
        for (name, dependency) in &mut dependencies {
            if let Some(table) = dependency.as_table_mut() {
                if let Some(path) = table.get_mut("path") {
                    let relative = path
                        .as_str()
                        .ok_or_else(|| format!("dependency '{name}' path must be a string"))?;
                    let absolute = root
                        .join(relative)
                        .canonicalize()
                        .map_err(|e| format!("dependency '{name}': {e}"))?;
                    *path = toml::Value::String(
                        absolute
                            .to_str()
                            .ok_or("dependency path must be UTF-8")?
                            .into(),
                    );
                }
            }
        }
        Ok(Some(Self {
            root,
            entry,
            name: name.into(),
            dependencies,
            async_overflow: None,
        }))
    }

    pub fn dependency_names(&self) -> impl Iterator<Item = String> + '_ {
        self.dependencies.keys().map(|name| name.replace('-', "_"))
    }
}

impl Project {
    pub fn standalone(root: PathBuf, entry: PathBuf) -> Self {
        Self {
            root,
            entry,
            name: "clust_application".into(),
            dependencies: toml::Table::new(),
            async_overflow: None,
        }
    }
    pub fn with_overflow(mut self, checked: bool) -> Self {
        self.async_overflow = Some(checked);
        self
    }
    pub fn with_tokio(mut self, checked: bool) -> Result<Self, String> {
        self.async_overflow = Some(checked);
        let dependency: toml::Table = "version = '=1.54.1'\nfeatures = ['rt-multi-thread','sync']"
            .parse()
            .map_err(|e: toml::de::Error| e.to_string())?;
        if self
            .dependencies
            .insert(
                "clust_tokio".into(),
                toml::Value::Table({
                    let mut dependency = dependency;
                    dependency.insert("package".into(), "tokio".into());
                    dependency
                }),
            )
            .is_some()
        {
            return Err("clust_tokio is reserved by the async backend".into());
        }
        Ok(self)
    }
}
