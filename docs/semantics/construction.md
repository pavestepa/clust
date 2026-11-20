# Construction and Definite Initialization

**Status:** defined core for explicit, synchronous `new` constructors and `super.new`; general failure and control-flow details remain partial.

The [canonical example](../syntax.md#classes-constructors-and-methods) constructs `Animal` and `Cat`. This document defines the initialization rules behind that example and refines foundation Sections 5.6 and 5.8.6 in [SPEC.md](../../SPEC.md).

## 1. Constructor Contract

A declaration `new(parameters) { body }` inside class `C` is syntax sugar for an always-public associated construction operation whose successful result is a complete instance of `C`.

- `new` and `pub new` have the same visibility, bounded by the declaring class's accessibility.
- The source signature has no explicit receiver or return type.
- `self` is an implicit initialization place. It is not an ordinary already-initialized object supplied by the caller.
- Successful completion at the closing brace implicitly yields the constructed instance. No explicit `return self` is required.
- Constructor calls use `C.new(arguments)` and are statically selected.
- Constructors are not inherited and cannot override one another.

Ordinary named factories remain ordinary associated functions. The constructor syntax does not imply heap allocation, reference counting, a special Rust ABI, or a runtime constructor protocol.

The approved MVP requires one explicit constructor per class and excludes constructor overloading, automatic constructor generation, field defaults, early returns from constructors, and fallible constructor syntax. Named factories returning Result can validate inputs before infallible construction. Further creation forms are post-MVP questions. See [approved decisions](../mvp-decisions.md).

## 2. Initialization Capability

The implicit `self` permits initialization writes without a `var self` parameter. This capability exists only while constructing the new object; it does not turn an existing read-only reference into mutable access.

An uninitialized field MUST NOT be read. Every assignment expression must be type-correct and may read only values that are already valid.

Before the complete object is initialized, `self` MUST NOT escape into a field of another object, a closure, a returned user-visible reference, or an external call accepting the whole object. Direct writes to construction fields and reads of already initialized fields do not by themselves publish `self`.

An ordinary instance method cannot be invoked on an incompletely initialized receiver. Even when the current constructor's own fields are initialized, dispatch while a base constructor is active MUST NOT enter an override belonging to a not-yet-initialized derived class.

The compiler MUST NOT lower partial initialization by fabricating an initialized Rust value or an `&mut C` referring to uninitialized storage. Staged local values, builders, or another representation with proven invariants must be used instead.

## 3. Base Construction

Inside a derived constructor, `super.new(arguments)` initializes the direct base subobject of the same complete instance.

For the explicit form documented here:

1. The direct base constructor must be selected and its arguments type-checked.
2. The base initialization must complete exactly once on every successful constructor path.
3. The base must be initialized before any derived field is initialized.
4. A second base-construction call is an error, even if the calls have identical arguments.
5. Assigning inherited fields manually does not substitute for constructing the base.

Computing independent constructor arguments before `super.new` is permitted, provided it neither accesses uninitialized state nor publishes incomplete `self`. A root-class constructor has no `super.new` operation.

Within a direct call to `Animal.new()`, the completed result is an independent `Animal`. Within `Cat.new`, `super.new()` uses the base constructor to initialize `Cat`'s base portion; it does not introduce a second independent owner for that portion.

## 4. Initialization State

Let:

- `C` be the class whose constructor body is being checked.
- `F(C)` be the set of its own required instance fields.
- `I` be the subset of `F(C)` definitely initialized at the current program point.
- `B` be `not-required`, `uninitialized`, or `initialized`, describing the direct base.
- `Gamma` be the type environment for constructor arguments and other available values.

The initialization state is `K = (B, I)`.

| Operation | Preconditions | Successful successor state |
| --- | --- | --- |
| Enter a root constructor. | `C` has no direct base. | `(not-required, empty)` |
| Enter a derived constructor. | `C` has a direct base. | `(uninitialized, empty)` |
| `super.new(arguments)` | Direct base exists; `B` is `uninitialized`; `I` is empty; arguments satisfy its constructor contract. | `(initialized, empty)` |
| First assignment to own field `f`. | `B` is `initialized` or `not-required`; `f` belongs to `F(C)` but not to `I`; expression is valid and assignable to `f`; access is permitted. | `(B, I union {f})` |
| Read own field `f`. | `f` is in `I`; access is permitted. | Unchanged. |
| Read or mutate an inherited field. | `B` is `initialized`; ordinary field access and mutation rules hold. | Unchanged. |
| Successfully finish the constructor. | Base is `initialized` or `not-required`; `I` equals `F(C)`. | A complete instance is yielded. |

Reassigning an already initialized field is an ordinary replacement of that field's value. It must preserve typing, access, ownership, and destruction rules; it does not create another initialization obligation or permit dropping the old value incorrectly.

No implicit zero initialization is assumed. `Animal.age` is initialized because the constructor assigns `0`, not because an unsigned integer happens to admit zero.

### Checking the Canonical Example

| Constructor step | State afterward |
| --- | --- |
| Enter `Animal.new`. | `(not-required, empty)` |
| `self.age = 0;` | `(not-required, {age})` |
| Finish `Animal.new`. | A complete `Animal` or a completed base portion for its caller's construction. |
| Enter `Cat.new(color)`. | `(uninitialized, empty)` |
| `super.new();` | `(initialized, empty)` |
| `self.color = color;` | `(initialized, {color})` |
| Finish `Cat.new`. | A complete `Cat` with `age = 0` and `color` equal to the argument. |

## 5. Branches and Successful Paths

Every successful path must meet the completion condition; assigning a field on just one branch is insufficient.

When paths merge, the definitely initialized set is the intersection of their initialized sets. The base is definitely initialized only if every incoming path has initialized it. The analysis must also reject an operation that could initialize the base twice on any path.

This fixes the obligation but is not yet a complete control-flow algorithm. Loop fixed points, divergent paths, early returns, and general effects still require formalization alongside the missing statement syntax. The compiler must not accept a program merely because one execution path happens to initialize everything.

## 6. Failure and Cleanup

On a construction failure path that performs cleanup:

- Each successfully initialized owned value must be released exactly once.
- An uninitialized field must not be read or destroyed.
- A destructor for a complete derived instance must not be run on an incomplete instance.
- A completed base subobject must be cleaned up when its owning construction is abandoned.

For a successfully constructed object, normal destruction follows the foundation's derived-to-base order: the current class's destructor, its declared fields in declaration order, then the direct base recursively.

Partial construction cleanup runs in reverse successful initialization order under the approved MVP. Panic modes, failure propagation details, cancellation outside the synchronous MVP, and user-defined destructor syntax still require specification. The absence of those details must not weaken the initialization and exactly-once cleanup requirements above.

## 7. Illustrative Rust Lowering

For these concrete, non-polymorphic construction calls, the initialization may be expressed using safe Rust values:

```rust
struct AnimalData {
    pub age: u32,
}

impl AnimalData {
    pub fn new() -> Self {
        let age: u32 = 0;
        Self { age }
    }
}

struct CatData {
    pub color: u8,
    base: AnimalData,
}

impl CatData {
    pub fn new(color: u8) -> Self {
        let base = AnimalData::new();
        Self { color, base }
    }
}
```

The base value is initialized first. Its embedded field is placed after the derived fields to preserve the required normal destruction order. The source classes remain private here while their constructors are public within type accessibility.

This is an illustration of valid initialization, not a mandated code generator design or a complete class-polymorphism implementation. Class-typed references and owning polymorphic values additionally need the dispatch and ownership representation specified in [inheritance semantics](inheritance.md).

## 8. Required Validation Cases

| Case | Expected result |
| --- | --- |
| Canonical `Animal` and `Cat` declarations. | Accept; `Cat.new(7)` completes with `age = 0` and `color = 7`. |
| `new` without `pub`. | Same public constructor contract as `pub new`. |
| Root constructor omits initialization of `age`. | Reject successful completion with an uninitialized field. |
| Derived constructor initializes `color` but omits `super.new()`. | Reject missing base initialization. |
| Derived constructor initializes `color` before `super.new()`. | Reject derived initialization before the base. |
| Derived constructor calls `super.new()` twice. | Reject repeated base initialization. |
| Root constructor calls `super.new()`. | Reject the absence of a direct base. |
| Constructor reads `self.color` before assigning it. | Reject the uninitialized read. |
| Constructor publishes incomplete `self`. | Reject escape of an incomplete object. |
| Only one successful branch initializes a required field. | Reject missing definite initialization. |
| Base construction fails. | Do not execute derived initialization; clean up only valid initialized state on cleanup paths. |
| Construction fails after some fields are initialized. | Clean up those fields without invoking a complete-object destructor on invalid state. |
