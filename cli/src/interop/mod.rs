//! Native mixed crates use rustc for item expansion, types, generics and drop glue.
pub(crate) mod adapt;
mod contracts;
mod data;
mod data_privacy;
mod lower;
pub mod metadata;
mod modules;
mod origins;
mod parameters;
mod scalars;
mod tokens;
use crate::project::Project;
use clust_frontend::{
    lexer::{lex, TokenKind},
    source::SourceId,
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
pub struct Mixed {
    pub rust: String,
    pub(crate) native_items: Option<std::ops::Range<usize>>,
    pub asynchronous: bool,
    pub sources: BTreeSet<PathBuf>,
}

pub fn compile(
    entry: &Path,
    source: &str,
    project: Option<&Project>,
    checks: clust_semantics::OverflowChecks,
) -> Result<Option<Mixed>, String> {
    let deps: BTreeSet<_> = project
        .into_iter()
        .flat_map(Project::dependency_names)
        .collect();
    let root = entry.canonicalize().map_err(|e| e.to_string())?;
    let dir = root.parent().unwrap();
    if !modules::mixed(dir, source, &deps, &mut BTreeSet::new())? {
        return Ok(None);
    }
    if let Some(mixed) = scalars::compile(&root, source, checks, &deps)? {
        return Ok(Some(mixed));
    }
    let mut files = BTreeSet::from([root.clone()]);
    let code = modules::emit(dir, source, &deps, &mut files)?;
    let mut parsed = syn::parse_file(&code)
        .map_err(|e| format!("unsupported mixed Clust syntax in {}: {e}", entry.display()))?;
    data_privacy::seal(&mut parsed.items);
    let mut asynchronous = false;
    let mut entry_found = false;
    for item in &mut parsed.items {
        if let syn::Item::Fn(f) = item {
            if f.sig.ident == "main" {
                asynchronous = f.sig.asyncness.is_some();
                f.sig.ident = syn::parse_quote!(__clust_entry);
                entry_found = true;
            }
        }
    }
    if !entry_found {
        return Err("mixed application requires fn main()".into());
    }
    let code = quote::quote!(#parsed).to_string();
    let standard = if code.contains("crate :: __clust_standard") {
        include_str!("standard.rs")
    } else {
        ""
    };
    let (support, invoke) = if asynchronous {
        (clust_runtime::TOKIO_SUPPORT,"tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(__clust_entry())")
    } else {
        ("trait ExitStatus {fn code(self)->i32;} impl ExitStatus for i32{fn code(self)->i32{self}} impl ExitStatus for (){fn code(self)->i32{0}}","__clust_entry()")
    };
    let support = format!("{standard}\n{support}");
    Ok(Some(Mixed {
        rust: format!(
            "{support}\n{code}\nfn main() {{std::process::exit(ExitStatus::code({invoke}));}}\n"
        ),
        native_items: Some(support.len() + 1..support.len() + 1 + code.len()),
        asynchronous,
        sources: files,
    }))
}
fn tokens(source: &str) -> Vec<&str> {
    lex(SourceId(0), source)
        .tokens
        .into_iter()
        .filter(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::Eof))
        .map(|t| &source[t.span.start..t.span.end])
        .collect()
}
