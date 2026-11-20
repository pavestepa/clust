use super::*;

pub(super) fn number(value: u64, span: Span) -> a::Expr {
    a::Expr {
        kind: a::ExprKind::Literal(a::Literal::Integer(value.to_string())),
        span,
    }
}

pub(crate) fn ast_type(ty: &Type, span: Span) -> a::Type {
    let name = match ty {
        Type::Function {
            unsafe_,
            parameters,
            result,
            ..
        } => {
            return a::Type {
                kind: a::TypeKind::Function {
                    unsafe_: *unsafe_,
                    parameters: parameters
                        .iter()
                        .map(|(mutable, ty)| (*mutable, ast_type(ty, span)))
                        .collect(),
                    result: Box::new(ast_type(result, span)),
                },
                span,
            }
        }
        Type::Unit => {
            return a::Type {
                kind: a::TypeKind::Tuple(vec![]),
                span,
            }
        }
        Type::Pointer { mutable, pointee } => {
            return a::Type {
                kind: a::TypeKind::Pointer {
                    mutable: *mutable,
                    pointee: Box::new(ast_type(pointee, span)),
                },
                span,
            }
        }
        Type::Tuple(fields) => {
            return a::Type {
                kind: a::TypeKind::Tuple(fields.iter().map(|t| ast_type(t, span)).collect()),
                span,
            }
        }
        Type::Array(element, count) => {
            return a::Type {
                kind: a::TypeKind::Array {
                    element: Box::new(ast_type(element, span)),
                    length: Box::new(number(*count, span)),
                },
                span,
            }
        }
        Type::Record { name, .. } | Type::Enum { name, .. } | Type::Class { name, .. } => {
            name.clone()
        }
        Type::Integer(i) => format!("{i:?}").to_lowercase(),
        other => format!("{other:?}").to_lowercase(),
    };
    a::Type {
        kind: a::TypeKind::Named {
            path: vec![name],
            arguments: vec![],
        },
        span,
    }
}
