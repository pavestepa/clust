# ast

Shared source entities describe declarations, types, expressions and patterns. Every consumer uses these definitions before semantic lowering. Spans refer to the original source.

Module entry: [ast](../ast.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [declarations](declarations.rs) | `Module`, `Item`, `Attribute`, `ItemKind`, `UseBinding` |
| [expressions](expressions.rs) | `Block`, `Statement`, `StatementKind`, `Expr`, `Literal` |
| [patterns](patterns.rs) | `Arm`, `Pattern`, `PatternKind`, `PatternFields` |
| [types](types.rs) | `Type`, `TypeKind`, `TypeArgument` |
