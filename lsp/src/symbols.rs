use crate::{analysis::Documents, document::Document};
use std::collections::BTreeSet;
pub fn ident(s: &str) -> bool {
    let mut b = s.bytes();
    b.next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        && b.all(|c| c.is_ascii_alphanumeric() || c == b'_')
}
#[derive(Clone)]
pub struct Symbol {
    pub uri: String,
    pub i: usize,
    pub name: String,
    pub kind: u8,
    pub owner: String,
    pub scope: usize,
    pub ty: String,
    pub signature: String,
    pub params: Vec<String>,
    pub public: bool,
}
pub struct Index<'a> {
    pub docs: &'a Documents,
    pub symbols: Vec<Symbol>,
}
impl<'a> Index<'a> {
    pub fn new(docs: &'a Documents, group: Option<&BTreeSet<String>>, uri: &str) -> Self {
        let mut symbols = vec![];
        for (u, d) in docs {
            if u != uri && !group.is_some_and(|g| g.contains(u)) {
                continue;
            }
            let mut signature_end = 0;
            let mut owners = std::collections::BTreeMap::new();
            for (i, w) in d.words.iter().enumerate() {
                let name = d.word(i);
                if i < signature_end || !ident(name) {
                    continue;
                }
                let prev = d.word(i.wrapping_sub(1));
                let next = d.word(i + 1);
                let owner = owners.get(&w.scope).cloned().unwrap_or_else(String::new);
                let kind = match prev {
                    "class" | "interface" | "enum" => 7,
                    "fn" => 3,
                    "var" | "let" if name != "self" => 6,
                    _ if !owner.is_empty() && (next == "(" || next == "<") => 2,
                    _ if !owner.is_empty() && next == ":" => 5,
                    _ => continue,
                };
                let mut s = Symbol {
                    uri: u.clone(),
                    i,
                    name: name.into(),
                    kind,
                    owner,
                    scope: w.scope,
                    ty: String::new(),
                    signature: name.into(),
                    params: vec![],
                    public: prev == "pub" || d.word(i.wrapping_sub(2)) == "pub" || kind == 6,
                };
                if kind == 7 {
                    if let Some(j) =
                        (i + 1..d.words.len()).find(|&j| matches!(d.word(j), "{" | ";"))
                    {
                        owners.insert(j + 1, name.to_string());
                    }
                }
                if matches!(kind, 2 | 3) {
                    let end = (i + 1..d.words.len())
                        .find(|&j| matches!(d.word(j), "{" | ";"))
                        .unwrap_or(d.words.len());
                    signature_end = end;
                    if end > i + 1 {
                        s.signature = d.text[w.start..d.words[end - 1].end].into();
                        let mut level = 0;
                        for j in i + 1..end {
                            match d.word(j) {
                                "(" => level += 1,
                                ")" => level -= 1,
                                "->" if level == 0 => s.ty = d.word(j + 1).into(),
                                ":" if level == 1 && j > 0 => {
                                    let param = d.word(j - 1);
                                    if ident(param) {
                                        s.params.push(format!("{param}: {}", d.word(j + 1)));
                                        let mut p = s.clone();
                                        p.i = j - 1;
                                        p.kind = 6;
                                        p.owner.clear();
                                        p.scope = end + 1;
                                        p.name = param.into();
                                        p.ty = d.word(j + 1).into();
                                        p.signature = format!("{param}: {}", p.ty);
                                        symbols.push(p);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                } else if next == ":"
                    || kind == 6 && next == "=" && d.word(i + 3) == "." && d.word(i + 4) == "new"
                {
                    s.ty = d.word(i + 2).into();
                    s.signature = format!("{name}: {}", s.ty);
                }
                symbols.push(s);
            }
        }
        Self { docs, symbols }
    }
    pub fn ancestor(d: &Document, mut scope: usize, target: usize) -> bool {
        while scope != 0 && scope != target {
            scope = d.words[scope - 1].scope;
        }
        scope == target
    }
    pub fn receiver(&self, u: &str, i: usize) -> String {
        let d = &self.docs[u];
        let name = d.word(i);
        if name == "self" {
            return self
                .symbols
                .iter()
                .filter(|s| s.uri == u && s.kind == 7 && s.i < i)
                .max_by_key(|s| s.i)
                .map_or(String::new(), |s| s.name.clone());
        }
        self.local(u, i, name)
            .map_or_else(|| name.into(), |s| s.ty.clone())
    }
    fn local(&self, u: &str, i: usize, name: &str) -> Option<&Symbol> {
        let d = &self.docs[u];
        self.symbols
            .iter()
            .filter(|s| {
                s.uri == u
                    && s.name == name
                    && s.kind == 6
                    && s.i <= i
                    && Self::ancestor(d, d.words[i].scope, s.scope)
            })
            .max_by_key(|s| s.i)
    }
    pub fn resolve(&self, u: &str, i: usize) -> Option<&Symbol> {
        let d = &self.docs[u];
        let name = d.word(i);
        if let Some(s) = self.symbols.iter().find(|s| s.uri == u && s.i == i) {
            return Some(s);
        }
        if i >= 2 && d.word(i - 1) == "." {
            let ty = self.receiver(u, i - 2);
            let mut options = self.symbols.iter().filter(|s| {
                s.name == name
                    && ((!s.owner.is_empty() && s.owner == ty)
                        || (s.owner.is_empty() && s.public && module_name(&s.uri) == ty))
            });
            let found = options.next()?;
            return options.next().is_none().then_some(found);
        }
        if let Some(s) = self.local(u, i, name) {
            return Some(s);
        }
        let mut options = self.symbols.iter().filter(|s| {
            s.name == name && s.kind != 6 && s.owner.is_empty() && (s.uri == u || s.public)
        });
        let found = options.next()?;
        options.next().is_none().then_some(found)
    }
}
pub fn module_name(uri: &str) -> &str {
    uri.rsplit('/').next().unwrap_or("").trim_end_matches(".cl")
}
