# function_values

Resolve direct and generic functions used as values. An indirect call checks both the callable signature and the access contracts of its arguments.

Module entry: [function_values](../function_values.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [generic_values](generic_values.rs) | `generic_function_value` |
| [indirect](indirect.rs) | `indirect_call`, `split_temporaries` |
| [pointers](pointers.rs) | `pointer_entry`, `pointer_from_call` |
