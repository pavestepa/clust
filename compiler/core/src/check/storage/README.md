# storage

Infer how aggregate components hold owners and views. Merge compatible storage constraints and feed requested modes back into specialization.

Module entry: [storage](../storage.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [arrays](arrays.rs) | `array_element`, `storage_array` |
| [assignment](assignment.rs) | `storage_assignment`, `class_field`, `storage_value` |
| [inference](inference.rs) | `promote`, `infers_storage_place`, `request_storage_mode`, `request_expression_mode` |
| [modes](modes.rs) | `same_mode`, `same_storage_mode`, `request_parameter_view` |
| [paths](paths.rs) | `key`, `union`, `place_paths` |
