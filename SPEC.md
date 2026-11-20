# CLUST — Foundation Specification

**Status:** foundational architecture specification  
**Version:** Foundation 1.0  
**Date:** November 18, 2026  
**Compiler implementation language:** Rust  
**Compilation target language:** Rust  
**Source file extension:** `.cl`

---

# 1. Purpose of This Document

This document defines the fundamental principles for designing and implementing the **Clust** programming language.

It is the primary source of architectural requirements for developers of the compiler, libraries, and tools, as well as AI agents participating in the project.

The document describes:

- The language's philosophy and purpose.
- Mandatory semantic guarantees.
- Type system principles.
- The memory management model.
- The object model.
- Rust compatibility rules.
- The compiler architecture.
- The asynchronous execution and multithreading model.
- Safety and performance requirements.
- Project development principles.

This document **is not a complete language grammar** and must not constrain future decisions about secondary syntax details.

Standard library function names, expression style, syntactic sugar, formatting, and source file organization may change without revising this specification, provided the semantics described here are preserved.

If the implementation conflicts with this specification, the specification or an explicitly approved amendment to its provisions takes precedence.

## 1.1. Requirement Levels

The following terms denote requirements:

- **MUST** — a fundamental requirement.
- **MUST NOT** — prohibited behavior.
- **SHOULD** — a preferred architectural decision.
- **MAY** — an option that is not mandatory for every compiler version.

Experimental decisions do not become part of the standard without separate approval.

## 1.2. Detailed Language Documents

The [language documentation](docs/README.md) develops the semantic core required by Stage 1. The [syntax document](docs/syntax.md) records the current source forms, while documents under [docs/semantics](docs/semantics/) define their meaning and identify remaining questions.

The [Stage 1 checklist](docs/stage-1.md) tracks which requirements have been formalized. An open question or an illustrative lowering in those documents is not a completed language guarantee.

---

# 2. Clust Philosophy

## 2.1. Primary Goal

Clust is a high-level, statically typed programming language that uses Rust as its primary and mandatory compilation platform.

Its goal is to give developers a convenient programming model similar to modern object-oriented languages while preserving Rust's memory safety and systems programming performance.

The central statement:

**Clust lets developers write application code without manually managing moves, borrows, and lifetimes, and compiles that code into efficient Rust.**

Clust must minimize the amount of information developers are required to provide about a program's low-level behavior.

The absence of manual memory management does not mean abandoning strict memory guarantees.

Instead, responsibility for proving these guarantees shifts to the compiler.

## 2.2. Key Principles

### I. Rust Is the Foundation of the Language

Clust does not compete with Rust at the level of machine code, ABI, system primitives, or standard platform libraries.

Clust uses Rust to obtain these capabilities.

### II. Simplicity of User Code

Developers must describe types, operations, data, and necessary constraints.

Developers must not be required to needlessly describe the mechanisms of their low-level implementation.

### III. Memory Safety by Default

Safe Clust code must not allow:

- Use-after-free.
- Double-free.
- Invalid memory accesses.
- Unauthorized simultaneous mutation.
- Data races in safe multithreaded code.
- Violations of object lifetime invariants.

### IV. Performance as a Fundamental Requirement

Language convenience must not automatically imply:

- Garbage collection.
- Heap allocation for every object.
- Pervasive reference counting.
- Unnecessary dynamic dispatch when the concrete call target can be proven.
- Automatic object cloning.
- Mandatory additional layers of indirection.

### V. Predictable Compilation

Developers must be able to inspect the generated Rust code and understand the compiler's decisions.

### VI. Rust Compatibility Takes Priority over Syntactic Similarity

Clust must support a semantically correct mapping of its types and operations to Rust.

Superficial syntactic similarity to Rust is not a goal in itself.

### VII. A Minimal Set of Language Mechanisms

Where possible, new features must build on the existing type system, methods, interfaces, and memory model.

Separate constructs should not be introduced when existing ones can already express the required semantics.

---

# 3. Architectural Dependency on Rust

## 3.1. Mandatory Compilation to Rust

The output of Clust compilation **must always be Rust code**.

Direct generation of LLVM IR, machine code, or a proprietary object format is not part of the core architecture.

The main pipeline:

```text
Clust Source
    ↓
Parsing
    ↓
Name Resolution
    ↓
Type Analysis
    ↓
Ownership & Lifetime Analysis
    ↓
Semantic IR
    ↓
Rust Code Generation
    ↓
Rust Compiler / Cargo
    ↓
Native Binary / Library
```

The Clust compiler is responsible for the correctness of the source program and its translation into Rust.

The Rust compiler is responsible for final validation of the generated program, optimization, and production of executable artifacts.

## 3.2. Relationship with Rust

Clust has a one-way dependency on Rust.

Required properties:

1. Clust can use its own modules.
2. Clust can use Rust modules.
3. Clust can use Rust crates.
4. Clust can use Clust crates.
5. Clust crates can depend on Rust crates.
6. Rust is not required to understand Clust source code.
7. Every Clust crate must be translatable into a Rust crate or a set of Rust modules.

Rust projects are not required to support `.cl` files.

However, Rust code generated by Clust may expose an ordinary Rust API if the developer intentionally exports such an API.

## 3.3. Unified Build System

Clust must use existing Rust and Cargo capabilities rather than implementing an independent build ecosystem where it is unnecessary.

Compatibility is recommended with:

- Cargo dependency resolution.
- Cargo.lock.
- Cargo features.
- Rust targets.
- Cargo workspaces.
- The build profile system.
- Build scripts.
- Rust linking.
- Existing Rust checking and testing tools.

A dedicated Clust CLI may manage source translation, diagnostics, and Cargo invocations.

The manifest format and specific command names are not fundamental parts of the specification.

## 3.4. Mixed Projects

A single project may contain both Clust and Rust.

Both languages must be able to contribute to the same final Rust crate.

The compiler must preserve the correctness of:

- Module boundaries.
- Visibility.
- Types.
- Generic constraints.
- Ownership.
- The Rust trait system.
- Rust coherence rules.
- Linking and dependency resolution.

---

# 4. Fundamental Type System

Clust is a statically typed language.

Every meaningful operation must have defined type semantics before the final Rust code is generated.

## 4.1. Compatibility with Rust Types

Clust's type system must be compatible with Rust's type system.

Clust is not required to reproduce all of Rust's user-facing syntax.

However, the compiler must have an internal representation for the types and constraints required for Rust interoperability.

This applies to:

- Primitive types.
- Compound types.
- Enumerations.
- Generic types.
- Interfaces and Rust traits.
- Associated types.
- Function types.
- Closures.
- Types with dynamic dispatch.
- Types with special lifetime requirements.
- Types with Send and Sync constraints.
- Types with unsafe internal Rust implementations.

No Clust type may require an independent representation system or ABI that is incompatible with Rust.

## 4.2. Core Type Constructs

The user-facing type model includes:

**class** — a concrete named data type supporting nominal subtyping through single inheritance.

**interface** — a behavioral contract.

**enum** — a type with multiple variants, including variants carrying data.

**type** — an alias for an existing type.

**Generic types** — parameterized types.

**Function types** — types of callable values.

**dyn Interface** — a dynamically polymorphic representation of an interface.

**Weak references** — non-owning references that require checking whether the object is still valid.

Other types needed for Rust compatibility may be represented directly or through adaptation.

## 4.3. No Separate struct Construct

Clust has no separate user-facing `struct` construct.

Its role is fulfilled by `class`.

This does not mean that class objects must be allocated on the heap.

The compiler may translate a Clust class into an ordinary Rust struct.

The word `class` itself defines a type category, not an allocation strategy.

---

# 5. Object Model

## 5.1. Class

A class is a concrete type that may contain:

- Fields.
- Methods.
- Associated functions.
- Constants.
- Static data, if the corresponding declaration form is supported.
- Generic parameters.
- A single base class.
- Interface implementations.

A class is not required to have a virtual table.

A class is not required to be a reference type in the sense of mandatory heap allocation.

A class is not required to have a special object header, runtime type metadata, or a reference counter.

The need for such mechanisms is determined by the specific semantics of its use. A polymorphic class value MUST preserve the complete object's runtime class and dispatch behavior, but this information need not be stored in a header on every concrete instance.

## 5.2. Interface

An interface defines a behavioral contract.

Clust interfaces contain only methods.

Interfaces contain no required fields, instance state of their own, or hidden data storage.

Every interface method is public.

Declaring methods public inside an interface does not require `pub`.

An interface method may:

- Have no implementation.
- Have a default implementation.
- Use other methods of the same contract.
- Have generic parameters, if the corresponding context permits them.

Methods without implementations must be implemented by the corresponding class.

