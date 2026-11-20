use super::*;
impl Checker<'_> {
    pub(super) fn check_binary(
        &mut self,
        operator: &&'static str,
        left: &a::Expr,
        right: &a::Expr,
        span: Span,
        expected: Option<&Type>,
    ) -> Result<h::Expr> {
        let (kind, ty) = {
            let comparison = ["==", "!=", "<", ">", "<=", ">="].contains(operator);
            let logical = ["&&", "||"].contains(operator);
            let shift = ["<<", ">>"].contains(operator);
            let pointer_subtraction =
                *operator == "-" && matches!(self.known_type(left), Some(Type::Pointer { .. }));
            let hint = if pointer_subtraction {
                None
            } else if logical {
                Some(&Type::Bool)
            } else if comparison {
                None
            } else {
                expected
            };
            let inferred = if hint.is_none()
                && !pointer_subtraction
                && !shift
                && !matches!(self.known_type(left), Some(Type::Pointer { .. }))
            {
                self.known_type(right)
            } else {
                None
            };
            let hint = hint.or(inferred.as_ref());
            let left = self.expr(left, hint, true)?;
            if matches!(left.ty, Type::Pointer { .. }) && matches!(*operator, "+" | "-") {
                self.require_raw_unsafe(span)?;
                if !matches!(&left.ty, Type::Pointer { pointee, .. } if pointee.raw_storage_value())
                {
                    return Err(error(
                        span,
                        "E0900",
                        "pointer arithmetic requires a supported owned pointee",
                    ));
                }
                let right_hint = if *operator == "-"
                    && (matches!(self.known_type(right), Some(Type::Pointer { .. }))
                        || matches!(right.kind, a::ExprKind::Call { .. }))
                {
                    None
                } else {
                    Some(Type::Integer(I::Isize))
                };
                let right = self.expr(right, right_hint.as_ref(), true)?;
                if *operator == "-" && matches!(right.ty, Type::Pointer { .. }) {
                    let (Type::Pointer { pointee: a, .. }, Type::Pointer { pointee: b, .. }) =
                        (&left.ty, &right.ty)
                    else {
                        unreachable!()
                    };
                    same(a, b, span)?;
                    if crate::layout::layout(a).is_ok_and(|layout| layout.size() == 0) {
                        return Err(error(
                            span,
                            "E0503",
                            "pointer difference requires a non-zero-sized pointee",
                        ));
                    }
                    let ty = Type::Integer(I::Isize);
                    if let Some(expected) = expected {
                        same(expected, &ty, span)?;
                    }
                    return Ok(h::Expr {
                        kind: h::ExprKind::Binary(
                            "raw_difference",
                            Box::new(left),
                            Box::new(right),
                        ),
                        ty,
                        span,
                    });
                }
                same(&Type::Integer(I::Isize), &right.ty, span)?;
                let ty = left.ty.clone();
                if let Some(expected) = expected {
                    same(expected, &ty, span)?;
                }
                return Ok(h::Expr {
                    kind: h::ExprKind::Binary(
                        if *operator == "+" {
                            "raw_offset"
                        } else {
                            "raw_offset_sub"
                        },
                        Box::new(left),
                        Box::new(right),
                    ),
                    ty,
                    span,
                });
            }
            let right = self.expr(right, if shift { None } else { Some(&left.ty) }, true)?;
            if matches!(left.ty, Type::Class { .. }) && ["==", "!="].contains(operator) {
                if let Some(expected) = expected {
                    same(expected, &Type::Bool, span)?;
                }
                let identity = h::Expr {
                    kind: h::ExprKind::Identity(Box::new(left), Box::new(right)),
                    ty: Type::Bool,
                    span,
                };
                return Ok(if *operator == "==" {
                    identity
                } else {
                    h::Expr {
                        kind: h::ExprKind::Unary("!", Box::new(identity)),
                        ty: Type::Bool,
                        span,
                    }
                });
            }
            let valid = if logical {
                left.ty == Type::Bool
            } else if shift {
                matches!(left.ty, Type::Integer(_)) && matches!(right.ty, Type::Integer(_))
            } else if ["&", "|", "^"].contains(operator) {
                matches!(left.ty, Type::Integer(_))
            } else if comparison {
                left.ty.numeric() || matches!(left.ty, Type::Bool | Type::Char)
            } else {
                left.ty.numeric()
            };
            if !valid {
                return Err(error(
                    span,
                    "E0311",
                    "binary operator is not available for these types",
                ));
            }
            let ty = if comparison || logical {
                Type::Bool
            } else {
                left.ty.clone()
            };
            (
                h::ExprKind::Binary(operator, Box::new(left), Box::new(right)),
                ty,
            )
        };
        finish(kind, ty, span, expected)
    }
}
