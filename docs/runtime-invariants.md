# Native Runtime Invariants

The Rust backend emits a synchronous application with explicit ownership state and cleanup. Its internal unsafe Rust does not introduce unsafe blocks into Clust source. The compiler-crate `unsafe_code = forbid` lint does not validate generated code; native execution tests and the invariants below cover that separate boundary.

## Class Storage and Views

Each non-Copy class has one owning handle. A move transfers that handle without relocating the payload. Proven nonescaping payloads may live in a stable function frame; escaping payloads use heap storage. Views carry access to existing storage, not ownership or a reference count. Effect, borrow and escape checks run before emission. Raw pointers in generated code do not grant source-level access rights.

Initialization state and initialization sequence numbers are separate from payload bytes. Drop glue only touches initialized resources. Complete objects run their own destructor, fields in declaration order, then the embedded base. Incomplete objects skip their own destructor and clean initialized resources in reverse successful initialization order. A second panic during cleanup aborts.

## Inheritance and Dispatch

A derived object contains an owned base component as its final hidden field. Inherited bodies preserve their declaration's private access while using the complete concrete receiver. Public calls select actual-class methods; `super.method` selects the appropriate ancestor body. Constructors and destructors are not ordinary virtual methods.

An owning base-typed or dynamic-interface value uses a closed-world tagged payload retaining the full concrete object. Borrowed entry specialization avoids materializing this container when the actual class is known. Container destruction releases the concrete payload exactly once. Identity comparisons unwrap dispatch containers and compare actual object identity; a base conversion does not slice the payload.

## Weak Registry

The execution context registers an address, generation and concrete type only when weak identity is requested. It never owns a target. New generations use a checked monotonic counter; overflow aborts rather than reusing an identity. Drop glue marks the payload as finalizing and removes its registry entry before calling user code. A finalizing payload cannot register a live weak handle again.

`Weak.with` validates both address and generation before reading a target. A direct-class callback receives a read-only descriptor. A base/interface callback can require a temporary stack dispatch container holding a borrowed concrete pointer. That container is never subjected to owning drop glue. Its lifetime is exactly the synchronous callback, including unwinding. The checker forbids consuming callbacks, mutable callbacks and escaping resource results; any temporary identity registration is removed on both success and failure. Callback execution cannot run concurrently with destruction in the synchronous runtime.

Union payload accesses use explicit raw projections to the selected variant; generated code must not create an implicit Rust reference through `ManuallyDrop`. The variant tag and concrete type identifier determine every such projection.

## Validation

`class_bases`, `approved_runtime`, `borrowed_storage`, `borrowed_destructors`, constructor, partial-cleanup and ownership tests check interpreter agreement and rejection behavior. CLI tests execute approved examples at optimization levels 0 and 3 and compare exit codes and destructor output. The allocation-counting test checks that ordinary frame objects without weak handles require no heap allocations. Rust adapters are tested through actual Rust dependencies; interpreters reject foreign execution explicitly.

These checks are evidence, not a formal soundness proof. Specialized generic base mappings, higher-order scoped mutable closures and more general weak access need additional representation and effect contracts before acceptance.

## Owned buffer growth

Only the embedded virtual storage module receives the Rust lowering for its private slot instructions. The public buffer checks capacity and initialized length. Growth allocates a fresh destination, transfers the initialized prefix in order with the existing checked range-move primitive, and replaces the old allocation. Allocation failure occurs before the transfer. Raw storage releases allocation bytes, while the owning buffer destroys its initialized elements. This distinction preserves exactly one element owner across growth. Both interpreters and the native backend execute the same lowered operations.

## Compiler IR Fixtures

The `compiler-tests` feature exposes explicit Rust AST factories for backend-only inline records. Test inputs request these factories through `compiler_fixtures` fixture bindings. The factory builds `ItemKind::Aggregate { class: false, ... }` directly, with explicit field types, methods, destructor bodies and Copy metadata. Dynamically chosen test bodies use `testing::record::inline` and `parse_with_items`.

The production parser and loader keep their source semantics. A Clust class annotated `#[repr(C)]` retains its constructor and class layout in backend tests. A dedicated regression compares the production AST with the fixture-enabled parser for this case. Backend-only unsafe operation probes opt into `parse_ir` or `load_ir`. The ordinary fixture parser preserves function bodies and safety flags, including user functions with the same names as IR markers. Rust adapter tests exercise the production source boundary.