Default methods may be overridden by the class implementation.

## 5.3. Interface Implementation

A class may implement multiple interfaces.

An implementation may be placed:

- Directly inside the class definition.
- In a separate implementation or extension block.

The location of an implementation must not change its semantic properties.

Implementing an interface method in a class requires an explicit `pub`.

This rule applies whether the implementation is inside the class or separate.

The compiler must check that signatures and interface requirements match.

## 5.4. No override or virtual

Clust does not use the keywords `override` and `virtual`.

Whether a method implements or replaces another method is determined by semantic analysis.

If a method's name and signature match a method in an implemented interface, it is considered an implementation of that method.

If the interface already provides a default implementation, the matching class method replaces it.

Matching inherited class methods are overridden according to Section 5.8.3. Calls through a base-class type preserve class polymorphism and select the implementation of the actual object without requiring `override`, `virtual`, or `dyn` in user code.

The compiler must diagnose ambiguity.

When multiple interfaces have conflicting requirements, the compiler must not arbitrarily select an implementation.

There must be an unambiguous way to resolve such a conflict.

The specific syntax for conflict resolution is defined by a separate specification.

## 5.5. Methods

Clust method declarations do not require the `fn` keyword.

The instance receiver is denoted by `self`.

A mutable receiver is denoted by `var self`.

These constructs specify instance access semantics rather than Rust reference syntax directly.

The compiler infers the corresponding Rust mechanism.

The constructor form `new(...)` is an exception to explicit receiver syntax: it has an implicit initialization receiver, as defined in Section 5.6.

## 5.6. Object Construction

`new(...) { ... }` is constructor syntax sugar for a public associated operation that creates and returns a complete instance of its declaring class.

A declared `new` constructor is always public, even when `pub` is omitted. Writing `pub new(...)` is permitted and has the same visibility. This does not make an otherwise inaccessible class public.

The constructor has no explicit receiver or result annotation. Its implicit `self` denotes the instance being initialized, and successful completion yields that instance. Field initialization does not require `var self` in the constructor signature.

Constructor calls use `ClassName.new(...)`. Constructors are not inherited and do not participate in class-polymorphic method overriding.

In a derived constructor, `super.new(...)` statically selects a direct base constructor and initializes the base subobject exactly once before derived fields are initialized. It does not create an unrelated base object or dispatch to a derived override.

Every required field MUST be initialized on every successful completion path. Reading uninitialized fields, exposing incomplete `self`, and treating an uninitialized value as a valid Rust reference are prohibited. The precise initialization-state rules are defined in [construction semantics](docs/semantics/construction.md).

This syntax does not require a special runtime constructor mechanism. It may lower to ordinary Rust functions and valid staged initialization. Other named factory methods remain ordinary associated operations. Additional creation forms, implicit constructors, constructor overloads, and fallible construction require separate definition.

Placement of the created object is determined by Clust's memory model.

Construction of a derived object must also satisfy the base initialization rules in Section 5.8.6.

## 5.7. Class Extensions

Methods and interface implementations may be separated from the class definition.

Conceptual form:

```text
extend ClassName
extend ClassName : InterfaceName
```

The exact grammar may change.

Extensions are not a mechanism for bypassing Rust restrictions.

Only implementation categories that can be correctly represented in Rust, taking crate boundaries, orphan rules, and coherence into account, are allowed.

A class declaration and its corresponding implementation blocks may reside in different modules if Rust permits the corresponding implementation.

An extension must not change a class's declared base class or add inherited instance storage after the class has been defined.

## 5.8. Inheritance and Composition

Clust MUST support single class inheritance for reuse of state and behavior and for nominal class polymorphism.

If `Dog` and `Cat` inherit from `Animal`, both MUST be usable wherever an `Animal` value is expected, subject to the same access, ownership, lifetime, and thread-safety requirements as any other value. No explicit cast, adapter, or `dyn Animal` annotation is required.

Interfaces provide static generic polymorphism and explicitly dynamic interface polymorphism. Class polymorphism is provided directly by inheritance. Composition remains available independently of both.

Class polymorphism MUST preserve the actual object's identity, complete state, method implementations, and destruction behavior. It MUST NOT imply heap allocation or reference counting for every class instance; the compiler may introduce the representation and indirection needed by a particular polymorphic use.

### 5.8.1. Declaration and Hierarchy

A class MAY declare one direct base class and any number of implemented interfaces.

Conceptual form:

```text
class Derived : Base, InterfaceA, InterfaceB
```

The compiler MUST distinguish class inheritance from interface implementation by resolving the named types. The exact declaration grammar may be refined separately.

The base MUST resolve to a Clust class. A type alias may name that class. An imported Rust type may be wrapped or composed into a Clust class, but importing it does not make it an inheritable Clust class.

Multiple class inheritance and inheritance cycles are prohibited. Multilevel single inheritance is permitted. There is no implicit universal root class.

A generic base MUST have a valid generic instantiation, and the derived class MUST satisfy its constraints. Inheritance alone introduces neither abstract classes nor additional visibility modifiers.

### 5.8.2. Inherited State and Member Access

Each derived instance contains exactly one direct base subobject, which recursively contains its own base subobject, if any. These subobjects are parts of the same complete object, not independently allocated or reference-counted owners.

Accessible base instance fields and methods are available on the derived instance. An inherited field always denotes its original storage in the corresponding base subobject.

Inheritance MUST preserve the visibility rules in Section 6. A derived class gains no special permission to access private base members. Access that is already permitted by the module privacy rules remains permitted.

A derived class MUST NOT hide an accessible inherited field with another field of the same name. Private base fields remain distinct base storage and are not exposed as inherited members to code without access.

Associated functions, factories, constants, and static declarations remain associated with their declaring class. They are not inherited as declarations rebound to the derived class. In particular, a base factory returning `Base` does not become a factory returning `Derived`.

### 5.8.3. Method Overriding and Class Dispatch

Non-private instance methods are overridable by default. A derived instance method with the same name and a compatible signature as an inherited method in this contract overrides that method. No `override` or `virtual` keyword is required.

The override MUST preserve the receiver access mode, parameter types, result type, generic constraints, and any other callable contract after base generic arguments are substituted. It MUST NOT reduce visibility or strengthen the caller's requirements. A same-name declaration with an incompatible signature MUST produce a diagnostic rather than silently introduce an overload or hide the inherited method.

Private base methods do not establish an overridable contract. They remain bound to their declaring class. Associated functions and static declarations do not participate in instance dispatch.

The static receiver type determines which members are accessible and which method contract is checked. The actual runtime class determines the most-derived applicable implementation of an overridable instance method. A value used through `Animal` MUST therefore call `Dog.speak` or `Cat.speak` when its actual class is respectively `Dog` or `Cat`.

An inherited method body retains its declaring class's field access and visibility rules while preserving the complete receiver's runtime class. A call to an overridable method on `self` from an inherited base body MUST select the most-derived implementation, subject to the initialization and destruction rules in Section 5.8.6.

The language MUST provide a way for a derived method to explicitly call the direct base implementation on the same object. Conceptual notation is `super.method(...)`; selection of that method body is statically bound and does not recurse into the override. Overridable calls on `self` inside that body still use the complete receiver's runtime class. Final surface syntax may be specified separately.

The compiler MAY devirtualize a call when its target can be proven. This optimization MUST preserve the same implementation selection and observable behavior as class-polymorphic dispatch.

### 5.8.4. Inherited Interfaces and Dynamic Dispatch

A derived class MUST satisfy the interfaces implemented by its base classes, in addition to interfaces it declares directly. Each interface contract MUST be revalidated for the complete derived type.

Compatible inherited methods may supply an implementation. A matching derived `pub` method supplies the derived implementation of that contract. Declaring an inherited interface again does not create a second implementation of the same interface for the same concrete type.

An inherited body that cannot satisfy a contract for the derived type, including a contract involving interface `Self` or an owned result, requires a valid explicit implementation. Otherwise, the compiler MUST reject the derived class. Inheritance MUST NOT introduce implicit result covariance, object slicing, or invalid Rust trait implementations.

Rust auto traits and eligibility for traits such as Copy and Clone MUST be determined from the complete derived type and the applicable Rust rules, not copied from the base class's implementations.

Converting an object to `dyn Interface`, directly or through a polymorphic base-class value, MUST preserve the actual object's interface implementation. Upcasting a `Dog` to `Animal` MUST NOT change its interface dispatch to the implementation of an independently created `Animal` instance.

For a compatible `describe` method declared by both classes and implementing `Describable`, dispatch follows these rules:

