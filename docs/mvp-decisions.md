# Approved MVP Decisions

**Date:** November 19, 2026. **Status:** Q01–Q10 answered; Q11–Q22 remain pending except where their contents are already settled by these answers.

This record captures the user's answers to the numbered questions in the conversation. It supersedes conflicting recommendations in the [assessment](mvp-assessment.md) and [questionnaire](mvp-questions.md). A required feature is a delivery obligation, not an implementation-status claim.

## 1. Approved Scope

| Decision | Approved contract |
| --- | --- |
| Q01 | Synchronous native CLI compiler for multi-file applications. Async/concurrency and automatic arbitrary Rust API importing are deferred. |
| Q02 | Interfaces, `dyn Interface`, user generics, enums/match, Option/Result, closures, weak references, and user-defined destructors are all required in the MVP. |
| Q03 | One inferred owner with safe aliases and automatic ownership transfer where proven valid; simple read-only results tied to an input owner. Reject unresolved ownership, strong cycles and independent shared owning graphs in the MVP. |
| Q04 | `let` gives read access; `var` needs write permission and exclusive access. Mutable parameters mutate objects without replacing caller bindings. Local `var` bindings can be reassigned. |
| Q05 | Destruction at ownership-scope exit; locals in reverse initialization order; replacement cleans up the replaced value. Partial construction cleanup reverses successful initialization. Complete objects retain the documented own destructor, own fields in declaration order, then base order. Last use may end a borrow without destroying its owner. |
| Q06 | Necessary owning polymorphic containers and dispatch metadata are allowed; no blanket Rc/Arc. Prefer a safe Rust lowering and inline concrete values where practical. Any unsafe support requires explicit invariants and validation. No fixed performance target yet. |
| Q07 | Free functions require `fn`; methods omit `fn`. Results use `->`; omitted result means unit; value returns use explicit `return`. Bindings use `let`/`var` with optional annotations; mutable parameters use `var name: T`; call sites need no mutation marker. Statements use semicolons and fields retain optional trailing commas. |
| Q08 | One explicit always-public `new` per class; explicit `super.new` for a derived constructor. No constructor overloads, defaults, implicit generation or early return in MVP. Ordinary named factories returning Result handle recoverable validation before infallible construction. |
| Q09 | Rust-style `mod` declarations and `use` bindings with dot-separated paths, inline/file modules, aliases, re-exports and module visibility. Multiple modules form one application compilation unit with cross-module inheritance. External Clust inheritance ABI and separately distributed Clust libraries are deferred. |
| Q10 | Apple Silicon macOS first; tested stable Rust minimum before release. Commands: `clust check`, `build`, `run`, `emit-rust`; debug/release profiles, artifact paths, minimal manifest and Cargo-backed builds. Compiler tests required; source-language test runner deferred. |

Q02 refers to the feature list in the conversational summary. It does not override Q01's explicit deferral of automatic arbitrary Rust library importing. Fine-grained generic/interface restrictions in Q20 are not silently approved by selecting user generics as a feature.

## 2. Syntax Example

```clust
class Counter {
    value: i32

    new(value: i32) {
        self.value = value;
    }

    pub read(self) -> i32 {
        return self.value;
    }

    pub add(var self, amount: i32) {
        self.value += amount;
    }
}

fn advance(var counter: Counter, amount: i32) {
    counter.add(amount);
}

fn main() -> i32 {
    var counter = Counter.new(0);
    advance(counter, 2);
    let result = counter.read();
    return result;
}
```

This form is implemented and exercised by the counter fixture. Associated factories omit the receiver and use ordinary class-qualified calls. See the syntax and CLI documents for runnable examples.

## 3. Consequences of Including Closures and Weak References

Closures and weak references are required, not optional follow-up features. They must obey the accepted single-owner model.

- A closure may borrow or take inferred ownership according to use. An escaping closure must not retain a borrow of a destroyed owner. Mutable captures require exclusive access. Multiple independent owning captures cannot be synthesized by hidden cloning or reference counting.
- A weak reference does not own its target. Its presence does not keep the target alive. Access must safely detect expiration, and successful access must not outlive or conflict with the target's valid access rights.
- Weak back-edges may represent relationships that would otherwise require strong ownership cycles. The prohibition on strong cycles does not prohibit weak references or graph-shaped data.
- Weak identity metadata may need stable storage even when the payload has one owner. Representation choices must preserve identity through permitted moves without making every class reference-counted.

The implemented working syntax and access contracts are now documented in [closures and weak references](semantics/closures-and-weak.md). These implement the approved ownership obligations without claiming that every higher-order capture or weak-upgrade form is available.

## 4. Remaining Decisions and Delivery Gates

Q11–Q22 remain pending. In particular, numeric defaults, overflow/casts, panic mode, destructor syntax, detailed generic boundaries, module syntax, library API and lexical conventions have not been adopted merely because Q01–Q10 were answered.

Where earlier answers already settle part of a later question, preserve that decision: Option/Result are required; value return is explicit; statements use semicolons; constructor overloading is excluded. Ask only about the remaining parts.

Before implementing the entire MVP, extend the semantic and acceptance corpus to cover:

1. The approved function and method forms and constructor restrictions.
2. Borrowed and owned class polymorphism, base calls and mixed base-typed collections.
3. Closure capture, invocation, mutation, ownership transfer and invalid escape.
4. Weak access while alive, expiration after destruction, weak back-edges and identity preservation.
5. User destructor execution, field replacement and partial cleanup, under the eventual panic contract.
6. Interface defaults, static and dynamic interface use, generics, enum matching and Option/Result.

This decision record alone is not evidence of execution. The current acceptance mapping and restrictions are in [implementation status](implementation-status.md); historical Stage 1 questions must be read alongside that implementation record.

## 5. Unsafe Interoperability Decision

Clust source has no unsafe blocks. Imported Rust unsafe functions may nevertheless be invoked using ordinary call syntax. Unsafe implementation and the required emitted unsafe call region remain on the Rust side. Unverified safety preconditions remain the caller's responsibility; imported documentation and tooling must preserve them. A safe wrapper is optional, not a prerequisite for every such call.

This decision settles the unsafe-call policy without approving automatic arbitrary Rust API importing for the MVP. It does not weaken ordinary COLI checking or authorize unsafe ownership repair. See SPEC.md Section 12.7.
