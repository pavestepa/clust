# interfaces

Collect nominal interface declarations and implementations. Check signatures, bounds, default bodies and marker capabilities before constructing callable interface entries.

Module entry: [interfaces](../interfaces.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [claims](claims.rs) | `claims`, `surface` |
| [declarations](declarations.rs) | `contains`, `validate_declarations` |
| [implementation](implementation.rs) | `validate_implementation` |
| [instances](instances.rs) | `instance`, `require` |
| [markers](markers.rs) | `prepare_markers`, `marker_methods`, `apply_generics` |
| [signatures](signatures.rs) | `signature`, `symbolic_constants`, `expression`, `ty` |

Further reading: [signatures](signatures/README.md).
