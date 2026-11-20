use super::*;

impl Integer {
    pub(super) fn divide(self, op: BinaryOp, rhs: Self) -> Result<Self, ArithmeticError> {
        if rhs.raw == 0 {
            return Err(ArithmeticError::DivisionByZero);
        }
        let raw = if self.ty.is_signed() {
            let (a, b) = (self.sign_extended(), rhs.sign_extended());
            // Check every width, including MIN % -1, before the host operation.
            if a == self.ty.signed_min() && b == -1 {
                return Err(ArithmeticError::Overflow);
            }
            if op == BinaryOp::Divide {
                (a / b) as u128
            } else {
                (a % b) as u128
            }
        } else if op == BinaryOp::Divide {
            self.raw / rhs.raw
        } else {
            self.raw % rhs.raw
        };
        Ok(Self::from_bits(self.ty, raw))
    }
}

impl Integer {
    pub(super) fn shift(
        self,
        op: BinaryOp,
        rhs: Self,
        checks: OverflowChecks,
    ) -> Result<Self, ArithmeticError> {
        let negative = rhs.ty.is_signed() && rhs.sign_extended() < 0;
        if checks == OverflowChecks::Checked && (negative || rhs.raw >= u128::from(self.ty.bits()))
        {
            return Err(ArithmeticError::InvalidShift);
        }
        let count = (rhs.raw & u128::from(self.ty.bits() - 1)) as u32;
        let raw = if op == BinaryOp::ShiftLeft {
            self.raw << count
        } else if self.ty.is_signed() {
            (self.sign_extended() >> count) as u128
        } else {
            self.raw >> count
        };
        Ok(Self::from_bits(self.ty, raw))
    }
}
