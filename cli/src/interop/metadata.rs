//! rustdoc JSON supplies expanded signatures. Cargo owns feature selection,
//! freshness, dependency renames and the compiler-version-specific cache.
use super::contracts::Catalog;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    ffi::OsStr,
    fs,
    path::Path,
    process::Command,
};

pub(crate) fn lower(
    manifest: &Path,
    target: &Path,
    source: &str,
    release: bool,
) -> Result<String, String> {
    let cargo = env::var_os("CLUST_CARGO").unwrap_or_else(|| "cargo".into());
    let mut graph = Command::new(&cargo);
    graph
        .args([
            "metadata",
            "--format-version",
            "1",
            "--offline",
            "--manifest-path",
        ])
        .arg(manifest);
    let info: Value = serde_json::from_slice(&run(&mut graph)?).map_err(|e| e.to_string())?;
    let root = info["resolve"]["root"]
        .as_str()
        .ok_or("missing Cargo root")?;
    let node = info["resolve"]["nodes"]
        .as_array()
        .and_then(|v| v.iter().find(|n| n["id"] == root))
        .ok_or("missing Cargo dependency graph")?;
    let mut aliases = BTreeMap::from([(root.to_owned(), String::new())]);
    for dep in node["deps"].as_array().into_iter().flatten() {
        if let (Some(id), Some(alias)) = (dep["pkg"].as_str(), dep["name"].as_str()) {
            if alias != "clust_tokio" {
                aliases.insert(id.into(), alias.into());
            }
        }
    }
    let mut cmd = Command::new(cargo);
    cmd.args([
        "doc",
        "--offline",
        "--no-deps",
        "--document-private-items",
        "--manifest-path",
    ])
    .arg(manifest)
    .arg("--target-dir")
    .arg(target);
    if release {
        cmd.arg("--release");
    }
    let mut outputs = vec![];
    for pkg in info["packages"].as_array().into_iter().flatten() {
        let Some(id) = pkg["id"].as_str() else {
            continue;
        };
        let Some(prefix) = aliases.get(id) else {
            continue;
        };
        cmd.arg("-p").arg(id);
        for t in pkg["targets"].as_array().into_iter().flatten() {
            if t["kind"].as_array().is_some_and(|k| {
                k.iter()
                    .any(|s| s == "lib" || s == "proc-macro" || (id == root && s == "bin"))
            }) {
                if let Some(name) = t["name"].as_str() {
                    outputs.push((name.replace('-', "_"), prefix));
                }
            }
        }
    }
    cmd.env(
        "CLUST_METADATA_RUSTDOC",
        env::var_os("RUSTDOC").unwrap_or_else(|| "rustdoc".into()),
    );
    cmd.env("RUSTDOC", env::current_exe().map_err(|e| e.to_string())?);
    cmd.env("RUSTDOCFLAGS", "-Z unstable-options --output-format json");
    if let Some(rustc) = env::var_os("CLUST_RUSTC") {
        cmd.env("RUSTC", rustc);
    }
    run(&mut cmd)?;
    let mut catalog = Catalog::default();
    for (name, prefix) in outputs {
        let path = target.join("doc").join(format!("{name}.json"));
        let doc: Value = serde_json::from_slice(
            &fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
        )
        .map_err(|e| e.to_string())?;
        if doc["format_version"].as_u64().is_none() || !doc["index"].is_object() {
            return Err("unsupported rustdoc item metadata".into());
        }
        visit(
            &doc,
            &doc["root"],
            prefix,
            &mut BTreeSet::new(),
            &mut catalog,
        )?;
    }
    let mut file = syn::parse_file(source).map_err(|e| e.to_string())?;
    catalog.validate(&file)?;
    crate::async_program::lower_with_foreign(&mut file, &|c, i| catalog.borrowed_input(c, i))?;
    Ok(quote::quote!(#file).to_string())
}
fn visit(
    doc: &Value,
    id: &Value,
    prefix: &str,
    stack: &mut BTreeSet<u64>,
    catalog: &mut Catalog,
) -> Result<(), String> {
    let Some(id) = id.as_u64() else { return Ok(()) };
    if !stack.insert(id) {
        return Ok(());
    }
    let item = &doc["index"][id.to_string()];
    let inner = &item["inner"];
    if let Some(items) = inner["module"]["items"].as_array() {
        for child in items {
            let c = &doc["index"][child.to_string()];
            let u = &c["inner"]["use"];
            let Some(name) = c["name"].as_str().or_else(|| u["name"].as_str()) else {
                continue;
            };
            let key = if prefix.is_empty() {
                name.to_owned()
            } else {
                format!("{prefix}::{name}")
            };
            if u.is_null() {
                visit(doc, child, &key, stack, catalog)?
            } else if u["is_glob"] == true {
                visit(doc, &u["id"], prefix, stack, catalog)?
            } else {
                visit(doc, &u["id"], &key, stack, catalog)?
            }
        }
    } else if let Some(items) = inner["trait"]["items"].as_array() {
        associated(doc, prefix, items, catalog)?;
    } else if let Some(impls) = ["struct", "enum", "union"]
        .iter()
        .find_map(|k| inner[k]["impls"].as_array())
    {
        for id in impls {
            let implementation = &doc["index"][id.to_string()]["inner"]["impl"];
            if implementation["trait"].is_null() {
                if let Some(items) = implementation["items"].as_array() {
                    associated(doc, prefix, items, catalog)?;
                }
            }
        }
    } else if !inner["function"].is_null() {
        catalog.expanded(prefix, &inner["function"])?;
    }
    stack.remove(&id);
    Ok(())
}
fn associated(
    doc: &Value,
    prefix: &str,
    items: &[Value],
    catalog: &mut Catalog,
) -> Result<(), String> {
    for id in items {
        let member = &doc["index"][id.to_string()];
        if let (Some(name), Some(f)) = (member["name"].as_str(), member["inner"].get("function")) {
            catalog.expanded(&format!("{prefix}::{name}"), f)?;
        }
    }
    Ok(())
}
fn run(cmd: &mut Command) -> Result<Vec<u8>, String> {
    let out = cmd.output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}
/// Only the documentation frontend opts into its unstable JSON output format.
/// Cargo, proc-macro compilation and the application rustc keep their toolchain settings.
pub fn rustdoc(tool: &OsStr) -> Result<std::process::ExitStatus, String> {
    let mut cmd = Command::new(tool);
    cmd.args(env::args_os().skip(1))
        .env_remove("CLUST_METADATA_RUSTDOC");
    if env::var_os("RUSTC_BOOTSTRAP").is_none() {
        cmd.env("RUSTC_BOOTSTRAP", "1");
    }
    cmd.status().map_err(|e| e.to_string())
}
