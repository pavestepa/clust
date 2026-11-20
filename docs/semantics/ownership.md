# Ownership, Access, and Lifetimes

**Status:** partial COLI semantic model. This document defines obligations and an initial vocabulary, not a complete inference algorithm.

The architectural guarantees come from foundation Section 9 in [SPEC.md](../../SPEC.md). [Construction](construction.md) and [class polymorphism](inheritance.md) add requirements that COLI must preserve.

## Approved MVP Boundary

The [approved decisions](../mvp-decisions.md) select one inferred owner with safe aliases, conservative rejection of unprovable ownership, and no strong ownership cycles or independently shared owning graphs. Closures and weak references remain required: weak references do not own their targets, and closure capture must preserve the same ownership and access rules.

Mutable parameters permit object mutation without replacement of the caller's binding. Owners are destroyed at ownership-scope exit, with local resources cleaned up in reverse initialization order; replacement cleans up the old value, and partial construction unwinds successfully initialized values in reverse order. Last use may end a borrow without triggering owner destruction. Complete-object field/base order remains as defined in SPEC.md.

The rules below still need an inference algorithm and formal judgments; these accepted observable choices are no longer open product questions.

## 1. Values, Places, and Responsibility

A logical value is distinct from its current storage representation. A place identifies storage or an access path, such as an own field, an inherited field, or an element of a container.

For every owned resource, COLI must determine responsibility for destruction along every permitted execution path. Source variable names do not by themselves establish separate copies or owners.

For non-Copy types, ordinary use is reference-like. The compiler may realize that use as a Rust owned value or a borrow when the complete lifetime and ownership constraints permit it. It must not repair an invalid program by implicitly invoking arbitrary Clone, leaking the owner, or introducing unsafe aliasing.

The general rules for transferring destruction responsibility between aliases, fields, calls, and escaping results are still open. The phrase "last owner" is an obligation to identify a valid owner, not an algorithm based on the last textual variable use.

## 2. Access Capabilities

The following notation describes analysis state, not source-language types:

| Capability | Meaning |
| --- | --- |
| Ownership responsibility | The resource must eventually be released by this ownership path. It does not automatically grant mutation. |
| Read access | The holder may inspect valid initialized data. Compatible read accesses may coexist. |
| Unique mutable access | The holder may perform permitted mutations; conflicting accesses outside its valid reborrow chain must not coexist. |
| Initialization access | The constructor may initialize its own not-yet-complete object under the separate construction rules. |

Ordinary source `T` provides readable access. `var` requests mutable access under uniqueness requirements; on a method receiver or parameter it does not itself request ownership transfer.

The compiler must prove both permission and non-conflict. Proving that a read-only reference happens to be unique does not authorize converting it into mutable access by assignment or rebinding.

## 3. Core Operation Obligations

| Operation | Required proof |
| --- | --- |
| Read a place. | The value is live and initialized, the access is permitted, and it does not conflict with an exclusive access. |
| Initialize a field. | The constructor's initialization state permits the write and the value has an assignable type. |
| Mutate or replace an initialized value. | Mutable access is permitted and exclusive, and any replaced resource retains correct destruction behavior. |
| Pass a value to a call. | The callee's read, write, consume, retain, capture, and result-dependency effects are satisfied. |
| Transfer ownership responsibility. | The new ownership path is valid and all remaining uses and borrows remain safe under the selected representation. |
| Return or retain a borrow. | Its complete owner and required storage remain valid for every use of the borrow. |
| End ownership. | No valid use requires the resource to remain live; the complete concrete value is released according to its destruction contract. |

These are proof obligations. A complete Stage 1 model still needs inference judgments and rules for solving them across control flow and function boundaries.

## 4. Aliasing and Places

Read access may be shared. A unique mutable access excludes conflicting reads and writes to overlapping data, except for valid accesses performed through that capability and its reborrows.

