# entities

Shared typed entities are the contracts between checking, lowering, interpreters and Rust emission. Types, signatures, values and expression nodes have a single definition. The hir path re-exports these types for existing consumers. Change a node here and handle its semantics in the relevant passes.

| Source | Responsibilities or entry points |
| --- | --- |
| [expressions](expressions.rs) | `Block`, `Statement`, `Expr`, `ExprKind`, `MatchArm` |
| [mod](mod.rs) | Shared typed entities used by checking, lowering and execution. |
| [program](program.rs) | `Program`, `Function`, `RustAdapter`, `valid_path`, `scalar` |
| [raw_layout](raw_layout.rs) | `raw_storage_value`, `raw_copy_value`, `raw_reinterpret_to`, `raw_bits_value`, `raw_scalar` |
| [type_properties](type_properties.rs) | `copyable`, `owns_storage`, `contains_views`, `logical`, `class_key` |
| [types](types.rs) | `Type`, `FunctionContract`, `CallContract`, `Variant`, `VariantShape` |
| [values](values.rs) | `ClassDefinition`, `Value` |
