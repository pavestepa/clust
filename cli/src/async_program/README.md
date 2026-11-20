# async_program

Parse and validate the supported async source subset, infer function parameter contracts and emit Rust for Tokio. Command execution handles temporary projects and Cargo artifacts.

Module entry: [async_program](../async_program.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [borrows](borrows.rs) | Infer borrow contracts, then let Rust track each loan inside the generated future. |
| [command](command.rs) | `BuildOptions`, `execute` |

Further reading: [borrows](borrows/README.md).
