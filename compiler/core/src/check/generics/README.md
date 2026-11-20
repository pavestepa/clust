# generics

Infer values for type and const parameters, apply defaults and verify bounds. A specialization key identifies the checked instance in the worklist.

Module entry: [generics](../generics.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [arguments](arguments.rs) | `generic_arguments` |
| [complexity](complexity.rs) | `validate_complexity` |
| [inference](inference.rs) | `infer` |
| [substitute](substitute.rs) | `Substitution`, `ty`, `arguments`, `function`, `pattern` |

Further reading: [substitute](substitute/README.md).
