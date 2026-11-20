//! Source locations, diagnostics, and reusable lexical infrastructure.
//! Lexical recognition does not imply acceptance by the Clust language grammar.
pub mod lexer;
pub mod source;

pub mod ast;
mod buffer;
pub mod intrinsics;
pub mod modules;
pub mod parser;

/// Inputs for compiler regression tests that directly exercise lowered operations.
#[cfg(feature = "compiler-tests")]
pub mod testing;
