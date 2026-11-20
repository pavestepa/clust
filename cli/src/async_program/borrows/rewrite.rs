use super::*;

impl Rewrite<'_> {
    pub(super) fn call(&mut self, c: &mut syn::ExprCall, spawned: bool) {
        let modes = name(&c.func).and_then(|id| self.contracts.get(&id.to_string()));
        let owned = spawned && modes.is_some_and(|m| !m.contains(&Mode::Exclusive));
        if modes.is_some_and(|m| m.contains(&Mode::Shared) && !m.contains(&Mode::Exclusive)) {
            let original = name(&c.func).unwrap();
            if owned && self.native {
                let wrapper = owned_name(original);
                c.func = parse_quote!(#wrapper);
            } else if !owned && !self.native {
                let target = borrowed_name(original);
                c.func = parse_quote!(#target);
            }
        }
        for (i, arg) in c.args.iter_mut().enumerate() {
            if (self.foreign)(&c.func, i)
                && name(arg).is_some_and(|id| self.borrowed.contains(&id.to_string()))
            {
                continue;
            }
            self.visit_expr_mut(arg);
            if !owned {
                match modes.and_then(|m| m.get(i)) {
                    Some(Mode::Shared) => *arg = parse_quote!(&#arg),
                    Some(Mode::Exclusive) => *arg = parse_quote!(&mut #arg),
                    _ => (),
                }
            }
        }
    }
}

pub(super) fn bindings(pat: &Pat) -> Vec<String> {
    #[derive(Default)]
    struct Names(Vec<String>);
    impl<'a> Visit<'a> for Names {
        fn visit_pat_ident(&mut self, pat: &'a syn::PatIdent) {
            self.0.push(pat.ident.to_string());
            visit::visit_pat_ident(self, pat);
        }
    }
    let mut names = Names::default();
    names.visit_pat(pat);
    names.0
}
