# interpret

Execute the lowered control-flow graph against the reference memory model. Compare results and cleanup traces with HIR and native programs.

Module entry: [interpret](../interpret.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [drop](drop.rs) | `DropMemory`, `class`, `initialization`, `initialization_order`, `begin` |
| [execution](execution.rs) | `run` |
| [invocation](invocation.rs) | `call` |
| [operands](operands.rs) | `operand`, `indices`, `resolve` |
| [operations](operations.rs) | `operation` |
