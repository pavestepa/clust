# rust

Emit Rust from verified MIR. Register layouts and dispatch types, lower operations and attach source positions to faults. File and manifest builds consume the same emitted source.

Module entry: [rust](../rust.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [classes](classes.rs) | Explicit Copy allocates a fresh class identity; moves only transfer its pointer. |
| [constants](constants.rs) | `constant`, `aggregate`, `enum_value`, `typed_constant` |
| [lifecycle](lifecycle.rs) | Explicit resource state and typed drop glue. Payloads never implement Rust Drop: Clust's checked moves and cleanup boundaries control their lifetime. |
| [operands](operands.rs) | `operand`, `operand_ty`, `place` |
| [operations](operations.rs) | `operation` |
| [program](program.rs) | `emit_internal` |
| [stack](stack.rs) | Conservative frame allocation: closed scalar/class programs only. No class escapes through results, owned parameters, aggregates, function pointers or identity observations. A slot is reused only after every owner is dead. |
| [structured](structured.rs) | Structured, infallible scalar entries. Keep the Result ABI wrapper for code pointers and fallback MIR callers; calls within this subset use plain values. |
| [types](types.rs) | `class_payload`, `register`, `ty`, `borrow_ty`, `captured_slot` |

Further reading: [lifecycle](lifecycle/README.md), [structured](structured/README.md).
