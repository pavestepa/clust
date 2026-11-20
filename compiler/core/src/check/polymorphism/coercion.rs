use super::*;

pub(in super::super) fn is_polymorphic(ty: &Type) -> bool {
    matches!(ty,Type::Class {name,..} if name.starts_with(PREFIX))
}

impl Checker<'_> {
    pub(in super::super) fn class_subtype(&self, expected: &Type, actual: &Type) -> bool {
        if !is_polymorphic(expected) {
            return false;
        }
        let Some(fields) = self.nominals.fields(expected) else {
            return false;
        };
        let Type::Enum { variants, .. } = &fields[0].ty else {
            return false;
        };
        variants
            .iter()
            .any(|v| v.fields.len() == 1 && v.fields[0].logical() == actual.logical())
    }
}

impl Checker<'_> {
    pub(in super::super) fn coerce_class(
        &mut self,
        source: &a::Expr,
        expected: &Type,
    ) -> Result<h::Expr> {
        let value = self.expr(source, None, true)?;
        if value.ty.logical() == expected.logical() {
            return Ok(value);
        }
        let fields = self.nominals.fields(expected).unwrap();
        let Type::Enum {
            name: cases,
            variants,
            ..
        } = &fields[0].ty
        else {
            unreachable!()
        };
        let variant = variants
            .iter()
            .find(|v| v.fields.len() == 1 && v.fields[0].logical() == value.ty.logical())
            .ok_or_else(|| {
                error(
                    source.span,
                    "E0312",
                    "class value is not a subtype of the expected base class",
                )
            })?;
        let span = source.span;
        let argument = a::Expr {
            kind: a::ExprKind::Call {
                callee: Box::new(a::Expr {
                    kind: a::ExprKind::Member {
                        value: Box::new(expr(cases, span)),
                        name: variant.name.clone(),
                    },
                    span,
                }),
                types: vec![],
                arguments: vec![source.clone()],
            },
            span: site(span, 21),
        };
        let Type::Class { name, .. } = expected else {
            unreachable!()
        };
        self.new_class(&ty(name, span), &[argument], Some(expected), site(span, 20))
    }
}

pub(super) fn name_of(ty: &a::Type) -> String {
    match &ty.kind {
        a::TypeKind::Named { path, arguments } => {
            let name = path.join("::");
            if arguments.is_empty() {
                name
            } else {
                format!(
                    "{name}<{}>",
                    arguments
                        .iter()
                        .map(|argument| match argument {
                            a::TypeArgument::Type(ty) => name_of(ty),
                            a::TypeArgument::Const(value) => const_name(value),
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        }
        a::TypeKind::Tuple(fields) => format!(
            "({})",
            fields.iter().map(name_of).collect::<Vec<_>>().join(",")
        ),
        a::TypeKind::Array { element, length } => {
            format!("{{{};{}}}", name_of(element), const_name(length))
        }
        a::TypeKind::Pointer { mutable, pointee } => format!("*{mutable}{}", name_of(pointee)),
        a::TypeKind::Function {
            unsafe_,
            parameters,
            result,
        } => format!(
            "fn{unsafe_}({})->{}",
            parameters
                .iter()
                .map(|(mutable, ty)| format!("{mutable}{}", name_of(ty)))
                .collect::<Vec<_>>()
                .join(","),
            name_of(result)
        ),
    }
}

pub(super) fn const_name(value: &a::Expr) -> String {
    match &value.kind {
        a::ExprKind::Name(name) => name.clone(),
        a::ExprKind::Literal(a::Literal::Integer(value)) => value.clone(),
        a::ExprKind::Group(value) => const_name(value),
        a::ExprKind::Unary { operator, value } => format!("({operator}{})", const_name(value)),
        a::ExprKind::Binary {
            operator,
            left,
            right,
        } => format!("({}{operator}{})", const_name(left), const_name(right)),
        _ => format!(
            "invalid-constant-at-{}-{}",
            value.span.source.0, value.span.start
        ),
    }
}
