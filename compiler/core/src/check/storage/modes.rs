use super::*;

pub(in super::super) fn same_mode(expected: &Type, actual: &Type, span: Span) -> Result<()> {
    if let (
        Type::Function {
            unsafe_: a_unsafe,
            parameters: a,
            result: ar,
            contract: Some(ac),
        },
        Type::Function {
            unsafe_: b_unsafe,
            parameters: b,
            result: br,
            contract: Some(bc),
        },
    ) = (expected, actual)
    {
        let view = match (&ac.view, &bc.view) {
            (Some(a), Some(b)) => a.access == b.access && b.origins.is_subset(&a.origins),
            (None, None) => true,
            _ => false,
        };
        if a_unsafe == b_unsafe && a == b && ar == br && ac.modes == bc.modes && view {
            return Ok(());
        }
    }
    if expected != actual
        && (expected.contains_views() || actual.contains_views() || callable_mode(expected))
    {
        Err(error(
            span,
            "E0396",
            "incompatible static storage or callable ABI modes",
        ))
    } else {
        same(expected, actual, span)
    }
}

impl Checker<'_> {
    pub(in super::super) fn same_storage_mode(
        &self,
        expected: &Type,
        actual: &Type,
        span: Span,
    ) -> Result<()> {
        // A caller can be checked before its callee's representation is known.
        // Recheck physical compatibility once result inference has converged.
        if !self.mono.borrow().pending_results.is_empty() {
            same(expected, actual, span)
        } else {
            same_mode(expected, actual, span)
        }
    }
}

impl Checker<'_> {
    pub(in super::super) fn request_parameter_view(&mut self, value: &h::Expr) {
        if views::view_root(value) {
            return;
        }
        for (root, _) in place_paths(self, value, &mut Vec::new()) {
            if self.storage_parameters.contains(&root) && self.locals[root].view.is_none() {
                self.storage_requests.insert(
                    key(self.locals[root].span),
                    Type::StoredView(Box::new(self.locals[root].ty.clone())),
                );
            }
        }
    }
}
