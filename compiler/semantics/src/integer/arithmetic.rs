use super::*;

impl Integer {
    pub fn negate(self, checks: OverflowChecks) -> Result<Self, ArithmeticError> {
        if !self.ty.is_signed() {
            return Err(ArithmeticError::UnsignedNegation);
        }
        if checks == OverflowChecks::Checked && self.sign_extended() == self.ty.signed_min() {
            return Err(ArithmeticError::Overflow);
        }
        Ok(Self::from_bits(self.ty, self.raw.wrapping_neg()))
    }
}

impl Integer {
    pub fn bit_not(self) -> Self {
        Self::from_bits(self.ty, !self.raw)
    }
}

impl Integer {
    pub fn compare(self, rhs: Self) -> Result<std::cmp::Ordering, ArithmeticError> {
        self.require_same_type(rhs)?;
        Ok(if self.ty.is_signed() {
            self.sign_extended().cmp(&rhs.sign_extended())
        } else {
            self.raw.cmp(&rhs.raw)
        })
    }
}

impl Integer {
    pub fn binary(
        self,
        op: BinaryOp,
        rhs: Self,
        checks: OverflowChecks,
    ) -> Result<Self, ArithmeticError> {
        use BinaryOp::*;
        if matches!(op, ShiftLeft | ShiftRight) {
            return self.shift(op, rhs, checks);
        }
        self.require_same_type(rhs)?;
        let raw = match op {
            BitAnd => self.raw & rhs.raw,
            BitOr => self.raw | rhs.raw,
            BitXor => self.raw ^ rhs.raw,
            Divide | Remainder => return self.divide(op, rhs),
            Add | Subtract | Multiply => {
                if checks == OverflowChecks::Checked {
                    self.check_arithmetic(op, rhs)?;
                }
                match op {
                    Add => self.raw.wrapping_add(rhs.raw),
                    Subtract => self.raw.wrapping_sub(rhs.raw),
                    Multiply => self.raw.wrapping_mul(rhs.raw),
                    _ => unreachable!(),
                }
            }
            ShiftLeft | ShiftRight => unreachable!(),
        };
        Ok(Self::from_bits(self.ty, raw))
    }
}

impl Integer {
    /// Const arithmetic always checks overflow, regardless of build profile.
    pub fn const_binary(self, op: BinaryOp, rhs: Self) -> Result<Self, ArithmeticError> {
        self.binary(op, rhs, OverflowChecks::Checked)
    }
}

impl Integer {
    pub(super) fn require_same_type(self, rhs: Self) -> Result<(), ArithmeticError> {
        if self.ty == rhs.ty {
            Ok(())
        } else {
            Err(ArithmeticError::TypeMismatch)
        }
    }
}

impl Integer {
    pub(super) fn check_arithmetic(self, op: BinaryOp, rhs: Self) -> Result<(), ArithmeticError> {
        use BinaryOp::*;
        let fits = if self.ty.is_signed() {
            let (a, b) = (self.sign_extended(), rhs.sign_extended());
            let value = match op {
                Add => a.checked_add(b),
                Subtract => a.checked_sub(b),
                Multiply => a.checked_mul(b),
                _ => unreachable!(),
            };
            value.is_some_and(|v| v >= self.ty.signed_min() && v <= self.ty.signed_max())
        } else {
            let value = match op {
                Add => self.raw.checked_add(rhs.raw),
                Subtract => self.raw.checked_sub(rhs.raw),
                Multiply => self.raw.checked_mul(rhs.raw),
                _ => unreachable!(),
            };
            value.is_some_and(|v| v <= self.ty.mask())
        };
        if fits {
            Ok(())
        } else {
            Err(ArithmeticError::Overflow)
        }
    }
}
