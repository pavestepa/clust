use clust_frontend::{
    lexer::{lex, TokenKind},
    source::SourceId,
};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone)]
pub struct Document {
    pub text: String,
    pub words: Vec<Word>,
    lines: Vec<usize>,
    pub version: Option<i64>,
}
#[derive(Clone)]
pub struct Word {
    pub start: usize,
    pub end: usize,
    pub scope: usize,
}
impl Document {
    pub fn new(text: String, version: Option<i64>) -> Arc<Self> {
        let mut lines = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' || (b == b'\r' && text.as_bytes().get(i + 1) != Some(&b'\n')) {
                lines.push(i + 1);
            }
        }
        let mut scopes = vec![0];
        let mut words = vec![];
        for token in lex(SourceId(0), &text).tokens {
            if matches!(token.kind, TokenKind::Newline | TokenKind::Eof) {
                continue;
            }
            let raw = &text[token.span.start..token.span.end];
            if raw == "}" && scopes.len() > 1 {
                scopes.pop();
            }
            words.push(Word {
                start: token.span.start,
                end: token.span.end,
                scope: *scopes.last().unwrap(),
            });
            if raw == "{" {
                scopes.push(words.len());
            }
        }
        Arc::new(Self {
            text,
            words,
            lines,
            version,
        })
    }
    pub fn word(&self, i: usize) -> &str {
        self.words.get(i).map_or("", |t| &self.text[t.start..t.end])
    }
    pub fn pos(&self, byte: usize) -> Value {
        let mut byte = byte.min(self.text.len());
        while !self.text.is_char_boundary(byte) {
            byte -= 1;
        }
        let line = self.lines.partition_point(|&s| s <= byte) - 1;
        json!({"line":line,"character":self.text[self.lines[line]..byte].encode_utf16().count()})
    }
    pub fn offset(&self, p: &Value) -> Option<usize> {
        let line = p["line"].as_u64()? as usize;
        let start = *self.lines.get(line)?;
        let mut units = p["character"].as_u64()?;
        for (i, c) in self.text[start..].char_indices() {
            if units == 0 {
                return Some(start + i);
            }
            if matches!(c, '\r' | '\n') {
                return None;
            }
            units = units.checked_sub(c.len_utf16() as u64)?;
        }
        (units == 0).then_some(self.text.len())
    }
    pub fn range(&self, start: usize, end: usize) -> Value {
        json!({"start":self.pos(start),"end":self.pos(end)})
    }
    pub fn token_range(&self, i: usize) -> Value {
        self.range(self.words[i].start, self.words[i].end)
    }
}
pub fn path(uri: &str) -> Option<PathBuf> {
    let raw = uri.strip_prefix("file://")?;
    if !raw.starts_with('/') {
        return None;
    }
    let decoded = percent_encoding::percent_decode_str(raw)
        .decode_utf8()
        .ok()?;
    Some(PathBuf::from(decoded.as_ref()))
}
pub fn uri(path: &Path) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    const SAFE: percent_encoding::AsciiSet = NON_ALPHANUMERIC
        .remove(b'/')
        .remove(b'-')
        .remove(b'_')
        .remove(b'.')
        .remove(b'~');
    format!(
        "file://{}",
        utf8_percent_encode(&path.to_string_lossy(), &SAFE)
    )
}
