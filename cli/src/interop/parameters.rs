//! Convert a native data-class receiver and explicit mutable parameters to
//! their Rust signatures. Plain constructor inputs can move into owned fields.
use super::lower::source;
use quote::ToTokens;
use std::collections::BTreeSet;
use syn::{parse_quote, FnArg, Pat};

pub(super) fn parameters(tokens: &[&str], roots: &BTreeSet<String>) -> Result<String, String> {
    let args = source(tokens, roots)?;
    let sig: syn::Signature = syn::parse_str(&format!("fn member{args}"))
        .map_err(|e| format!("invalid data class parameters: {e}"))?;
    let mut inputs = sig.inputs;
    for arg in &mut inputs {
        match arg {
            FnArg::Receiver(r) if r.mutability.is_some() => *arg = parse_quote!(&mut self),
            FnArg::Receiver(_) => *arg = parse_quote!(&self),
            FnArg::Typed(arg) => {
                let Pat::Ident(p) = arg.pat.as_mut() else {
                    return Err("expected a named parameter".into());
                };
                if p.mutability.take().is_some() {
                    let ty = &arg.ty;
                    arg.ty = parse_quote!(&mut #ty);
                }
            }
        }
    }
    Ok(inputs.to_token_stream().to_string())
}
