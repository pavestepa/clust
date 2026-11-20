use super::*;
impl Checker<'_> {
    pub(super) fn check_unary(
        &mut self,
        operator: &&'static str,
        value: &a::Expr,
        span: Span,
        expected: Option<&Type>,
    ) -> Result<h::Expr> {
        let (kind, ty) = {
            if *operator == "-" {
                if let a::ExprKind::Literal(a::Literal::Integer(_)) = &value.kind {
                    let a::ExprKind::Literal(literal_value) = &value.kind else {
                        unreachable!()
                    };
                    let (kind, ty) = literal(literal_value, expected, true, span)?;
                    return Ok(h::Expr { kind, ty, span });
                }
            }
            fn view_name(value: &a::Expr) -> bool {
                match &value.kind {
                    a::ExprKind::Name(_) => true,
                    a::ExprKind::Group(v) => view_name(v),
                    _ => false,
                }
            }
            if ["&", "&var"].contains(operator) {
                let value = self.expr(value, None, true)?;
                if !views::place(&value) || !value.ty.raw_copy_value() {
                    return Err(error(
                        span,
                        "E0502",
                        "raw address requires an existing addressable plain-data place",
                    ));
                }
                if *operator == "&var" {
                    self.mutable_place(&value)?;
                }
                let ty = Type::Pointer {
                    mutable: *operator == "&var",
                    pointee: Box::new(value.ty.clone()),
                };
                if let Some(expected) = expected {
                    same(expected, &ty, span)?;
                }
                return Ok(h::Expr {
                    kind: h::ExprKind::Unary(
                        if *operator == "&var" {
                            "raw_addr_mut"
                        } else {
                            "raw_addr"
                        },
                        Box::new(value),
                    ),
                    ty,
                    span,
                });
            }
            let named_view = view_name(value);
            let value = self.expr(value, if *operator == "*" { None } else { expected }, true)?;
            if *operator == "*" && named_view && matches!(value.kind, h::ExprKind::View(_)) {
                if let Some(expected) = expected {
                    same(expected, &value.ty, span)?;
                }
                return Ok(value);
            }
            if let Type::Pointer { pointee, .. } = &value.ty {
                if *operator == "*" {
                    self.require_raw_unsafe(span)?;
                    if !pointee.raw_copy_value() {
                        return Err(error(
                            span,
                            "E0900",
                            "raw dereference currently requires a plain-data pointee",
                        ));
                    }
                    let ty = (**pointee).clone();
                    if let Some(expected) = expected {
                        same(expected, &ty, span)?;
                    }
                    return Ok(h::Expr {
                        kind: h::ExprKind::Unary("raw_read", Box::new(value)),
                        ty,
                        span,
                    });
                }
            }
            let ty = value.ty.clone();
            let valid = match *operator {
                "-" => {
                    matches!(ty,Type::Integer(i) if i.is_signed())
                        || matches!(ty, Type::F32 | Type::F64)
                }
                "!" => matches!(ty, Type::Bool | Type::Integer(_)),
                _ => false,
            };
            if !valid {
                return Err(error(
                    span,
                    "E0311",
                    "operator is not available for this type",
                ));
            }
            (h::ExprKind::Unary(operator, Box::new(value)), ty)
        };
        finish(kind, ty, span, expected)
    }
}