| Access path | Selected implementation |
| --- | --- |
| `derived.describe()` with static type `Derived` | The derived declaration. |
| `base_view.describe()` with static type `Base` and actual class `Derived` | The derived override. |
| An inherited base method calling `self.describe()` on a complete `Derived` | The derived override. |
| `super.describe()` inside the derived method | The direct base implementation, with the complete receiver preserved. |
| `dyn Describable` formed directly from the derived value | The derived interface implementation. |
| `dyn Describable` formed from a base-class view of the derived value | The derived interface implementation. |

Class dispatch and interface dispatch MUST agree on applicable derived overrides. Neither requires a virtual table stored inside every ordinary class object; dispatch metadata may reside in a polymorphic reference or owning representation.

### 5.8.5. Base Views, Ownership, and COLI

Inheritance defines a nominal subtype relationship: `Derived` is usable as `Base`, transitively through the ancestor chain. This applies to arguments, assignments, return values, fields, and collection elements declared with the base-class type, including contexts that retain ownership.

COLI MUST infer the appropriate borrowed or owning representation when a derived value is used as its base type. A readable base-class view retains the complete object's identity and dispatch capability. Mutable access additionally requires valid unique access to the complete object because a dispatched method may access derived state.

A base-class parameter is a polymorphic class contract, not an implicit generic parameter that requires a single statically known subtype. One `Animal` binding or collection may hold instances of different subclasses over its lifetime. Interface-based generic polymorphism continues to follow Section 7.

For initialized values `dog: Dog` and `cat: Cat`, where both classes inherit from `Animal` and override its public `speak` method, the following conceptual example is valid:

```text
listen(animal: Animal) -> String {
    return animal.speak()
}

listen(dog) // Calls Dog.speak.
listen(cat) // Calls Cat.speak.
```

A return type of `Animal` MAY return either a `Dog` or a `Cat` on different execution paths. An `Animal` field MAY retain either subclass, and a `Vec<Animal>` MAY contain both at the same time. The representation MUST retain each element's complete concrete object and class dispatch behavior.

Subtype conversion MUST NOT copy only base fields, detach the base subobject, implicitly clone a non-Copy value, or discard derived state. Borrowed representations MUST NOT outlive their complete owner. Owning representations MUST retain ownership of the complete concrete object and run its full destructor chain when ownership ends.

Passing a `Dog` through a mutable `Animal` view permits mutations supported by the `Animal` contract, but MUST NOT permit replacing a storage slot declared as `Dog` with a `Cat`. A slot declared as `Animal` may be assigned either subtype according to its ordinary mutability and ownership rules.

Class subtyping does not imply covariance of generic containers. In particular, `Vec<Dog>` MUST NOT implicitly become `Vec<Animal>` merely because individual `Dog` values are usable as `Animal`; inserting a `Cat` through such a conversion would violate the original container's element type.

Access to disjoint fields is permitted only when COLI can prove non-overlap for the actual operation. An overridable call through a mutable base-class view MUST NOT be treated as accessing only base fields. Address-sensitive values, Send/Sync constraints, and projections MUST preserve their applicable Rust requirements through subtype conversion.

Upcasting preserves dispatch information but does not grant access to members declared only on a derived class. Implicit downcasting is prohibited; a checked downcast facility, if provided separately, MUST verify the actual type.

### 5.8.6. Initialization and Destruction

Construction MUST initialize a valid base subobject before initializing the derived class's own fields, and MUST initialize all required fields before exposing a complete derived instance. In constructor syntax, `super.new(...)` performs base initialization under Section 5.6. A base constructor or factory is not an inherited derived constructor.

Failure during initialization MUST release successfully initialized resources according to Rust's cleanup semantics. Uninitialized state MUST NOT be exposed or destroyed as an initialized value.

When a complete derived object is destroyed, its own destructor, if any, runs first, followed by its declared fields in declaration order, followed by destruction of the direct base subobject under the same rules. Every initialized resource MUST be destroyed exactly once on cleanup paths that run destructors.

Dispatch during initialization and destruction MUST NOT target state that is not yet initialized or has already been destroyed. While a base subobject is being initialized or destroyed, class method calls are limited to that class and its live ancestors; they MUST NOT enter derived overrides. An ordinary borrowed base-class view does not acquire independent responsibility for destruction. An owning base-class value retains responsibility for the complete actual object.

### 5.8.7. Rust Representation and Compatibility

The normal lowering represents inherited state through embedded Rust fields. Class-polymorphic operations additionally require a representation that preserves the complete concrete receiver and its method dispatch. Generated Rust traits, dispatch adapters, borrowed trait objects, and owning erased representations MAY provide this behavior. Generic inheritance lowers to ordinary Rust generic types and constraints.

A bare reference to an embedded base-state field is insufficient to represent a polymorphic base-class value because it loses derived dispatch and complete-object ownership. Generated base method bodies and interface conversions MUST retain or recover the full receiver through a valid Rust contract.

The compiler MUST preserve class substitution in both borrowing and ownership-retaining contexts. A borrowed trait object may avoid allocation; an owning polymorphic value may require an owning container or another valid representation. These costs MUST NOT be hidden by slicing the object or changing its dispatch semantics.

Methods that cannot be dispatched directly through a Rust trait object require valid specialization or generated adaptation. An unsupported method form MUST produce a precise diagnostic rather than silently fall back to base-only dispatch. The detailed lowering strategy is separate from the mandatory class-subtyping semantics.

The generated field arrangement and drop glue MUST preserve the initialization and destruction order above. The compiler MUST NOT depend on a base subobject being at offset zero, on a stable Rust struct layout, or on pointer reinterpretation to establish the ancestor relationship.

Clust MUST resolve inherited members itself rather than delegate language semantics to implicit Rust Deref lookup. Any generated projection must be valid under Rust's ownership and access rules.

Inherited interfaces MUST map to valid implementations for the derived Rust type. The compiler may reuse an applicable Rust implementation or generate a valid implementation, but MUST NOT introduce overlapping implementations or bypass orphan rules, coherence, or visibility restrictions.

Inheritance across Clust crates is permitted when the base is accessible and the required operations can be expressed through the exported contract. Generated code MUST NOT access another crate's private fields to implement inheritance.

Classes without a base retain their existing semantics and representation requirements. Adding or changing a public base, inherited member, or interface implementation MUST be assessed for source compatibility, generic constraints, and ownership effects.

### 5.8.8. Required Validation

Inheritance support MUST include compile-pass, compile-fail, and equivalent-Rust behavioral tests covering:

- Single and multilevel inheritance, generic bases, and cross-crate access.
- Rejection of multiple class bases, cycles, visibility violations, field hiding, and incompatible method replacements.
- `Dog` and `Cat` substitutability for `Animal` in arguments, assignments, returns, fields, and a heterogeneous `Vec<Animal>`.
- Class-polymorphic dispatch, explicit base calls, inherited method bodies, interface overrides, and both forms of interface conversion in the table above.
- Borrowed and owning base-class values, overlapping mutable access, rejection of mutable container covariance, complete-object destruction, and partial initialization cleanup.
- Rust trait coherence, whole-derived-type constraints, preserved Send/Sync and Pin requirements, allocation-free borrowed representations, and devirtualization.

Benchmarks MUST compare equivalent inherited and composed Rust representations when evaluating inheritance overhead. Safety, COLI correctness, and observable behavior take priority over removing forwarding code.

---

# 6. Visibility and Modules

## 6.1. Private by Default

All Clust declarations are private by default, except interface methods and `new` constructors, which are implicitly public.

The sole primary public visibility modifier is `pub`.

This rule applies to:

- Classes.
- Fields.
- Methods.
- Functions.
- Types.
- Constants.
- Static declarations.
- Modules.
- Exported API items.

An interface as a type also follows the ordinary visibility rules.

Methods inside an interface are always public wherever the interface itself is accessible.

A `new` constructor is always public wherever its declaring class is accessible, whether or not its declaration includes `pub`.

Inheritance does not grant access to private base members or introduce a `protected` visibility level. Inherited member access follows the same module and declaration visibility rules as other access.

## 6.2. Compatibility with Rust

Clust must preserve the access restrictions of imported Rust types.

A private Rust API must not become public merely because it is imported into Clust.

Adopting Rust's module-based privacy model is recommended when it avoids unnecessary adapters.

The exact rules for accessing private class members must be defined with these restrictions in mind.

## 6.3. Module Declarations and Paths

Clust uses Rust-style module declarations and `use` bindings. All Clust namespace paths use `.` as their separator, including explicit Rust adapter paths.

```clust
mod animals;
use crate.animals.{Animal, Cat as Pet};
use core.vec.Vec;

mod helpers {
    pub(crate) fn count() -> usize { return 0; }
}
```

