use clust_semantics::{
    ArithmeticError as Error, BinaryOp as Op, Integer as Int, IntegerType as Ty,
    OverflowChecks as Checks,
};

fn signed(ty: Ty, value: i128) -> Int {
    Int::literal(ty, value.unsigned_abs(), value < 0).unwrap()
}

#[path = "arithmetic/exhaustive_eight_bit_arithmetic.rs"]
mod exhaustive_eight_bit_arithmetic;
#[path = "arithmetic/float_casts_at_large.rs"]
mod float_casts_at_large;
#[path = "arithmetic/shifts_check_counts_and.rs"]
mod shifts_check_counts_and;
