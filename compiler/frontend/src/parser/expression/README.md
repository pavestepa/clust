# expression

Parse literals and prefix forms, then assemble calls, projections and operators by precedence. Operands carry the spans required by later diagnostics.

Module entry: [expression](../expression.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [operands](operands.rs) | `expression_list`, `infix`, `expression_type` |
| [prefix](prefix.rs) | `prefix` |
