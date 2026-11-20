use crate::{
    analysis::{entry, Documents},
    document::path,
};
use clust_frontend::{modules::load_buffers, parser::parse};
use serde_json::{json, Value};
use std::collections::BTreeMap;

// Re-run the compiler on the proposed edits before offering the workspace transaction.
pub fn checked(
    docs: &Documents,
    changes: &BTreeMap<String, Vec<Value>>,
    uri: &str,
) -> Result<Value, &'static str> {
    let file = path(uri).ok_or("Save the file before renaming")?;
    let root = entry(&file).map_err(|_| "Cannot resolve project entry")?;
    let mut buffers = BTreeMap::new();
    let mut operations = vec![];
    for (u, d) in docs {
        let Some(path) = path(u) else { continue };
        let mut text = d.text.clone();
        if let Some(edits) = changes.get(u) {
            for edit in edits.iter().rev() {
                let r = &edit["range"];
                let start = d.offset(&r["start"]).ok_or("Invalid range")?;
                let end = d.offset(&r["end"]).ok_or("Invalid range")?;
                text.replace_range(start..end, edit["newText"].as_str().unwrap());
            }
            operations.push(json!({"textDocument":{"uri":u,"version":d.version},"edits":edits}));
        }
        buffers.insert(path, text);
    }
    let loaded = load_buffers(&root, &buffers, parse);
    let module = loaded
        .module
        .ok_or("Rename would leave an invalid module")?;
    clust_core::check::check_clust(&module)
        .map_err(|_| "Rename would leave unresolved names or change a checked contract")?;
    Ok(json!({"documentChanges":operations}))
}
