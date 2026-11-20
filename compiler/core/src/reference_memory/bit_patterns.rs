// Only padding-free, provenance-free payloads cross this byte representation.
// Native code operates directly on the original address; this is diagnostic-only.
use super::*;

pub(super) fn bits_bytes(value: Value, ty: &Type) -> Result<Vec<u8>, TrapKind> {
    if !ty.raw_bits_value() {
        return Err(TrapKind::InvalidIr);
    }
    if let Value::Enum { variant, fields } = value {
        let Type::Enum { variants, .. } = ty else {
            return Err(TrapKind::InvalidIr);
        };
        let definition = variants.get(variant).ok_or(TrapKind::InvalidIr)?;
        if definition.fields.len() != fields.len() {
            return Err(TrapKind::InvalidIr);
        }
        let mut bytes = (variant as u32).to_ne_bytes().to_vec();
        for (value, ty) in fields.into_iter().zip(&definition.fields) {
            bytes.extend(bits_bytes(value, ty)?);
        }
        return Ok(bytes);
    }
    if let Value::Aggregate(values) = value {
        let mut types = bits_fields(ty).ok_or(TrapKind::InvalidIr)?;
        let mut bytes = Vec::new();
        for value in values {
            bytes.extend(bits_bytes(value, types.next().ok_or(TrapKind::InvalidIr)?)?);
        }
        if types.next().is_some() {
            return Err(TrapKind::InvalidIr);
        }
        return Ok(bytes);
    }
    let bits = match value {
        Value::Integer(n) if *ty == Type::Integer(n.ty()) => n.bits(),
        Value::F32(n) if *ty == Type::F32 => u128::from(n.to_bits()),
        Value::F64(n) if *ty == Type::F64 => u128::from(n.to_bits()),
        Value::Bool(n) if *ty == Type::Bool => u128::from(n),
        Value::Char(n) if *ty == Type::Char => u128::from(n as u32),
        _ => return Err(TrapKind::InvalidIr),
    };
    let size = crate::layout::layout(ty)
        .map_err(|_| TrapKind::InvalidIr)?
        .size() as usize;
    let bytes = bits.to_ne_bytes();
    Ok(if cfg!(target_endian = "little") {
        bytes[..size].to_vec()
    } else {
        bytes[16 - size..].to_vec()
    })
}

pub(super) fn bits_fields(ty: &Type) -> Option<Box<dyn Iterator<Item = &Type> + '_>> {
    match ty {
        Type::Array(element, count) => {
            Some(Box::new(std::iter::repeat_n(&**element, *count as usize)))
        }
        Type::Tuple(fields) => Some(Box::new(fields.iter())),
        Type::Record {
            copy: true, fields, ..
        } => Some(Box::new(fields.iter().map(|f| &f.ty))),
        _ => None,
    }
}

pub(super) fn bits_value(bytes: &[u8], ty: &Type) -> Result<Value, TrapKind> {
    if !ty.raw_bits_value() {
        return Err(TrapKind::InvalidIr);
    }
    let size = crate::layout::layout(ty)
        .map_err(|_| TrapKind::InvalidIr)?
        .size() as usize;
    if size != bytes.len() {
        return Err(TrapKind::InvalidIr);
    }
    if let Type::Enum { variants, .. } = ty {
        let tag = u32::from_ne_bytes(
            bytes
                .get(..4)
                .ok_or(TrapKind::InvalidIr)?
                .try_into()
                .map_err(|_| TrapKind::InvalidIr)?,
        ) as usize;
        let variant = variants.get(tag).ok_or(TrapKind::InvalidIr)?;
        let mut offset = 4;
        let mut fields = Vec::new();
        for ty in &variant.fields {
            let size = crate::layout::layout(ty)
                .map_err(|_| TrapKind::InvalidIr)?
                .size() as usize;
            fields.push(bits_value(
                bytes
                    .get(offset..offset + size)
                    .ok_or(TrapKind::InvalidIr)?,
                ty,
            )?);
            offset += size;
        }
        return Ok(Value::Enum {
            variant: tag,
            fields,
        });
    }
    if let Some(types) = bits_fields(ty) {
        let mut offset = 0;
        let mut values = Vec::new();
        for ty in types {
            let size = crate::layout::layout(ty)
                .map_err(|_| TrapKind::InvalidIr)?
                .size() as usize;
            values.push(bits_value(
                bytes
                    .get(offset..offset + size)
                    .ok_or(TrapKind::InvalidIr)?,
                ty,
            )?);
            offset += size;
        }
        return Ok(Value::Aggregate(values));
    }
    if size > 16 {
        return Err(TrapKind::InvalidIr);
    }
    let mut padded = [0; 16];
    if cfg!(target_endian = "little") {
        padded[..size].copy_from_slice(bytes)
    } else {
        padded[16 - size..].copy_from_slice(bytes)
    }
    let bits = u128::from_ne_bytes(padded);
    Ok(match ty {
        Type::Integer(ty) => Value::Integer(clust_semantics::Integer::from_bits(*ty, bits)),
        Type::F32 => Value::F32(f32::from_bits(bits as u32)),
        Type::F64 => Value::F64(f64::from_bits(bits as u64)),
        Type::Bool if bits <= 1 => Value::Bool(bits == 1),
        Type::Char => Value::Char(char::from_u32(bits as u32).ok_or(TrapKind::InvalidIr)?),
        _ => return Err(TrapKind::InvalidIr),
    })
}
