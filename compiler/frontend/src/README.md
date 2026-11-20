# frontend

Read source with spans, lex and parse it into the AST, then resolve modules. The production path retains original source locations for semantic diagnostics.

| Source | Responsibilities or entry points |
| --- | --- |
| [ast](ast.rs) | Module declarations and re-exports |
| [buffer](buffer.rs) | Lower the private instruction markers in the embedded core.storage module. Source modules cannot select this pass by path or declare trusted operations. |
| [intrinsics](intrinsics.rs) | Compiler-owned module. Private symbols cannot be spelled by source programs. |
| [lexer](lexer.rs) | `TokenKind`, `Token`, `Lexed`, `lex`, `Lexer` |
| [lib](lib.rs) | Source locations, diagnostics, and reusable lexical infrastructure. Lexical recognition does not imply acceptance by the Clust language grammar. |
| [modules](modules.rs) | Rust-style module declarations and imports, with dot-separated paths. |
| [parser](parser.rs) | `Parsed`, `parse`, `Parser`, `module` |
| [source](source.rs) | `SourceId`, `Span`, `synthetic`, `source_origin`, `join` |
| [testing](testing.rs) | Backend tests also exercise IR operations unavailable to source programs. Parse with the production grammar first, then construct those nodes in Rust. Module linking uses the same pass for test inputs, never for bundled modules. |

Further reading: [ast](ast/README.md), [intrinsics](intrinsics/README.md), [lexer](lexer/README.md), [modules](modules/README.md), [parser](parser/README.md), [source](source/README.md), [testing](testing/README.md).
