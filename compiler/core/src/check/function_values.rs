use super::*;
pub(super) const PENDING_CALLABLE: &str = "E0397";

// Plain code pointers retain their statically selected ABI, provenance and
// effects through annotations, aggregates and higher-order parameters.

impl Checker<'_> {
    pub(super) fn static_function_value(
        &mut self,
        source: &a::Expr,
        arguments: &[a::TypeArgument],
        expected: Option<&Type>,
        span: Span,
    ) -> Result<Option<h::Expr>> {
        let a::ExprKind::Member { value, .. } = &source.kind else {
            return Ok(None);
        };
        let owner = match &value.kind {
            a::ExprKind::Name(owner) => owner,
            a::ExprKind::Specialize { value, .. } => {
                let a::ExprKind::Name(owner) = &value.kind else {
                    return Ok(None);
                };
                owner
            }
            _ => return Ok(None),
        };
        if self.lookup(owner).is_some() {
            return Ok(None);
        }
        let Some((name, mut types)) = methods::resolved_static_call(source, self.nominals) else {
            return Ok(None);
        };
        types.extend_from_slice(arguments);
        self.function_value(&name, &types, expected, span)
    }
    pub(super) fn function_value(
        &mut self,
        name: &str,
        types: &[a::TypeArgument],
        expected: Option<&Type>,
        span: Span,
    ) -> Result<Option<h::Expr>> {
        self.check_function_access(name, span)?;
        if self.mono.borrow().templates.contains_key(name) {
            return self
                .generic_function_value(name, types, expected, span)
                .map(Some);
        }
        let Some(signature) = self.signatures.get(name) else {
            return Ok(None);
        };
        if !types.is_empty() {
            return Err(error(span, "E0370", "function is not generic"));
        }
        self.pointer_entry(signature.clone(), expected, span)
            .map(Some)
    }
}

mod generic_values;
mod indirect;
mod pointers;
use self::indirect::split_temporaries;
