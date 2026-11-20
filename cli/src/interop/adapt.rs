//! Adapt checked call arguments using borrow suggestions and native standard types.
//! Each rewritten call must pass a fresh Rust type and borrow check.
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path, process::Command};
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    Expr,
};
#[derive(Default)]
struct Arguments(BTreeMap<usize, usize>);
impl<'ast> Visit<'ast> for Arguments {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        self.add(&call.args);
        visit::visit_expr_call(self, call);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.add(&call.args);
        visit::visit_expr_method_call(self, call);
    }
}
impl Arguments {
    fn add(&mut self, args: &syn::punctuated::Punctuated<Expr, syn::Token![,]>) {
        for a in args {
            let range = a.span().byte_range();
            self.0.insert(range.start, range.end);
        }
    }
}
pub fn build(command: &mut Command, path: &Path, code: &mut String) -> Result<(), String> {
    let target_source = path.canonicalize().map_err(|e| e.to_string())?;
    loop {
        let output = command
            .output()
            .map_err(|e| format!("cannot start Cargo: {e}"))?;
        if output.status.success() {
            return Ok(());
        }
        let mut args = Arguments::default();
        args.visit_file(&syn::parse_file(code).map_err(|e| e.to_string())?);
        let mut edits = BTreeMap::new();
        let mut errors = String::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let Ok(json) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            let msg = &json["message"];
            if msg["level"] != "error" {
                continue;
            }
            errors.push_str(msg["rendered"].as_str().unwrap_or("Rust compilation error"));
            if msg["code"]["code"] != "E0308"
                || json["target"]["src_path"]
                    .as_str()
                    .and_then(|p| Path::new(p).canonicalize().ok())
                    .as_ref()
                    != Some(&target_source)
            {
                continue;
            }
            for span in msg["spans"].as_array().into_iter().flatten() {
                if span["is_primary"] != true || span["file_name"] != "src/main.rs" {
                    continue;
                }
                if let (Some(start), Some(end), Some(prefix)) = (
                    span["byte_start"].as_u64(),
                    span["byte_end"].as_u64(),
                    span["label"].as_str().and_then(standard_conversion),
                ) {
                    let range = (start as usize, end as usize);
                    if args.0.get(&range.0) == Some(&range.1) {
                        edits.insert(range, prefix);
                    }
                }
            }
            for hint in msg["children"].as_array().into_iter().flatten() {
                let Some(spans) = hint["spans"].as_array() else {
                    continue;
                };
                if spans.len() != 1 {
                    continue;
                }
                let s = &spans[0];
                let Some(pos) = s["byte_start"].as_u64().map(|v| v as usize) else {
                    continue;
                };
                let Some(insert) = s["suggested_replacement"].as_str() else {
                    continue;
                };
                if s["file_name"] == "src/main.rs"
                    && s["byte_end"] == s["byte_start"]
                    && matches!(insert, "&" | "&mut ")
                    && args.0.contains_key(&pos)
                    && !code[pos..].starts_with('&')
                {
                    edits.insert((pos, args.0[&pos]), format!("{insert}("));
                }
            }
        }
        if edits.is_empty() {
            return Err(format!(
                "{errors}{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let mut insertions = edits
            .into_iter()
            .flat_map(|((start, end), prefix)| [(start, prefix), (end, ")".to_owned())])
            .collect::<Vec<_>>();
        insertions.sort_by_key(|(pos, _)| *pos);
        for (offset, text) in insertions.into_iter().rev() {
            code.insert_str(offset, &text);
        }
        fs::write(path, code.as_bytes()).map_err(|e| e.to_string())?;
    }
}

// Convert only a compiler-owned standard wrapper and its exact Rust container.
// The selected method transfers the allocation. A subsequent Rust type check
// validates element types and ownership.
fn standard_conversion(label: &str) -> Option<String> {
    let (expected, found) = label
        .strip_prefix("expected `")?
        .trim_end_matches('`')
        .split_once("`, found `")?;
    for tail in ["vec::Vec", "string::String"] {
        let wrapper = format!("__clust_standard::{tail}");
        let native = |s: &str| {
            s.strip_prefix("std::")
                .or_else(|| s.strip_prefix("alloc::"))
                .is_some_and(|p| p.split('<').next() == Some(tail))
        };
        let wrapped = |s: &str| s.split('<').next() == Some(wrapper.as_str());
        if native(expected) && wrapped(found) {
            return Some(format!("crate::{wrapper}::into_native("));
        }
        if wrapped(expected) && native(found) {
            return Some(format!("crate::{wrapper}::from_native("));
        }
    }
    None
}
