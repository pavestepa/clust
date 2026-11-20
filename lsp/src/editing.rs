use crate::document::Document;
use serde_json::{json, Value};

pub fn format(d: &Document, options: &Value) -> Value {
    let unit = if options["insertSpaces"] == false {
        "\t".into()
    } else {
        " ".repeat(options["tabSize"].as_u64().unwrap_or(4).min(16) as usize)
    };
    let mut edits = vec![];
    let mut depth = 0usize;
    for (i, w) in d.words.iter().enumerate() {
        let raw = d.word(i);
        if raw == "}" {
            depth = depth.saturating_sub(1)
        }
        let start = d.text[..w.start].rfind(['\r', '\n']).map_or(0, |j| j + 1);
        let prefix = &d.text[start..w.start];
        if prefix.bytes().all(|b| b == b' ' || b == b'\t') {
            let indent = unit.repeat(depth);
            if prefix != indent {
                edits.push(json!({"range":d.range(start,w.start),"newText":indent}));
            }
        }
        if raw == "{" {
            depth += 1
        }
    }
    json!(edits)
}
pub fn fixes(d: &Document, uri: &str, params: &Value) -> Value {
    let mut out = vec![];
    if let Some(errors) = params["context"]["diagnostics"].as_array() {
        for error in errors {
            let message = error["message"].as_str().unwrap_or("");
            if message.contains("';'") || message.contains("semicolon") {
                let Some(offset) = d.offset(&error["range"]["start"]) else {
                    continue;
                };
                let i = d.words.partition_point(|w| w.end <= offset);
                if i == 0 || d.word(i - 1) == ";" {
                    continue;
                }
                let at = d.words[i - 1].end;
                out.push(json!({"title":"Insert missing semicolon","kind":"quickfix","diagnostics":[error],"edit":{"changes":{(uri):[{"range":d.range(at,at),"newText":";"}]}}}));
            }
        }
    }
    json!(out)
}

pub fn imports(index: &crate::symbols::Index, uri: &str, params: &Value) -> Vec<Value> {
    let mut actions = vec![];
    let Some(errors) = params["context"]["diagnostics"].as_array() else {
        return actions;
    };
    let Some(root) = crate::document::path(uri).and_then(|p| crate::analysis::entry(&p).ok())
    else {
        return actions;
    };
    let Some(dir) = root.parent() else {
        return actions;
    };
    for e in errors {
        let msg = e["message"].as_str().unwrap_or("");
        if !msg.starts_with("unknown ") {
            continue;
        }
        let Some(name) = msg.split('\'').nth(1) else {
            continue;
        };
        for s in &index.symbols {
            if s.uri == uri || !s.public || s.name != name || !s.owner.is_empty() || s.kind == 6 {
                continue;
            }
            let Some(path) = crate::document::path(&s.uri) else {
                continue;
            };
            let Ok(relative) = path.strip_prefix(dir) else {
                continue;
            };
            let mut parts = vec!["crate".to_owned()];
            if path != root {
                parts.extend(
                    relative
                        .with_extension("")
                        .iter()
                        .map(|p| p.to_string_lossy().into_owned()),
                );
                if parts.last().is_some_and(|p| p == "mod") {
                    parts.pop();
                }
            }
            parts.push(name.into());
            let import = parts.join(".");
            actions.push(json!({"title":format!("Import {import}"),"kind":"quickfix","diagnostics":[e],"edit":{"changes":{(uri):[{"range":index.docs[uri].range(0,0),"newText":format!("use {import};\n")}]}}}));
        }
    }
    actions
}
