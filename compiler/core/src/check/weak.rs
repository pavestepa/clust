//! Weak identity is observable only through a scoped readonly callback. The
//! callback receives no owner, mutable capability or escaping result view.
use super::*;
use clust_frontend::intrinsics::Intrinsic;
pub(crate) fn scalar_result(ty: &Type) -> bool {
    match ty {
        Type::Unit
        | Type::Bool
        | Type::Char
        | Type::Str
        | Type::Integer(_)
        | Type::F32
        | Type::F64 => true,
        Type::Tuple(fields) => fields.iter().all(scalar_result),
        Type::Array(element, _) => scalar_result(element),
        _ => false,
    }
}
impl Checker<'_> {
    fn weak_owned_target(&self, ty: &Type) -> bool {
        if polymorphism::is_polymorphic(ty) {
            let Some(fields) = self.nominals.fields(ty) else {
                return false;
            };
            let Type::Enum { variants, .. } = &fields[0].ty else {
                return false;
            };
            variants.iter().all(|variant| match &variant.fields[0] {
                Type::StoredView(target) => self.weak_owned_target(target),
                target => self.weak_owned_target(target),
            })
        } else {
            !ty.contains_views()
        }
    }

    pub(super) fn weak_intrinsic(
        &mut self,
        id: Intrinsic,
        types: &[a::TypeArgument],
        args: &[a::Expr],
        expected: Option<&Type>,
        span: Span,
    ) -> Result<h::Expr> {
        let access = id == Intrinsic::WeakAccess;
        if types.len() != if access { 2 } else { 1 } || args.len() != if access { 2 } else { 1 } {
            return Err(error(span, "E0313", "invalid weak intrinsic arguments"));
        }
        let a::TypeArgument::Type(source) = &types[0] else {
            return Err(error(span, "E0313", "weak target must be a class type"));
        };
        let target = resolve_type_with(source, self.constants, self.nominals)?;
        if !matches!(target, Type::Class { copy: false, .. }) && !self.mono.borrow().auditing {
            return Err(error(
                span,
                "E0404",
                "weak references require a non-Copy class",
            ));
        }
        let token = Type::Tuple(vec![Type::Integer(I::Usize); 2]);
        let (kind, result) = if id == Intrinsic::WeakIdentity {
            let value = self.expr(&args[0], None, true)?;
            if !self.class_subtype(&target, &value.ty) {
                same(&target, &value.ty, span)?;
            }
            let target = value.ty.clone();
            if !self.weak_owned_target(&value.ty) {
                return Err(error(
                    span,
                    "E0404",
                    "weak targets must own their class fields",
                ));
            }
            if !views::place(&value) {
                return Err(error(
                    span,
                    "E0404",
                    "weak creation requires a live class place",
                ));
            }
            (
                h::ExprKind::Unary(
                    "weak_identity",
                    Box::new(h::Expr {
                        kind: h::ExprKind::ArgumentView {
                            place: Box::new(value),
                            access: crate::mir::loans::Access::Readonly,
                        },
                        ty: target,
                        span,
                    }),
                ),
                token,
            )
        } else {
            let value = self.expr(&args[0], Some(&token), true)?;
            if id == Intrinsic::WeakAlive {
                (
                    h::ExprKind::Unary("weak_alive", Box::new(value)),
                    Type::Bool,
                )
            } else {
                let a::TypeArgument::Type(source) = &types[1] else {
                    return Err(error(span, "E0313", "weak callback requires a result type"));
                };
                let result = resolve_type_with(source, self.constants, self.nominals)?;
                if !scalar_result(&result) && !self.mono.borrow().auditing {
                    return Err(error(
                        span,
                        "E0404",
                        "weak callback result must contain only scalar values",
                    ));
                }
                let callback_type = Type::Function {
                    unsafe_: false,
                    parameters: vec![(false, target)],
                    result: Box::new(result.clone()),
                    contract: None,
                };
                let callback = self.expr(&args[1], Some(&callback_type), true)?;
                if let Type::Function {
                    contract: Some(contract),
                    ..
                } = &callback.ty
                {
                    if contract.modes != [Some(crate::mir::loans::Access::Readonly)]
                        || contract.view.is_some()
                    {
                        return Err(error(
                            span,
                            "E0404",
                            "weak callback must only borrow its target readonly",
                        ));
                    }
                }
                let option = expected.cloned().ok_or_else(|| {
                    error(
                        span,
                        "E0404",
                        "weak access requires an Option result context",
                    )
                })?;
                if !matches!(&option,Type::Enum {variants,..} if variants.len()==2 && variants[0].name=="None" && variants[0].fields.is_empty() && variants[1].name=="Some" && variants[1].fields==[result])
                {
                    return Err(error(span, "E0404", "weak access result must be Option<R>"));
                }
                (
                    h::ExprKind::Binary("weak_access", Box::new(value), Box::new(callback)),
                    option,
                )
            }
        };
        if let Some(expected) = expected {
            same(expected, &result, span)?;
        }
        Ok(h::Expr {
            kind,
            ty: result,
            span,
        })
    }
}
