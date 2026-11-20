use super::*;
impl Checker<'_> {
    pub(super) fn check_assign(
        &mut self,
        operator: &&'static str,
        place: &a::Expr,
        value: &a::Expr,
        source: &a::Expr,
        span: Span,
        expected: Option<&Type>,
    ) -> Result<h::Expr> {
        let (kind, ty) = {
            if let a::ExprKind::Index {
                value: receiver,
                index,
            } = &place.kind
            {
                let checked = self.expr(receiver, None, true)?;
                if matches!(checked.ty, Type::Class { .. }) {
                    if *operator != "=" {
                        return Err(error(
                            span,
                            "E0315",
                            "compound indexed assignment requires an explicit read and set",
                        ));
                    }
                    let callee = a::Expr {
                        kind: a::ExprKind::Member {
                            value: receiver.clone(),
                            name: "set".into(),
                        },
                        span,
                    };
                    return self
                        .instance_call(
                            &callee,
                            &[],
                            &[(**index).clone(), value.clone()],
                            expected,
                            span,
                        )?
                        .ok_or_else(|| {
                            error(span, "E0315", "class does not support indexed assignment")
                        });
                }
            }
            if let a::ExprKind::Member {
                value: receiver,
                name,
            } = &place.kind
            {
                if !name.starts_with('$') {
                    let checked = self.expr(receiver, None, true)?;
                    if polymorphism::is_polymorphic(&checked.ty) {
                        let callee = a::Expr {
                            kind: a::ExprKind::Member {
                                value: receiver.clone(),
                                name: polymorphism::accessor(name, operator),
                            },
                            span,
                        };
                        return self
                            .instance_call(
                                &callee,
                                &[],
                                std::slice::from_ref(value),
                                expected,
                                span,
                            )?
                            .ok_or_else(|| {
                                error(span, "E0352", "base field assignment is not available")
                            });
                    }
                }
            }
            let place = self.expr(place, None, true)?;
            if let h::ExprKind::Unary("raw_read", pointer) = &place.kind {
                if !matches!(pointer.ty, Type::Pointer { mutable: true, .. }) {
                    return Err(error(
                        span,
                        "E0503",
                        "raw write requires a writable pointer",
                    ));
                }
                let shift = ["<<=", ">>="].contains(operator);
                let value = self.expr(value, if shift { None } else { Some(&place.ty) }, true)?;
                if let Some(expected) = expected {
                    same(expected, &Type::Unit, span)?;
                }
                if *operator != "=" {
                    let integer_only = ["&=", "|=", "^=", "<<=", ">>="].contains(operator);
                    if !place.ty.numeric()
                        || integer_only && !matches!(place.ty, Type::Integer(_))
                        || shift && !matches!(value.ty, Type::Integer(_))
                    {
                        return Err(error(span, "E0311", "invalid raw compound assignment"));
                    }
                    // Capture the address and old value exactly once, before
                    // evaluating the RHS, just like ordinary compound assignment.
                    let address = self.locals.len();
                    let previous = address + 1;
                    for (name, ty) in [
                        ("<raw address>", pointer.ty.clone()),
                        ("<raw previous>", place.ty.clone()),
                    ] {
                        self.locals.push(h::Local {
                            name: name.into(),
                            ty,
                            mutable: false,
                            view: None,
                            span,
                        });
                    }
                    let address_expr = h::Expr {
                        kind: h::ExprKind::Local(address),
                        ty: pointer.ty.clone(),
                        span,
                    };
                    let old = h::Expr {
                        kind: h::ExprKind::Unary("raw_read", Box::new(address_expr.clone())),
                        ty: place.ty.clone(),
                        span,
                    };
                    let computed = h::Expr {
                        kind: h::ExprKind::Binary(
                            operator.trim_end_matches('='),
                            Box::new(h::Expr {
                                kind: h::ExprKind::Local(previous),
                                ty: place.ty.clone(),
                                span,
                            }),
                            Box::new(value),
                        ),
                        ty: place.ty.clone(),
                        span,
                    };
                    let write = h::Expr {
                        kind: h::ExprKind::Binary(
                            "raw_write",
                            Box::new(address_expr),
                            Box::new(computed),
                        ),
                        ty: Type::Unit,
                        span,
                    };
                    return Ok(h::Expr {
                        kind: h::ExprKind::Block(h::Block {
                            construction: vec![],
                            storage: vec![address, previous],
                            statements: vec![
                                h::Statement::Initialize(address, (**pointer).clone()),
                                h::Statement::Initialize(previous, old),
                                h::Statement::Evaluate(write),
                            ],
                            result: Type::Unit,
                            diverges: false,
                            span,
                        }),
                        ty: Type::Unit,
                        span,
                    });
                }
                return Ok(h::Expr {
                    kind: h::ExprKind::Binary("raw_write", pointer.clone(), Box::new(value)),
                    ty: Type::Unit,
                    span,
                });
            }
            if let h::ExprKind::StoredView(slot) = &place.kind {
                if *operator != "=" {
                    return Err(error(
                        span,
                        "E0396",
                        "stored readonly view only supports rebinding",
                    ));
                }
                self.mutable_place(slot)?;
                let value = self.storage_value(value, Some(&slot.ty))?;
                self.same_storage_mode(&slot.ty, &value.ty, value.span)?;
                return Ok(h::Expr {
                    kind: h::ExprKind::Assign {
                        place: slot.clone(),
                        operator,
                        value: Box::new(value),
                    },
                    ty: Type::Unit,
                    span,
                });
            }
            if let h::ExprKind::View(local) = place.kind {
                // Only explicit dereference writes the whole target.
                fn explicit(v: &a::Expr) -> bool {
                    match &v.kind {
                        a::ExprKind::Unary { operator: "*", .. } => true,
                        a::ExprKind::Group(v) => explicit(v),
                        _ => false,
                    }
                }
                let a::ExprKind::Assign { place: source, .. } = &source.kind else {
                    unreachable!()
                };
                if explicit(source) && matches!(place.ty, Type::Class { .. }) {
                    return Err(error(
                        span,
                        "E0394",
                        "class view cannot replace its owning slot",
                    ));
                }
                if !explicit(source) {
                    if !self.locals[local].mutable {
                        return Err(error(
                            span,
                            "E0382",
                            "rebinding requires a mutable view binding",
                        ));
                    }
                    if *operator != "=" {
                        return Err(error(
                            span,
                            "E0382",
                            "use *view for compound assignment to the target",
                        ));
                    }
                    let value = self.expr(value, Some(&place.ty), true)?;
                    self.same_storage_mode(&place.ty, &value.ty, value.span)?;
                    if !views::place(&value) {
                        return Err(error(
                            span,
                            "E0382",
                            "view rebinding requires an existing place",
                        ));
                    }
                    if self.locals[local].view == Some(crate::mir::loans::Access::Mutable) {
                        self.mutable_place(&value)?;
                    }
                    self.storage_sources
                        .entry(local)
                        .or_default()
                        .push(value.clone());
                    return Ok(h::Expr {
                        kind: h::ExprKind::Borrow {
                            local,
                            place: Box::new(value),
                        },
                        ty: Type::Unit,
                        span,
                    });
                }
            }
            self.mutable_place(&place)?;
            let shift = ["<<=", ">>="].contains(operator);
            let value = if *operator == "="
                && self.infers_storage_place(&place)
                && matches!(
                    place.kind,
                    h::ExprKind::Field(..) | h::ExprKind::Index(..) | h::ExprKind::Payload(..)
                ) {
                self.storage_assignment(value, &place)?
            } else {
                self.expr(value, if shift { None } else { Some(&place.ty) }, true)?
            };
            if !shift && !self.request_storage_mode(&place, &value.ty)? {
                self.same_storage_mode(&place.ty, &value.ty, value.span)?;
            }
            if *operator != "=" {
                let integer_only = ["&=", "|=", "^=", "<<=", ">>="].contains(operator);
                if !place.ty.numeric()
                    || integer_only && !matches!(place.ty, Type::Integer(_))
                    || shift && !matches!(value.ty, Type::Integer(_))
                {
                    return Err(error(span, "E0311", "invalid compound assignment"));
                }
            }
            (
                h::ExprKind::Assign {
                    place: Box::new(place),
                    operator,
                    value: Box::new(value),
                },
                Type::Unit,
            )
        };
        finish(kind, ty, span, expected)
    }
}
