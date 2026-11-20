# check

Start at check_clust. Source normalization resolves closures, adapters, inheritance and interfaces before the checker specializes declarations and infers contracts. The checker owns the specialization worklist and delegates ownership validation to MIR.

Module entry: [check](../check.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [access](access.rs) | `require_raw_unsafe`, `mutable_place` |
| [blocks](blocks.rs) | `block` |
| [branches](branches.rs) | `branch`, `known_type` |
| [class_bases](class_bases.rs) | Concrete inheritance lowering. A hidden, uniquely owned base component is destroyed after derived fields. Inherited bodies keep lexical privileges while their receiver remains the complete, most-derived concrete class. |
| [classes](classes.rs) | `prepare`, `new_class` |
| [classes_init](classes_init.rs) | Definite constructor field initialization on successful typed MIR paths. Borrow aliases carry provenance; taking the address of an uninitialized field for assignment is distinct from reading or exposing its contents. |
| [closures](closures.rs) | Anonymous functions lower to checked named functions or uniquely owned capture classes. No runtime reference counting or hidden cloning is added. |
| [construction_contracts](construction_contracts.rs) | Source-level base initialization obligations, checked before payload lowering. |
| [constructors](constructors.rs) | Definite field initialization before a constructor exposes its whole receiver. The AST check is retained for nominal-registry validation. Source compilation validates declarations here and uses typed MIR field analysis in classes_init. |
| [consume](consume.rs) | Consuming-parameter inference (NMM §11.1, §20.3).  A non-Copy ordinary parameter is a borrow unless the body passes the parameter root itself to an owning position: an aggregate or enum payload, a stored assignment, or a consuming parameter of a callee. Such a parameter gets an owned entry and callers must supply an owner token. Aliases of the parameter and `return param` are not consumption. |
| [declarations](declarations.rs) | `check_declaration`, `check_declaration_modes` |
| [default_methods](default_methods.rs) | Instantiate interface defaults before ordinary method collection. Explicit implementations win; unrelated defaults with the same name are ambiguous. |
| [divergence](divergence.rs) | `expression_diverges`, `block_breaks`, `expr_breaks`, `same`, `error` |
| [entries](entries.rs) | Demand-driven static owned/borrowed entry specialization. |
| [enums](enums.rs) | `enum_pattern`, `enum_variant`, `enum_unit` |
| [expressions](expressions.rs) | `expr`, `finish` |
| [function_values](function_values.rs) | `static_function_value`, `function_value` |
| [future](future.rs) | Conservative place-sensitive future reads before definite replacement. |
| [generics](generics.rs) | `Monomorphization`, `generic_call` |
| [inheritance](inheritance.rs) | Declaration hierarchy, independent of allocation and dispatch lowering. Inherited declarations retain the lexical owner of their private members. |
| [interface_parameters](interface_parameters.rs) | A plain interface parameter is static sugar for an independently inferred type parameter. It never introduces an erased value or shared ownership. |
| [interfaces](interfaces.rs) | Nominal interface obligations and abstract generic method surfaces. Concrete calls use the existing entry specializer; no interface value or vtable is made. |
| [intrinsics](intrinsics.rs) | `intrinsic` |
| [lifecycle](lifecycle.rs) | Destructor declarations become private borrowed entries with checked effects. |
| [literals](literals.rs) | `constant_value`, `literal`, `magnitude` |
| [methods](methods.rs) | `collect`, `static_call`, `resolved_static_call`, `parameter_modes` |
| [patterns](patterns.rs) | `match_expression` |
| [pipeline](pipeline.rs) | `check_pass` |
| [polymorphism](polymorphism.rs) | Closed-world class values retain a complete concrete payload. Runtime tag dispatch reuses verified enum borrowing and destruction, without slicing. |
| [privacy](privacy.rs) | Access is checked in the lexical declaring type, independent of entry modes, concrete generic arguments, receiver aliases, and the caller's source module. |
| [records](records.rs) | `Argument`, `Nominals`, `register_storage`, `interface_claims`, `field_declaration` |
| [rust_adapters](rust_adapters.rs) | Explicit scalar Rust adapters. No Clust owner or borrow crosses this ABI. Adapter effects and panics remain calls; reference execution refuses them. |
| [scope](scope.rs) | `local`, `lookup` |
| [storage](storage.rs) | Static readonly aggregate components. Descriptor storage is never ownership of its target, even if the enclosing aggregate is moved or copied. |
| [text](text.rs) | Checked UTF-8 byte primitives used by the owned String library. |
| [type_resolution](type_resolution.rs) | `resolve_type`, `resolve_type_in`, `resolve_type_with` |
| [type_walk](type_walk.rs) | Visit type occurrences without confusing declarations, expressions or binders. |
| [uses](uses.rs) | Conservative source-use hint for choosing pattern move/view lowering. |
| [views](views.rs) | `pending_result_place`, `view_binding`, `place`, `view_root` |
| [weak](weak.rs) | Weak identity is observable only through a scoped readonly callback. The callback receives no owner, mutable capability or escaping result view. |

Further reading: [class_bases](class_bases/README.md), [classes](classes/README.md), [classes_init](classes_init/README.md), [closures](closures/README.md), [construction_contracts](construction_contracts/README.md), [constructors](constructors/README.md), [consume](consume/README.md), [default_methods](default_methods/README.md), [entries](entries/README.md), [enums](enums/README.md), [expressions](expressions/README.md), [function_values](function_values/README.md), [future](future/README.md), [generics](generics/README.md), [inheritance](inheritance/README.md), [interface_parameters](interface_parameters/README.md), [interfaces](interfaces/README.md), [intrinsics](intrinsics/README.md), [lifecycle](lifecycle/README.md), [methods](methods/README.md), [patterns](patterns/README.md), [pipeline](pipeline/README.md), [polymorphism](polymorphism/README.md), [privacy](privacy/README.md), [records](records/README.md), [rust_adapters](rust_adapters/README.md), [storage](storage/README.md), [text](text/README.md), [type_walk](type_walk/README.md), [views](views/README.md), [weak](weak/README.md).
