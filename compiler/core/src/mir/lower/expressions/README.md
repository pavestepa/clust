# expressions

Lower expression operands to MIR. Branching expressions create explicit block successors and result slots. Borrowed results retain the selected access mode across joins.

Module entry: [expressions](../expressions.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [control_flow](control_flow.rs) | `conditional`, `while_loop`, `match_arms`, `short_circuit`, `assignment` |
