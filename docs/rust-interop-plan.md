# Rust interoperability implementation plan

[Foundation specification, section 12](../SPEC.md#12-rust-interoperability) defines direct use, generated adaptation and an explicit native boundary. This plan extends the current scalar adapter without requiring third-party libraries to change. Each stage has an executable acceptance case.

| Stage | Compiler and build changes | Acceptance case |
| --- | --- | --- |
| 1. One application crate | Extend project resolution to register Rust modules beside Clust sources. Preserve module-relative paths, visibility, Cargo dependency renames, features, target configuration and the lockfile. Detect collisions when both languages define the same module. Use the existing Cargo build driver for mixed projects. | A project combines a nested local Rust module, a renamed path dependency, and two Clust files. Both debug and release builds resolve the same names. Private items remain inaccessible. |
| 2. Rust item discovery | Introduce an importer with versioned symbol identities and a cache keyed by the dependency graph, target, enabled features and Rust toolchain. Prototype a toolchain-pinned post-expansion metadata provider. Cargo metadata supplies the build graph, not item signatures. Keep the provider separate from the language model, and retain explicit declarations when metadata is unavailable. | Resolve a public re-export, a feature-gated function and a macro-generated type. Changing the feature or toolchain invalidates the affected metadata. |
| 3. Native type and ownership adapters | Add foreign type identity and call contracts to shared entities. Describe each argument and result as owned, shared, exclusive, or borrowed from specific inputs. Feed these facts into COLI and MIR. Emit direct Rust calls for identical representations. Use typed wrappers for different representations and opaque owning storage for Rust values with private layout. Rust performs the actual drop. | Pass an owned Rust value through Clust, mutate it in Rust and read a borrowed result. Reject use after move, invalidated views and borrowed results that outlive their owner. Check exactly one destructor invocation on return and unwind. |
| 4. Methods, generics, traits and async | Resolve inherent and trait methods, associated items and concrete generic substitutions. Let rustc enforce bounds, coherence, Send, Sync and Unpin. Cover standard String, Vec, Option and Result with representation-specific adapters. Extend the async path to retain borrow dependencies inside returned futures. | Use a crate method and a generic trait function directly from Clust. Test an awaited borrowed operation, reject a borrowed future escaping into spawn and check cancellation cleanup. Measure copies and allocations at representative boundaries. |
| 5. Attributes and macros | Preserve attribute paths, arguments and token groups in the frontend. Route Clust semantic attributes to their checker. Apply Rust derives and attributes to a source-facing generated item whose layout and drop behavior support that attribute. Run derive, attribute, declarative and procedural function-like macros through the Rust compiler. Import expanded items before exposing them to Clust. Define which macro positions accept Rust tokens and which can produce Clust-visible declarations. | A derive on an eligible data type generates a callable trait implementation. An attribute macro generates an imported method. A function-like macro works from a local module and a dependency. Reject incompatible Copy/drop combinations and report macro failures at the Clust invocation. |

The CLI now exercises all five stages in [mixed-crate tests](../cli/tests/commands/mixed.rs).
The [user guide](rust-interop.md) records the supported combinations. Remaining
work includes unified rich imports in the full Clust frontend, stable metadata,
source-mapped diagnostics and editor indexing. The table describes design goals
and acceptance criteria, including coverage beyond the current tests.

## Implementation and verification order

1. Resolve mixed module trees and Cargo dependency names at the normal CLI entry points.
2. Exercise owned handles, borrowed results and exclusive access with a destructor counter and compile-fail fixtures.
3. Test associated items, trait dispatch, generic methods and native standard-library containers.
4. Check both an awaited borrowed future and a task that illegally outlives its owner.
5. Test derives, attribute macros and function-like macros in the same Cargo compilation as their consumers.
