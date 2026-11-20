# Implementation Status

The workspace contains `clust-frontend`, `clust-semantics`, `clust-core`, `clust` and `clust-runtime`. It builds independently from this checkout. The CLI accepts the syntax described in [Clust syntax](syntax.md).

## Implemented and Exercised

| Area | Current behavior | Evidence |
| --- | --- | --- |
| Frontend | Source locations, ASCII identifiers, UTF-8 literals, comments, one parser, Rust-style mod/use declarations, dotted paths and module visibility | Frontend suites; module path, privacy and re-export integration tests |
| Constructors | Explicit always-public `new`, definite field initialization, branch-aware exactly-once base construction | Construction contracts and partial cleanup tests |
| Inheritance | Embedded bases, private lexical access, implicit public overrides, super calls, complete derived payloads | `class_bases`; `clust_animals` |
| Class polymorphism | Borrowed/mutable parameters, owning returns/fields/Vec elements, actual-class dispatch and identity | `class_bases`; `clust_weak_identity` |
| Interfaces | Static parameters and bounds, audited defaults, dynamic interfaces with concrete type arguments | Interface/default suites; `clust_dynamic` |
| Generics and enums | Monomorphized type/const generics, uniform generic inheritance, interface bounds, exhaustive match, Option and Result | Generic/enum suites; `clust_results` |
| Ownership | Inferred ownership and borrowed entries/results, liveness, exclusivity, storage contracts and checked escapes | Ownership, effect, view, storage and destructor suites |
| Closures | Function literals, noncapturing function values, read-only/owning captures, owned mutable capture, scoped mutable local capture | `clust_closures`; `clust_captures`; negative CLI tests |
| Weak references | Nonowning generation-checked identity, scoped read-only access, expiration before destruction, weak back-edges | `clust_weak`; `clust_weak_identity`; `clust_weak_backedge` |
| Library | Owned UTF-8 String, Vec growth/index/replacement, collection for loops, scalar output/assertions | `clust_string`; `clust_iteration`; native cleanup output |
| Cleanup | Scope-exit and replacement destruction, declaration-order complete cleanup, reverse actual initialization on failure | Native partial-construction and aggregate tests |
| Native tools | check/build/run/emit-rust, debug/release, artifacts, Clust.toml, Cargo path dependencies | CLI integration suite |
| Rust interop | Explicit safe/unsafe scalar adapters, ordinary Clust calls, caught Rust panic and cleanup | Native adapter integration tests |

HIR and MIR reference interpreters are separate test oracles. Native tests also check independently expected values and destructor output. Passing an interpreter comparison alone is not sufficient evidence for generated unsafe Rust.

## Supported MVP Boundaries

The synchronous compiler implements each required MVP feature category above. It does not implement the entire foundation specification or every combination of these features. The following restrictions remain explicit:

- Generic inheritance forwards every class parameter in order to the base, including renamed type and const parameters. Specialized/reordered base arguments are rejected with E0412.
- Dynamic interfaces require concrete interface arguments and nongeneric instance methods. Generic implementers can forward their class parameters to the interface. More general parameter mappings and generic virtual methods are not implemented.
- Closures with scoped mutable captures support local direct calls. Passing or storing those closures, general FnOnce extraction, and broader capture-type inference remain unimplemented. Borrowing and owning escaping closures have narrower checked support.
- Weak access uses the documented `Weak<T>.with` callback API. General weak upgrade, borrowed-field targets and the old `weak` field modifier are not implemented.
- `for` uses read-only `len`/`at` collection iteration. Mutable iteration, general iterator traits and range syntax are not implemented. Vec compound indexed assignment requires an explicit read followed by `set`.
- Explicit Rust adapters handle scalar boundaries only. Automatic arbitrary Rust API importing, concurrency and a stable external Clust inheritance ABI are deferred by the approved scope.

See [closure/weak semantics](semantics/closures-and-weak.md) and [runtime invariants](runtime-invariants.md). These restrictions describe conservative implementation boundaries; they do not prohibit later language extensions or silently relax ownership obligations. The earlier assessment is historical; its absence-of-code inventory is superseded by this document.

## Reproducible Checks

```sh
cargo test --workspace --offline
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
```

The minimum compiler version is Rust 1.88. Native tests require a local Rust compiler. `CLUST_RUSTC` selects it. The test suite includes debug and release execution; both use checked integer arithmetic by default. The recursive HIR oracle uses a larger test-thread stack for nested library calls. This is separate from native program stack requirements.

## Verified Snapshot — November 19, 2026

The complete workspace suite passed on aarch64 macOS with Rust 1.88: **273 tests passed, zero failures, one manual performance probe ignored by default**, including native debug/release execution, foreign Rust adapters, allocation accounting and failure-path cleanup.

```sh
RUSTUP_TOOLCHAIN=1.88 CARGO_INCREMENTAL=0 cargo +1.88 test --workspace --locked --offline --target-dir target/msrv
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
```

Formatting and Clippy passed without warnings on the installed default toolchain. All-target compilation was also checked with Rust 1.88. The suite includes 12 module-path checks and source/IR fixture regressions. All 13 storage checks pass, including native allocation accounting, growth, panic, double panic and simulated allocation failure. The generic inheritance and generic dynamic-interface examples were separately executed with the Rust 1.88 native backend in release mode. The closed-world class/interface conversion example returns 14 in both reference and native execution.

This is validation of the synchronous MVP and the boundaries above. It is not a claim of complete foundation-language coverage, a stable generated ABI, a formal soundness proof or performance parity with handwritten Rust.

## Async implementation update

The CLI also builds a single-file async subset with Tokio tasks, bounded channels,
actors and mutexes. This extends the original synchronous scope above. See
[async applications](cli.md#async-applications) for supported types and ownership
boundaries. Async support for classes and COLI across suspension remains open.
