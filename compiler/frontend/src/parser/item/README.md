# item

Parse declarations, imports, members and signatures. Constructor and receiver rules are recorded in the resulting AST for validation by the checker.

Module entry: [item](../item.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [aggregates](aggregates.rs) | `aggregate` |
| [attributes](attributes.rs) | `string` |
| [constants](constants.rs) | `constant` |
| [functions](functions.rs) | `function`, `parameters` |
| [imports](imports.rs) | `use_tree`, `use_tree_inner` |
