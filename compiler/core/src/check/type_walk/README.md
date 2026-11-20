# type_walk

Visit types nested in declarations, arguments, blocks and expressions. Clients supply a visitor so discovery and rewriting use the same traversal.

Module entry: [type_walk](../type_walk.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [expressions](expressions.rs) | `block`, `expression` |
| [types](types.rs) | `arg`, `ty` |
