# lexer

Scan the original bytes into tokens and spans. Cursor movement is shared by comment, escape and numeric scanning so diagnostics refer to the correct offsets.

Module entry: [lexer](../lexer.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [comments](comments.rs) | `comment` |
| [cursor](cursor.rs) | `rest`, `peek`, `bump`, `span`, `emit` |
| [numbers](numbers.rs) | `number` |
| [strings](strings.rs) | `quoted`, `escape`, `digits` |
