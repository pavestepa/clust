# expressions

Evaluate HIR operands in source order. The expression dispatcher delegates calls, storage operations, weak access, patterns and place reads to typed helpers on the same execution engine.

Module entry: [expressions](../expressions.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [calls](calls.rs) | `invoke` |
| [operators](operators.rs) | `unary`, `binary_value` |
| [patterns](patterns.rs) | `tag`, `match_value` |
| [places](places.rs) | `read_view`, `identity`, `take`, `read_place`, `assign` |
| [storage](storage.rs) | `storage_view`, `address_conversion`, `allocate`, `storage_operation` |
| [weak](weak.rs) | `weak_identity`, `weak_access` |
