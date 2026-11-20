# verify

Validate block targets, projections, call contracts, ownership and cleanup before emission. Both generated and independently constructed MIR enter through verify.

Module entry: [verify](../verify.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [cleanup_edges](cleanup_edges.rs) | `cleanup_exits`, `writable` |
| [operands](operands.rs) | `operand_type` |
| [places](places.rs) | `place_type` |
| [raw_operations](raw_operations.rs) | `verify_raw` |
