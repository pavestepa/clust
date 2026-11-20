# interpret

Execute typed HIR with a bounded step count. Each frame tracks locals, views and temporaries. Scope exit and failure both use the explicit cleanup model.

Module entry: [interpret](../interpret.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [blocks](blocks.rs) | `block`, `block_mode`, `block_body` |
| [drop](drop.rs) | `DropMemory`, `class`, `initialization`, `initialization_order`, `begin` |
| [expressions](expressions.rs) | `expr`, `expr_mode`, `expr_body` |
| [invocation](invocation.rs) | `tick`, `call` |
| [places](places.rs) | `place` |
| [scalars](scalars.rs) | `trap`, `index_value`, `binary`, `comparison`, `cast` |

Further reading: [drop](drop/README.md), [expressions](expressions/README.md).
