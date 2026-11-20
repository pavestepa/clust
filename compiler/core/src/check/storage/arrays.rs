use super::*;

impl Checker<'_> {
    pub(super) fn array_element(
        &mut self,
        source: &a::Expr,
        expected: Option<&Type>,
    ) -> Result<h::Expr> {
        let value = self.storage_value(source, expected)?;
        if !value.ty.copyable() && views::place(&value) && self.call_view_needed(source) {
            Ok(h::Expr {
                ty: Type::StoredView(Box::new(value.ty.clone())),
                span: value.span,
                kind: h::ExprKind::StoreView(Box::new(value)),
            })
        } else {
            Ok(value)
        }
    }
}

impl Checker<'_> {
    pub(in super::super) fn storage_array(
        &mut self,
        values: &[a::Expr],
        expected: Option<&Type>,
        span: Span,
    ) -> Result<h::Expr> {
        let hint = match expected {
            Some(Type::Array(element, _)) => Some((**element).clone()),
            _ => None,
        };
        let snapshot = self.clone();
        let mono = self.mono.borrow().clone();
        let mut logical_hint = hint.clone();
        let mut checked = Vec::new();
        for source in values {
            let value = self.array_element(source, logical_hint.as_ref())?;
            if logical_hint.is_none() {
                logical_hint = Some(value.ty.logical());
            }
            checked.push(value);
        }
        let mut element = hint;
        for value in &checked {
            element = Some(match element {
                Some(old) => union(&old, &value.ty, value.span)?,
                None => value.ty.clone(),
            });
        }
        let element =
            element.ok_or_else(|| error(span, "E0314", "empty array requires an element type"))?;
        if checked.iter().any(|value| value.ty != element) {
            let requests = self.storage_requests.clone();
            *self = snapshot;
            *self.mono.borrow_mut() = mono;
            self.storage_requests.extend(requests);
            checked = values
                .iter()
                .map(|value| self.array_element(value, Some(&element)))
                .collect::<Result<Vec<_>>>()?;
        }
        for value in &checked {
            if !self.request_storage_mode(value, &element)? {
                self.same_storage_mode(&element, &value.ty, value.span)?;
            }
        }
        let ty = Type::Array(Box::new(element), values.len() as u64);
        if let Some(expected) = expected {
            same(expected, &ty, span)?;
        }
        Ok(h::Expr {
            kind: h::ExprKind::Array(checked),
            ty,
            span,
        })
    }
}
