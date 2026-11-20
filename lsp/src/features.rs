use crate::{
    analysis::{Analysis, Documents},
    editing, language,
    symbols::{ident, Index},
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
pub fn handle(
    method: &str,
    p: &Value,
    open: &Documents,
    a: &Analysis,
) -> Result<Value, &'static str> {
    let u = p["textDocument"]["uri"]
        .as_str()
        .ok_or("Missing document")?;
    let mut docs = a.files.clone();
    docs.extend(open.clone());
    let Some(d) = docs.get(u) else {
        return Ok(Value::Null);
    };
    match method {
        "textDocument/formatting" => return Ok(editing::format(d, &p["options"])),
        "textDocument/codeAction" => {
            let mut actions = editing::fixes(d, u, p);
            actions.as_array_mut().unwrap().extend(editing::imports(
                &Index::new(&docs, a.groups.get(u), u),
                u,
                p,
            ));
            return Ok(actions);
        }
        "textDocument/inlayHint" => {
            let from = d.offset(&p["range"]["start"]).unwrap_or(0);
            let to = d.offset(&p["range"]["end"]).unwrap_or(d.text.len());
            return Ok(json!(a
                .hints
                .get(u)
                .into_iter()
                .flatten()
                .filter(|h| d
                    .offset(&h["position"])
                    .is_some_and(|n| (from..=to).contains(&n)))
                .collect::<Vec<_>>()));
        }
        _ => {}
    }
    let pos = d.offset(&p["position"]).ok_or("Invalid position")?;
    let end = d.words.partition_point(|w| w.start < pos);
    let under = d.words.partition_point(|w| w.end <= pos);
    let i = if d
        .words
        .get(under)
        .is_some_and(|w| w.start <= pos && ident(d.word(under)))
    {
        under
    } else {
        end.saturating_sub(1)
    };
    let index = Index::new(&docs, a.groups.get(u), u);
    if method == "textDocument/completion" {
        let dot = (end > 0 && d.word(end - 1) == ".")
            .then(|| end - 1)
            .or_else(|| (end >= 2 && d.word(end - 2) == ".").then(|| end - 2));
        let owner = dot
            .and_then(|j| j.checked_sub(1))
            .map(|j| index.receiver(u, j));
        let mut result = if owner.is_some() {
            vec![]
        } else {
            language::completions().as_array().unwrap().clone()
        };
        for s in &index.symbols {
            let visible = match &owner {
                Some(ty) => {
                    &s.owner == ty
                        && (s.public || dot.is_some_and(|j| j > 0 && d.word(j - 1) == "self"))
                }
                None => {
                    s.owner.is_empty()
                        && (s.public || s.uri == u)
                        && (matches!(s.name.as_str(), "main" | "new" | "drop")
                            || (s.uri == u
                                && s.i < i
                                && d.words.get(i).is_some_and(|w| {
                                    super::symbols::Index::ancestor(d, w.scope, s.scope)
                                })))
                }
            };
            if visible {
                result.push(json!({"label":s.name,"kind":s.kind,"detail":s.signature}));
            }
        }
        return Ok(json!(result));
    }
    if method == "textDocument/signatureHelp" {
        let mut depth = 0;
        let mut active = 0;
        for j in (0..end).rev() {
            match d.word(j) {
                ")" | "]" | "}" => depth += 1,
                "(" | "[" | "{" if depth > 0 => depth -= 1,
                "," if depth == 0 => active += 1,
                "(" if depth == 0 && j > 0 => {
                    if let Some(s) = index.resolve(u, j - 1) {
                        return Ok(
                            json!({"signatures":[{"label":s.signature,"parameters":s.params.iter().map(|p|json!({"label":p})).collect::<Vec<_>>()}],"activeSignature":0,"activeParameter":active}),
                        );
                    }
                    break;
                }
                _ => {}
            }
        }
        return Ok(Value::Null);
    }
    let Some(s) = index.resolve(u, i) else {
        return Ok(Value::Null);
    };
    let location = json!({"uri":s.uri,"range":docs[&s.uri].token_range(s.i)});
    match method {
        "textDocument/hover" => Ok(
            json!({"contents":{"kind":"plaintext","value":s.signature},"range":d.token_range(i)}),
        ),
        "textDocument/definition" => Ok(location),
        "textDocument/prepareRename" if s.name != "main" && s.name != "new" && s.name != "drop" => {
            Ok(json!({"range":d.token_range(i),"placeholder":s.name}))
        }
        "textDocument/prepareRename" => Ok(Value::Null),
        "textDocument/references" | "textDocument/rename" => {
            let rename = method.ends_with("/rename");
            let name = p["newName"].as_str().unwrap_or("");
            if rename
                && (matches!(s.name.as_str(), "main" | "new" | "drop")
                    || !ident(name)
                    || index.symbols.iter().any(|x| x.name == name)
                    || crate::language::completions()
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|v| v["label"] == name))
            {
                return Err("Choose a fresh, non-reserved identifier");
            }
            let mut refs = vec![];
            let mut changes = BTreeMap::<String, Vec<Value>>::new();
            for (uri, file) in &docs {
                if uri != u && !a.groups.get(u).is_some_and(|g| g.contains(uri)) {
                    continue;
                }
                for j in 0..file.words.len() {
                    if file.word(j) != s.name {
                        continue;
                    }
                    if index
                        .resolve(uri, j)
                        .is_some_and(|x| x.uri == s.uri && x.i == s.i)
                    {
                        let range = file.token_range(j);
                        if rename {
                            changes
                                .entry(uri.clone())
                                .or_default()
                                .push(json!({"range":range,"newText":name}));
                        } else if p["context"]["includeDeclaration"] == true
                            || uri != &s.uri
                            || j != s.i
                        {
                            refs.push(json!({"uri":uri,"range":range}));
                        }
                    }
                }
            }
            Ok(if rename {
                super::rename::checked(&docs, &changes, u)?
            } else {
                json!(refs)
            })
        }
        _ => Err("Unsupported request"),
    }
}
