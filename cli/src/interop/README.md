# Native Rust imports

The CLI calls `compile` to discover mixed modules and produce a Rust application.
`modules` resolves file layouts and Cargo aliases, and `scalars` can supply
virtual declarations to the full Clust checker. For richer imports, `tokens`
preserves macro invocations, `lower` emits Rust expressions and `data` with
`data_privacy` emits source-facing data types with private members.

During a native Cargo build, `metadata` reads expanded items and re-exports
from rustdoc JSON. `contracts` maps functions and associated signatures into
COLI access modes for Clust parameter inference. `origins` checks local uses
of reference results. Rust checks method resolution, generic substitutions,
future lifetimes and destruction.

`standard` supplies typed wrappers for explicit Clust Vec and String imports.
`adapt` inserts borrows or transfers their backing allocations at generated
call arguments, then reruns Cargo. See the [CLI guide](../../../docs/rust-interop.md).
