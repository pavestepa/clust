//! Safe Rust lowering for scalar async applications and owned runtime handles.
use clust_frontend::{
    lexer::{lex, TokenKind},
    source::SourceId,
};
use quote::ToTokens;
use syn::{
    visit::{self, Visit},
    Expr, Item,
};

mod borrows;
pub(crate) mod command;

const BUILTINS: &[&str] = &["channel", "actor", "spawn", "mutex", "drop"];
const METHODS: &[&str] = &[
    "send",
    "recv",
    "close",
    "clone",
    "join",
    "cancel",
    "sender",
    "finish",
    "update",
    "read",
    "is_ok",
    "is_err",
    "is_some",
    "is_none",
    "unwrap_or",
];

pub fn compile(source: &str) -> Result<Option<String>, String> {
    let lexed = lex(SourceId(0), source);
    let tokens: Vec<_> = lexed
        .tokens
        .iter()
        .filter(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::Eof))
        .collect();
    let has_async = tokens.windows(2).any(|pair| {
        matches!(
            (&pair[0].kind, &pair[1].kind),
            (TokenKind::Word(a), TokenKind::Word(b)) if a == "async" && b == "fn"
        )
    });
    if !has_async {
        return Ok(None);
    }
    if let Some(error) = lexed.diagnostics.first() {
        return Err(error.message.clone());
    }
    let mut input = String::new();
    for (index, token) in tokens.iter().enumerate() {
        let raw = &source[token.span.start..token.span.end];
        match raw {
            "var"
                if index > 0 && matches!(tokens[index - 1].kind, TokenKind::Symbol("(" | ",")) =>
            {
                input.push_str("mut")
            }
            "var" => input.push_str("let mut"),
            name if name.starts_with("__clust_") => {
                return Err("reserved compiler identifier".into())
            }
            "mut" | "unsafe" | "extern" | "#" | "!" | "::" | "&" | "|" => {
                return Err(format!("unsupported async syntax '{raw}'"))
            }
            _ => input.push_str(raw),
        }
        input.push(' ');
    }
    let mut file = syn::parse_file(&input).map_err(|e| format!("async syntax: {e}"))?;
    let names: Vec<_> = file
        .items
        .iter()
        .filter_map(|i| {
            if let Item::Fn(f) = i {
                Some(f.sig.ident.to_string())
            } else {
                None
            }
        })
        .collect();
    let mut validator = Validator {
        names: &names,
        error: None,
    };
    let mut entry = false;
    for item in &mut file.items {
        let Item::Fn(f) = item else {
            return Err("async applications currently support top-level functions only".into());
        };
        if !f.sig.generics.params.is_empty()
            || f.sig.generics.where_clause.is_some()
            || f.sig.constness.is_some()
            || BUILTINS.contains(&f.sig.ident.to_string().as_str())
        {
            return Err("unsupported async function signature".into());
        }
        validator.visit_item_fn(f);
        if f.sig.ident == "main" {
            if f.sig.asyncness.is_none() || !f.sig.inputs.is_empty() {
                return Err("expected async fn main()".into());
            }
            f.vis = syn::parse_quote!(pub(super));
            entry = true;
        }
    }
    if let Some(e) = validator.error {
        return Err(e);
    }
    if !entry {
        return Err("async program requires async fn main()".into());
    }
    borrows::lower(&mut file)?;
    let code = file.into_token_stream();
    let runtime = clust_runtime::TOKIO_SUPPORT;
    Ok(Some(format!(
        r#"{runtime}
mod app {{
    use super::{{channel, actor, mutex, spawn, Sender, Receiver, Actor, Task, Mutex}};
    {code}
}}
fn main() {{
    let runtime = tokio::runtime::Builder::new_multi_thread().build().expect("Tokio runtime");
    let status = ExitStatus::code(runtime.block_on(app::main()));
    drop(runtime);
    std::process::exit(status);
}}
"#
    )))
}
struct Validator<'a> {
    names: &'a [String],
    error: Option<String>,
}
impl<'ast> Visit<'ast> for Validator<'_> {
    fn visit_stmt(&mut self, s: &'ast syn::Stmt) {
        if matches!(s, syn::Stmt::Item(_) | syn::Stmt::Macro(_)) {
            self.error = Some("nested declarations are unsupported".into());
            return;
        }
        visit::visit_stmt(self, s);
    }
    fn visit_type(&mut self, t: &'ast syn::Type) {
        match t {
            syn::Type::Tuple(_) => (),
            syn::Type::Path(p)
                if p.qself.is_none()
                    && p.path.segments.len() == 1
                    && [
                        "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "bool",
                        "Task", "Actor", "Mutex", "Sender", "Receiver", "Option",
                    ]
                    .contains(&p.path.segments[0].ident.to_string().as_str()) => {}
            _ => {
                self.error = Some("unsupported async type".into());
                return;
            }
        }
        visit::visit_type(self, t);
    }
    fn visit_expr(&mut self, e: &'ast Expr) {
        let ok = match e {
            Expr::Call(c) => {
                matches!(&*c.func,Expr::Path(p) if p.path.get_ident().is_some_and(|i|self.names.contains(&i.to_string())||BUILTINS.contains(&i.to_string().as_str())))
            }
            Expr::MethodCall(c) => METHODS.contains(&c.method.to_string().as_str()),
            Expr::Path(p) => p.qself.is_none() && p.path.get_ident().is_some(),
            Expr::Lit(l) => matches!(l.lit, syn::Lit::Int(_) | syn::Lit::Bool(_)),
            Expr::Await(_)
            | Expr::Binary(_)
            | Expr::Assign(_)
            | Expr::Block(_)
            | Expr::If(_)
            | Expr::Paren(_)
            | Expr::Return(_)
            | Expr::Tuple(_)
            | Expr::While(_)
            | Expr::Break(_)
            | Expr::Continue(_) => true,
            _ => false,
        };
        if !ok {
            self.error = Some(format!(
                "unsupported async expression: {}",
                e.to_token_stream()
            ));
            return;
        }
        visit::visit_expr(self, e);
    }
}
