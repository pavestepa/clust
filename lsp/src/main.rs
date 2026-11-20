mod analysis;
mod async_check;
mod call_hints;
mod document;
mod editing;
mod features;
mod language;
mod rename;
mod symbols;
mod transport;

use analysis::{Analysis, Documents};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    io::{self, BufReader},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

enum Event {
    Message(Value),
    End,
    Ready(u64, Analysis),
}

fn main() -> io::Result<()> {
    let (tx, rx) = mpsc::channel();
    let reader = tx.clone();
    thread::spawn(move || {
        let mut input = BufReader::new(io::stdin().lock());
        while let Ok(Some(m)) = transport::read(&mut input) {
            if reader.send(Event::Message(m)).is_err() {
                return;
            }
        }
        let _ = reader.send(Event::End);
    });
    let mut output = io::stdout().lock();
    let async_cache = std::sync::Arc::new(async_check::Cache::create()?);
    let mut open = Documents::new();
    let mut current = Analysis::default();
    let mut revision = 0;
    let mut dirty = None;
    let mut busy = false;
    let mut shutdown = false;
    let mut published = BTreeSet::new();
    loop {
        if dirty.is_some_and(|when| Instant::now() >= when) && !busy && !shutdown {
            dirty = None;
            busy = true;
            let snapshot = open.clone();
            let cache = async_cache.clone();
            let tx = tx.clone();
            let rev = revision;
            thread::spawn(move || {
                let a = Analysis::build(&snapshot, &cache.0);
                let _ = tx.send(Event::Ready(rev, a));
            });
        }
        let event = if let Some(when) = dirty.filter(|_| !busy && !shutdown) {
            match rx.recv_timeout(when.saturating_duration_since(Instant::now())) {
                Ok(m) => m,
                Err(_) => continue,
            }
        } else {
            rx.recv().map_err(io::Error::other)?
        };
        let message = match event {
            Event::End => break,
            Event::Ready(rev, a) => {
                busy = false;
                if rev != revision || shutdown {
                    continue;
                }
                let next: BTreeSet<_> = open.keys().chain(a.errors.keys()).cloned().collect();
                for u in next.union(&published) {
                    let mut p =
                        json!({"uri":u,"diagnostics":a.errors.get(u).cloned().unwrap_or_default()});
                    if let Some(v) = open.get(u).and_then(|d| d.version) {
                        p["version"] = json!(v)
                    }
                    transport::write(
                        &mut output,
                        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":p}),
                    )?;
                }
                published = next;
                current = a;
                continue;
            }
            Event::Message(m) => m,
        };
        let p = &message["params"];
        let method = message["method"].as_str().unwrap_or("");
        if method == "exit" {
            if shutdown {
                break;
            } else {
                return Err(io::Error::other("Exit without shutdown"));
            }
        }
        if let Some(id) = message.get("id") {
            let result = match method {
                _ if shutdown => Err((-32600, "Server has shut down")),
                "initialize" => Ok(
                    json!({"capabilities":{"textDocumentSync":{"openClose":true,"change":1},"completionProvider":{"triggerCharacters":["."]},"signatureHelpProvider":{"triggerCharacters":["(",","]},"hoverProvider":true,"definitionProvider":true,"referencesProvider":true,"renameProvider":{"prepareProvider":true},"documentFormattingProvider":true,"inlayHintProvider":true,"codeActionProvider":true},"serverInfo":{"name":"clust-lsp","version":env!("CARGO_PKG_VERSION")}}),
                ),
                "shutdown" => {
                    shutdown = true;
                    Ok(Value::Null)
                }
                m if m.starts_with("textDocument/") => {
                    features::handle(m, p, &open, &current).map_err(|e| (-32602, e))
                }
                _ => Err((-32601, "Method not found")),
            };
            let reply = match result {
                Ok(r) => json!({"jsonrpc":"2.0","id":id,"result":r}),
                Err((code, msg)) => {
                    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":msg}})
                }
            };
            transport::write(&mut output, reply)?;
        } else if !shutdown {
            if method == "workspace/didChangeWatchedFiles" {
                revision += 1;
                dirty = Some(Instant::now());
                continue;
            }
            let doc = &p["textDocument"];
            let Some(u) = doc["uri"].as_str() else {
                continue;
            };
            match method {
                "textDocument/didClose" => {
                    open.remove(u);
                }
                "textDocument/didChange" | "textDocument/didOpen" => {
                    let v = doc["version"].as_i64();
                    if open.get(u).is_some_and(|d| d.version >= v) {
                        continue;
                    }
                    let Some(text) = doc["text"]
                        .as_str()
                        .or_else(|| p["contentChanges"].as_array()?.last()?["text"].as_str())
                    else {
                        continue;
                    };
                    open.insert(u.into(), document::Document::new(text.into(), v));
                }
                _ => continue,
            }
            revision += 1;
            dirty = Some(Instant::now() + Duration::from_millis(65));
        }
    }
    Ok(())
}
