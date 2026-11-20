use clust_frontend::{
    lexer::{lex, TokenKind},
    parser::parse,
    source::SourceId,
};
use serde_json::{json, Value};

pub fn completions() -> Value {
    let words = "class interface enum fn pub new self var let if else while for in match return mod use extend dyn async await";
    Value::Array(
        words
            .split_whitespace()
            .map(|word| json!({"label":word,"kind":14}))
            .collect(),
    )
}

fn position(text: &str, offset: usize) -> Value {
    let mut line = 0;
    let mut character = 0;
    let mut previous_cr = false;
    for c in text[..offset].chars() {
        match c {
            '\r' => {
                line += 1;
                character = 0;
            }
            '\n' => {
                line += usize::from(!previous_cr);
                character = 0;
            }
            _ => character += c.len_utf16(),
        }
        previous_cr = c == '\r';
    }
    json!({"line":line,"character":character})
}

pub fn diagnostics(text: &str) -> Vec<Value> {
    let id = SourceId(0);
    let lexed = lex(id, text);
    // Async files have a separate compiler frontend. Only report lexical errors there.
    let mut previous_async = false;
    let async_file = lexed
        .tokens
        .iter()
        .filter(|token| token.kind != TokenKind::Newline)
        .any(|token| {
            let found =
                previous_async && matches!(&token.kind, TokenKind::Word(word) if word=="fn");
            previous_async = matches!(&token.kind,TokenKind::Word(word) if word=="async");
            found
        });
    let errors = if async_file || !lexed.diagnostics.is_empty() {
        lexed.diagnostics
    } else {
        parse(id, text).diagnostics
    };
    errors.into_iter().map(|error| json!({
        "severity":1,"source":"clust","code":error.code,"message":error.message,
        "range":{"start":position(text,error.span.start),"end":position(text,error.span.end)}
    })).collect()
}
