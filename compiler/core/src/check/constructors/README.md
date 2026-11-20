# constructors

Resolve constructor declarations and check field initialization. The branch state records which fields may be read and which still require assignment.

Module entry: [constructors](../constructors.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [bindings](bindings.rs) | `is_this`, `join`, `bindings` |
| [expressions](expressions.rs) | `expr` |
| [initialization](initialization.rs) | `complete`, `field`, `block` |
