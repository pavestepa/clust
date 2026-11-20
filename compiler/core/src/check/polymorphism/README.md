# polymorphism

Construct closed-world dispatch representations and rewrite conversions to base classes or dynamic interfaces. The representation keeps the complete concrete object and its identity.

Module entry: [polymorphism](../polymorphism.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [coercion](coercion.rs) | `is_polymorphic`, `class_subtype`, `coerce_class`, `name_of`, `const_name` |
| [dispatch](dispatch.rs) | `Interfaces`, `DispatchRoot`, `prepare` |
| [rewrite_expressions](rewrite_expressions.rs) | `rewrite_block`, `rewrite_expr` |
| [rewrite_items](rewrite_items.rs) | `rewrite_function`, `rewrite_item` |
| [rewrite_types](rewrite_types.rs) | `rewrite_type`, `rewrite_argument`, `scoped` |

Further reading: [dispatch](dispatch/README.md).
