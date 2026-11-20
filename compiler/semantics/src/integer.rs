use crate::TARGET_POINTER_BITS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerType {
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
}

impl IntegerType {
    pub const ALL: [Self; 12] = [
        Self::I8,
        Self::I16,
        Self::I32,
        Self::I64,
        Self::I128,
        Self::Isize,
        Self::U8,
        Self::U16,
        Self::U32,
        Self::U64,
        Self::U128,
        Self::Usize,
    ];

    pub const fn bits(self) -> u32 {
        match self {
            Self::I8 | Self::U8 => 8,
            Self::I16 | Self::U16 => 16,
            Self::I32 | Self::U32 => 32,
            Self::I64 | Self::U64 => 64,
            Self::I128 | Self::U128 => 128,
            Self::Isize | Self::Usize => TARGET_POINTER_BITS,
        }
    }

    pub const fn is_signed(self) -> bool {
        matches!(
            self,
            Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::I128 | Self::Isize
        )
    }

    pub const fn mask(self) -> u128 {
        u128::MAX >> (128 - self.bits())
    }

    fn signed_min(self) -> i128 {
        i128::MIN >> (128 - self.bits())
    }

    fn signed_max(self) -> i128 {
        i128::MAX >> (128 - self.bits())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverflowChecks {
    /// Debug default and mandatory for constant evaluation.
    Checked,
    /// Release default; division faults remain errors.
    Wrapping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithmeticError {
    TypeMismatch,
    LiteralOutOfRange,
    Overflow,
    DivisionByZero,
    InvalidShift,
    UnsignedNegation,
}

/// A typed, normalized two's-complement bit pattern, including all u128 values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Integer {
    ty: IntegerType,
    raw: u128,
}

impl Integer {
    /// Constructs a bit pattern, not a source literal. High bits are truncated.
    pub fn from_bits(ty: IntegerType, raw: u128) -> Self {
        Self {
            ty,
            raw: raw & ty.mask(),
        }
    }

    /// Checks the magnitude before narrowing. `negative` handles signed MIN
    /// without requiring its positive magnitude to fit the signed type.
    pub fn literal(
        ty: IntegerType,
        magnitude: u128,
        negative: bool,
    ) -> Result<Self, ArithmeticError> {
        let limit = if ty.is_signed() {
            ty.signed_max() as u128 + u128::from(negative)
        } else {
            ty.mask()
        };
        if magnitude > limit || (negative && !ty.is_signed()) {
            return Err(ArithmeticError::LiteralOutOfRange);
        }
        Ok(Self::from_bits(
            ty,
            if negative {
                magnitude.wrapping_neg()
            } else {
                magnitude
            },
        ))
    }

    pub fn ty(self) -> IntegerType {
        self.ty
    }
    pub fn bits(self) -> u128 {
        self.raw
    }

    /// Returns the mathematical signed value only for signed types.
    pub fn signed_value(self) -> Option<i128> {
        self.ty.is_signed().then(|| self.sign_extended())
    }

    fn sign_extended(self) -> i128 {
        let shift = 128 - self.ty.bits();
        ((self.raw << shift) as i128) >> shift
    }

    /// Numeric `as`: sign/zero extension followed by truncation to destination.
    pub fn cast(self, destination: IntegerType) -> Self {
        let raw = if self.ty.is_signed() {
            self.sign_extended() as u128
        } else {
            self.raw
        };
        Self::from_bits(destination, raw)
    }

    /// Saturating float-to-int cast: truncate toward zero, NaN -> 0.
    /// An f32 source is widened exactly before entering this function.
    pub fn from_float(destination: IntegerType, value: f64) -> Self {
        if destination.is_signed() {
            let clamped = (value as i128).clamp(destination.signed_min(), destination.signed_max());
            Self::from_bits(destination, clamped as u128)
        } else {
            Self::from_bits(destination, (value as u128).min(destination.mask()))
        }
    }
}

mod arithmetic;
mod division;
