use crate::document::{self, Document};
use clust_frontend::{
    modules::load_buffers,
    parser::parse,
    source::{Diagnostic, SourceId, Span},
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};
pub type Documents = BTreeMap<String, Arc<Document>>;
#[derive(Default)]
pub struct Analysis {
    pub files: Documents,
    pub groups: BTreeMap<String, BTreeSet<String>>,
    pub errors: BTreeMap<String, Vec<Value>>,
    pub hints: BTreeMap<String, Vec<Value>>,
}
pub fn entry(path: &Path) -> Result<PathBuf, String> {
    for dir in path.ancestors().skip(1) {
        let manifest = dir.join("Clust.toml");
        if manifest.is_file() {
            return Ok(clust::project::Project::resolve(&manifest)?.unwrap().entry);
        }
        let main = dir.join("main.cl");
        if main.is_file() {
            return Ok(main);
        }
    }
    Ok(path.into())
}
impl Analysis {
    pub fn build(open: &Documents, cache: &Path) -> Self {
        let buffers = open
            .iter()
            .filter_map(|(u, d)| Some((document::path(u)?, d.text.clone())))
            .collect();
        let mut out = Self {
            files: open.clone(),
            ..Self::default()
        };
        let mut done = BTreeSet::new();
        for (uri, doc) in open {
            match clust::async_program::compile(&doc.text) {
                Ok(Some(source)) => {
                    if let Err(msg) = crate::async_check::check(cache, &source) {
                        out.error(uri, &msg);
                    }
                    continue;
                }
                Err(message) => {
                    out.error(uri, &message);
                    continue;
                }
                Ok(None) => {}
            }
            let Some(path) = document::path(uri) else {
                out.errors
                    .insert(uri.clone(), crate::language::diagnostics(&doc.text));
                continue;
            };
            let root = match entry(&path) {
                Ok(p) => p,
                Err(e) => {
                    out.error(uri, &e);
                    continue;
                }
            };
            if !done.insert(root.clone()) {
                continue;
            }
            let loaded = load_buffers(&root, &buffers, parse);
            let mut ids = vec![];
            let mut group = BTreeSet::new();
            for source in &loaded.sources {
                let path = Path::new(&source.name);
                if !path.is_absolute() {
                    ids.push(None);
                    continue;
                }
                let u = document::uri(path);
                out.files
                    .entry(u.clone())
                    .or_insert_with(|| Document::new(source.text.clone(), None));
                group.insert(u.clone());
                ids.push(Some(u));
            }
            for u in &group {
                out.groups.insert(u.clone(), group.clone());
            }
            let errors = match loaded.module {
                Some(module) => match clust_core::check::check_clust(&module) {
                    Ok(program) => {
                        out.types(&program, &ids);
                        super::call_hints::collect(&mut out, &program, &ids);
                        vec![]
                    }
                    Err(errors) => errors,
                },
                None => loaded.diagnostics,
            };
            for error in errors {
                out.diagnostic(&ids, error);
            }
        }
        out
    }
    fn error(&mut self, u: &str, msg: &str) {
        self.errors.entry(u.into()).or_default().push(
            json!({"range":self.files[u].range(0,0),"severity":1,"source":"clust","message":msg}),
        );
    }
    fn location(&self, ids: &[Option<String>], span: Span) -> Option<Value> {
        let span = span.source_origin();
        let u = ids.get(span.source.0)?.as_ref()?;
        Some(json!({"uri":u,"range":self.files[u].range(span.start,span.end)}))
    }
    fn diagnostic(&mut self, ids: &[Option<String>], error: Diagnostic) {
        let Some(loc) = self.location(ids, error.span) else {
            return;
        };
        let notes: Vec<_> = error
            .notes
            .into_iter()
            .filter_map(|n| {
                Some(json!({"message":n.message,"location":self.location(ids,n.span)?}))
            })
            .collect();
        self.errors.entry(loc["uri"].as_str().unwrap().into()).or_default().push(json!({"severity":1,"source":"clust","code":error.code,"message":error.message,"range":loc["range"],"relatedInformation":notes}));
    }
    fn types(&mut self, program: &clust_core::hir::Program, ids: &[Option<String>]) {
        use clust_core::hir::Type;
        for f in &program.functions {
            for l in &f.locals {
                let Span {
                    source: SourceId(id),
                    start,
                    end,
                } = l.span.source_origin();
                let Some(Some(u)) = ids.get(id) else { continue };
                let d = &self.files[u];
                let first = d.words.partition_point(|w| w.end <= start);
                let Some(i) = (first..d.words.len())
                    .take_while(|&i| d.words[i].start < end)
                    .find(|&i| d.word(i) == l.name)
                else {
                    continue;
                };
                if d.word(i + 1) == ":" && l.view.is_none() {
                    continue;
                }
                let ty = match &l.ty {
                    Type::Class { name, .. }
                    | Type::Enum { name, .. }
                    | Type::Record { name, .. } => name.clone(),
                    Type::Integer(t) => format!("{t:?}").to_lowercase(),
                    Type::Bool => "bool".into(),
                    Type::Str => "str".into(),
                    _ => continue,
                };
                let mode = l.view.map_or(String::new(), |v| format!(" ({v:?} borrow)"));
                let hint = json!({"position":d.pos(d.words[i].end),"label":format!(": {ty}{mode}"),"kind":1});
                let hints = self.hints.entry(u.clone()).or_default();
                if !hints.contains(&hint) {
                    hints.push(hint);
                }
            }
        }
    }
}
