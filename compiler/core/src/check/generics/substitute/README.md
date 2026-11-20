# substitute

Apply a substitution recursively to source types, patterns, expressions and declarations. Lexical bindings and const expressions must keep the correct definition context.

Module entry: [substitute](../substitute.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [constants](constants.rs) | `number`, `ast_type` |
| [expressions](expressions.rs) | `block`, `expr` |
