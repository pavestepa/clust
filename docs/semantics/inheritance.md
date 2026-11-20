# Inheritance and Class Polymorphism

**Status:** defined class-substitution and dispatch contracts; partial formalization of generic and Rust-adapted cases.

This document refines foundation Section 5.8 in [SPEC.md](../../SPEC.md). It complements [types](types.md), [construction](construction.md), and [ownership](ownership.md).

## 1. Single Class Inheritance

A class may have at most one direct base class and may implement multiple interfaces. Multilevel inheritance is permitted; cycles and multiple class bases are errors.

Each instance contains one direct base subobject recursively. Inherited fields keep their original storage, and access remains subject to visibility. There is no implicit root class and no automatic access to private base members.

Constructors are associated with their declaring class. `Cat` does not inherit `Animal.new`; its own `new` uses `super.new(...)` to initialize the base portion.

## 2. Nominal Subtyping

Write `D <: B` to mean that `D` is usable where class `B` is expected. For class types, the core relation is the reflexive, transitive closure of the declared base relation:

1. `C <: C`.
2. If `base(D) = B`, then `D <: B`.
3. If `D <: B` and `B <: A`, then `D <: A`.

Sharing fields or compatible method names does not establish class subtyping. An unrelated class remains unrelated unless an independent interface contract applies.

If `Dog <: Animal` and `Cat <: Animal`, values of both subclasses are accepted in arguments, assignments, returns, fields, and collection elements declared as `Animal`, subject to the same access and COLI requirements as other values.

An `Animal` return may choose between `Dog` and `Cat` at runtime. A `Vec<Animal>` may contain both simultaneously. These uses do not require `dyn Animal`, an explicit cast, or an interface introduced by the programmer.

## 3. Static Contract and Actual Class

The static type determines accessible fields, callable signatures, visibility, and argument checks. The actual class determines the most-derived applicable implementation of an overridable instance method.

Non-private instance methods are overridable by default. A compatible inherited method is overridden by a derived declaration with the same name; `override` and `virtual` keywords are unnecessary. Receiver access, parameter and result types, constraints, and required visibility must remain compatible under the foundation's invariant override contract.

Private methods, associated functions, and constructors do not participate in virtual instance overriding.

For an actual `Cat` used through an `Animal` contract:

| Operation | Meaning |
| --- | --- |
| Read inherited `age`. | Read the original `Animal.age` storage. |
| Call an overridden public instance method. | Invoke the applicable `Cat` implementation. |
| Run an inherited method that calls another overridable method on `self`. | Preserve the complete receiver and select the applicable `Cat` override. |
| Explicitly call `super.method(...)`. | Select the base method body while preserving the complete receiver for nested overridable calls. |
| Call `super.new(...)` inside construction. | Initialize the direct base; no virtual constructor dispatch. |

During base construction or destruction, dispatch cannot enter derived state that is not yet initialized or has already been destroyed. An optimizer may replace a virtual call with a direct call only when the selected implementation and observable behavior are preserved.

## 4. Object Preservation

Converting a `Cat` to `Animal` preserves:

- The complete object's logical identity.
- Its base and derived state.
- Its actual-class method and interface implementations.
- Its lifetime and ownership obligations.
- Its complete destruction chain.

Copying only the `Animal` fields is object slicing and is not an implementation of the subtype conversion.

A borrowed base-class view must not outlive the complete owner. An owning `Animal` value holding a `Cat` owns the complete `Cat`; releasing it must perform the complete derived-to-base destruction sequence.

Converting through an `Animal` contract to an inherited `dyn Interface` must retain the actual class's interface implementation rather than switch to the implementation of an independent `Animal` object.

## 5. Mutable Views and Containers

Mutable access through a base-class contract requires exclusivity for the complete object because a dispatched method may mutate derived fields.

A mutable `Animal` view of a `Cat` does not authorize replacing a storage slot declared as `Cat` with a `Dog`. A slot declared as `Animal` may hold either subclass when ordinary assignment, mutability, and ownership rules permit it.

Class subtyping does not automatically lift through mutable generic containers. `Vec<Cat>` is not implicitly converted to `Vec<Animal>`: that would allow an unrelated subclass to be inserted into storage that promises `Cat` elements.

Similarly, no subtype conversion may erase Send/Sync, lifetime, or Pin requirements. A base-class view does not expose derived-only members or authorize an unchecked downcast.

## 6. Rust Representation Obligations

The compiler may represent base storage through embedded Rust fields and class contracts through generated Rust traits, dispatch adapters, borrowed trait objects, or owning erased representations.

A raw reference to only the embedded base field is insufficient for class-polymorphic use. The representation must preserve complete-receiver dispatch and, when necessary, complete-object ownership. Neither a fixed base offset nor pointer reinterpretation defines the language relationship.

An ordinary known concrete instance may remain inline. A borrowed polymorphic view may avoid allocation. An owning polymorphic value may require allocation or another representation that can hold different actual subclasses. The language does not promise that every such use has zero overhead.

The synchronous backend uses closed-world tagged payloads and borrowed entry specialization, described in [runtime invariants](../runtime-invariants.md). A stable public Rust representation and generic virtual methods remain outside the implemented boundary.

## 7. Required Validation Cases

| Case | Expected result |
| --- | --- |
| Pass `Dog` or `Cat` to an `Animal` parameter. | Accept with complete-object preservation. |
| Return either subclass as `Animal` on different paths. | Accept with ownership and actual-class dispatch preserved. |
| Store both subclasses in `Vec<Animal>`. | Accept without slicing. |
| Call an overridden method through `Animal`. | Select the actual class's override. |
| An inherited method invokes an overridable method on `self`. | Preserve actual-class dispatch. |
| Call `super.new()` during construction. | Select the direct base constructor exactly once. |
| Destroy an owning `Animal` containing a `Cat`. | Destroy the complete `Cat`, including its base. |
| Convert `Vec<Cat>` implicitly into `Vec<Animal>`. | Reject unsupported mutable container covariance. |
| Replace a `Cat`-typed caller slot with `Dog` through a base view. | Reject violation of the concrete slot type. |
| Declare a base cycle or multiple class bases. | Reject the hierarchy. |
