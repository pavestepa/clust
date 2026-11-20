use super::*;

pub(super) fn trap(kind: TrapKind, span: Span) -> Signal {
    Signal::Trap(Trap { kind, span })
}

pub(super) fn index_value(value: Value, span: Span) -> Result<usize> {
    let Value::Integer(value) = value else {
        return Err(trap(TrapKind::InvalidIr, span));
    };
    usize::try_from(value.bits()).map_err(|_| trap(TrapKind::Bounds, span))
}

pub fn binary(
    operator: &str,
    left: Value,
    right: Value,
    checks: OverflowChecks,
) -> std::result::Result<Value, TrapKind> {
    match (operator, &left, &right) {
        ("str_byte_at", Value::Str(text), Value::Integer(index)) => {
            return usize::try_from(index.bits())
                .ok()
                .and_then(|index| text.as_bytes().get(index).copied())
                .map(|byte| {
                    Value::Integer(Integer::from_bits(
                        clust_semantics::IntegerType::U8,
                        byte as u128,
                    ))
                })
                .ok_or(TrapKind::Bounds)
        }
        ("checked_index", Value::Integer(index), Value::Integer(length)) => {
            return if index.bits() < length.bits() {
                Ok(left)
            } else {
                Err(TrapKind::Bounds)
            };
        }
        ("raw_write", Value::Pointer(p), value) => return p.write(value.clone()),
        ("raw_difference", Value::Pointer(a), Value::Pointer(b)) => {
            return a.difference(b).map(|n| {
                Value::Integer(Integer::from_bits(
                    clust_semantics::IntegerType::Isize,
                    n as u128,
                ))
            });
        }
        ("raw_offset_sub", Value::Pointer(p), Value::Integer(n)) => {
            return p
                .clone()
                .raw_offset(-n.signed_value().ok_or(TrapKind::InvalidIr)?)
                .map(Value::Pointer);
        }
        ("raw_offset", Value::Pointer(p), Value::Integer(n)) => {
            return p
                .clone()
                .raw_offset(n.signed_value().ok_or(TrapKind::InvalidIr)?)
                .map(Value::Pointer)
        }
        _ => (),
    }
    if let (Value::Integer(a), Value::Integer(b)) = (&left, &right) {
        let operation = match operator {
            "+" => Some(BinaryOp::Add),
            "-" => Some(BinaryOp::Subtract),
            "*" => Some(BinaryOp::Multiply),
            "/" => Some(BinaryOp::Divide),
            "%" => Some(BinaryOp::Remainder),
            "&" => Some(BinaryOp::BitAnd),
            "|" => Some(BinaryOp::BitOr),
            "^" => Some(BinaryOp::BitXor),
            "<<" => Some(BinaryOp::ShiftLeft),
            ">>" => Some(BinaryOp::ShiftRight),
            _ => None,
        };
        if let Some(operation) = operation {
            return a
                .binary(operation, *b, checks)
                .map(Value::Integer)
                .map_err(TrapKind::Arithmetic);
        }
        return comparison(operator, Some(a.compare(*b).map_err(TrapKind::Arithmetic)?))
            .map(Value::Bool)
            .ok_or(TrapKind::InvalidIr);
    }
    macro_rules! float {
        ($a:expr,$b:expr,$variant:ident) => {{
            match operator {
                "+" => Ok(Value::$variant($a + $b)),
                "-" => Ok(Value::$variant($a - $b)),
                "*" => Ok(Value::$variant($a * $b)),
                "/" => Ok(Value::$variant($a / $b)),
                "%" => Ok(Value::$variant($a % $b)),
                _ => comparison(operator, $a.partial_cmp(&$b))
                    .map(Value::Bool)
                    .ok_or(TrapKind::InvalidIr),
            }
        }};
    }
    match (left, right) {
        (Value::F32(a), Value::F32(b)) => float!(a, b, F32),
        (Value::F64(a), Value::F64(b)) => float!(a, b, F64),
        (Value::Bool(a), Value::Bool(b)) => match operator {
            "&&" => Ok(Value::Bool(a && b)),
            "||" => Ok(Value::Bool(a || b)),
            _ => comparison(operator, Some(a.cmp(&b)))
                .map(Value::Bool)
                .ok_or(TrapKind::InvalidIr),
        },
        (Value::Char(a), Value::Char(b)) => comparison(operator, Some(a.cmp(&b)))
            .map(Value::Bool)
            .ok_or(TrapKind::InvalidIr),
        _ => Err(TrapKind::InvalidIr),
    }
}

pub(super) fn comparison(operator: &str, ordering: Option<std::cmp::Ordering>) -> Option<bool> {
    use std::cmp::Ordering::*;
    Some(match operator {
        "==" => ordering == Some(Equal),
        "!=" => ordering != Some(Equal),
        "<" => ordering == Some(Less),
        ">" => ordering == Some(Greater),
        "<=" => matches!(ordering, Some(Less | Equal)),
        ">=" => matches!(ordering, Some(Greater | Equal)),
        _ => return None,
    })
}

pub fn cast(value: Value, target: &Type) -> Option<Value> {
    match target {
        Type::Pointer { pointee, .. } => match value {
            Value::Pointer(pointer) => pointer.reinterpret(pointee).map(Value::Pointer),
            _ => None,
        },
        Type::Integer(ty) => Some(Value::Integer(match value {
            Value::Integer(value) => value.cast(*ty),
            Value::F32(value) => Integer::from_float(*ty, f64::from(value)),
            Value::F64(value) => Integer::from_float(*ty, value),
            Value::Bool(value) => Integer::from_bits(*ty, u128::from(value)),
            Value::Char(value) => Integer::from_bits(*ty, u128::from(u32::from(value))),
            _ => return None,
        })),
        Type::F32 => Some(Value::F32(match value {
            Value::Integer(value) => {
                if let Some(signed) = value.signed_value() {
                    signed as f32
                } else {
                    value.bits() as f32
                }
            }
            Value::F32(v) => v,
            Value::F64(v) => v as f32,
            _ => return None,
        })),
        Type::F64 => Some(Value::F64(match value {
            Value::Integer(value) => {
                if let Some(signed) = value.signed_value() {
                    signed as f64
                } else {
                    value.bits() as f64
                }
            }
            Value::F32(v) => f64::from(v),
            Value::F64(v) => v,
            _ => return None,
        })),
        Type::Char => {
            if let Value::Integer(v) = value {
                if v.ty() == clust_semantics::IntegerType::U8 {
                    Some(Value::Char(char::from(v.bits() as u8)))
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
