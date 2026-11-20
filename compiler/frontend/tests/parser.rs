use clust_frontend::{ast::*, parser::parse, source::SourceId};

fn module(text: &str) -> Module {
    let parsed = parse(SourceId(0), text);
    assert!(
        parsed.diagnostics.is_empty(),
        "{:#?}\n{text}",
        parsed.diagnostics
    );
    parsed.module.unwrap()
}
fn body(text: &str) -> Block {
    let mut module = module(text);
    let ItemKind::Function(function) = module.items.remove(0).kind else {
        panic!("expected function")
    };
    function.body.unwrap()
}
fn failure(text: &str) -> String {
    let parsed = parse(SourceId(0), text);
    assert!(parsed.module.is_none(), "accepted {text}");
    parsed.diagnostics[0].code.into()
}

#[path = "parser/unsafe_declarations_are_limited.rs"]
mod unsafe_declarations_are_limited;
#[path = "parser/unsupported_syntax_is_rejected.rs"]
mod unsupported_syntax_is_rejected;
