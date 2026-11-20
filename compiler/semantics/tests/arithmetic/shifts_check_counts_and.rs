use super::*;

#[test]
fn shifts_check_counts_and_mask_only_in_wrapping_mode() {
    for ty in Ty::ALL {
        let one = signed(ty, 1);
        for count_ty in Ty::ALL {
            let width = signed(count_ty, i128::from(ty.bits().min(127)));
            // I8 cannot represent positive 128. Test it through all wider count types.
            let width = if ty.bits() == 128 && count_ty == Ty::I8 {
                None
            } else {
                Some(if ty.bits() == 128 {
                    signed(count_ty, 128)
                } else {
                    width
                })
            };
            if let Some(width) = width {
                for op in [Op::ShiftLeft, Op::ShiftRight] {
                    assert_eq!(
                        one.binary(op, width, Checks::Checked),
                        Err(Error::InvalidShift)
                    );
                    assert_eq!(one.binary(op, width, Checks::Wrapping), Ok(one));
                }
            }
        }
        let negative = signed(Ty::I128, -1);
        assert_eq!(
            one.const_binary(Op::ShiftLeft, negative),
            Err(Error::InvalidShift)
        );
        assert_eq!(
            one.binary(Op::ShiftLeft, negative, Checks::Wrapping)
                .unwrap()
                .bits(),
            1 << (ty.bits() - 1)
        );
        let last = signed(Ty::Usize, i128::from(ty.bits() - 1));
        assert_eq!(
            one.const_binary(Op::ShiftLeft, last).unwrap().bits(),
            1 << (ty.bits() - 1)
        );
        let all = Int::from_bits(ty, ty.mask());
        assert_eq!(
            all.const_binary(Op::ShiftRight, signed(Ty::U8, 1))
                .unwrap()
                .bits(),
            if ty.is_signed() {
                ty.mask()
            } else {
                ty.mask() >> 1
            }
        );
        assert_eq!(all.bit_not().bits(), 0);
        assert_eq!(
            one.const_binary(Op::ShiftLeft, Int::from_bits(Ty::U128, u128::MAX)),
            Err(Error::InvalidShift)
        );
    }
}

#[test]
fn casts_preserve_sign_before_destination_truncation() {
    assert_eq!(signed(Ty::I8, -1).cast(Ty::U128).bits(), u128::MAX);
    assert_eq!(signed(Ty::U8, 255).cast(Ty::I16).signed_value(), Some(255));
    assert_eq!(signed(Ty::I16, -129).cast(Ty::I8).signed_value(), Some(127));
    assert_eq!(
        Int::from_bits(Ty::U128, u128::MAX)
            .cast(Ty::I128)
            .signed_value(),
        Some(-1)
    );
    for ty in Ty::ALL {
        let max = Int::from_bits(ty, ty.mask());
        assert_eq!(max.cast(ty), max);
    }
}

#[test]
fn float_casts_saturate_instead_of_producing_poison() {
    for ty in Ty::ALL {
        assert_eq!(Int::from_float(ty, f64::NAN).bits(), 0);
        assert_eq!(Int::from_float(ty, -0.0).bits(), 0);
        assert_eq!(Int::from_float(ty, 1.99).bits(), 1);
        let max = if ty.is_signed() {
            ty.mask() >> 1
        } else {
            ty.mask()
        };
        assert_eq!(Int::from_float(ty, f64::INFINITY).bits(), max);
        assert_eq!(
            Int::from_float(ty, f64::NEG_INFINITY).bits(),
            if ty.is_signed() {
                1 << (ty.bits() - 1)
            } else {
                0
            }
        );
    }
    assert_eq!(Int::from_float(Ty::I8, -128.9).signed_value(), Some(-128));
    assert_eq!(Int::from_float(Ty::U8, -1.0).bits(), 0);
    assert_eq!(Int::from_float(Ty::U8, 256.0).bits(), 255);
    assert_eq!(
        Int::from_float(Ty::I128, 2_f64.powi(127)).signed_value(),
        Some(i128::MAX)
    );
    assert_eq!(Int::from_float(Ty::U128, 2_f64.powi(128)).bits(), u128::MAX);
}

#[test]
fn integer_types_remain_distinct_even_with_equal_width() {
    let a = signed(Ty::Usize, 1);
    let b = signed(Ty::U64, 1);
    assert_eq!(a.const_binary(Op::Add, b), Err(Error::TypeMismatch));
    assert_eq!(a.compare(b), Err(Error::TypeMismatch));
    assert_eq!(Ty::Usize.bits(), 64);
    assert_eq!(Ty::Isize.bits(), 64);
}

#[test]
fn exhaustive_eight_bit_shifts_and_negation() {
    for raw in 0_u128..=255 {
        for ty in [Ty::I8, Ty::U8] {
            let left = Int::from_bits(ty, raw);
            for count in -128_i128..=255 {
                let right = signed(Ty::I16, count);
                let reduced = count.rem_euclid(8) as u32;
                let expected_left = ((raw as u16) * 2_u16.pow(reduced)) as u8;
                // Wider signed host arithmetic is independent of the oracle's
                // normalized u128 representation and always fits here.
                let expected_right = if ty.is_signed() {
                    ((raw as u8 as i8 as i16) >> reduced) as u8
                } else {
                    ((raw as u16) / 2_u16.pow(reduced)) as u8
                };
                for (op, expected) in [
                    (Op::ShiftLeft, expected_left),
                    (Op::ShiftRight, expected_right),
                ] {
                    assert_eq!(
                        left.binary(op, right, Checks::Wrapping).unwrap().bits(),
                        u128::from(expected)
                    );
                    let checked = left.const_binary(op, right);
                    if !(0..8).contains(&count) {
                        assert_eq!(checked, Err(Error::InvalidShift));
                    } else {
                        assert_eq!(checked.unwrap().bits(), u128::from(expected));
                    }
                }
            }
        }
        let left = Int::from_bits(Ty::I8, raw);
        let wide = -(raw as u8 as i8 as i16);
        assert_eq!(
            left.negate(Checks::Wrapping).unwrap().bits(),
            u128::from(wide as u8)
        );
        if raw == 128 {
            assert_eq!(left.negate(Checks::Checked), Err(Error::Overflow));
        } else {
            assert_eq!(
                left.negate(Checks::Checked).unwrap().signed_value(),
                Some(i128::from(wide))
            );
        }
    }
}
