use super::*;
impl Checker<'_> {
    pub(super) fn check_call(
        &mut self,
        callee: &a::Expr,
        types: &[a::TypeArgument],
        arguments: &[a::Expr],
        span: Span,
        expected: Option<&Type>,
    ) -> Result<h::Expr> {
        if let a::ExprKind::Name(name) = &callee.kind {
            if let Some(value) = self.text_builtin(name, types, arguments, span)? {
                if let Some(expected) = expected {
                    same(expected, &value.ty, span)?;
                }
                return Ok(value);
            }
            if matches!(name.as_str(), "print" | "println" | "assert")
                && self.lookup(name).is_none()
                && !self.signatures.contains_key(name)
                && !self.mono.borrow().templates.contains_key(name)
            {
                if !types.is_empty() || arguments.len() != 1 {
                    return Err(error(
                        span,
                        "E0313",
                        "print, println and assert take exactly one argument and no type arguments",
                    ));
                }
                let assertion = name == "assert";
                let value = self.expr(
                    &arguments[0],
                    if assertion { Some(&Type::Bool) } else { None },
                    true,
                )?;
                if !assertion
                    && matches!(&value.ty, Type::Class {name,..} if self.nominals.origin(name).rsplit("::").next()==Some("String"))
                {
                    let callee = a::Expr {
                        kind: a::ExprKind::Member {
                            value: Box::new(arguments[0].clone()),
                            name: "write".into(),
                        },
                        span,
                    };
                    let write = self
                        .instance_call(&callee, &[], &[], Some(&Type::Unit), span)?
                        .ok_or_else(|| error(span, "E0312", "String requires a write method"))?;
                    let mut statements = vec![h::Statement::Evaluate(write)];
                    if name == "println" {
                        statements.push(h::Statement::Evaluate(h::Expr {
                            kind: h::ExprKind::Unary(
                                "io_println",
                                Box::new(h::Expr {
                                    kind: h::ExprKind::Value(Value::Str(String::new())),
                                    ty: Type::Str,
                                    span,
                                }),
                            ),
                            ty: Type::Unit,
                            span,
                        }));
                    }
                    if let Some(expected) = expected {
                        same(expected, &Type::Unit, span)?;
                    }
                    return Ok(h::Expr {
                        kind: h::ExprKind::Block(h::Block {
                            construction: vec![],
                            storage: vec![],
                            statements,
                            result: Type::Unit,
                            diverges: false,
                            span,
                        }),
                        ty: Type::Unit,
                        span,
                    });
                }
                if !assertion
                    && !matches!(
                        value.ty,
                        Type::Str
                            | Type::Bool
                            | Type::Char
                            | Type::Integer(_)
                            | Type::F32
                            | Type::F64
                    )
                {
                    return Err(error(
                        span,
                        "E0312",
                        "output requires a string or scalar value",
                    ));
                }
                if let Some(expected) = expected {
                    same(expected, &Type::Unit, span)?;
                }
                let operation = match name.as_str() {
                    "print" => "io_print",
                    "println" => "io_println",
                    _ => "assert",
                };
                return Ok(h::Expr {
                    kind: h::ExprKind::Unary(operation, Box::new(value)),
                    ty: Type::Unit,
                    span,
                });
            }
        }
        if let a::ExprKind::Name(name) = &callee.kind {
            if let Some(id) = clust_frontend::intrinsics::Intrinsic::resolve(name) {
                return self.intrinsic(id, types, arguments, expected, span);
            }
        }
        if matches!(&callee.kind, a::ExprKind::Name(name) if name=="same_identity" && self.lookup(name).is_none() && !self.signatures.contains_key(name) && !self.mono.borrow().templates.contains_key(name))
        {
            if !types.is_empty() || arguments.len() != 2 {
                return Err(error(
                    span,
                    "E0313",
                    "same_identity requires two class operands",
                ));
            }
            let left = self.expr(&arguments[0], None, true)?;
            let right = self.expr(&arguments[1], None, true)?;
            if !matches!(left.ty, Type::Class { .. }) || !matches!(right.ty, Type::Class { .. }) {
                return Err(error(
                    span,
                    "E0311",
                    "same_identity requires class operands",
                ));
            }
            if let Some(expected) = expected {
                same(expected, &Type::Bool, span)?;
            }
            return Ok(h::Expr {
                kind: h::ExprKind::Identity(Box::new(left), Box::new(right)),
                ty: Type::Bool,
                span,
            });
        }
        if types.is_empty() {
            if let Some(value) = self.enum_call(callee, arguments, expected, span)? {
                return Ok(value);
            }
        }
        if let Some(value) = self.instance_call(callee, types, arguments, expected, span)? {
            return self.contextual_result(value, expected);
        }
        if let Some(value) = self.indirect_call(callee, types, arguments, span)? {
            if let Some(expected) = expected {
                same(expected, &value.ty, span)?;
            }
            return self.contextual_result(value, expected);
        }
        let (name, mut generic_arguments) = match &callee.kind {
            a::ExprKind::Name(name) => (name.clone(), Vec::new()),
            _ => methods::resolved_static_call(callee, self.nominals).ok_or_else(|| {
                error(
                    span,
                    "E0900",
                    "instance/function pointer calls require further checking",
                )
            })?,
        };
        if let a::ExprKind::Member { value, .. } = &callee.kind {
            if let a::ExprKind::Name(owner) = &value.kind {
                if self.lookup(owner).is_some() {
                    return Err(error(
                        span,
                        "E0900",
                        "instance calls require receiver checking",
                    ));
                }
            }
        }
        generic_arguments.extend(types.iter().cloned());
        let types = &generic_arguments;
        let name = &name;
        self.check_function_access(name, span)?;
        if self.lookup(name).is_some() {
            return Err(error(callee.span, "E0313", "local binding shadows this function; callable values require function-pointer checking"));
        }
        if self.mono.borrow().templates.contains_key(name) {
            let value = self.generic_call(name, types, arguments, expected, span)?;
            return self.contextual_result(value, expected);
        }
        if !types.is_empty() {
            return Err(error(span, "E0370", "function has no generic parameters"));
        }
        let signature = self
            .signatures
            .get(name)
            .cloned()
            .ok_or_else(|| error(callee.span, "E0310", format!("unknown function '{name}'")))?;
        if signature.parameters.len() != arguments.len() {
            return Err(error(span, "E0313", "wrong number of arguments"));
        }
        let mut checked = Vec::new();
        for (arg, ty) in arguments.iter().zip(&signature.parameters) {
            checked.push(self.expr(
                arg,
                if polymorphism::is_polymorphic(ty) {
                    None
                } else {
                    Some(ty)
                },
                true,
            )?);
        }
        let value = self.call_entry(signature, arguments, checked, span)?;
        self.contextual_result(value, expected)
    }
}
