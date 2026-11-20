//! Reusable arithmetic and layout utilities for the compiler.
//! These utilities do not select Clust's pending overflow or layout policies.
mod integer;
mod layout;

pub use integer::{ArithmeticError, BinaryOp, Integer, IntegerType, OverflowChecks};
pub use layout::{AggregateLayout, EnumLayout, Layout, LayoutError, MAX_OBJECT_SIZE};

/// Initial utility target profile; not a language-wide pointer-width guarantee.
pub const TARGET_POINTER_BITS: u32 = 64;

mod storage;
pub use storage::{ElementMode, ModeError, ModeRequirement, ModeVar, StorageModes};
