//! Shared typed entities used by checking, lowering and execution.
use clust_frontend::source::Span;
use clust_semantics::{Integer, IntegerType};
mod expressions;
mod program;
mod raw_layout;
mod type_properties;
mod types;
mod values;
pub use expressions::*;
pub use program::*;
pub use types::*;
pub use values::*;
