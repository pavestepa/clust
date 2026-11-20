//! Infer borrow contracts, then let Rust track each loan inside the generated future.
use std::collections::{BTreeMap, BTreeSet};
use syn::{
    parse_quote,
    visit::{self, Visit},
    visit_mut::{self, VisitMut},
    Expr, FnArg, Item, ItemFn, Pat, Type,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Value,
    Shared,
    Exclusive,
}
type Contracts = BTreeMap<String, Vec<Mode>>;

fn name(expr: &Expr) -> Option<&syn::Ident> {
    match expr {
        Expr::Path(p) => p.path.get_ident(),
        Expr::Paren(p) => name(&p.expr),
        _ => None,
    }
}
fn is_copy(ty: &Type) -> bool {
    match ty {
        Type::Tuple(t) => t.elems.iter().all(is_copy),
        Type::Path(p) => p.path.get_ident().is_some_and(|id| {
            matches!(
                id.to_string().as_str(),
                "bool" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "usize"
            )
        }),
        _ => false,
    }
}
type BorrowedInput<'a> = dyn Fn(&Expr, usize) -> bool + 'a;
struct Uses<'a> {
    receiver_types: BTreeMap<String, syn::Path>,
    contracts: &'a Contracts,
    foreign: &'a BorrowedInput<'a>,
    consumed: BTreeSet<String>,
    hidden: BTreeSet<String>,
}
impl<'ast> Visit<'ast> for Uses<'_> {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let outer = self.hidden.clone();
        for stmt in &block.stmts {
            self.visit_stmt(stmt);
            if let syn::Stmt::Local(local) = stmt {
                self.hidden.extend(bindings(&local.pat));
            }
        }
        self.hidden = outer;
    }
    fn visit_expr(&mut self, e: &'ast Expr) {
        match e {
            Expr::MethodCall(m) => {
                let borrows_receiver = name(&m.receiver)
                    .and_then(|id| self.receiver_types.get(&id.to_string()))
                    .is_some_and(|receiver| {
                        let method = &m.method;
                        let callee = syn::parse_quote!(#receiver::#method);
                        (self.foreign)(&callee, 0)
                    });
                if !matches!(
                    m.method.to_string().as_str(),
                    "read"
                        | "update"
                        | "clone"
                        | "send"
                        | "sender"
                        | "is_ok"
                        | "is_err"
                        | "is_some"
                        | "is_none"
                ) && !borrows_receiver
                    || name(&m.receiver).is_none()
                {
                    self.visit_expr(&m.receiver);
                }
                for arg in &m.args {
                    self.visit_expr(arg)
                }
            }
            Expr::Call(c) => {
                let modes = name(&c.func).and_then(|id| self.contracts.get(&id.to_string()));
                if name(&c.func).is_some_and(|id| id == "spawn") {
                    // A spawned invocation owns its inputs.
                    visit::visit_expr(self, e);
                    if let Some(Expr::Call(inner)) = c.args.first() {
                        for arg in &inner.args {
                            self.visit_expr(arg)
                        }
                    }
                    return;
                }
                for (i, arg) in c.args.iter().enumerate() {
                    if name(arg).is_none()
                        || (modes.and_then(|m| m.get(i)) != Some(&Mode::Shared)
                            && !(self.foreign)(&c.func, i))
                    {
                        self.visit_expr(arg)
                    }
                }
            }
            Expr::Path(_) => {
                if let Some(id) = name(e) {
                    if !self.hidden.contains(&id.to_string()) {
                        self.consumed.insert(id.to_string());
                    }
                }
            }
            _ => visit::visit_expr(self, e),
        }
    }
}
fn owned_name(id: &syn::Ident) -> syn::Ident {
    quote::format_ident!("__clust_owned_{}", id)
}
fn borrowed_name(id: &syn::Ident) -> syn::Ident {
    quote::format_ident!("__clust_borrow_{}", id)
}
fn params(f: &ItemFn) -> Result<Vec<(syn::Ident, Mode)>, String> {
    f.sig
        .inputs
        .iter()
        .map(|a| {
            let FnArg::Typed(a) = a else {
                return Err("expected a function parameter".into());
            };
            let Pat::Ident(p) = &*a.pat else {
                return Err("expected a named async profile parameter".into());
            };
            if p.by_ref.is_some() || p.subpat.is_some() {
                return Err("unsupported parameter pattern".into());
            }
            Ok((
                p.ident.clone(),
                if p.mutability.is_some() {
                    Mode::Exclusive
                } else if is_copy(&a.ty) {
                    Mode::Value
                } else {
                    Mode::Shared
                },
            ))
        })
        .collect()
}

struct Rewrite<'a> {
    native: bool,
    foreign: &'a BorrowedInput<'a>,
    contracts: &'a Contracts,
    borrowed: BTreeSet<String>,
}
impl VisitMut for Rewrite<'_> {
    fn visit_block_mut(&mut self, b: &mut syn::Block) {
        let saved = self.borrowed.clone();
        for s in &mut b.stmts {
            self.visit_stmt_mut(s);
            if let syn::Stmt::Local(l) = s {
                for id in bindings(&l.pat) {
                    self.borrowed.remove(&id);
                }
            }
        }
        self.borrowed = saved;
    }
    fn visit_expr_mut(&mut self, e: &mut Expr) {
        if let Expr::Call(c) = e {
            if name(&c.func).is_some_and(|id| id == "spawn") && c.args.len() == 1 {
                if let Some(Expr::Call(inner)) = c.args.first_mut() {
                    self.call(inner, true);
                    return;
                }
            }
            self.call(c, false);
            return;
        }
        if let Expr::Path(_) = e {
            if name(e).is_some_and(|id| self.borrowed.contains(&id.to_string())) {
                *e = parse_quote!((*#e));
                return;
            }
        }
        visit_mut::visit_expr_mut(self, e);
    }
}

mod inference;
mod rewrite;
pub(crate) use self::inference::{lower, lower_with_foreign};
use self::rewrite::bindings;
