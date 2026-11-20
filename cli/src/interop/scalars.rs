//! Keep the full Clust language pipeline for local Rust scalar APIs. Expose
//! compiler-generated declarations through the loader's existing overlay API.
use super::{modules::module, tokens, Mixed};
use clust_frontend::{modules::load_buffers, parser::parse};
use clust_semantics::OverflowChecks;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
#[derive(Default)]
struct Imports {
    root_dir: Option<PathBuf>,
    deps: BTreeSet<String>,
    buffers: BTreeMap<PathBuf, String>,
    modules: String,
    files: BTreeSet<PathBuf>,
}
impl Imports {
    fn collect(&mut self, dir: &Path, source: &str) -> Result<bool, String> {
        let mut definitions = String::new();
        let t = tokens(source);
        if t.windows(2)
            .any(|w| (w[0] == "std" || self.deps.contains(w[0])) && w[1] == ".")
            || t.windows(3).any(|w| w == ["#", "[", "derive"])
        {
            return Ok(false);
        }
        for t in t.windows(3).filter(|w| w[0] == "mod" && w[2] == ";") {
            let name = t[1];
            let Some(path) = module(dir, name)? else {
                continue;
            };
            let path = path.canonicalize().map_err(|e| e.to_string())?;
            if !self.files.insert(path.clone()) {
                return Ok(false);
            }
            let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if path.extension().is_some_and(|e| e == "cl") {
                if !self.collect(&dir.join(name), &text)? {
                    return Ok(false);
                }
            } else {
                if dir != self.root_dir.as_deref().unwrap_or(dir) {
                    return Ok(false);
                }
                let module_name = name;
                let Some(declarations) = declarations(&text, module_name) else {
                    return Ok(false);
                };
                self.buffers
                    .insert(dir.join(format!("{name}.cl")), declarations);
                definitions.push_str(&format!("mod {module_name};\n"));
            }
        }
        self.modules.push_str(&definitions);
        Ok(true)
    }
}
fn scalar(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Tuple(t) if t.elems.is_empty() => Some("()".into()),
        syn::Type::Path(p) => {
            let n = p.path.get_ident()?.to_string();
            [
                "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
                "usize", "f32", "f64", "bool", "char",
            ]
            .contains(&n.as_str())
            .then_some(n)
        }
        _ => None,
    }
}
fn declarations(text: &str, root: &str) -> Option<String> {
    let file = syn::parse_file(text).ok()?;
    let mut out = String::new();
    if !file.attrs.is_empty() {
        return None;
    }
    for item in file.items {
        let syn::Item::Fn(f) = item else { return None };
        if !matches!(f.vis, syn::Visibility::Public(_)) {
            continue;
        }
        let s = f.sig;
        if s.unsafety.is_some()
            || s.asyncness.is_some()
            || !s.generics.params.is_empty()
            || !f.attrs.is_empty()
        {
            return None;
        }
        let mut args = vec![];
        for (i, a) in s.inputs.iter().enumerate() {
            let syn::FnArg::Typed(a) = a else { return None };
            args.push(format!("arg{i}:{}", scalar(&a.ty)?));
        }
        let result = match s.output {
            syn::ReturnType::Type(_, t) => scalar(&t)?,
            _ => "()".into(),
        };
        let name = s.ident;
        out.push_str(&format!(
            "#[rust(\"{root}.{name}\")] pub extern \"Rust\" fn {name}({})->{result};\n",
            args.join(",")
        ));
    }
    Some(out)
}
pub(super) fn compile(
    root: &Path,
    text: &str,
    checks: OverflowChecks,
    deps: &BTreeSet<String>,
) -> Result<Option<Mixed>, String> {
    let mut imports = Imports {
        deps: deps.clone(),
        root_dir: root.parent().map(Path::to_path_buf),
        ..Imports::default()
    };
    if !imports.collect(root.parent().unwrap(), text)? || imports.buffers.is_empty() {
        return Ok(None);
    }
    let loaded = load_buffers(root, &imports.buffers, parse);
    let Some(module) = loaded.module else {
        return Ok(None);
    };
    let program = clust_core::check::check_clust(&module).map_err(|errors| {
        errors
            .iter()
            .map(|e| e.render(&loaded.sources))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let mut rust = clust_core::rust::emit_with_sources(&program, checks, &loaded.sources)
        .map_err(|e| e.message)?;
    rust.push_str(&imports.modules);
    imports.files.insert(root.into());
    Ok(Some(Mixed {
        rust,
        asynchronous: false,
        native_items: None,
        sources: imports.files,
    }))
}