`mod name;` declares a child module using `name.cl` or `name/mod.cl`; declaring both file forms is an error. Inline modules use `mod name { ... }`. Child modules of a file module are resolved under the directory named for that module. Only declared source modules belong to the compilation unit.

`use` supports single names, module aliases, nested groups, glob bindings and `as` aliases. `pub use` re-exports declarations. `crate`, `self` and `super` identify the crate root, the current module and the parent module. Explicit names take precedence over glob bindings, and an ambiguous glob reference MUST be rejected. Qualified paths MUST work in type and expression positions and in variant patterns.

A private module-level declaration is accessible from its declaring module and descendants. `pub`, `pub(crate)`, `pub(super)`, `pub(self)` and `pub(in ancestor.path)` define the permitted visibility. A re-export MUST preserve the original declaration's visibility limits. Class-member privacy follows the class rules in this specification.

The synchronous MVP bundles the `core` library. Automatic discovery of arbitrary Rust APIs and separately distributed Clust crates remain subject to their documented implementation boundaries.

---

# 7. Generic System

## 7.1. Primary Goal

Clust's generic system must be substantially simpler for everyday use than Rust's user-facing generic system.

At the same time, the compiler's internal model must remain expressive enough to use Rust crates.

Rust's complexity must not automatically carry over into Clust's user-facing syntax.

## 7.2. Implicit Generic Parameters

An interface used as an ordinary static argument type represents a generic constraint.

For example, a function accepting the `Reader` interface is conceptually equivalent to a function accepting `T`, where `T` implements `Reader`.

For each independent use of an interface, the compiler must infer a corresponding generic parameter.

It must not automatically assume that two arguments of the same interface type have the same concrete type.

If a function requires several values to have the same concrete type, that relationship must be expressed explicitly.

Naming a base class is not an implicit generic constraint. A class-typed parameter accepts instances of that class and its descendants with class-polymorphic dispatch, as defined in Section 5.8.5.

## 7.3. Explicit Generic Parameters

Explicit generic parameters are used to express:

- Relationships between argument types.
- Relationships between input and output types.
- Parameterized classes.
- Parameterized interfaces.
- Multiple constraints on a single type.
- Const generics.
- Generic methods.

A parameter may be constrained by multiple interfaces.

Complex Rust constraints are not required to have a direct syntactic equivalent in Clust.

## 7.4. Generic Fields and Classes

A field declared using an interface without `dyn` has statically polymorphic semantics.

The compiler must translate this into parameterization of the containing type when necessary.

The public representation of such a type must be unambiguous.

Hidden generic parameters must not lead to an ambiguous public API.

A field declared with a class type, such as `Animal`, instead follows class-subtyping semantics: it can hold an `Animal`, `Dog`, or `Cat` without turning its containing type into a generic type for each concrete subclass.

## 7.5. Generic Results

A return value of an interface type without `dyn` does not mean an arbitrary runtime type.

The compiler must prove that a valid static representation of the result exists.

If a function can return different concrete types and they cannot be represented by a single static type, it must use an appropriate explicitly dynamic or algebraic type.

A class return type already provides the class-polymorphic contract defined in Section 5.8.5. A function returning `Animal` may return either `Dog` or `Cat` without an additional `dyn` marker or an enum.

## 7.6. Associated Types and Complex Constraints

Clust's internal type system must support representations of Rust's:

- Associated types.
- Generic associated types.
- Higher-ranked trait bounds.
- Lifetime constraints.
- Const generic constraints.
- Trait projections.
- Auto traits.
- Blanket implementations.

This does not mean that all of these constructs must exist in Clust syntax.

The preferred approach is adaptation into simple generic abstractions where it is semantically correct.

If translation is impossible without losing information or guarantees, a Rust adapter is used.

## 7.7. Monomorphization

Statically polymorphic operations must compile into Rust generics.

Final specialization and machine-level optimization are performed by Rust.

The Clust compiler must not force dynamic dispatch in place of supported static dispatch.

---

# 8. Dynamic Polymorphism

## 8.1. Explicit dyn for Interfaces

Clust supports the `dyn` keyword.

`dyn Interface` denotes dynamically polymorphic use of an interface.

The absence of `dyn` means static polymorphism when a type is specified through an interface.

This is a fundamental distinction.

This explicit `dyn` distinction applies to interfaces. A class type such as `Animal` already admits its subclasses and class-polymorphic dispatch without `dyn Animal`, as defined in Section 5.8. Runtime polymorphism over a class hierarchy does not require the programmer to introduce an interface.

## 8.2. dyn Representation

A dynamic interface must correspond to a valid Rust trait object representation.

The compiler may use:

- A borrowed representation.
- An owning container.
- Heap allocation of the object.
- Another representation that can be correctly adapted to Rust.

However, `dyn` itself must not automatically imply a particular allocation strategy.

The ownership mechanism is determined by COLI.

## 8.3. dyn Restrictions

Not every interface is required to support dynamic dispatch.

If an interface contains methods or constraints incompatible with Rust trait objects, the compiler must emit a diagnostic error.

The compiler must not silently change interface semantics to support `dyn`.

## 8.4. Cost of Dynamic Dispatch

The compiler must preserve the possibility of devirtualization when the concrete type is actually known.

However, the language does not guarantee a direct call for every `dyn`.

Using `dyn Interface` explicitly selects interface runtime polymorphism. Using a base class such as `Animal` admits class runtime polymorphism directly. Both may require indirect calls when the actual target cannot be proven statically; neither guarantees that every call incurs this cost.

---

# 9. COLI — Clust Ownership & Lifetime Inference

## 9.1. Purpose

**COLI is the compiler system for inferring ownership, borrowing, lifetimes, access permissions, and object placement.**

COLI is a fundamental component specific to Clust.

Its goal is to provide the safety of Rust's ownership model without requiring most of its low-level operations to be described manually in source code.

COLI is not a garbage collector.

COLI does not imply mandatory reference counting for every object.

COLI is a system for static analysis and program transformation.

## 9.2. General Principle

Clust source code describes logical values and the permitted ways of interacting with them.

The compiler determines:

- Value creation.
- Ownership transfer.
- Borrowing.
- Read access.
- Unique mutable access.
- Lifetimes.
- The last valid use.
- The destruction scope.
- Whether heap allocation is necessary.
- Whether an object may be moved.
- Whether shared ownership is necessary.
- Whether capturing a value in a closure is safe.
- Whether transferring a value between threads is safe.

The result must be expressed as correct Rust code.

## 9.3. No Explicit move or borrow

Developers must not be required to use Rust-like markers:

```text
&T
&mut T
move
' lifetime
```

in ordinary safe Clust code.

The compiler determines the corresponding Rust operations itself.

This does not prohibit low-level Rust constructs inside Rust modules.

## 9.4. Semantics of T

Ordinary access to a value of type `T` provides read access.

For non-Copy types, Clust's user-facing model is reference-like: using a variable must not automatically imply copying the entire object.

At the same time, the compiler may store the value as a fully owned Rust value.

The concept of a logical reference in Clust does not have to correspond to a physical pointer in machine memory.

Two accesses to the same object may refer to the same logical entity even if, after optimization, the object is represented by registers or local values.

## 9.5. Semantics of var

`var` provides mutable access with a uniqueness requirement.

For an argument or method receiver, `var` means that exclusive mutable access is required.

The compiler may represent such access through Rust's `&mut T`.

For an ordinary local variable, `var` also denotes permission to change the corresponding value or binding within the language's rules.

`var` must not mean automatic ownership transfer.

An ordinary immutable reference must not be converted into a mutable one merely through assignment or rebinding.

The compiler must prove that unique mutable access is actually permitted.

## 9.6. Aliasing

Multiple read accesses may coexist.

Mutable access must not coexist with conflicting accesses to the same data.

COLI must analyze not only variable names but also possible overlap between memory regions.

It must account for:

- Object fields.
- Base subobjects and polymorphic views of the same complete derived object, including derived state reachable through dispatched methods.
- Array elements.
- Nested values.
- Access through methods.
- Closures.
- Returned references.
- Shared containers.
- Object transfers between functions.

The compiler may split access to independent fields when it can prove the absence of conflicting aliasing.

## 9.7. Ownership Inference

Responsibility for releasing each resource must be defined.

COLI constructs an ownership model based on a value's usage and lifetime.

Conceptually, responsibility may be transferred to a subsequent consumer of the value.

The last valid owner is responsible for ending the resource's lifetime.

This principle does not imply mandatory destruction at the last textual mention of a variable.

The analysis must account for:

- Branches.
- Loops.
- Exceptional termination paths.
- Early returns.
- Passing values to functions.
- Storing values in fields.
- Returning values from functions.
- Closures.
- Asynchronous execution.
- Multiple potential ownership paths.

