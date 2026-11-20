use super::*;

#[test]
fn exhaustive_eight_bit_arithmetic_against_wider_mathematical_values() {
    for ty in [Ty::I8, Ty::U8] {
        let (min, max) = if ty.is_signed() {
            (-128_i128, 127_i128)
        } else {
            (0, 255)
        };
        for a in min..=max {
            for b in min..=max {
                let (left, right) = (signed(ty, a), signed(ty, b));
                for (op, exact) in [
                    (Op::Add, a + b),
                    (Op::Subtract, a - b),
                    (Op::Multiply, a * b),
                ] {
                    let expected = if (min..=max).contains(&exact) {
                        Ok(Int::from_bits(ty, exact as u128))
                    } else {
                        Err(Error::Overflow)
                    };
                    assert_eq!(
                        left.binary(op, right, Checks::Checked),
                        expected,
                        "{ty:?}: {a} {op:?} {b}"
                    );
                    assert_eq!(
                        left.binary(op, right, Checks::Wrapping).unwrap().bits(),
                        (exact as u128) & 255
                    );
                }
                for op in [Op::Divide, Op::Remainder] {
                    let expected = if b == 0 {
                        Err(Error::DivisionByZero)
                    } else if ty.is_signed() && a == min && b == -1 {
                        Err(Error::Overflow)
                    } else {
                        Ok(Int::from_bits(
                            ty,
                            if op == Op::Divide { a / b } else { a % b } as u128,
                        ))
                    };
                    for checks in [Checks::Checked, Checks::Wrapping] {
                        assert_eq!(left.binary(op, right, checks), expected);
                    }
                }
                assert_eq!(left.compare(right).unwrap(), a.cmp(&b));
                for (op, expected) in [(Op::BitAnd, a & b), (Op::BitOr, a | b), (Op::BitXor, a ^ b)]
                {
                    assert_eq!(
                        left.const_binary(op, right).unwrap().bits(),
                        (expected as u128) & 255
                    );
                }
            }
        }
    }
}

#[test]
fn every_integer_width_checks_boundaries_without_host_overflow() {
    for ty in Ty::ALL {
        let one = Int::literal(ty, 1, false).unwrap();
        let zero = Int::literal(ty, 0, false).unwrap();
        let max = Int::from_bits(
            ty,
            if ty.is_signed() {
                ty.mask() >> 1
            } else {
                ty.mask()
            },
        );
        assert_eq!(max.const_binary(Op::Add, one), Err(Error::Overflow));
        assert_eq!(
            max.const_binary(Op::Multiply, signed(ty, 2)),
            Err(Error::Overflow)
        );
        assert_eq!(
            max.binary(Op::Add, one, Checks::Wrapping).unwrap().bits(),
            if ty.is_signed() {
                1 << (ty.bits() - 1)
            } else {
                0
            }
        );
        assert_eq!(
            max.binary(Op::Divide, zero, Checks::Wrapping),
            Err(Error::DivisionByZero)
        );
        assert_eq!(
            max.binary(Op::Remainder, zero, Checks::Wrapping),
            Err(Error::DivisionByZero)
        );
        if ty.is_signed() {
            let min = Int::from_bits(ty, 1_u128 << (ty.bits() - 1));
            let minus_one = signed(ty, -1);
            for checks in [Checks::Checked, Checks::Wrapping] {
                assert_eq!(
                    min.binary(Op::Divide, minus_one, checks),
                    Err(Error::Overflow)
                );
                assert_eq!(
                    min.binary(Op::Remainder, minus_one, checks),
                    Err(Error::Overflow)
                );
            }
            assert_eq!(min.const_binary(Op::Subtract, one), Err(Error::Overflow));
            assert_eq!(min.negate(Checks::Checked), Err(Error::Overflow));
            assert_eq!(min.negate(Checks::Wrapping), Ok(min));
            assert_eq!(Int::literal(ty, min.bits(), true), Ok(min));
            assert_eq!(
                Int::literal(ty, min.bits(), false),
                Err(Error::LiteralOutOfRange)
            );
        } else {
            assert_eq!(zero.const_binary(Op::Subtract, one), Err(Error::Overflow));
            assert_eq!(one.negate(Checks::Wrapping), Err(Error::UnsignedNegation));
            assert_eq!(Int::literal(ty, 0, true), Err(Error::LiteralOutOfRange));
        }
        if ty.bits() < 128 {
            assert_eq!(
                Int::literal(ty, ty.mask() + 1, false),
                Err(Error::LiteralOutOfRange)
            );
        }
    }
}
