# returns

Infer return-view origins and component storage contracts across calls. Merge reachable returns and reject results without a valid surviving owner.

Module entry: [returns](../returns.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [paths](paths.rs) | `concrete_paths`, `owned`, `merge`, `normalize`, `project` |
| [view_inference](view_inference.rs) | `infer`, `analyze` |