Responsibility for release must be determined for all valid execution paths.

## 9.8. Transfers between Functions

COLI must have an interprocedural model of ownership effects.

For a function call, it is necessary to determine which parameters are:

- Only read.
- Mutated.
- Consumed.
- Retained beyond the call.
- Returned as values related to the original data.

The compiler may internally use equivalents of Rust move, borrow, mutable borrow, clone, and drop.

However, user code is not required to specify these operations explicitly.

The compiler must not silently clone arbitrary non-Copy objects to repair invalid ownership.

## 9.9. Lifetimes

COLI must construct lifetime relationships between values.

A value's lifetime is determined by the possibility of safely using it, not solely by its lexical scope.

The compiler must support:

- Local scopes.
- Temporary values.
- References between objects.
- Borrowed arguments.
- Returned borrows.
- Values inside containers.
- Closures.
- Asynchronous functions.
- Cross-thread transfers.

If a lifetime cannot be proven safe, the compiler must emit an error or use a correct representation permitted by the program's semantics.

Lifetimes must not be extended arbitrarily through memory leaks.

## 9.10. Object Placement

COLI determines the physical placement of values itself.

The compiler must prefer the simplest and most efficient valid representation.

Possible outcomes include:

- Register representation.
- Stack allocation.
- Inline storage inside another object.
- Heap allocation.
- Representation through an owning container.
- Representation through a borrow.

If an object can be stored safely and efficiently on the stack, the compiler must prefer that placement.

However, stack placement is not a mandatory guarantee for all local variables.

The exact placement may be further changed by the Rust and LLVM optimizers.

## 9.11. Escape Analysis

COLI must analyze values escaping their original scope of use.

Particular attention is required for:

- Returned objects.
- Storage in long-lived structures.
- Closures.
- Asynchronous tasks.
- References inside containers.
- Dynamic polymorphism.
- Address-sensitive values.

Escape analysis must not assume that every escape from a local scope automatically requires heap allocation.

In some cases, Rust can safely pass the value itself onward without an additional allocation.

## 9.12. Explicit heap

Clust retains the `heap` keyword.

Its purpose is to let developers explicitly require the corresponding heap placement when it is necessary for semantic reasons or compatibility with a particular API.

`heap` must not mean automatic reference counting.

`heap` must not automatically make an object shareable between threads.

An object's heap placement is not the same as its ownership.

The exact syntax is defined by a separate specification.

## 9.13. Box

`Box` is permitted as a type needed for Rust interoperability and explicit control over data representation.

Ordinary Clust application code must not require pervasive use of Box.

COLI may generate Rust Box when necessary and consistent with the specified semantics.

If a Rust API type requires a specific `Box<T>`, the compiler must preserve compliance with its contract.

## 9.14. Shared Ownership

When shared ownership is necessary, COLI may use safe Rust mechanisms.

Possible options include Rc and Arc.

However:

- They must not be used for all objects by default.
- They must not be inserted solely to bypass an ownership error without a corresponding semantic basis.
- Atomic operations must not appear in single-threaded code unnecessarily.
- The distinction between shared access and unique mutable access must be preserved.

If the required ownership model cannot be proven, a diagnostic error is preferred.

## 9.15. Weak References

Clust must support weak references.

A weak reference does not own an object.

Using a weak reference does not guarantee that the object exists.

The operation that obtains strong access must safely handle the case where the object has already been destroyed.

The weak reference mechanism may require a supported stable representation of object identity.

The compiler must reconcile weak references with the actual ownership system.

## 9.16. Cyclic Graphs

Clust must support data structures representing graphs.

When ownership cycles exist, the memory system must avoid uncontrolled leaks.

Permitted solutions include:

- Non-owning edges.
- Weak references.
- Explicitly defined ownership boundaries.
- Other safe strategies provided by the implementation.

COLI is not required to automatically reclaim arbitrary cycles of strong references without additional information.

## 9.17. Object Destruction

Each resource must be destroyed in accordance with its actual Rust semantics.

Destructors and Rust Drop must be invoked correctly.

The following are prohibited:

- Double destruction.
- Access after destruction.
- Destruction before valid use has completed.
- Hidden suppression of Drop.
- Changing observable destructor behavior for optimization.

The specific destruction point may be determined by ownership inference, but it must comply with the program's public semantics.

For an inherited class, COLI MUST retain ownership of the complete derived value while borrowed base-class views remain live. An owning value of a base-class type MUST retain the actual derived value's destruction behavior. Destruction MUST preserve the derived-to-base cleanup order in Section 5.8.6; ending a base borrow MUST NOT independently destroy its subobject.

## 9.18. Copy and Clone

Clust must preserve the distinction between types that permit copying and objects that require an explicit cloning operation.

Copy values may be copied automatically according to the type's semantics.

For other types, the compiler must not implicitly invoke a potentially expensive Clone.

The Clone implementation must be compatible with Rust.

## 9.19. Optimizer Restrictions

COLI must not perform an optimization if it violates:

- Lifetimes.
- Uniqueness of mutable access.
- The order of observable effects.
- Rust API requirements.
- Address stability where required.
- Destructor semantics.
- Thread safety.
- Type correctness.

**Safety and semantic equivalence take priority over optimization.**

---

# 10. Functions and Closures

## 10.1. Functions

Free function declarations require the `fn` keyword. Class methods omit `fn`. Result types use `-> T`; an omitted result means unit, and ordinary value returns use explicit `return`.

Clust functions have statically determined argument and result types.

Functions may be:

- Ordinary functions.
- Generic functions.
- Asynchronous functions.
- Class methods.
- Interface methods.
- Associated functions.

Parameter semantics are integrated with COLI.

## 10.2. Closures

Clust supports anonymous functions and closures.

Developers are not required to explicitly specify how an external variable is captured.

Clust source code must not require a special `move` keyword for closures.

COLI automatically determines:

- Capture for reading.
- Capture with mutable access.
- Capture with ownership transfer.
- The lifetime of captured data.
- Whether a closure may escape the current scope.

The compiler may choose the corresponding Rust Fn, FnMut, or FnOnce representation.

## 10.3. Mutable Closures

The type system must allow closures that require mutable access to be denoted.

This capability may use `var` semantics.

The exact syntax is not a fundamental part of the specification.

## 10.4. Closure Restrictions

The compiler must prevent:

- Capturing invalid references.
- Unsafe lifetime extension.
- Conflicting mutable captures.
- Using a resource whose ownership has already been transferred.
- Violating Send/Sync when moving a closure between threads.

---

# 11. Constants, Static Data, and Type Aliases

## 11.1. Const

`const` represents a constant value that must satisfy the corresponding constant evaluation constraints.

The semantics must be compatible with Rust const wherever Clust maps directly to Rust.

## 11.2. Static

`static` is a separate construct.

It is intended for static storage or corresponding long-lived data.

Clust must not merge `const` and `static` into a single semantic category.

The distinction between a constant value and a static object is fundamental.

Access to mutable static state must comply with Rust's safety requirements.

## 11.3. Type

`type` defines a type alias.

An alias does not automatically create a new nominal type.

It must preserve the semantics of the original type, including generic parameters, ownership requirements, and Rust compatibility.

---

# 12. Rust Interoperability

## 12.1. Primary Goal

Clust must provide the most complete practical compatibility possible with the Rust ecosystem.

Users must be able to add Rust crates as dependencies of a Clust project.

Clust must not require existing Rust libraries to be rewritten.

At the same time, it is necessary to distinguish between:

1. The ability to use a crate as a dependency.
2. The ability to directly express every part of its public API in Clust.
3. The ability to safely invoke a particular operation.

Support for the first item is mandatory.

Support for the second must expand gradually.

The third is always subject to safety requirements.

## 12.2. Integration Levels

### Direct integration

Direct use of Rust APIs.

Used for types and operations that Clust can correctly represent.

### Generated adaptation

The compiler generates Rust wrappers to adapt representations and contracts.

Used only when type correctness and safety are preserved.

### Native Rust boundary

A complex operation is implemented in a Rust module and exported to Clust through a valid API.

This approach must remain available even when automatic adaptation is impossible.

## 12.3. Rust API Importer

The compiler must have a subsystem for importing Rust type information.

It must account for:

- Public API.
- Type definitions.
- Trait implementations.
- Generic parameters.
- Associated types.
- Lifetime relationships.
- Trait bounds.
- Visibility.
- Async functions.
- Constants and static declarations.
- Macros and generated items.
- Safety requirements.

Importing must not assume that Rust source text alone sufficiently describes the public API after macro expansion.

