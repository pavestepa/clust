//! Execute emitted Rust on the host and compare both independent interpreters.
use clust_core::{check::check, hir::Value, interpret::execute, rust::emit_with_sources};
use clust_frontend::{
    source::{Source, SourceId},
    testing::parse_ir as parse,
};
use clust_semantics::OverflowChecks;
use std::{fs, process::Command};
#[path = "native/rust_native_matches_reference.rs"]
mod rust_native_matches_reference;
