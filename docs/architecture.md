# Compiler architecture

| Boundary | Input and result | Implementation guide |
| --- | --- | --- |
| Frontend | Source text and reachable modules → AST with resolved names and original spans | [Frontend](../compiler/frontend/src/README.md) |
| Semantic checking | AST → checked program with specialized calls and access contracts | [Checker](../compiler/core/src/check/README.md) |
| Shared entities | Type, value, function, expression and foreign call definitions used by compiler passes | [Entities](../compiler/core/src/entities/README.md) |
| Ownership | Lowered control flow → loan and initialization diagnostics. Return contracts and storage writes also feed the checker’s inference loop | [COLI](../compiler/core/src/mir/coli/README.md), [MIR](../compiler/core/src/mir/README.md) |
| Native backend | Verified MIR → standalone Rust source | [Rust emission](../compiler/core/src/rust/README.md) |
| Host tools | Source or Clust.toml → check, evaluation, native binary or generated source | [CLI](../cli/src/README.md) |
| Runtime | Allocation, scope cleanup, Tokio tasks and concurrency primitives | [Runtime](../runtime/src/README.md) |

The synchronous compiler owns its semantic decisions. HIR and MIR interpreters provide independent execution checks. The async frontend currently selects a separate Rust lowering with its own parameter inference. Its programs still use Cargo and the Tokio runtime. A future Rust importer will need an explicit integration point for each path until they converge.

Use the feature owner for a new operation. Source syntax belongs in the frontend, its typed representation in entities, access and transfer rules in the checker and MIR, and its physical representation in the backend. The existing public HIR module re-exports shared entities. Introducing a second copy of a type would break compatibility and require conversion work in hot paths.

Keep concrete data structures and static calls across internal module boundaries. A Rust source module introduces a namespace without requiring a service object, heap allocation or dynamic dispatch. Aim for 200 nonblank, noncomment lines per implementation file. When a pass grows, extract a complete operation with its inputs and outputs. A single exhaustive verifier or instruction match may exceed the target while its operations still share one set of invariants.

Validation uses `cargo test --workspace --locked --offline`, `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings`. The [implementation status](implementation-status.md) documents tested language behavior.

## Size profile

Of the 390 Rust implementation and test files in this checkout, 350 are at or below 200 nonblank, noncomment lines. Forty exceed the guideline. The largest is the closure capture analysis (324 lines), followed by pattern binding (318), MIR instruction execution (311) and Rust operation emission (304). The guideline is a design target, not an enforced build limit.
