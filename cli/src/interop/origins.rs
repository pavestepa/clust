//! Check straight-line uses of a Rust result borrowed from a named local.
//! Rust additionally checks nested control flow, reborrows and generic futures.
use super::contracts::Catalog;
use clust_core::mir::coli::foreign::Parameter;
use std::collections::BTreeMap;
use syn::{visit::Visit, Expr, Pat, Stmt};

#[derive(Default)]
struct Names(Vec<String>);
impl<'a> Visit<'a> for Names {
    fn visit_expr_path(&mut self, p: &'a syn::ExprPath) {
        if let Some(n) = p.path.get_ident() {
            self.0.push(n.to_string())
        }
    }
}
fn local(e: &Expr) -> Option<String> {
    let Expr::Path(p) = e else { return None };
    Some(p.path.get_ident()?.to_string())
}
pub(super) fn check(b: &syn::Block, catalog: &Catalog) -> Result<(), String> {
    let mut last = BTreeMap::new();
    for (i, s) in b.stmts.iter().enumerate() {
        let mut names = Names::default();
        names.visit_stmt(s);
        for n in names.0 {
            last.insert(n, i);
        }
    }
    let mut views = Vec::<(String, String, Parameter, usize)>::new();
    for (i, s) in b.stmts.iter().enumerate() {
        let (binding, expr) = match s {
            Stmt::Local(l) => {
                let Pat::Ident(id) = &l.pat else {
                    views.clear();
                    continue;
                };
                let name = id.ident.to_string();
                // A shadowed binding has a new identity.
                views.retain(|(origin, view, _, _)| origin != &name && view != &name);
                let Some(init) = &l.init else { continue };
                (Some(name), &*init.expr)
            }
            Stmt::Expr(e, _) => (None, e),
            _ => {
                views.clear();
                continue;
            }
        };
        let Expr::Call(c) = expr else {
            views.clear();
            continue;
        };
        let Some(contract) = catalog.contract(c) else {
            views.clear();
            continue;
        };
        for (index, (arg, mode)) in c.args.iter().zip(&contract.inputs).enumerate() {
            let Some(owner) = local(arg) else { continue };
            for (base, view, access, end) in &views {
                if base == &owner
                    && *end >= i
                    && (*mode == Parameter::Exclusive
                        || (*access == Parameter::Exclusive && *mode == Parameter::Shared))
                {
                    return Err(format!(
                        "E0381: cannot borrow '{owner}' while Rust result '{view}' is live"
                    ));
                }
            }
            if contract.result_origins.contains(&index) {
                if let Some(view) = &binding {
                    if let Some(end) = last.get(view) {
                        views.push((owner, view.clone(), *mode, *end));
                    }
                }
            }
        }
    }
    Ok(())
}
