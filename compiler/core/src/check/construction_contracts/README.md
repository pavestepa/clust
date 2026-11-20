# construction_contracts

Walk constructor branches while tracking the base initialization obligation. Check every reachable successful exit and reject receiver access before required construction.

Module entry: [construction_contracts](../construction_contracts.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [bindings](bindings.rs) | `bind` |
| [expressions](expressions.rs) | `expr` |
