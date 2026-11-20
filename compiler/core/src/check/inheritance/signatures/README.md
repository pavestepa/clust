# signatures

Instantiate parent arguments in inherited signatures and method bodies. Preserve shadowed bindings while substituting type and const parameters, then check override compatibility.

Module entry: [signatures](../signatures.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [compatibility](compatibility.rs) | `compatible` |
| [constants](constants.rs) | `freeze_constants` |
| [instantiation](instantiation.rs) | `method`, `expr`, `block`, `argument`, `ty` |
| [shadowing](shadowing.rs) | `hide_pattern`, `body`, `signature` |
| [substitution](substitution.rs) | `parent`, `argument`, `ty`, `expr`, `block` |
