# inheritance

Validate a single-base class hierarchy, instantiate inherited members and check overrides before constructing polymorphic values.

Module entry: [inheritance](../inheritance.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [signatures](signatures.rs) | `Bindings`, `module_constants`, `Keys` |
| [tests](tests.rs) | `module`, `clust_methods_infer_overrides_across_generic_base_chains`, `clust_overrides_preserve_receiver_signature_and_visibility`, `valid_hierarchies_substitute_generics_and_keep_override_chains`, `invalid_hierarchies_fail_before_layout_or_instantiation` |
| [validation](validation.rs) | `validate`, `node` |

Further reading: [signatures](signatures/README.md).