The method of obtaining type information may depend on the chosen Rust toolchain version.

Using unstable rustc APIs is permitted as a separate implementation strategy, but must not be declared a stable public guarantee of Clust without appropriate support measures.

## 12.4. Rust Traits

Clust's internal type system must understand the Rust trait system regardless of which user-facing language constructs express it.

It must account for:

- Supertraits.
- Blanket implementations.
- Coherence.
- Orphan rules.
- Associated types.
- Generic associated types.
- Auto traits.
- Dyn compatibility.
- Method resolution.
- Trait bounds.
- Generic substitutions.

The compiler must not silently create implementations prohibited by Rust's rules.

## 12.5. Implementation Restrictions

Clust must not allow inherent implementations to be added to external Rust types when Rust prohibits such implementations.

This rule also applies to class extension mechanisms.

Embedding a base class does not inherit its Rust trait implementations automatically. Implementations for a derived type must satisfy the complete inherited contract and the Rust restrictions described in Section 5.8.7.

Interface implementations must comply with the corresponding orphan rules and coherence requirements.

Clust must not provide a hidden mechanism for bypassing these restrictions.

## 12.6. Macros

Clust must aim to support Rust macros through the existing Rust system.

Using the original macro implementation is preferred over creating an independent Rust macro engine.

Support includes several categories:

- Derive macros.
- Attribute macros.
- Function-like macros.
- Declarative macros.

Macros applied to generated Rust types may execute during Rust compilation.

For macros that produce complex Rust code, type information and effects after expansion must be taken into account.

An opaque macro must not be executed in a context where COLI needs to know its effects unless there is a correct way to determine them.

A Rust adapter is used for incompatible cases.

## 12.7. Unsafe Rust APIs

Clust MUST NOT provide source-level `unsafe { ... }` blocks. Low-level unsafe implementations and Rust's required unsafe blocks belong on the Rust side of the interoperability boundary.

Clust MAY call an imported Rust function declared `unsafe fn` using ordinary call syntax. The user is not required to introduce an unsafe block or rewrite the function as a safe Rust wrapper merely to call it from Clust.

The compiler MUST retain the imported function's unsafe contract and emit the required Rust unsafe call at the Rust boundary. This generated block authorizes the Rust operation syntactically; it does not prove that the operation is safe.

Type, visibility, ownership, lifetime, and effect checks still apply. Preconditions that can be checked statically or through an adapter should be enforced. Preconditions that cannot be verified remain the caller's responsibility and MUST be exposed through imported API documentation and tooling. A call is not prohibited solely because some safety preconditions cannot be proven by Clust.

Violating an imported unsafe function's contract may cause undefined behavior. Clust's ordinary memory-safety guarantees are conditional on those external contracts being respected; an unsafe function does not become a safe API merely because its call has no unsafe marker in Clust syntax.

A safe Rust wrapper remains available when it can enforce the contract, but it is not mandatory for every imported unsafe function. Direct low-level operations that require unsafe implementation remain in Rust rather than becoming an unrestricted unsafe region in Clust.

## 12.8. FFI

Low-level integration with other languages and system APIs must primarily be performed through Rust.

Clust does not need an independent FFI ABI model of its own.

The corresponding Rust mechanisms apply when using the C ABI, system calls, and platform libraries.

## 12.9. Preservation of Rust API Semantics

Automatic adaptation must not change the observable semantics of an imported function.

In particular, it must not:

- Change ownership contrary to the Rust contract.
- Silently clone expensive values.
- Change Send/Sync guarantees.
- Remove mandatory lifetime constraints.
- Convert borrowing into ownership without a valid basis.
- Violate Drop semantics.
- Ignore Pin requirements.
- Hide unsafe preconditions.

---

# 13. Standard Library and Ecosystem

## 13.1. Reuse Principle

Clust must reuse the Rust standard library implementation as much as possible.

Independent implementations of containers, strings, network I/O, or system primitives should not be created merely for the sake of a new language.

Clust's own library must primarily act as an adaptation layer.

## 13.2. User-Facing Semantics

Clust must not require developers to constantly perform the low-level conversions typical of Rust APIs.

In particular, ordinary code must, where possible, avoid manual calls equivalent to:

- `as_ref`.
- `as_mut`.
- Explicit borrowing.
- Explicit moves.
- Technical conversions between reference representations.

COLI and the adaptation system must infer such operations when it is possible and unambiguous.

This does not prohibit direct use of the corresponding Rust methods when necessary.

## 13.3. Type Representation

Ordinary Clust types must correspond to compatible Rust types.

When necessary, Clust may introduce transparent adapters or new types if their representation, operations, and ownership rules compile unambiguously into Rust.

Clust must not create an independent object runtime system as the mandatory foundation for all user-defined types.

## 13.4. Clust Libraries

Clust crates must use a unified module, dependency, and compilation system.

Every Clust crate must compile into a Rust representation.

A user-defined Clust library is not required to export its original API directly to Rust.

Exporting a Rust API may be a separate capability.

---

# 14. Async/Await

## 14.1. Asynchronous Model

Clust must support native async/await.

The model must compile into Rust's asynchronous mechanisms, including Future and the corresponding state machines.

There is no requirement to create a separate low-level asynchronous task execution mechanism in place of Rust's.

## 14.2. Base Runtime

Tokio is the base runtime for asynchronous execution in Clust.

However, the language's semantics must not depend on specific Tokio API names.

Developers work with Clust abstractions that are adapted to Tokio.

## 14.3. COLI and Async

COLI must account for the fact that an async function can suspend execution while retaining part of its local state.

It must correctly analyze:

- Values that live across await.
- Borrows across await.
- Whether a Future may be moved.
- Pin requirements.
- Destruction of cancelled tasks.
- Environment capture.
- Send/Sync when executing in a multithreaded runtime.
- Data ownership between tasks.

An async function must not automatically be assumed independent of the lifetimes of its inputs.

## 14.4. Minimal Syntax Principle

Ordinary asynchronous programming must feel natural in the language.

Developers should not be forced to describe low-level details of the scheduler, executor, or Future state machine unnecessarily.

At the same time, task behavior and lifecycle must remain predictable.

---

# 15. Concurrency and Multithreading

## 15.1. General Principle

Multithreading must be an integral part of Clust.

Developers must not be required to write substantially more complex code for safe interaction between threads.

At the same time, Clust must not hide the fundamental restrictions of concurrent access.

## 15.2. Core Primitives

Clust's public model of safe concurrency is built around three primary abstractions:

**Channel** — transfer of messages and values.

**Actor** — isolated state and sequential message processing.

**Mutex** — synchronized exclusive access to shared state.

Other Rust mechanisms may be used through corresponding libraries and adapters.

## 15.3. Channel

A channel provides communication between tasks or threads.

COLI must infer correct ownership transfer when values are sent.

Developers must not be required to specify move manually for ordinary message passing.

A channel must support the corresponding safe send, receive, and termination operations.

The specific API is defined separately.

## 15.4. Actor

An Actor encapsulates mutable state and provides access through messages or corresponding operations.

During ordinary message processing, the actor's state must be isolated from conflicting external mutations.

The Actor model must support use with the Rust async runtime.

The actor lifecycle, processing order, termination, and failure behavior must be explicitly defined.

## 15.5. Mutex

A Mutex provides safe exclusive access to shared state.

Clust must provide a minimal way to use it.

The compiler must correctly constrain the lock's lifetime.

The guard must not be released unsafely before data access is complete.

Asynchronous code must account for restrictions on holding locks across await.

## 15.6. Threads and Tasks

Clust must support actual parallel execution on multiple threads.

Parallel execution may use Tokio and Rust mechanisms.

The compiler must check whether values may be transferred between threads.

Internal Send/Sync constraints must be preserved even if Clust's user-facing syntax makes them less visible.

## 15.7. Shared Memory

Mutable memory may be shared between multiple threads only through provably safe mechanisms.

Ordinary safe Clust code must not be able to create a data race.

COLI must account for thread boundaries when analyzing ownership, aliasing, and lifetimes.

## 15.8. Concurrency Performance

Clust must not automatically turn every shared object into Arc<Mutex<T>>.

The compiler must choose the minimum sufficient mechanism.

If only ownership transfer is required between tasks, adding shared ownership should preferably be avoided.

If only single-threaded execution is required, atomic synchronization should not be added unnecessarily.

---

# 16. Compiler Architecture

## 16.1. Main Subsystems

The following structure is recommended:

