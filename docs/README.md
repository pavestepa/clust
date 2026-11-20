# Clust Language Documentation

These documents describe the language foundation, the implemented synchronous compiler and remaining delivery boundaries. [Implementation status](implementation-status.md) distinguishes tested behavior from broader specification requirements.

| Document | Purpose |
| --- | --- |
| [Syntax](syntax.md) | Source syntax, core forms and supported boundaries. |
| [Types and members](semantics/types.md) | Nominal types, field typing, member visibility, and constructor signatures. |
| [Construction](semantics/construction.md) | Always-public `new`, base initialization, definite initialization, and cleanup obligations. |
| [Inheritance and polymorphism](semantics/inheritance.md) | Class subtyping, actual-class dispatch, base calls, and complete-object preservation. |
| [Ownership and lifetimes](semantics/ownership.md) | COLI obligations for reading, mutation, ownership transfer, lifetimes, and placement. |
| [Stage 1 checklist](stage-1.md) | Current coverage, unresolved decisions, validation cases, and the completion criteria. |
| [Implementation status](implementation-status.md) | Integrated compiler components and remaining differences from the approved language. |
| [Closures and weak references](semantics/closures-and-weak.md) | Capture, escape, exclusive access and weak identity contracts. |
| [Runtime invariants](runtime-invariants.md) | Generated Rust storage, dispatch, cleanup and weak-access safety obligations. |
| [Native Rust integration](rust-interop.md) | Mixed source files, Cargo dependencies, ownership, async and macros. |
| [CLI and Rust adapters](cli.md) | Executable commands, project manifests, explicit scalar Rust imports and bundled library modules. |
| [MVP assessment](mvp-assessment.md) | Verified implementation state, semantic gaps, reuse limits, and proposed delivery gates. |
| [Approved MVP decisions](mvp-decisions.md) | Accepted Q01–Q10 answers, required closures and weak references, and remaining decisions. |
| [MVP questions](mvp-questions.md) | Prioritized user decisions with recommended defaults and an answer template. |

## Reading the Status

- **Defined core:** a rule is specified for the documented subset, including the initial `Animal`/`Cat` example.
- **Partial:** architectural guarantees or initial rules exist, but the general formalization is incomplete.
- **Open:** no final language rule has been chosen; examples or possible lowerings must not be treated as a decision.

`MUST`, `MUST NOT`, `SHOULD`, and `MAY` have the meanings defined in SPEC.md. Requirements in these documents refine that specification within their stated scope. Open questions do not override existing guarantees.

The syntax examples describe the approved Clust language. The CLI checks examples against the current grammar and ownership rules. Semantic validation tables are requirements, not claims that the current pipeline implements every rule. See [implementation status](implementation-status.md).

Implementation guides: [compiler architecture](architecture.md) and [Rust interoperability plan](rust-interop-plan.md).
