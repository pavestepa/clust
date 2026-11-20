use super::*;
impl Checker<'_> {
    pub(super) fn enum_pattern(
        &self,
        ty: &Type,
        path: &[String],
        span: Span,
    ) -> Result<Option<(Type, usize, h::Variant)>> {
        let Type::Enum { name, variants, .. } = ty else {
            return Ok(None);
        };
        if path.len() != 2 || self.nominals.origin(name) != path[0] && *name != path[0] {
            return Ok(None);
        }
        let index = variants
            .iter()
            .position(|v| v.name == path[1])
            .ok_or_else(|| error(span, "E0360", "unknown enum variant"))?;
        Ok(Some((ty.clone(), index, variants[index].clone())))
    }
    fn enum_variant(
        &self,
        path: &[String],
        arguments: &[a::TypeArgument],
        expected: Option<&Type>,
        span: Span,
    ) -> Result<Option<(Type, usize, h::Variant)>> {
        if path.len() != 2 || !self.nominals.is_enum(&path[0]) {
            return Ok(None);
        }
        let ty = if arguments.is_empty() {
            if let Some(Type::Enum { name, .. }) = expected {
                if self.nominals.origin(name) == path[0] {
                    expected.unwrap().clone()
                } else {
                    self.nominals
                        .resolve(&path[0], arguments, self.constants, span)?
                }
            } else {
                self.nominals
                    .resolve(&path[0], arguments, self.constants, span)?
            }
        } else {
            self.nominals
                .resolve(&path[0], arguments, self.constants, span)?
        };
        let ty = expected
            .filter(|expected| {
                matches!(expected, Type::Enum { .. }) && expected.logical() == ty.logical()
            })
            .cloned()
            .unwrap_or(ty);
        self.enum_pattern(&ty, path, span)
    }
    pub(super) fn enum_unit(
        &self,
        base: &a::Expr,
        name: &str,
        expected: Option<&Type>,
        span: Span,
    ) -> Result<Option<h::Expr>> {
        let Some((base, arguments)) = type_expression(base) else {
            return Ok(None);
        };
        if self.lookup(base).is_some() {
            return Ok(None);
        }
        let Some((ty, variant, definition)) =
            self.enum_variant(&[base.to_owned(), name.into()], arguments, expected, span)?
        else {
            return Ok(None);
        };
        if definition.shape != h::VariantShape::Unit {
            return Err(error(
                span,
                "E0360",
                "variant requires payload construction",
            ));
        }
        if let Some(expected) = expected {
            same(expected, &ty, span)?;
        }
        Ok(Some(h::Expr {
            kind: h::ExprKind::Enum {
                variant,
                fields: vec![],
            },
            ty,
            span,
        }))
    }
}

mod construction;
use self::construction::type_expression;
