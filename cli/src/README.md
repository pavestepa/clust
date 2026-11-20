# cli

Resolve the command, source or manifest, select the appropriate compiler path and build or execute the resulting program. Native process management is separate from compiler semantics.

| Source | Responsibilities or entry points |
| --- | --- |
| [async_program](async_program.rs) | Safe Rust lowering for scalar async applications and owned runtime handles. |
| [main](main.rs) | `main`, `render`, `run`, `RunDirectory`, `drop` |
| [interop](interop/README.md) | Resolve mixed modules, consume expanded Rust signatures and adapt native calls. |
| [sync_command](sync_command.rs), [mixed_command](mixed_command.rs) | Execute synchronous Clust and mixed Cargo commands. |
| [native](native.rs) | `execute_native`, `execute_binary`, `build` |
| [project](project.rs) | Minimal application manifests and Cargo-backed native builds. |

Further reading: [async_program](async_program/README.md), [native](native/README.md), [project](project/README.md).
