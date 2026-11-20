# reference_memory

Provide the storage model shared by both reference interpreters. Class identity, captured projections, simulated allocations, raw storage and weak generations are tracked independently of native host addresses.

Module entry: [reference_memory](../reference_memory.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [addresses](addresses.rs) | `expose`, `recover`, `storage_view` |
| [allocations](allocations.rs) | `allocate`, `drop_range_arguments` |
| [bit_patterns](bit_patterns.rs) | `bits_bytes`, `bits_fields`, `bits_value` |
| [class_storage](class_storage.rs) | `allocate_class`, `class_reference`, `complete_class`, `copy_value` |
| [offsets](offsets.rs) | `projection_byte_offset` |
| [pointer_access](pointer_access.rs) | `bits_window`, `read`, `read_storage`, `initialize`, `take` |
| [pointer_math](pointer_math.rs) | `new`, `reinterpret`, `storage_stride`, `byte_position`, `raw_offset` |
| [references](references.rs) | `normalized`, `captured`, `read`, `parts`, `projected` |
| [storage_operations](storage_operations.rs) | `operation` |
| [weak_registry](weak_registry.rs) | `create`, `get`, `access`, `invalidate` |
