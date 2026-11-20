# coli

Use validate on a lowered program after return and storage contracts settle. It emits loan conflicts first and moved or uninitialized place diagnostics second. The verifier can request the first loan error lazily. Provenance and conflict bookkeeping stay inside the compiler.

Module entry: [coli](../coli.rs).

The foreign contract API records Rust argument access and the input indices
that can lend to a returned reference or future. It checks known local places
using the same loan domain. The Rust compiler verifies the emitted native code,
including generic substitutions, projections, outlives constraints and Drop.
