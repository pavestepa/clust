use super::*;

impl Checker<'_> {
    pub(in super::super) fn storage_assignment(
        &mut self,
        source: &a::Expr,
        place: &h::Expr,
    ) -> Result<h::Expr> {
        fn class_field(v: &h::Expr) -> bool {
            match &v.kind {
                h::ExprKind::Field(base, _) => {
                    matches!(base.ty, Type::Class { .. }) || class_field(base)
                }
                h::ExprKind::Index(base, _) | h::ExprKind::Payload(base, _) => class_field(base),
                _ => false,
            }
        }
        if !class_field(place) {
            return self.storage_value(source, Some(&place.ty));
        }
        let value = self.expr(source, Some(&place.ty), true)?;
        if matches!(place.ty, Type::StoredView(_)) {
            self.request_parameter_view(&value);
        }
        if !value.ty.copyable()
            && views::place(&value)
            && (views::view_root(&value) || matches!(place.ty, Type::StoredView(_)))
        {
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
    pub(in super::super) fn storage_value(
        &mut self,
        source: &a::Expr,
        expected: Option<&Type>,
    ) -> Result<h::Expr> {
        let value = self.expr(source, expected, true)?;
        if let Some(expected) = expected {
            let expected = match expected {
                Type::StoredView(target) => &**target,
                ty => ty,
            };
            self.request_storage_mode(&value, expected)?;
        }
        if matches!(expected, Some(Type::StoredView(_))) {
            self.request_parameter_view(&value);
        }
        if !value.ty.copyable()
            && views::place(&value)
            && (views::view_root(&value)
                || matches!(expected, Some(Type::StoredView(_)))
                || self.ownership_view_needed(source))
        {
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
