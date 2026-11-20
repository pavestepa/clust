mod assign_expression;
mod binary_expression;
mod call_expression;
mod cast_expression;
mod if_expression;
mod member_expression;
mod unary_expression;
use super::*;

impl Checker<'_> {
    pub(super) fn expr(
        &mut self,
        source: &a::Expr,
        expected: Option<&Type>,
        value_context: bool,
    ) -> Result<h::Expr> {
        if let Some(expected) = expected.filter(|ty| polymorphism::is_polymorphic(ty)) {
            if !matches!(
                source.kind,
                a::ExprKind::If { .. } | a::ExprKind::Match { .. } | a::ExprKind::Branch(_)
            ) {
                return self.coerce_class(source, expected);
            }
        }
        let span = source.span;
        let expression_hint = self.storage_hints.get(&storage::key(span)).cloned();
        let expected = expression_hint.as_ref().or(expected);
        let (kind, ty) = match &source.kind {
            a::ExprKind::Closure(_) => return Err(error(span, "E0405", "closure was not lowered")),
            a::ExprKind::Match { value, arms } => {
                return self.match_expression(value, arms, expected, value_context, span);
            }
            a::ExprKind::Branch(block) => {
                let block = self.block(block, expected, value_context)?;
                let ty = block.result.clone();
                (h::ExprKind::Block(block), ty)
            }
            a::ExprKind::New { ty, arguments } => {
                return self.new_class(ty, arguments, expected, span)
            }
            a::ExprKind::Group(value) => return self.expr(value, expected, value_context),
            a::ExprKind::Literal(value) => literal(value, expected, false, span)?,
            a::ExprKind::Name(name) => {
                if self.lookup(name).is_none() {
                    if let Some(receiver) = self.lookup("this") {
                        if self
                            .inherited_field_path(&self.locals[receiver].ty, name)
                            .is_some()
                        {
                            let member = a::Expr {
                                kind: a::ExprKind::Member {
                                    value: Box::new(a::Expr {
                                        kind: a::ExprKind::Name("this".into()),
                                        span,
                                    }),
                                    name: name.clone(),
                                },
                                span,
                            };
                            return self.expr(&member, expected, value_context);
                        }
                    }
                    if let Some((value, ty)) = self.constants.get(name) {
                        if let Some(expected) = expected {
                            same(expected, ty, span)?;
                        }
                        return Ok(h::Expr {
                            kind: h::ExprKind::Value(value.clone()),
                            ty: ty.clone(),
                            span,
                        });
                    }
                    if let Some(value) = self.function_value(name, &[], expected, span)? {
                        if let Some(expected) = expected {
                            same(expected, &value.ty, span)?;
                        }
                        return Ok(value);
                    }
                }
                let id = self
                    .lookup(name)
                    .ok_or_else(|| error(span, "E0310", format!("unknown value '{name}'")))?;
                (
                    if self.locals[id].view.is_some() {
                        h::ExprKind::View(id)
                    } else {
                        h::ExprKind::Local(id)
                    },
                    self.locals[id].ty.clone(),
                )
            }
            a::ExprKind::Unary { operator, value } => {
                return self.check_unary(operator, value, span, expected)
            }
            a::ExprKind::Binary {
                operator,
                left,
                right,
            } => return self.check_binary(operator, left, right, span, expected),
            a::ExprKind::Cast { value, ty } => return self.check_cast(value, ty, span, expected),
            a::ExprKind::Call {
                callee,
                types,
                arguments,
            } => return self.check_call(callee, types, arguments, span, expected),
            a::ExprKind::Specialize { value, arguments } => {
                if let Some(pointer) =
                    self.static_function_value(value, arguments, expected, span)?
                {
                    return Ok(pointer);
                }
                let a::ExprKind::Name(name) = &value.kind else {
                    return Err(error(
                        span,
                        "E0900",
                        "specialized function values require a named function",
                    ));
                };
                if self.lookup(name).is_some() {
                    return Err(error(
                        span,
                        "E0390",
                        "stored function values cannot be specialized",
                    ));
                }
                return self
                    .function_value(name, arguments, expected, span)?
                    .ok_or_else(|| error(span, "E0310", "unknown function"));
            }
            a::ExprKind::Construct { ty, fields } => {
                return self.record(ty, fields, expected, span)
            }
            a::ExprKind::Tuple(values) => {
                if values.is_empty() {
                    (h::ExprKind::Value(Value::Unit), Type::Unit)
                } else {
                    let hints = if let Some(Type::Tuple(types)) = expected {
                        Some(types)
                    } else {
                        None
                    };
                    let mut checked = Vec::new();
                    for (index, value) in values.iter().enumerate() {
                        checked.push(self.storage_value(value, hints.and_then(|v| v.get(index)))?);
                    }
                    let ty = Type::Tuple(checked.iter().map(|e| e.ty.clone()).collect());
                    (h::ExprKind::Tuple(checked), ty)
                }
            }
            a::ExprKind::Array(values) => return self.storage_array(values, expected, span),
            a::ExprKind::Index { value, index } => {
                let source_value = value;
                let value = self.expr(value, None, true)?;
                if matches!(value.ty, Type::Class { .. }) {
                    let callee = a::Expr {
                        kind: a::ExprKind::Member {
                            value: source_value.clone(),
                            name: "at".into(),
                        },
                        span,
                    };
                    return self
                        .instance_call(
                            &callee,
                            &[],
                            std::slice::from_ref(&**index),
                            expected,
                            span,
                        )?
                        .ok_or_else(|| error(span, "E0315", "class does not support indexing"));
                }
                let index = self.expr(index, Some(&Type::Integer(I::Usize)), true)?;
                let Type::Array(element, _) = &value.ty else {
                    return Err(error(span, "E0315", "indexing requires a fixed array"));
                };
                let ty = (**element).clone();
                (h::ExprKind::Index(Box::new(value), Box::new(index)), ty)
            }
            a::ExprKind::Member { value, name } => {
                return self.check_member(value, name, source, span, expected)
            }
            a::ExprKind::Assign {
                operator,
                place,
                value,
            } => return self.check_assign(operator, place, value, source, span, expected),
            a::ExprKind::If {
                condition,
                then_block,
                else_branch,
            } => {
                return self.check_if(
                    condition,
                    then_block,
                    else_branch,
                    value_context,
                    span,
                    expected,
                )
            }
            a::ExprKind::While { condition, body } => {
                let condition = Box::new(self.expr(condition, Some(&Type::Bool), true)?);
                self.loops += 1;
                let checked = self.block(body, None, false);
                self.loops -= 1;
                let body = checked?;
                (h::ExprKind::While { condition, body }, Type::Unit)
            }
        };
        finish(kind, ty, span, expected)
    }
}

fn finish(kind: h::ExprKind, ty: Type, span: Span, expected: Option<&Type>) -> Result<h::Expr> {
    let checked = storage::read(h::Expr { kind, ty, span });
    if !expression_diverges(&checked) {
        if let Some(expected) = expected {
            same(expected, &checked.ty, span)?;
        }
    }
    Ok(checked)
}
