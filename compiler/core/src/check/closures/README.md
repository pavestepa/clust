# closures

Find free variables with lexical scope and infer their access from later uses. Lower a closure to a function or a callable object. A scoped mutable capture limits where the callable may escape.

Module entry: [closures](../closures.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [captures](captures.rs) | `generic_arguments`, `function`, `scoped`, `infer`, `block` |
| [compatibility](compatibility.rs) | `crate_name`, `compatible` |
| [conversion](conversion.rs) | `expression` |
| [mutation](mutation.rs) | `mutations`, `scan`, `root`, `expression` |
| [preparation](preparation.rs) | `prepare` |
| [receivers](receivers.rs) | `nominal_name`, `find_member`, `receiver_type` |
| [rewrite](rewrite.rs) | `rewrite_block`, `rewrite` |

Further reading: [conversion](conversion/README.md).
