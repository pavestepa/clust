//! Keep the original token tree of a Rust macro invocation. Its punctuation,
//! literals and nested delimiters belong to the macro's Rust grammar.
use proc_macro2::{TokenStream, TokenTree};
pub(super) fn mixed(source: &str) -> Result<Vec<&str>, String> {
    let stream = source.parse::<TokenStream>().map_err(|e| e.to_string())?;
    let mut tokens = Vec::new();
    flatten(source, stream, &mut tokens);
    Ok(tokens)
}
fn flatten<'a>(source: &'a str, stream: TokenStream, out: &mut Vec<&'a str>) {
    let mut iter = stream.into_iter().peekable();
    while let Some(t) = iter.next() {
        let span = t.span().byte_range();
        if matches!(&t,TokenTree::Punct(p) if p.as_char()=='!')
            && matches!(iter.peek(), Some(TokenTree::Group(_)))
            && out.last().is_some_and(|s| {
                s.starts_with(|c: char| c.is_alphabetic() || c == '_')
                    && !matches!(*s, "if" | "while" | "return")
            })
        {
            let group = iter.next().unwrap();
            out.push(&source[span.start..group.span().byte_range().end]);
            continue;
        }
        if let TokenTree::Group(g) = t {
            out.push(&source[g.span_open().byte_range()]);
            flatten(source, g.stream(), out);
            out.push(&source[g.span_close().byte_range()]);
            continue;
        }
        if let TokenTree::Punct(p) = &t {
            let mut end = span.end;
            let mut joint = p.spacing() == proc_macro2::Spacing::Joint;
            while joint && matches!(iter.peek(), Some(TokenTree::Punct(_))) {
                let candidate = iter.peek().unwrap();
                let next = candidate.span().byte_range();
                if !matches!(
                    &source[span.start..next.end],
                    "::" | "->"
                        | "=>"
                        | "=="
                        | "!="
                        | "<="
                        | ">="
                        | "&&"
                        | "||"
                        | "+="
                        | "-="
                        | "*="
                        | "/="
                        | "%="
                        | ">>"
                        | "<<"
                        | ".."
                        | "..="
                ) {
                    break;
                }
                let Some(TokenTree::Punct(next_p)) = iter.next() else {
                    unreachable!()
                };
                end = next.end;
                joint = next_p.spacing() == proc_macro2::Spacing::Joint;
            }
            out.push(&source[span.start..end]);
        } else {
            out.push(&source[span])
        }
    }
}
