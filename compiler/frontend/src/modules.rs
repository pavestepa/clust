//! Rust-style module declarations and imports, with dot-separated paths.
use crate::{
    ast::*,
    parser::parse,
    source::{Diagnostic, Source, SourceId, Span},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub struct Loaded {
    pub sources: Vec<Source>,
    pub module: Option<Module>,
    pub diagnostics: Vec<Diagnostic>,
}
pub use load as load_clust;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Origin {
    Declaration,
    Explicit,
    Glob,
    Ambiguous,
}

#[derive(Clone)]
struct Symbol {
    canonical: String,
    module: Option<Vec<String>>,
    restriction: Option<Vec<String>>,
    origin: Origin,
}
type Symbols = BTreeMap<String, Symbol>;
struct Unit {
    items: Vec<Item>,
    children: Symbols,
}
struct Loader<'a> {
    buffers: &'a BTreeMap<PathBuf, String>,
    root: PathBuf,
    sources: Vec<Source>,
    units: BTreeMap<Vec<String>, Unit>,
    files: BTreeSet<PathBuf>,
    trusted: BTreeSet<Vec<String>>,
    diagnostics: Vec<Diagnostic>,
    parser: fn(SourceId, &str) -> crate::parser::Parsed,
}

pub fn load(entry: &Path, package: &str) -> Loaded {
    load_with_parser(entry, package, parse)
}
pub(crate) fn load_with_parser(
    entry: &Path,
    _package: &str,
    parser: fn(SourceId, &str) -> crate::parser::Parsed,
) -> Loaded {
    load_buffers(entry, &BTreeMap::new(), parser)
}

pub fn load_buffers(
    entry: &Path,
    buffers: &BTreeMap<PathBuf, String>,
    parser: fn(SourceId, &str) -> crate::parser::Parsed,
) -> Loaded {
    let entry = match entry.canonicalize().or_else(|err| {
        if buffers.contains_key(entry) {
            Ok(entry.to_path_buf())
        } else {
            Err(err)
        }
    }) {
        Ok(path) => path,
        Err(error) => {
            return Loaded {
                sources: vec![Source::new(entry.to_string_lossy(), "")],
                module: None,
                diagnostics: vec![Diagnostic::new(
                    "E0500",
                    error.to_string(),
                    Span {
                        source: SourceId(0),
                        start: 0,
                        end: 0,
                    },
                )],
            }
        }
    };
    let mut loader = Loader {
        buffers,
        root: entry.parent().unwrap().into(),
        sources: vec![],
        units: BTreeMap::new(),
        files: BTreeSet::new(),
        trusted: BTreeSet::new(),
        diagnostics: vec![],
        parser,
    };
    let text = buffers
        .get(&entry)
        .cloned()
        .map_or_else(|| fs::read_to_string(&entry), Ok);
    match text {
        Ok(text) => loader.file(
            vec![],
            entry.clone(),
            entry.parent().unwrap().into(),
            text,
            None,
        ),
        Err(error) => loader.diagnostics.push(Diagnostic::new(
            "E0500",
            error.to_string(),
            Span {
                source: SourceId(0),
                start: 0,
                end: 0,
            },
        )),
    }
    if !loader.builtin_requests().is_empty()
        && !loader
            .units
            .get(&Vec::new())
            .is_some_and(|unit| unit.children.contains_key("core"))
    {
        loader.builtins();
    }
    let module = if loader.diagnostics.is_empty() {
        match loader.link() {
            Ok(module) => Some(module),
            Err(error) => {
                loader.diagnostics.push(error);
                None
            }
        }
    } else {
        None
    };
    Loaded {
        sources: loader.sources,
        module,
        diagnostics: loader.diagnostics,
    }
}

