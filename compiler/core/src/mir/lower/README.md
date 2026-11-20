# lower

Translate HIR into blocks, local slots and operations. The builder records panic successors, resource scopes and transfers as it emits each instruction.

Module entry: [lower](../lower.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [aggregates](aggregates.rs) | `aggregate_fields`, `check_place` |
| [builder](builder.rs) | `block_id`, `end`, `temp`, `evaluate_into`, `evaluate` |
| [expressions](expressions.rs) | `expr_mode` |
| [places](places.rs) | `place` |
| [results](results.rs) | `expr`, `result_into` |
| [scopes](scopes.rs) | `end_storage`, `end_locals`, `end_temporaries`, `block`, `block_mode` |

Further reading: [expressions](expressions/README.md).
