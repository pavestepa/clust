use super::*;
impl Checker<'_> {
    pub(super) fn check_cast(
        &mut self,
        value: &a::Expr,
        ty: &a::Type,
        span: Span,
        expected: Option<&Type>,
    ) -> Result<h::Expr> {
        let (kind, ty) = {
            let target = resolve_type_with(ty, self.constants, self.nominals)?;
            let value = self.expr(value, None, true)?;
            let address_cast = match (&value.ty, &target) {
                (Type::Pointer { mutable, pointee }, Type::Integer(I::Usize | I::Isize)) => Some((
                    if *mutable {
                        "raw_expose_mut"
                    } else {
                        "raw_expose"
                    },
                    pointee,
                )),
                (Type::Integer(I::Usize | I::Isize), Type::Pointer { pointee, .. }) => {
                    Some(("raw_recover", pointee))
                }
                _ => None,
            };
            if let Some((opcode, pointee)) = address_cast {
                self.require_raw_unsafe(span)?;
                if !pointee.raw_storage_value()
                    || !crate::layout::layout(pointee).is_ok_and(|l| l.size() != 0)
                {
                    return Err(error(
                        span,
                        "E0503",
                        "address casts require a non-zero-sized owned pointee",
                    ));
                }
                if let Some(expected) = expected {
                    same(expected, &target, span)?
                }
                return Ok(h::Expr {
                    kind: h::ExprKind::Unary(opcode, Box::new(value)),
                    ty: target,
                    span,
                });
            }
            if let (
                Type::Pointer {
                    mutable: from_mut,
                    pointee: from,
                },
                Type::Pointer {
                    mutable: to_mut,
                    pointee: to,
                },
            ) = (&value.ty, &target)
            {
                self.require_raw_unsafe(span)?;
                if (from != to && !from.raw_reinterpret_to(to))
                    || (*to_mut && !*from_mut)
                    || !from.raw_storage_value()
                {
                    return Err(error(
                            span,
                            "E0503",
                            "pointer cast requires identical pointees or scalar payloads without padding and cannot upgrade readonly access",
                        ));
                }
                if let Some(expected) = expected {
                    same(expected, &target, span)?;
                }
                return Ok(h::Expr {
                    kind: h::ExprKind::Cast(Box::new(value)),
                    ty: target,
                    span,
                });
            }
            let valid = value.ty.numeric() && target.numeric()
                || matches!(value.ty, Type::Bool | Type::Char)
                    && matches!(target, Type::Integer(_))
                || value.ty == Type::Integer(I::U8) && target == Type::Char;
            if !valid {
                return Err(error(span, "E0312", "invalid cast"));
            }
            (h::ExprKind::Cast(Box::new(value)), target)
        };
        finish(kind, ty, span, expected)
    }
}
