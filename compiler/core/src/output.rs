//! Shared scalar output for both reference engines.
use crate::{hir::Value, interpret::TrapKind};
use std::io::Write;
pub(crate) fn write(value: Value, newline: bool) -> Result<Value, TrapKind> {
    let mut text = match value {
        Value::Str(text) => text,
        Value::Bool(value) => value.to_string(),
        Value::Char(value) => value.to_string(),
        Value::Integer(value) => value
            .signed_value()
            .map_or_else(|| value.bits().to_string(), |value| value.to_string()),
        Value::F32(value) => value.to_string(),
        Value::F64(value) => value.to_string(),
        _ => return Err(TrapKind::InvalidIr),
    };
    if newline {
        text.push('\n');
    }
    std::io::stdout()
        .lock()
        .write_all(text.as_bytes())
        .map_err(|_| TrapKind::Output)?;
    Ok(Value::Unit)
}

pub(crate) fn write_byte(value: u8) -> Result<Value, TrapKind> {
    std::io::stdout()
        .lock()
        .write_all(&[value])
        .map_err(|_| TrapKind::Output)?;
    Ok(Value::Unit)
}
