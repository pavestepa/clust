use super::*;
impl Checker<'_> {
    pub(super) fn check_member(
        &mut self,
        value: &a::Expr,
        name: &str,
        source: &a::Expr,
        span: Span,
        expected: Option<&Type>,
    ) -> Result<h::Expr> {
        let (kind, ty) = {
            if let Some(pointer) = self.static_function_value(source, &[], expected, span)? {
                return Ok(pointer);
            }
            if let Some(value) = self.enum_unit(value, name, expected, span)? {
                return Ok(value);
            }
            let receiver = value;
            let value = self.expr(value, None, true)?;
            if polymorphism::is_polymorphic(&value.ty) && !name.starts_with('$') {
                let callee = a::Expr {
                    kind: a::ExprKind::Member {
                        value: Box::new(receiver.clone()),
                        name: polymorphism::accessor(name, "get"),
                    },
                    span,
                };
                return self
                    .instance_call(&callee, &[], &[], expected, span)?
                    .ok_or_else(|| error(span, "E0352", "base field is not publicly accessible"));
            }
            let (index, ty) = match &value.ty {
                Type::Tuple(fields) => {
                    let index = name.parse::<usize>().map_err(|_| {
                        error(span, "E0315", "tuple field must be an integer index")
                    })?;
                    let ty = fields
                        .get(index)
                        .cloned()
                        .ok_or_else(|| error(span, "E0315", "tuple field out of bounds"))?;
                    (index, ty)
                }
                Type::Record { .. } | Type::Class { .. } => {
                    let path = self
                        .inherited_field_path(&value.ty, name)
                        .ok_or_else(|| error(span, "E0351", "unknown class or struct field"))?;
                    let mut projected = value;
                    for index in path {
                        let field = &self.nominals.fields(&projected.ty).unwrap()[index];
                        self.check_field_access(&projected.ty, field, span)?;
                        let ty = field.ty.clone();
                        projected = h::Expr {
                            kind: h::ExprKind::Field(Box::new(projected), index),
                            ty,
                            span,
                        };
                    }
                    let projected = storage::read(projected);
                    if let Some(expected) = expected {
                        same(expected, &projected.ty, span)?;
                    }
                    return Ok(projected);
                }
                _ => {
                    return Err(error(
                        span,
                        "E0900",
                        format!(
                            "member access requires an aggregate, found {:?} for {name}",
                            value.ty
                        ),
                    ))
                }
            };
            (h::ExprKind::Field(Box::new(value), index), ty)
        };
        finish(kind, ty, span, expected)
    }
}
