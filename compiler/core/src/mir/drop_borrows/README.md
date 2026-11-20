# drop_borrows

Derive cleanup dependencies for destructors that read borrowed fields. Use provenance and initialization order to check that targets survive through destruction.

Module entry: [drop_borrows](../drop_borrows.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [ordering](ordering.rs) | `orders` |
