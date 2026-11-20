use super::*;
impl Engine<'_> {
    pub(super) fn unary(
        &mut self,
        operator: &&'static str,
        value: &Expr,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        let value = self.expr(value, frame)?;
        match (*operator, value) {
            ("str_byte_len", Value::Str(value)) => Ok(Value::Integer(Integer::from_bits(
                clust_semantics::IntegerType::Usize,
                value.len() as u128,
            ))),
            ("write_byte", Value::Integer(value)) => {
                crate::output::write_byte(value.bits() as u8).map_err(|kind| trap(kind, span))
            }
            ("io_print", value) => {
                crate::output::write(value, false).map_err(|kind| trap(kind, span))
            }
            ("io_println", value) => {
                crate::output::write(value, true).map_err(|kind| trap(kind, span))
            }
            ("assert", Value::Bool(true)) => Ok(Value::Unit),
            ("assert", Value::Bool(false)) => Err(trap(TrapKind::Assertion, span)),
            ("raw_read", Value::Pointer(p)) => p.read().map_err(|kind| trap(kind, span)),
            ("-", Value::Integer(value)) => value
                .negate(self.checks)
                .map(Value::Integer)
                .map_err(|e| trap(TrapKind::Arithmetic(e), span)),
            ("!", Value::Integer(value)) => Ok(Value::Integer(value.bit_not())),
            ("!", Value::Bool(value)) => Ok(Value::Bool(!value)),
            ("-", Value::F32(value)) => Ok(Value::F32(-value)),
            ("-", Value::F64(value)) => Ok(Value::F64(-value)),
            _ => Err(trap(TrapKind::InvalidIr, span)),
        }
    }
    pub(super) fn binary_value(
        &mut self,
        operator: &&'static str,
        left: &Expr,
        right: &Expr,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value> {
        let left = self.expr(left, frame)?;
        if (*operator == "&&" && left == Value::Bool(false))
            || (*operator == "||" && left == Value::Bool(true))
        {
            return Ok(left);
        }
        let right = self.expr(right, frame)?;
        binary(operator, left, right, self.checks).map_err(|kind| trap(kind, span))
    }
}
