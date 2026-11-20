# borrow_check

Internal loan analysis maps descriptors to their possible origins and follows them to later uses. Conflict checking and diagnostics consume this provenance. The source checker calls the COLI facade.

Module entry: [borrow_check](../borrow_check.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [components](components.rs) | `components`, `reachable` |
| [conflicts](conflicts.rs) | `check` |
| [diagnostics](diagnostics.rs) | `diagnostic`, `covers`, `compatible_prefix` |
| [origins](origins.rs) | `resolve` |
| [parameters](parameters.rs) | `parameter_loans`, `components`, `merge`, `places` |
| [provenance_analysis](provenance_analysis.rs) | `provenance` |
| [transfer](transfer.rs) | `transfer`, `call_origin`, `parameter_slot` |
