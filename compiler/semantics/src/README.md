# semantics

Provide integer semantics, layout calculations and storage constraint operations shared across compiler stages.

| Source | Responsibilities or entry points |
| --- | --- |
| [integer](integer.rs) | `IntegerType`, `bits`, `is_signed`, `mask`, `signed_min` |
| [layout](layout.rs) | Initial compiler object layout for the first 64-bit target.  This is not a C ABI argument classifier. Fields stay in declaration order, enums use an explicit u32 tag, and no niches or packing are introduced. |
| [lib](lib.rs) | Reusable arithmetic and layout utilities for the compiler. These utilities do not select Clust's pending overflow or layout policies. |
| [storage](storage.rs) | Constraint solving for the approved static storage specializations. These modes are compiler data; they are not per-element runtime tags. |

Further reading: [integer](integer/README.md), [layout](layout/README.md), [storage](storage/README.md).
