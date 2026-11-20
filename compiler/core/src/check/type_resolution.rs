use super::*;

pub fn resolve_type(ty: &a::Type) -> Result<Type> {
    resolve_type_in(ty, &BTreeMap::new())
}

pub(crate) fn resolve_type_in(
    ty: &a::Type,
    constants: &crate::constants::Constants,
) -> Result<Type> {
    resolve_type_with(ty, constants, &records::Nominals::default())
}

pub(super) fn resolve_type_with(
    ty: &a::Type,
    constants: &crate::constants::Constants,
    nominals: &records::Nominals,
) -> Result<Type> {
    let resolved = match &ty.kind {
        a::TypeKind::Named { path, arguments } if path.len() == 1 => {
            if !arguments.is_empty() {
                return nominals.resolve(&path[0], arguments, constants, ty.span);
            }
            match path[0].as_str() {
                "bool" => Type::Bool,
                "char" => Type::Char,
                "str" => Type::Str,
                "f32" => Type::F32,
                "f64" => Type::F64,
                "i8" => Type::Integer(I::I8),
                "i16" => Type::Integer(I::I16),
                "i32" => Type::Integer(I::I32),
                "i64" => Type::Integer(I::I64),
                "i128" => Type::Integer(I::I128),
                "isize" => Type::Integer(I::Isize),
                "u8" => Type::Integer(I::U8),
                "u16" => Type::Integer(I::U16),
                "u32" => Type::Integer(I::U32),
                "u64" => Type::Integer(I::U64),
                "u128" => Type::Integer(I::U128),
                "usize" => Type::Integer(I::Usize),
                name if nominals.contains(name) => {
                    nominals.resolve(name, arguments, constants, ty.span)?
                }
                _ => {
                    return Err(error(
                        ty.span,
                        "E0900",
                        "named type resolution is not implemented yet",
                    ))
                }
            }
        }
        a::TypeKind::Tuple(elements) => {
            if elements.is_empty() {
                Type::Unit
            } else {
                Type::Tuple(
                    elements
                        .iter()
                        .map(|ty| resolve_type_with(ty, constants, nominals))
                        .collect::<Result<_>>()?,
                )
            }
        }
        a::TypeKind::Function {
            unsafe_,
            parameters,
            result,
        } => {
            let parameters = parameters
                .iter()
                .map(|(mutable, ty)| Ok((*mutable, resolve_type_with(ty, constants, nominals)?)))
                .collect::<Result<Vec<_>>>()?;
            let result = resolve_type_with(result, constants, nominals)?;
            Type::Function {
                unsafe_: *unsafe_,
                parameters,
                result: Box::new(result),
                contract: None,
            }
        }
        a::TypeKind::Pointer { mutable, pointee } => {
            let pointee = resolve_type_with(pointee, constants, nominals)?;
            Type::Pointer {
                mutable: *mutable,
                pointee: Box::new(pointee),
            }
        }
        a::TypeKind::Array { element, length } => {
            let Value::Integer(length) =
                constant_value(length, &Type::Integer(I::Usize), constants)?
            else {
                unreachable!()
            };
            let size = u64::try_from(length.bits())
                .map_err(|_| error(ty.span, "E0318", "array length does not fit usize"))?;
            Type::Array(
                Box::new(resolve_type_with(element, constants, nominals)?),
                size,
            )
        }
        _ => {
            return Err(error(
                ty.span,
                "E0900",
                "this type requires a later checker stage",
            ))
        }
    };
    crate::layout::layout(&resolved).map_err(|message| error(ty.span, "E0318", message))?;
    Ok(resolved)
}