```text
Clust Compiler
│
├── Source Management
├── Lexer / Parser
├── AST
├── Module & Name Resolution
├── Type System
├── Rust API Importer
├── Interface / Trait Solver
├── Semantic HIR
├── COLI Analysis
│   ├── Ownership Analysis
│   ├── Aliasing Analysis
│   ├── Lifetime Analysis
│   ├── Effect Analysis
│   ├── Escape Analysis
│   └── Allocation Planning
├── Lowered MIR
├── Rust Code Generator
├── Diagnostics & Source Maps
└── Cargo / rustc Integration
```

These are architectural responsibilities, not mandatory software module names.

## 16.2. AST

The AST represents the source structure of Clust.

Final decisions about ownership or object placement should not be made at this stage.

## 16.3. HIR

The HIR represents the semantically resolved program.

It must contain:

- Type identities.
- Resolved class hierarchies, subtype conversions, base subobjects, override contracts, and class dispatch requirements.
- Resolved symbols.
- Generic constraints.
- Interface implementations.
- Method resolution results.
- Access semantics.
- Information about imported Rust APIs.

## 16.4. Effect Analysis

The compiler must be able to determine or account for the effects of operations on data.

In particular:

- Read.
- Write.
- Consume.
- Escape.
- Capture.
- Return dependency.
- Shared access.
- Synchronization requirements.

These effects may be part of the internal model rather than user-facing syntax.

## 16.5. MIR

The MIR must be expressive enough for control flow, ownership, and lifetime analysis.

It must support:

- Branches.
- Loops.
- Temporary values.
- Function calls.
- Data transfers.
- Destruction points.
- Class upcasts, complete-receiver dispatch, polymorphic ownership, and complete-object cleanup ordering.
- Closures.
- Async state.
- Objects with dynamic representations.

COLI must not be built solely on local syntactic analysis of individual expressions.

## 16.6. Rust Code Generator

The generator must produce correct Rust code.

A structured representation of Rust tokens or an AST is preferred over fragile string concatenation.

The result must satisfy Rust's syntactic, type, and ownership constraints.

## 16.7. Result Validation

The compiler must run the necessary Rust checks.

An important rule:

**Successful Rust validation is a mandatory condition for correct code generation, but does not replace Clust's responsibility for its own semantics.**

## 16.8. Diagnostics

Errors must point to the original Clust code.

Users must not be required to constantly interpret errors referring only to generated Rust files.

The compiler must preserve source mappings.

Diagnostics must explain:

- Which requirement was violated.
- Where the conflict arose.
- Why types or lifetimes are incompatible.
- Which Rust restriction caused the issue.
- Which safe actions are available to the developer.

---

# 17. Performance

## 17.1. Target Level

Clust's performance target is comparability with equivalent optimized Rust code.

This is a development goal, not an unconditional guarantee for every source expression.

## 17.2. Zero-Cost Abstractions

Clust must aim to eliminate additional overhead for high-level abstractions when they can be resolved statically.

In particular:

- Class methods are not required to have a vtable.
- Class inheritance uses embedded base state; class-polymorphic calls may become direct calls when the actual target is proven without changing dispatch behavior.
- An interface without dyn uses static polymorphism.
- Local objects are not required to be allocated on the heap.
- Closures are not required to have heap allocation.
- Shared ownership must not be added unnecessarily.
- Large objects must not be copied implicitly.

## 17.3. Permitted Overhead

Some capabilities inherently require additional mechanisms:

- Dynamic dispatch.
- Owning polymorphic class representations when complete derived values escape through a base type.
- Shared ownership.
- Weak references.
- Dynamic data structures.
- Synchronization.
- Asynchronous execution.
- Stable address identity.

The compiler must minimize overhead, but is not required to make these mechanisms free.

## 17.4. Choosing between Static and Dynamic Code

Static specialization may improve call performance while increasing generated code size.

Dynamic dispatch may reduce machine code duplication while introducing indirect calls.

The language must preserve an explicit semantic distinction between ordinary interfaces and `dyn`. Class types provide subtype polymorphism without a `dyn` marker; optimizing those calls into static code MUST preserve the actual object's applicable override.

## 17.5. Performance Monitoring

Clust development must include comparative benchmarks.

Required categories:

- Object creation and destruction.
- Argument passing.
- Container operations.
- Interface calls.
- Closures.
- Async/await.
- Channels.
- Mutexes.
- Actors.
- Multithreaded computation.
- Rust library calls.
- Object placement.
- Conversions and adapters.

Comparisons must use equivalent Rust programs, not only other high-level languages.

---

# 18. Safety and Correctness Guarantees

## 18.1. Safe by Default

Ordinary Clust operations must be memory-safe. Calls to imported unsafe Rust functions follow Section 12.7: their unverified external safety preconditions remain the caller's responsibility. The absence of unsafe blocks in Clust does not remove those preconditions.

Unsafe Rust must not be introduced silently to speed up a user program without rigorously proven invariants.

## 18.2. Correctness Goes beyond Successful Compilation

The compiler must not generate Rust that formally compiles but does not match the observable behavior of the original Clust program.

## 18.3. Errors Instead of Dangerous Heuristics

For ordinary Clust operations, if the compiler cannot prove safety, it must not arbitrarily assume it. Imported unsafe Rust calls are an explicit foreign-contract boundary under Section 12.7; their acceptance is not a proof of safety.

The options are:

- A correct conservative implementation, if permitted by the semantics.
- A diagnostic error.

Arbitrary cloning, memory leaks, or unsafe pointers must not be applied automatically as a universal solution to conflicts.

## 18.4. Validation

The following are required:

- Unit tests.
- Integration tests.
- Compile-pass tests.
- Compile-fail tests.
- Ownership tests.
- Lifetime tests.
- Concurrency tests.
- Rust interop tests.
- Differential tests against equivalent Rust programs.
- Regression tests.

Particular attention must be paid to branches, loops, aliasing, resource destruction, and asynchronous execution.

---

# 19. Development Tools

Clust must provide high-quality development support.

Strategic requirements:

- Clear diagnostics.
- Source locations.
- Incremental compilation.
- Cargo integration.
- Automatic retrieval of Rust crate information.
- Support for mixed projects.
- The ability to inspect generated Rust.
- IDE support.
- Autocompletion.
- Go to definition.
- Find usages.
- Type checking before a full build.

Specific implementation tools and protocols may change.

An important principle: users must primarily work with Clust sources rather than temporary Rust files.

---

# 20. Key Language Invariants

The following invariants take priority over the convenience of a particular implementation.

1. **Every Clust program compiles into Rust.**

2. **Clust does not require its own machine-code backend.**

3. **Every Clust type must have a correct Rust representation.**

4. **Class does not imply mandatory heap allocation.**

5. **An interface contains only methods.**

6. **An interface without dyn implies static generic semantics.**

7. **Dyn denotes explicitly chosen interface dynamic polymorphism; class subtyping requires no dyn marker.**

8. **Methods do not require fn.**

9. **An instance method receiver uses self or var self; a new constructor has an implicit initialization receiver.**

10. **The override and virtual keywords are absent; compatible inherited methods override automatically and preserve class-polymorphic dispatch.**

11. **All declarations are private by default, except interface methods and new constructors.**

12. **Public declarations use pub unless their construct is implicitly public.**

13. **Interface implementation methods in classes require pub.**

14. **Class methods may reside inside the class or in a separate implementation.**

15. **Rust coherence and orphan rules are not bypassed.**

16. **COLI infers low-level ownership and borrowing.**

17. **Ordinary Clust code does not require explicit move, &T, &mut T, or lifetime annotations.**

18. **Var denotes mutable access with the corresponding uniqueness requirements.**

19. **The compiler must not implicitly clone arbitrary non-Copy values.**

20. **Weak references do not own objects.**

21. **The compiler aims to use stack or inline storage when correct and beneficial.**

22. **Heap remains available as an explicit placement mechanism.**

23. **Const and static have different semantic purposes.**

24. **Type denotes a type alias.**

25. **Closures do not require explicit move for capture.**

26. **Async/await is based on Rust's Future model.**

27. **Tokio is the base runtime.**

28. **The primary concurrency abstractions are Channel, Actor, and Mutex.**

29. **Clust can use Rust source modules and Rust crates.**

30. **Rust libraries must not be rewritten for the sake of Clust.**

31. **Safety and correctness take priority over performance.**

32. **Equivalent Rust performance is the primary comparison target.**

33. **A class may inherit from one direct base class and implement multiple interfaces.**

34. **A derived class is usable wherever its base class is expected; overridable calls select the actual object's most-derived implementation without requiring dyn.**

35. **Borrowed and owning base-class values preserve the complete object's identity, derived state, dispatch, and destruction behavior without slicing.**

36. **Inherited state, interface contracts, and destruction order must have a valid Rust representation.**

37. **New is always-public constructor syntax sugar; successful construction yields a fully initialized instance.**