impl Loader<'_> {
    fn file(
        &mut self,
        id: Vec<String>,
        path: PathBuf,
        directory: PathBuf,
        text: String,
        builtin: Option<&str>,
    ) {
        if !self.files.insert(path.clone()) {
            self.diagnostics.push(Diagnostic::new(
                "E0501",
                "a source file cannot declare itself recursively",
                Span {
                    source: SourceId(0),
                    start: 0,
                    end: 0,
                },
            ));
            return;
        }
        let source_id = SourceId(self.sources.len());
        let parsed = if builtin.is_some() {
            parse(source_id, &text)
        } else {
            (self.parser)(source_id, &text)
        };
        let source_name = builtin.map_or_else(
            || path.to_string_lossy().into_owned(),
            |name| format!("core.{name}"),
        );
        self.sources.push(Source::new(source_name, text));
        self.diagnostics.extend(parsed.diagnostics);
        if let Some(mut module) = parsed.module {
            match builtin {
                Some("intrinsic") => {
                    self.trusted.insert(id.clone());
                    crate::intrinsics::lower_trusted_declarations(&mut module);
                }
                Some("storage") => crate::buffer::lower(&mut module),
                _ => {}
            }
            self.unit(id, directory, module);
        }
    }
    fn unit(&mut self, id: Vec<String>, directory: PathBuf, module: Module) {
        if id.len() > 32 {
            let span = module.items.first().map_or(
                Span {
                    source: SourceId(self.sources.len().saturating_sub(1)),
                    start: 0,
                    end: 0,
                },
                |item| item.span,
            );
            self.diagnostics.push(Diagnostic::new(
                "E0201",
                "module nesting limit exceeded",
                span,
            ));
            return;
        }
        let mut unit = Unit {
            items: vec![],
            children: Symbols::new(),
        };
        for item in module.items {
            let restriction = match visibility(&item, &id) {
                Ok(scope) => scope,
                Err(error) => {
                    self.diagnostics.push(error);
                    continue;
                }
            };
            let ItemKind::Module { name, body } = item.kind else {
                unit.items.push(item);
                continue;
            };
            let mut child = id.clone();
            child.push(name.clone());
            if unit.children.contains_key(&name) {
                self.diagnostics.push(Diagnostic::new(
                    "E0301",
                    "duplicate module declaration",
                    item.span,
                ));
                continue;
            }
            unit.children.insert(
                name.clone(),
                Symbol {
                    canonical: String::new(),

                    module: Some(child.clone()),
                    restriction,
                    origin: Origin::Declaration,
                },
            );
            let child_dir = directory.join(&name);
            if let Some(body) = body {
                self.unit(child, child_dir, body);
                continue;
            }
            let flat = directory.join(format!("{name}.cl"));
            let nested = child_dir.join("mod.cl");
            let path = match (
                flat.is_file() || self.buffers.contains_key(&flat),
                nested.is_file() || self.buffers.contains_key(&nested),
            ) {
                (true, false) => flat,
                (false, true) => nested,
                (true, true) => {
                    self.diagnostics.push(Diagnostic::new(
                        "E0501",
                        "module has both name.cl and name/mod.cl",
                        item.span,
                    ));
                    continue;
                }
                _ => {
                    self.diagnostics.push(Diagnostic::new(
                        "E0501",
                        format!("module file not found for '{name}'"),
                        item.span,
                    ));
                    continue;
                }
            };
            match path.canonicalize().or_else(|err| {
                if self.buffers.contains_key(&path) {
                    Ok(path.clone())
                } else {
                    Err(err)
                }
            }) {
                Ok(path) if path.starts_with(&self.root) => match self
                    .buffers
                    .get(&path)
                    .cloned()
                    .map_or_else(|| fs::read_to_string(&path), Ok)
                {
                    Ok(text) => self.file(child, path, child_dir, text, None),
                    Err(error) => self.diagnostics.push(Diagnostic::new(
                        "E0501",
                        error.to_string(),
                        item.span,
                    )),
                },
                Ok(_) => self.diagnostics.push(Diagnostic::new(
                    "E0501",
                    "module escapes package root",
                    item.span,
                )),
                Err(error) => {
                    self.diagnostics
                        .push(Diagnostic::new("E0501", error.to_string(), item.span))
                }
            }
        }
        self.units.insert(id, unit);
    }
}

struct Rewrite<'a> {
    globals: &'a Symbols,
    scopes: Vec<BTreeSet<String>>,
    modules: &'a BTreeMap<Vec<String>, Symbols>,
    current: &'a [String],
    error: Option<Diagnostic>,
}

mod builtins;
mod declarations;
mod expressions;
mod linker;
mod names;
mod patterns;
mod types;
mod visibility;
use self::names::item_name;
use self::visibility::accessible;
use self::visibility::resolve;
use self::visibility::visibility;
