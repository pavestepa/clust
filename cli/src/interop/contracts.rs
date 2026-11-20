use clust_core::mir::{
    coli::foreign::{Contract, Parameter},
    Place,
};
use std::collections::BTreeMap;
use syn::{
    visit::{self, Visit},
    Expr,
};
#[derive(Default)]
pub(super) struct Catalog(BTreeMap<String, Contract>);
impl Catalog {
    pub(super) fn expanded(&mut self, path: &str, item: &serde_json::Value) -> Result<(), String> {
        let sig = &item["sig"];
        let args = sig["inputs"]
            .as_array()
            .ok_or("unsupported rustdoc function signature")?;
        let mut inputs = Vec::new();
        let mut result_origins = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let r = &arg[1]["borrowed_ref"];
            if r.is_null() {
                inputs.push(Parameter::Value);
                continue;
            }
            inputs.push(if r["is_mutable"] == true {
                Parameter::Exclusive
            } else {
                Parameter::Shared
            });
            let output = &sig["output"]["borrowed_ref"];
            if item["header"]["is_async"] == true
                || (!output.is_null()
                    && ((output["lifetime"].is_null() && (args[0][0] != "self" || i == 0))
                        || !output["lifetime"].is_null()
                            && output["lifetime"] == r["lifetime"]
                            && output["lifetime"] != "'static"))
            {
                result_origins.push(i);
            }
        }
        self.0.insert(
            path.into(),
            Contract {
                inputs,
                result_origins,
            },
        );
        Ok(())
    }
    pub(super) fn contract(&self, c: &syn::ExprCall) -> Option<&Contract> {
        self.by_callee(&c.func)
    }
    fn by_callee(&self, callee: &Expr) -> Option<&Contract> {
        let Expr::Path(p) = callee else { return None };
        let key = p
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        self.0.get(key.strip_prefix("crate::").unwrap_or(&key))
    }
    pub(super) fn borrowed_input(&self, callee: &Expr, i: usize) -> bool {
        matches!(
            self.by_callee(callee).and_then(|c| c.inputs.get(i)),
            Some(Parameter::Shared | Parameter::Exclusive)
        )
    }
    pub fn validate(&self, file: &syn::File) -> Result<(), String> {
        let mut checker = Calls {
            catalog: self,
            error: None,
        };
        checker.visit_file(file);
        checker.error.map_or(Ok(()), Err)
    }
}
struct Calls<'a> {
    catalog: &'a Catalog,
    error: Option<String>,
}
impl<'a> Visit<'a> for Calls<'_> {
    fn visit_block(&mut self, b: &'a syn::Block) {
        if let Err(e) = super::origins::check(b, self.catalog) {
            self.error = Some(e);
        }
        visit::visit_block(self, b);
    }
    fn visit_expr_call(&mut self, c: &'a syn::ExprCall) {
        if let Expr::Path(p) = &*c.func {
            let path = p
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            if let Some(contract) = self.catalog.0.get(&path) {
                let mut locals = BTreeMap::new();
                let places = c
                    .args
                    .iter()
                    .map(|a| {
                        let Expr::Path(p) = a else { return None };
                        let ident = p.path.get_ident()?;
                        let next = locals.len();
                        Some(Place {
                            local: *locals.entry(ident.to_string()).or_insert(next),
                            projections: vec![],
                        })
                    })
                    .collect::<Vec<_>>();
                if let Err(e) = contract.check_arguments(&places) {
                    self.error = Some(format!("{path}: {e}"));
                }
            }
        }
        visit::visit_expr_call(self, c)
    }
}
