# parser

Build source entities with one token cursor. Item, type, expression and pattern parsers share nesting limits and diagnostics.

Module entry: [parser](../parser.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [boundaries](boundaries.rs) | `end`, `nested`, `unsupported`, `statement_end` |
| [cursor](cursor.rs) | `current`, `at_eof`, `at`, `word`, `newline` |
| [expression](expression.rs) | `literal`, `expr`, `expression` |
| [item](item.rs) | `item` |
| [iteration](iteration.rs) | Collection iteration evaluates its source once. Advance before the body so `continue` cannot repeat an element; each element binding has iteration scope. |
| [pattern](pattern.rs) | `pattern`, `pattern_inner`, `pattern_list` |
| [statements](statements.rs) | `block`, `block_with_tail`, `statement` |
| [types](types.rs) | `ty`, `type_inner`, `close_angle` |

Further reading: [expression](expression/README.md), [item](item/README.md), [types](types/README.md).
