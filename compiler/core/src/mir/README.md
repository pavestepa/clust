# mir

Represent typed control flow as basic blocks, operations, places and explicit failure edges. Lowering constructs the graph. The verifier checks its invariants before native emission.

Module entry: [mir](../mir.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [borrow_check](borrow_check.rs) | Flow-sensitive origins and last-use loan checking for local safe views. View slots carry pointers, while their logical type remains the target type. |
| [cleanup](cleanup.rs) | Lexical ownership of cleanup records. Initialization order within each scope is dynamic; replacing a root never changes its owning scope. |
| [coli](coli.rs) | Ownership diagnostics for checked control flow. |
| [dataflow](dataflow.rs) | Backward liveness with distinct successful and panic edges. |
| [drop_borrows](drop_borrows.rs) | Implicit reads made by user destructors, and possible root cleanup orders. The order domain stores may-before pairs, so joins and loops never assume a favorable initialization order. The runtime still uses its dynamic root list. |
| [drop_plan](drop_plan.rs) | Type-directed destruction recipes. Arrays stay compact even for huge lengths; execution consults initialization state before reading any payload or tag. |
| [drop_state](drop_state.rs) | Sparse runtime initialization oracle for cleanup. Unlike compile-time facts, these states distinguish incomplete construction from a moved-out subfield. |
| [effects](effects.rs) | Effects on borrowed parameter targets, including aliases and recursive calls. A field write cannot replace its containing target. Rebinding a view changes only the local pointer and therefore has no target-write effect. |
| [initialization](initialization.rs) | Sparse initialization facts for move paths, independent of ownership/access. A caller supplies all queried paths when joining control-flow alternatives. |
| [interpret](interpret.rs) | Reference execution of explicit CFG, independent of HIR control-flow traversal. |
| [loans](loans.rs) | Loan conflict domain for resolved origins. Binding aliases must first be normalized to their underlying root places by the ownership inference pass. |
| [lower](lower.rs) | `lower`, `function`, `Builder` |
| [reachability](reachability.rs) | Proven control-flow edges for local enum discriminants and scalar constants. Unknown inputs, calls and aliased writes retain all possible branches. This analysis never depends on loans, so provenance can use it without a cycle. |
| [returns](returns.rs) | Static result provenance. Facts flow through value temporaries before final ownership validation, allowing a borrowed entry to return its input view. |
| [storage_writes](storage_writes.rs) | Static may-write dependencies of mutable borrowed aggregate parameters. Include intermediate states, not just returns: writes survive panic and must never publish a descriptor to an owner destroyed inside the callee. |
| [verify](verify.rs) | Validation at the public MIR/backend boundary. Run structural checks before analyses, which deliberately assume valid indices. Recompute provenance rather than trusting serialized/caller-supplied result contracts. |

Further reading: [borrow_check](borrow_check/README.md), [cleanup](cleanup/README.md), [coli](coli/README.md), [dataflow](dataflow/README.md), [drop_borrows](drop_borrows/README.md), [drop_plan](drop_plan/README.md), [drop_state](drop_state/README.md), [effects](effects/README.md), [initialization](initialization/README.md), [interpret](interpret/README.md), [loans](loans/README.md), [lower](lower/README.md), [returns](returns/README.md), [storage_writes](storage_writes/README.md), [verify](verify/README.md).
