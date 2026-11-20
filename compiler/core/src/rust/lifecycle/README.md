# lifecycle

Emit Rust for resource initialization, drop state and slot cleanup. Only initialized pieces can run user destruction during normal execution or unwinding.

Module entry: [lifecycle](../lifecycle.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [resources](resources.rs) | `evaluate_resource` |
| [runtime_types](runtime_types.rs) | `lifecycle_types` |
| [slots](slots.rs) | `storage`, `end_storage`, `drop_local` |
