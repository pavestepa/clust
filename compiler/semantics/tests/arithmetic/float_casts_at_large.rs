use super::*;

#[test]
fn float_casts_at_large_integer_boundaries() {
    // The representable f64 directly below each upper boundary must not be
    // mistaken for the boundary itself (integer maxima round upwards as f64).
    for (ty, exponent) in [
        (Ty::I64, 63),
        (Ty::U64, 64),
        (Ty::I128, 127),
        (Ty::U128, 128),
    ] {
        let boundary = 2_f64.powi(exponent);
        let below = f64::from_bits(boundary.to_bits() - 1);
        let expected = if exponent == 128 {
            u128::MAX - ((1_u128 << 75) - 1)
        } else {
            (1_u128 << exponent) - (1_u128 << (exponent - 53))
        };
        assert_eq!(Int::from_float(ty, below).bits(), expected);
        assert_eq!(
            Int::from_float(ty, boundary).bits(),
            if ty.is_signed() {
                ty.mask() >> 1
            } else {
                ty.mask()
            }
        );
    }
}
