use super::*;

impl Checker<'_> {
    pub(in super::super) fn schedule_destructors(&mut self, ty: &Type, span: Span) -> Result<()> {
        if self.mono.borrow().auditing {
            return Ok(());
        }
        match ty {
            Type::Class { name, .. } | Type::Record { name, .. } => {
                if matches!(ty, Type::Class { .. })
                    && !self
                        .mono
                        .borrow_mut()
                        .class_destructor_payloads
                        .insert(ty.class_key())
                {
                    return Ok(());
                }
                let fields = self.nominals.fields(ty).unwrap();
                let candidate = format!("{}.$drop", self.nominals.origin(name));
                let generic = self.mono.borrow().templates.contains_key(&candidate);
                let has_destructor = generic || self.signatures.contains_key(&candidate);
                if has_destructor
                    && self
                        .mono
                        .borrow_mut()
                        .destructor_types
                        .insert(format!("{ty:?}"))
                {
                    let types = self
                        .nominals
                        .arguments(name)
                        .into_iter()
                        .map(|a| match a {
                            records::Argument::Type(t) => {
                                a::TypeArgument::Type(generics::substitute::ast_type(&t, span))
                            }
                            records::Argument::Const(n) => a::TypeArgument::Const(a::Expr {
                                kind: a::ExprKind::Literal(a::Literal::Integer(n.to_string())),
                                span,
                            }),
                        })
                        .collect::<Vec<_>>();
                    let original = self.locals.len();
                    self.scopes.push(BTreeMap::new());
                    self.local("<destructor receiver>".into(), ty.clone(), true, span)?;
                    let source = a::Expr {
                        kind: a::ExprKind::Name("<destructor receiver>".into()),
                        span,
                    };
                    let result = if generic {
                        self.generic_call(&candidate, &types, &[source], Some(&Type::Unit), span)
                    } else {
                        let signature = self.signatures[&candidate].clone();
                        let value = self.expr(&source, Some(&signature.parameters[0]), true)?;
                        self.call_entry(signature, &[source], vec![value], span)
                    };
                    self.locals.truncate(original);
                    self.scopes.pop();
                    result?;
                }
                for field in fields {
                    self.schedule_destructors(&field.ty, span)?;
                }
            }
            Type::Tuple(fields) => {
                for ty in fields {
                    self.schedule_destructors(ty, span)?;
                }
            }
            Type::Array(ty, _) => self.schedule_destructors(ty, span)?,
            Type::Enum { variants, .. } => {
                for ty in variants.iter().flat_map(|v| &v.fields) {
                    self.schedule_destructors(ty, span)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
