use super::*;

impl Checker<'_> {
    pub(super) fn local(
        &mut self,
        name: String,
        ty: Type,
        mutable: bool,
        span: Span,
    ) -> Result<usize> {
        let scope = self.scopes.last_mut().unwrap();
        // Same-scope shadowing follows declaration order; each binding has a new ID.
        let id = self.locals.len();
        scope.insert(name.clone(), id);
        self.locals.push(h::Local {
            view: None,
            name,
            ty,
            mutable,
            span,
        });
        Ok(id)
    }
}

impl Checker<'_> {
    pub(super) fn lookup(&self, name: &str) -> Option<usize> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }
}
