# records

Build nominal definitions and resolve instantiated record, class and enum layouts. Indirection for class fields separates recursive references from inline payload layout.

Module entry: [records](../records.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [arguments](arguments.rs) | `arguments`, `origin`, `get`, `contains` |
| [construction](construction.rs) | `build` |
| [definitions](definitions.rs) | `classes`, `fields`, `is_enum`, `marker`, `copyable` |
| [discovery](discovery.rs) | `collect`, `record` |
| [resolution](resolution.rs) | `resolve` |
