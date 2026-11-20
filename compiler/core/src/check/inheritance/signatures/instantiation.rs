use super::*;

impl Bindings {
    pub(in super::super) fn method(&self, method: &mut a::Function) {
        // Freshen method binders before substituting class arguments. Otherwise
        // A<T>.f<U>(x:T), inherited by B<U>:A<U>, would capture the class U as
        // the method's U. Declarations preserve their lexical scope across edges.
        let mut locals = Self::default();
        let mut renamed = Vec::new();
        for (index, generic) in method.generics.iter().enumerate() {
            let name = format!(
                "#local{:?}:{}:{index}",
                method.span.source, method.span.start
            );
            match generic.kind {
                a::GenericKind::Type { .. } => {
                    locals.types.insert(
                        generic.name.clone(),
                        a::Type {
                            kind: a::TypeKind::Named {
                                path: vec![name.clone()],
                                arguments: vec![],
                            },
                            span: generic.span,
                        },
                    );
                }
                a::GenericKind::Const { .. } => {
                    locals.constants.insert(
                        generic.name.clone(),
                        a::Expr {
                            kind: a::ExprKind::Name(name.clone()),
                            span: generic.span,
                        },
                    );
                }
            }
            renamed.push(name);
        }
        locals.signature(method);
        locals.body(method);
        for (generic, name) in method.generics.iter_mut().zip(renamed) {
            generic.name = name;
        }
        self.signature(method);
        self.body(method);
    }
}

impl Keys<'_> {
    pub(super) fn expr(&self, value: &a::Expr) -> Result<String> {
        if let Ok(Value::Integer(value)) =
            constant_value(value, &Type::Integer(I::Usize), self.constants)
        {
            return Ok(value.bits().to_string());
        }
        Ok(match &value.kind {
            a::ExprKind::Name(name) => name.clone(),
            a::ExprKind::Literal(value) => format!("{value:?}"),
            a::ExprKind::Group(value) => return self.expr(value),
            a::ExprKind::Unary { operator, value } => format!("{operator}({})", self.expr(value)?),
            a::ExprKind::Binary {
                operator,
                left,
                right,
            } => format!("{operator}({},{})", self.expr(left)?, self.expr(right)?),
            a::ExprKind::Cast { value, ty } => {
                format!("cast({},{})", self.expr(value)?, self.ty(ty)?)
            }
            a::ExprKind::If {
                condition,
                then_block,
                else_branch,
            } => format!(
                "if({},{},{})",
                self.expr(condition)?,
                self.block(then_block)?,
                else_branch
                    .as_ref()
                    .map(|e| self.expr(e))
                    .transpose()?
                    .unwrap_or_default()
            ),
            a::ExprKind::Branch(block) => return self.block(block),
            _ => {
                return Err(error(
                    value.span,
                    "E0411",
                    "unsupported symbolic const expression in override signature",
                ))
            }
        })
    }
}

impl Keys<'_> {
    pub(super) fn block(&self, block: &a::Block) -> Result<String> {
        block
            .statements
            .iter()
            .map(|statement| match &statement.kind {
                a::StatementKind::Expression(value) => self.expr(value),
                _ => Err(error(
                    statement.span,
                    "E0411",
                    "const signature requires pure expressions",
                )),
            })
            .collect::<Result<Vec<_>>>()
            .map(|v| format!("{{{}}}", v.join(";")))
    }
}

impl Keys<'_> {
    pub(super) fn argument(&self, argument: &a::TypeArgument) -> Result<String> {
        match argument {
            a::TypeArgument::Const(value) => self.expr(value),
            a::TypeArgument::Type(a::Type {
                kind: a::TypeKind::Named { path, arguments },
                span,
            }) if path.len() == 1
                && arguments.is_empty()
                && (path[0].starts_with("#const") || self.constants.contains_key(&path[0])) =>
            {
                self.expr(&a::Expr {
                    kind: a::ExprKind::Name(path[0].clone()),
                    span: *span,
                })
            }
            a::TypeArgument::Type(ty) => self.ty(ty),
        }
    }
}

impl Keys<'_> {
    pub(super) fn ty(&self, ty: &a::Type) -> Result<String> {
        Ok(match &ty.kind {
            a::TypeKind::Named { path, arguments } => format!(
                "{}<{}>",
                path.join("::"),
                arguments
                    .iter()
                    .map(|a| self.argument(a))
                    .collect::<Result<Vec<_>>>()?
                    .join(",")
            ),
            a::TypeKind::Tuple(fields) => format!(
                "({})",
                fields
                    .iter()
                    .map(|t| self.ty(t))
                    .collect::<Result<Vec<_>>>()?
                    .join(",")
            ),
            a::TypeKind::Array { element, length } => {
                format!("{{{};{}}}", self.ty(element)?, self.expr(length)?)
            }
            a::TypeKind::Pointer { mutable, pointee } => {
                format!("ptr({mutable},{})", self.ty(pointee)?)
            }
            a::TypeKind::Function {
                unsafe_,
                parameters,
                result,
            } => format!(
                "fn(unsafe={unsafe_};{})->{}",
                parameters
                    .iter()
                    .map(|(m, t)| Ok(format!("{m}:{}", self.ty(t)?)))
                    .collect::<Result<Vec<_>>>()?
                    .join(","),
                self.ty(result)?
            ),
        })
    }
}

impl Keys<'_> {
    pub(super) fn signature(&self, method: &a::Function) -> Result<String> {
        let mut method = method.clone();
        let mut alpha = Bindings::default();
        for (index, generic) in method.generics.iter().enumerate() {
            match generic.kind {
                a::GenericKind::Type { .. } => {
                    alpha.types.insert(
                        generic.name.clone(),
                        a::Type {
                            kind: a::TypeKind::Named {
                                path: vec![format!("#type{index}")],
                                arguments: vec![],
                            },
                            span: generic.span,
                        },
                    );
                }
                a::GenericKind::Const { .. } => {
                    alpha.constants.insert(
                        generic.name.clone(),
                        a::Expr {
                            kind: a::ExprKind::Name(format!("#const{index}")),
                            span: generic.span,
                        },
                    );
                }
            }
        }
        alpha.signature(&mut method);
        let generics = method
            .generics
            .iter()
            .map(|g| {
                Ok(match &g.kind {
                    a::GenericKind::Type {
                        constraints,
                        default,
                    } => format!(
                        "type({:?},{:?})",
                        constraints
                            .iter()
                            .map(|t| self.ty(t))
                            .collect::<Result<BTreeSet<_>>>()?,
                        default.as_ref().map(|t| self.ty(t)).transpose()?
                    ),
                    a::GenericKind::Const { ty, default } => format!(
                        "const({},{:?})",
                        self.ty(ty)?,
                        default.as_ref().map(|e| self.expr(e)).transpose()?
                    ),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let parameters = method
            .parameters
            .iter()
            .map(|p| {
                Ok((
                    p.receiver,
                    p.mutable,
                    p.ty.as_ref().map(|t| self.ty(t)).transpose()?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(format!(
            "{}:{generics:?}:{parameters:?}:{}",
            method.unsafe_,
            self.ty(&method.result)?
        ))
    }
}
