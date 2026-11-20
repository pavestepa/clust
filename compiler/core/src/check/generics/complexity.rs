use super::*;

pub(super) fn validate_complexity(ty: &Type, span: Span) -> Result<()> {
    let mut pending = vec![(ty, 0)];
    let mut count = 0;
    while let Some((ty, depth)) = pending.pop() {
        count += 1;
        if count > 4096 || depth > 32 {
            return Err(error(
                span,
                "E0372",
                "generic type complexity limit exceeded",
            ));
        }
        match ty {
            Type::Function {
                parameters, result, ..
            } => {
                pending.extend(parameters.iter().map(|(_, ty)| (ty, depth + 1)));
                pending.push((result, depth + 1));
            }
            Type::Tuple(fields) => pending.extend(fields.iter().map(|ty| (ty, depth + 1))),
            Type::Array(element, _)
            | Type::Pointer {
                pointee: element, ..
            } => pending.push((element, depth + 1)),
            Type::Record { fields, .. } => {
                pending.extend(fields.iter().map(|f| (&f.ty, depth + 1)))
            }
            Type::Enum { variants, .. } => pending.extend(
                variants
                    .iter()
                    .flat_map(|v| v.fields.iter())
                    .map(|ty| (ty, depth + 1)),
            ),
            _ => (),
        }
    }
    Ok(())
}