38. **Clust has no unsafe blocks; imported unsafe Rust functions may be called normally, with their safety contracts preserved and Rust-side unsafe call emission.**

---

# 21. What Is outside the Fundamental Specification

The following elements may change without changing the language's architectural philosophy:

- Code formatting.
- Library function names.
- Container method names.
- Most syntactic sugar rules.
- Literal declaration syntax.
- Specific names of internal compiler modules.
- CLI command names.
- The user-facing manifest format.
- Adaptation library names.
- The exact location of generated Rust files.
- The internal structure of Rust codegen.
- The choice of parser implementation libraries.
- Diagnostic code names.
- Caching strategy.
- The specific organization of runtime wrappers.

Such changes must not require revising the fundamental rules of ownership, types, safety, and compatibility.

---

# 22. Open Architectural Questions

Some questions must be resolved by separate specifications.

They are deliberately not settled prematurely.

## 22.1. COLI

The following require definitive formalization:

- Transfer of responsibility for destruction.
- Interprocedural effect analysis.
- Behavior in branches and loops.
- Relationships between unique access and logical object identity.
- Rules for graphs with complex ownership.
- The exact heap contract.
- Polymorphic class-value and owning dyn representations, preserving the complete-object contracts in Section 5.8.
- Pin support.
- Resource behavior when async tasks are cancelled.
- Correct interaction with Rust interior mutability.

## 22.2. Type System

The following must be defined:

- The complete semantics of generic parameter inference.
- Ways to express associated types.
- GAT representation.
- Method conflict resolution rules.
- Whether implementation specialization is permitted.
- Representation of types with nonstandard layouts.
- The final syntax of base-qualified calls and metadata for inherited APIs, within the semantics fixed by Section 5.8.
- Implicit constructor generation, constructor overload resolution, and fallible construction, within the guarantees fixed by Section 5.6.

## 22.3. Rust Interoperability

The following must be chosen:

- A mechanism for importing type metadata.
- A strategy for working with stable and nightly Rust tools.
- A macro integration approach.
- The scope of automatic adapters.
- Rules for supporting incompatible Rust APIs.
- A format for describing safe boundaries with Rust code.

## 22.4. Concurrency

The following require clarification:

- The Actor lifecycle.
- Task termination rules.
- Interaction between channels and ownership.
- Error behavior.
- The cancellation model.
- The semantics of automatically selecting single-threaded or multithreaded execution.

Solutions to these questions must comply with the fundamental invariants.

---

# 23. Development Strategy

Clust development must proceed in stages.

The initial delivery scope and accepted semantic choices are recorded in [docs/mvp-decisions.md](docs/mvp-decisions.md). The synchronous MVP includes closures, weak references, and user destructors, even though the staged implementation roadmap below groups some advanced analysis into later development stages. Stage labels do not make these required MVP features optional.

## Stage 1. Formalizing the Semantic Core

The priority is a precise type model and COLI.

The following must be defined:

- Types and interfaces.
- Single class inheritance, subtype substitution, inherited contracts, and class-polymorphic dispatch.
- Static and dynamic polymorphism.
- Ownership.
- Unique mutable access.
- Lifetimes.
- Destruction.
- Basic placement.
- Error rules.

The outcome of this stage is a formal specification suitable for implementation and validation.

Work in progress and remaining completion criteria are recorded in [docs/stage-1.md](docs/stage-1.md). Documenting one source example or initialization rule does not by itself complete this stage.

## Stage 2. Minimal Compiler

Implement:

- Parser.
- AST/HIR.
- Modules.
- Classes.
- Single class inheritance, borrowed and owning subtype conversions, and class-polymorphic dispatch.
- Interfaces.
- Methods.
- Functions.
- Basic generics.
- Basic COLI.
- Rust codegen.
- Cargo integration.
- Diagnostics.

The result is safe synchronous Clust programs that compile into Rust.

## Stage 3. Extending COLI

Implement more complex cases:

- Interprocedural analysis.
- Lifetime graphs.
- Escape analysis.
- Ownership through fields.
- Closures.
- Weak references.
- Placement optimizations.
- Complex combinations of generic types.

## Stage 4. Full Rust Integration

Expand support for:

- Rust crates.
- Rust traits.
- Complex generic constraints.
- Associated types.
- Macros.
- Adapters.
- Joint Clust/Rust builds.
- Unsafe Rust boundaries.

## Stage 5. Async and Concurrency

Implement:

- Async/await.
- Tokio integration.
- Channel.
- Actor.
- Mutex.
- Multithreaded execution.
- Send/Sync checking.
- Ownership analysis across await.

## Stage 6. Optimization and Ecosystem

Develop:

- COLI performance.
- Incremental compilation.
- Caching.
- IDE tooling.
- Library adaptation.
- Benchmarks.
- Diagnostics.
- Robust compatibility with the Rust toolchain.

Stages may overlap partially, but complex optimizations must not precede proof of semantic correctness.

---

# 24. Working Rules for AI Agents and Developers

The following instructions are mandatory when designing and implementing Clust.

## 24.1. Specification Takes Priority

Before changing the compiler, identify which fundamental principles are affected.

Language semantics must not be changed merely to simplify the current implementation.

If the existing architecture cannot satisfy a requirement, first identify the technical cause and possible ways to address it.

## 24.2. Do Not Transfer Rust's Complexity to the User

If Rust requires explicit borrowing, move, a lifetime annotation, or additional adaptation, first investigate whether COLI can infer it statically.

Introducing new user-facing syntax for manual memory management is considered only when automatic inference is insufficient and correct semantics cannot be ensured without an explicit constraint.

## 24.3. Do Not Create Independent Low-Level Mechanisms Unnecessarily

Before implementing a custom:

- Runtime.
- Allocator.
- Async scheduler.
- Set of system calls.
- Container.
- Synchronization mechanism.
- Low-level type representation.

check whether Rust can provide it.

## 24.4. Every Optimization Must Be Verifiable

An optimization must have:

1. A clearly described transformation.
2. Provable preconditions.
3. Preservation of semantics.
4. Positive and negative tests.
5. A comparative benchmark, when necessary.

## 24.5. Lack of Support Does Not Mean a Concept Is Prohibited

If Clust does not yet support a valid Rust pattern, that does not mean the pattern is fundamentally incompatible with Clust.

Distinguish between:

- Semantically prohibited behavior.
- Permitted behavior that is not yet implemented.
- Behavior requiring a Rust adapter.
- Behavior requiring a specification change.

## 24.6. Do Not Invent New Language Rules Unnecessarily

When a new task arises, first try to express it through:

- Class.
- Interface.
- Generics.
- Dyn.
- Methods.
- COLI.
- Existing Rust mechanisms.

Adding a new fundamental language construct must be justified.

## 24.7. When Changing the Architecture

Every significant change must be accompanied by:

- A description of the problem.
- Current behavior.
- Proposed behavior.
- The impact on COLI.
- The impact on Rust interop.
- The impact on performance.
- The impact on compatibility.
- A testing plan.
- A decision on whether this document needs to change.

---

# 25. Clust Success Criteria

The project can be considered to be achieving its primary goal if a developer can:

1. Create typed application programs without manual Rust borrow/move/lifetime annotations.

2. Use class subtyping and interface-based generic abstractions, with direct calls whenever the compiler can prove the applicable implementation.

3. Create complex data structures with correct ownership.

4. Work with Rust crates without extensively writing manual wrappers for common safe APIs.

5. Use async/await and multithreading with minimal syntax.

6. Obtain verifiable and performant Rust code.

7. Understand the causes of ownership errors from Clust diagnostics.

8. Inspect the compiler's decisions about memory and dispatch.

9. Use existing Rust tools, libraries, and platforms.

10. Achieve performance close to equivalent optimized Rust programs without being required to write low-level Rust code.

---

# 26. Final Architectural Statement

Clust must not become a copy of Rust with different syntax.

Its purpose is to change the level of abstraction at which developers interact with Rust's capabilities.

Rust provides:

- The type and systems foundation.
- Ownership mechanisms.
- Compiler infrastructure.
- Optimization.
- Native execution.
- The standard library.
- The ecosystem.

Clust provides:

- A more compact application-level type model.
- Classes with single inheritance and class polymorphism, alongside interfaces.
- Minimal user code.
- Automatic ownership and lifetime inference.
- Automatic data placement planning.
- High-level adaptation of Rust APIs.
- Native concurrency abstractions.
- Development convenience without mandatory manual memory management.

**Clust's fundamental goal is to make Rust's performance and safety accessible through a higher-level programming model without turning that model into mandatory additional runtime cost.**

This goal must guide all subsequent architectural decisions in the project.
