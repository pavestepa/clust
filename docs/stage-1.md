# Stage 1: Formalizing the Semantic Core

**Status:** historical formalization checklist, retained for the broader foundation. The implemented synchronous subset and its tested boundaries are recorded in [implementation status](implementation-status.md). Completion of compiler MVP scenarios does not claim a formal soundness proof of the entire foundation.

The outcome required by foundation Section 23 in [SPEC.md](../SPEC.md) is a formal semantic specification suitable for implementation and validation. A parser, Cargo project, runtime, or working compiler belongs to later implementation work and is not claimed by these documents.

The [approved MVP decisions](mvp-decisions.md) settle Q01–Q10, including free functions with `fn`, methods without it, explicit constructors, scope-based destruction, and required closures and weak references. Older open-question lists below describe formalization work; accepted product choices must not be reopened as undecided.

## 1. Established Starting Point

The [canonical example](syntax.md#classes-constructors-and-methods) fixes the initial forms for classes, typed fields, a single base class, explicit constructors, field initialization, and base construction.

The accompanying documents specify:

- `new` is always-public constructor syntax sugar, with implicit initialization `self` and an implicit complete-instance result.
- `super.new(...)` initializes the direct base exactly once before derived fields in the documented explicit form.
- Required fields must be definitely initialized before successful completion; incomplete objects cannot escape.
- `Cat` and `Dog` are usable as `Animal` with actual-class dispatch, complete state, and complete ownership preserved.
- Ordinary read access, unique mutable access, ownership, and initialization access are distinct obligations.

The example uses `self.color = color;`, because `u8` is a type rather than the argument value. Assignment statements use semicolons. Both shown field forms, with or without a trailing comma, are recorded in the initial syntax document.

## 2. Coverage against the Stage 1 Requirements

| Requirement | Status | Current definition | Still required |
| --- | --- | --- | --- |
| Types and interfaces | Partial | [Nominal classes, fields, primitive context, visibility, and constructor contracts](semantics/types.md). | A complete core type grammar, typing judgments, interface rules, generic substitution, aliases, and explicit feature boundaries. |
| Single class inheritance | Partial | [Single-base hierarchy, subtype relation, and complete-object preservation](semantics/inheritance.md). | Formal member resolution, generic inherited contracts, cross-crate metadata requirements, and a complete compatible Rust model. |
| Static and dynamic polymorphism | Partial | Class dispatch differs from static interface generics and `dyn Interface`; base calls preserve the appropriate receiver. | Complete override and interface-resolution judgments, object-compatible dispatch boundaries, and owning versus borrowed class representations. |
| Construction | Defined core | [Explicit always-public constructors, base initialization, and initialization-state transitions](semantics/construction.md). | Defaults, overloads, fallibility, early exits, complete control-flow analysis, and exact partial cleanup behavior. |
| Ownership | Partial | [Destruction responsibility and operation proof obligations](semantics/ownership.md). | Rules deciding move, borrow, retain, and return dependencies across calls and all paths. |
| Unique mutable access | Partial | Mutability requires permission and exclusivity; virtual calls can touch the complete object. | Formal aliasing, field projections, reborrows, and effect footprints. |
| Lifetimes | Partial | Borrowed views cannot outlive complete owners or invalidate Rust constraints. | Lifetime relations, propagation, joins, escape rules, and a sound core constraint-solving model. |
| Destruction | Partial | Complete-object derived-to-base ordering and incomplete-construction cleanup obligations. | Destructor declarations or imported Drop contracts, exact failure-path ordering, panic boundaries, and ownership joins. |
| Basic placement | Partial | Prefer simple valid representations; avoid mandatory heap or reference counting for every instance. | Conditions for direct values, borrows, owning erased values, explicit heap, and semantically justified shared ownership. |
| Error rules | Partial | Each semantic document includes acceptance and rejection cases. | Precise error categories, source locations, unsupported-feature boundaries, and coverage of every formal rule. |

## 3. Remaining Decisions in Recommended Order

### 3.1. Finish the Synchronous Surface and Core Type Rules

Define ordinary functions, methods, return types, local bindings, `self`, `var self`, and `var` parameters. Complete the lexical rules, expression grammar, control flow, and the module visibility model needed to write meaningful ownership examples.

Specify primitive conversions and numeric behavior. Select the first formal subset of enums, aliases, generics, interfaces, and function types. Features outside that subset need explicit boundaries rather than accidental acceptance.

Deliverable: grammar and typing judgments with accepted and rejected examples for each core form.

### 3.2. Close Constructor and Initialization Questions

Decide whether missing constructors or missing base calls are synthesized, whether field defaults exist, whether multiple `new` declarations are allowed, and how fallible construction and early exits are written.

Define initialization joins and loops, constructor argument effects, exact partial cleanup order, and the permitted dispatch phases while constructing or destroying base and derived state. Keep always-public `new` and complete-object initialization as fixed requirements.

Deliverable: complete rules for successful and unsuccessful construction paths, extending the current initialization-state model.

### 3.3. Formalize COLI Ownership and Effects

Specify what information determines whether an unmarked `T` use becomes a borrow or ownership transfer. Define when a callee reads, mutates, consumes, retains, or returns a dependency on an argument, including separate compilation and Rust API boundaries.

Resolve destruction responsibility through aliases, fields, returned values, branches, loops, and early exits. Establish when the compiler must reject a program instead of changing representation. Automatic cloning and leaking must not become ownership repair strategies.

Deliverable: ownership and effect judgments with deterministic composition rules and explicit conservative failure cases.

### 3.4. Formalize Exclusive Access and Lifetime Constraints

Define overlapping places, reborrows, field-sensitive access, borrow validity, movement restrictions, and lifetime propagation. Work through both ordinary object access and calls whose actual derived receiver may touch more state than the base contract's field layout suggests.

Deliverable: a core constraint model that justifies the intended Rust borrow and lifetime representation, including positive and negative derivations.

### 3.5. Specify Destruction, Placement, and Class Representation

Choose a valid model for borrowed and owning class-polymorphic values. It must support `Animal` parameters, fields, returns, and heterogeneous collections while retaining actual-class dispatch, Send/Sync and Pin obligations, and the complete destructor chain.

Describe a safe Rust lowering for that core, its observable destruction behavior, and when direct, stack, inline, heap, or shared representations are permitted. Exact compiler module names and a stable generated ABI are not Stage 1 requirements.

Deliverable: explicit representation invariants and representative Rust lowerings that preserve source semantics, rather than a promise that Rust will infer the missing language rules.

### 3.6. Complete the Semantic Validation and Diagnostic Contract

For each formal rule, provide acceptance and rejection cases and explain the expected source-level error. Include field initialization, branches and loops, aliasing, escaping borrows, base and derived method calls, owned base-typed results, and cleanup paths.

Classify failures as semantic errors, permitted but not yet implemented behavior, cases requiring a Rust adapter, or proposed specification changes. Record implementation-independent expectations before implementing the Stage 2 compiler checks.

Deliverable: an internally consistent validation corpus and diagnostic requirements linked to the corresponding rules.

## 4. Completion Gate

Stage 1 can be marked complete only when:

- The synchronous semantic core and its boundaries have explicit grammar and typing rules.
- Constructor, subtype, dispatch, access, ownership, lifetime, destruction, and basic placement rules agree with each other.
- Each operation has a defined validity judgment and an observable meaning, including failure paths.
- Representative programs have worked semantic derivations and valid Rust representation witnesses.
- Every core acceptance condition has a corresponding rejection or boundary case where applicable.
- No unresolved question can silently change the meaning of a program admitted by the chosen core.
- SPEC.md, syntax, semantics, and validation examples use the same rules and visibility exceptions.

Stage 1 does not require implementing all later inference optimizations, Rust crate importing, async scheduling, channels, actors, or a complete IDE. Their architectural contracts still constrain the core; detailed implementation and advanced feature coverage follow the foundation's later stages.

## 5. Implementation Follow-through

The synchronous source forms, closure and weak-reference subset, and runtime invariants are now described in [syntax](syntax.md), [capture/weak semantics](semantics/closures-and-weak.md) and [runtime invariants](runtime-invariants.md). The compiler exercises these contracts through reference and native tests. [Implementation status](implementation-status.md) is the current delivery record. Q11–Q22 recommendations used during implementation remain working defaults, not additional explicit answers. Broader formalization and unsupported combinations remain follow-up work.
