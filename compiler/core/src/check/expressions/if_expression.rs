use super::*;
impl Checker<'_> {
    pub(super) fn check_if(
        &mut self,
        condition: &a::Expr,
        then_block: &a::Block,
        else_branch: &Option<Box<a::Expr>>,
        value_context: bool,
        span: Span,
        expected: Option<&Type>,
    ) -> Result<h::Expr> {
        let (kind, ty) = {
            let condition = Box::new(self.expr(condition, Some(&Type::Bool), true)?);
            let yes = self.block(then_block, expected, value_context)?;
            let branch_hint = if yes.diverges {
                expected
            } else if value_context {
                Some(&yes.result)
            } else {
                None
            };
            let no = else_branch
                .as_ref()
                .map(|branch| self.branch(branch, branch_hint, value_context))
                .transpose()?;
            if let Some(no) = &no {
                if !yes.diverges && !no.diverges {
                    let joined = storage::union(&yes.result, &no.result, span)?;
                    if !self.request_expression_mode(span, &yes.result, &joined)?
                        && !self.storage_requests.contains_key(&storage::key(span))
                    {
                        storage::same_mode(&joined, &yes.result, span)?;
                        storage::same_mode(&joined, &no.result, span)?;
                    }
                }
            }
            let ty = if value_context && !yes.diverges && no.as_ref().is_some_and(|b| !b.diverges) {
                storage::union(&yes.result, &no.as_ref().unwrap().result, span)?
            } else if !value_context {
                Type::Unit
            } else if yes.diverges {
                no.as_ref().map_or(Type::Unit, |b| b.result.clone())
            } else {
                yes.result.clone()
            };
            if no.is_none() && value_context && ty != Type::Unit {
                return Err(error(span, "E0316", "value-producing if requires else"));
            }
            (h::ExprKind::If { condition, yes, no }, ty)
        };
        finish(kind, ty, span, expected)
    }
}