The compiler must reason about places rather than only names. Two names may denote the same logical object; fields can overlap through a whole-object access even when their source names differ.

A concrete projection to a known field may be disjoint from another field when the compiler can prove it. A polymorphic method call through `var Animal` may access the complete `Cat` or `Dog`, so its footprint is not restricted to the base fields.

The complete rules for reborrowing, field-sensitive access, indices, interprocedural aliasing, and conservative fallback diagnostics remain open.

## 5. Lifetime Relationships

A borrowed view must remain valid for its entire use. In the notation of the lifetime model, a necessary condition is:

```text
lifetime(borrowed view) <= lifetime(complete owner)
```

This is necessary but not sufficient: mutation, movement, replacement, destruction, or address changes must not invalidate the view either. The compiler must preserve any relevant Rust lifetime and Pin constraints.

An owning `Animal` value containing a `Cat` retains the complete `Cat`. A borrowed `Animal` view must not outlive that complete owner. A copied or erased base representation that forgets the derived owner is not a valid solution.

Stack escape does not automatically imply heap allocation. Returning an owned value may transfer it directly; retaining a borrow requires a representation and lifetime relationship that remain valid.

## 6. Control Flow and Effects

COLI must account for all permitted paths, including branches, loops, early exits, returned borrows, and values stored in fields or captured by closures.

The preliminary effect vocabulary is:

```text
Read(place)
Write(place)
Consume(value)
Retain(value)
Capture(value)
ReturnDependency(result, input)
```

These labels do not define an effect inference algorithm or a stable metadata format. Stage 1 must formalize their composition, substitution at calls, control-flow joins, and treatment of unavailable bodies.

A function's source signature alone must not be assumed to prove all of its ownership effects. Rust imports and separate compilation require trustworthy contracts. If a required ordinary ownership or lifetime proof is unavailable, the compiler must report an unsupported or unsafe operation rather than silently clone or leak data. Imported unsafe Rust functions follow the external-contract rule in SPEC.md Section 12.7: their unverified foreign safety preconditions remain the caller's responsibility. This exception does not authorize bypassing Clust type, access, or lifetime checks.

## 7. Destruction and Basic Placement

Every initialized resource must have correct exactly-once cleanup on paths where destructors run. Complete derived values follow the foundation's derived-to-base destruction order; incomplete construction follows its distinct cleanup obligations.

COLI must select the simplest valid representation: a direct value, inline field, borrow, owning container, or shared owner where semantically justified. It must prefer safe and efficient stack or inline storage when available, without promising stack placement for every local variable.

Class polymorphism may require borrowed or owning dispatch representations. It does not justify using reference counting for every class, and it does not permit slicing a subclass to fit a base layout.

The exact placement decision procedure, shared ownership boundaries, weak-reference identity, partial cleanup order, and interactions with panic and cancellation remain to be formalized. More advanced analysis and optimization can be implemented in later stages after the core contracts are fixed.

## 8. Required Validation Cases

| Case | Expected result |
| --- | --- |
| Multiple compatible read accesses. | Accept while their complete owner remains valid. |
| Conflicting independent accesses during unique mutable access. | Reject the conflict. |
| Attempt to obtain mutation only by rebinding a read-only reference. | Reject missing mutable access permission. |
| Return a borrow of a value whose owner is destroyed on return. | Reject the invalid lifetime. |
| Store a `Cat` in an owning `Animal` result. | Preserve complete ownership, class dispatch, and derived destruction. |
| Use a consumed non-Copy value without a remaining valid ownership path. | Reject; do not insert Clone automatically. |
| Retain a polymorphic view after destroying its complete owner. | Reject the invalid lifetime. |
| Treat a mutable virtual call as accessing only base fields. | Reject any resulting unsafe aliasing proof. |
| Move a value with address-sensitive obligations. | Require a representation and operation valid under its Pin contract. |
| Erase a non-Send actual object behind a class contract and transfer it between threads. | Reject unless the full concrete transfer contract is valid. |
