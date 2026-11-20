# dataflow

Compute facts across control-flow edges until they stabilize. Liveness includes cleanup edges. Initialization queries track places and projections so disjoint fields retain independent state.

Module entry: [dataflow](../dataflow.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [initialization](initialization.rs) | `required_initialized`, `place_initialization_errors`, `initialization_errors`, `consumed` |
