use super::*;
use std::fs;

fn candidates(dir: &Path, name: &str) -> Vec<PathBuf> {
    ["rs", "cl"]
        .into_iter()
        .flat_map(|ext| {
            [
                dir.join(format!("{name}.{ext}")),
                dir.join(name).join(format!("mod.{ext}")),
            ]
        })
        .filter(|p| p.is_file())
        .collect()
}
pub(super) fn module(dir: &Path, name: &str) -> Result<Option<PathBuf>, String> {
    let candidates = candidates(dir, name);
    match candidates.as_slice() {
        [] => Ok(None),
        [p] => Ok(Some(p.clone())),
        _ => Err(format!("ambiguous module '{name}' in {}", dir.display())),
    }
}
pub(super) fn mixed(
    dir: &Path,
    source: &str,
    deps: &BTreeSet<String>,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<bool, String> {
    let t = tokens(source);
    for (i, word) in t.iter().enumerate() {
        if t.get(i + 1) == Some(&"!")
            && matches!(t.get(i + 2), Some(&"(" | &"[" | &"{"))
            && word.starts_with(|c: char| c.is_alphabetic() || c == '_')
            && !matches!(*word, "return" | "if" | "while")
        {
            return Ok(true);
        }
        if *word == "#" && t.get(i + 1) == Some(&"[") && t.get(i + 2) == Some(&"derive") {
            return Ok(true);
        }
        if (*word == "std" || deps.contains(*word)) && t.get(i + 1) == Some(&".") {
            return Ok(true);
        }
        if *word == "mod" && t.get(i + 2) == Some(&";") {
            let name = t[i + 1];
            if let Some(path) = module(dir, name)? {
                if path.extension().is_some_and(|e| e == "rs") {
                    return Ok(true);
                }
                let path = path.canonicalize().map_err(|e| e.to_string())?;
                if seen.insert(path.clone())
                    && mixed(
                        &dir.join(name),
                        &fs::read_to_string(path).map_err(|e| e.to_string())?,
                        deps,
                        seen,
                    )?
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
pub(super) fn emit(
    dir: &Path,
    source: &str,
    deps: &BTreeSet<String>,
    files: &mut BTreeSet<PathBuf>,
) -> Result<String, String> {
    let t = tokens::mixed(source)?;
    let mut roots = deps.clone();
    roots.extend(["std", "core", "crate", "super", "Self"].map(str::to_owned));
    for w in t.windows(2) {
        if w[0] == "mod" {
            roots.insert(w[1].into());
        }
    }
    let mut out = String::new();
    let mut start = 0;
    for i in 0..t.len().saturating_sub(2) {
        if t[i] != "mod" || t[i + 2] != ";" {
            continue;
        }
        let name = t[i + 1];
        let Some(path) = module(dir, name)? else {
            return Err(format!(
                "missing mixed module '{name}' in {}",
                dir.display()
            ));
        };
        let path = path.canonicalize().map_err(|e| e.to_string())?;
        if !files.insert(path.clone()) {
            return Err(format!("recursive module {}", path.display()));
        }
        let public = i > 0 && t[i - 1] == "pub";
        out.push_str(&lower::source(
            &t[start..if public { i - 1 } else { i }],
            &roots,
        )?);
        let vis = if public { "pub " } else { "" };
        if path.extension().is_some_and(|e| e == "rs") {
            out.push_str(&format!("{vis}mod {name};\n"));
        } else {
            let body = emit(
                &dir.join(name),
                &fs::read_to_string(path).map_err(|e| e.to_string())?,
                deps,
                files,
            )?;
            out.push_str(&format!("{vis}mod {name} {{ {body} }}\n"));
        }
        start = i + 3;
    }
    out.push_str(&lower::source(&t[start..], &roots)?);
    Ok(out)
}
