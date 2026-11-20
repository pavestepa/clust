# core

The core consumes a resolved frontend module. Semantic checking produces a typed program. MIR lowering makes ownership, control flow and cleanup explicit. The Rust emitter and the two reference interpreters consume these representations.

| Source | Responsibilities or entry points |
| --- | --- |
| [check](check.rs) | `Signature`, `ReturnInfo`, `Pass`, `check`, `check_clust` |
| [constants](constants.rs) | `evaluate`, `Context`, `type_names`, `block_names`, `names` |
| [hir](hir.rs) | Compatibility path for typed compiler entities. |
| [interpret](interpret.rs) | Bounded reference execution for the currently checked value subset. This is an oracle, not the native runtime or a substitute for Rust source lowering. |
| [layout](layout.rs) | Target layout validation, independent of source code generation. |
| [lib](lib.rs) | Module declarations and re-exports |
| [mir](mir.rs) | Explicit control-flow representation. Every fallible operation has a panic edge. Current lowering admits Copy values; ownership operations are added only with checking. |
| [output](output.rs) | Shared scalar output for both reference engines. |
| [reference_memory](reference_memory.rs) | Host-side cells used only by the bounded reference interpreters. Clearing a slot ends its language lifetime even when host handles to that cell remain. Native views use raw addresses and never allocate or retain these cells. |
| [rust](rust.rs) | Standalone Rust source backend. Clust's checked MIR owns lifetime semantics; generated raw places do not introduce Rust reference aliasing assumptions. |

Further reading: [check](check/README.md), [entities](entities/README.md), [interpret](interpret/README.md), [layout](layout/README.md), [mir](mir/README.md), [reference_memory](reference_memory/README.md), [rust](rust/README.md).
