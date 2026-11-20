# Compiler MVP Decision Questions

**Status:** Q01–Q10 answered on November 19, 2026; see [approved decisions](mvp-decisions.md). Q11–Q22 remain pending except for portions already settled by those answers. The recommendations below are historical proposals, not the current decision record.

Read the [assessment](mvp-assessment.md) for the implementation evidence and delivery gates. Answer by ID. An answer such as "Use the recommended defaults except Q03 and Q07" is sufficient to adopt the remaining proposed choices explicitly. Scope choices must be reflected in SPEC.md and the Stage 1 documents before they govern implementation.

## 1. Requirements Already Known

Do not reopen these unless intentionally changing the language: Rust compiler implementation and Rust output; `.cl`; no separate source struct construct; single class inheritance; Dog/Cat substitutability for Animal with actual-class dispatch; no required override/virtual/dyn markers for class polymorphism; always-public new; implicit constructor self; English repository content; reuse existing components where compatible; no arbitrary implicit cloning, slicing, or unsafe ownership repair.

## 2. Priority Questions

### Q01 — What is the MVP demonstration and release boundary?

Is the target a working synchronous CLI compiler that runs a small multi-file object-oriented program, or must it already support a server, async application, or arbitrary Rust libraries?

**Recommended:** the synchronous native compiler described in the assessment, validated by a mixed Animal collection, inherited overrides, mutable methods, owned returns and cleanup. Treat async/concurrency and automatic Rust-crate importing as later milestones. This is the largest scope decision.

### Q02 — Which secondary features must ship in that first release?

For each of these, select required or deferred: interfaces, `dyn Interface`, user generics, enums/match, Option/Result, closures, weak references, user destructors, and arbitrary Rust imports.

**Recommended:** basic interfaces with default methods and static interface parameters, dyn-compatible interfaces, basic generic functions/classes, enums/match with Option/Result, and synchronous user destructors. Defer generic virtual methods, GATs, specialization, closures, weak references and automatic arbitrary Rust imports. Include generic virtual methods only with a separately demonstrated lowering. Required means tested implementation, not a placeholder API.

### Q03 — What ownership patterns must the MVP accept?

May two independent objects both retain the same non-Copy object? Must cycles of strong references work? Must a function return a borrow of an input or of an input field?

**Recommended:** one inferred owner plus temporary aliases; automatic transfer on return or storage when proven safe; simple read-only returns tied to one input owner; reject escaping aliases with no valid owner, unresolved ownership joins, strong cycles and independently shared owning graphs. Defer general shared ownership and interior mutability. This is a conservative MVP subset, not a permanent ban in Clust.

For example, decide whether storing the same Animal in two long-lived containers must succeed. The recommended subset rejects it when two independent owners would be required; explicit copying or cloning is a separate operation.

### Q04 — What should local aliases and mutation mean to programmers?

Should `let b = a` alias the same non-Copy object? Should `var b = a` create an exclusive mutable reborrow that temporarily prevents conflicting access through a? Can a var parameter replace the caller's entire value or only mutate it?

**Recommended:** let creates read access, var requires an existing write capability and creates an exclusive reborrow; read access cannot be upgraded by rebinding. Var parameters mutate the object but do not replace the caller's binding. Replacement uses a returned value assigned by the caller. A local var binding itself may be reassigned. This avoids replacing a Dog-typed caller slot with Cat through an Animal view.

These are observable rules; the compiler's dataflow and reborrow implementation can be chosen independently.

### Q05 — When should destructors execute?

Should a local resource be destroyed at its last use, or at the end of its ownership scope? What order should local resources use? What happens on field replacement or failed construction?

**Recommended:** end of ownership scope, locals in reverse initialization order, replaced values cleaned up at replacement, and partially constructed fields in reverse successful initialization order. Preserve the already documented complete-object order: own destructor, own fields in declaration order, then base recursively. Last use may end a borrow without immediately destroying its owner.

### Q06 — What runtime and performance restrictions are essential for MVP?

Is heap storage acceptable for an owning polymorphic Animal value that must hold different subclasses? Is a small generated support layer acceptable? Must all emitted Rust be safe, or may an audited unsafe representation be introduced when needed?

**Recommended:** allow necessary owning containers/dispatch metadata, avoid blanket Rc/Arc, keep proven concrete values inline when practical, and first target a safe Rust lowering. Any unsafe runtime requires a separate invariants review and targeted validation; the compiler-crate lint does not cover emitted Rust. No numeric performance target until a baseline exists. Specify any mandatory allocation or throughput limits now.

### Q07 — What ordinary syntax should become canonical?

Choose the free-function keyword, return separator, bindings and receiver notation. The following reflects the accepted Q07 correction; it is not a complete grammar:

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

**Accepted correction:** free functions require `fn`, while methods omit it. The remaining accepted syntax uses `->` results, omitted result means unit, explicit return for values, let/var bindings with optional type annotations, `var name: T` mutable parameters, and no mutation marker at the call site. Keep semicolons on executable statements; permit the currently documented optional trailing comma on fields. Select whether associated functions use `Class.function(...)` as constructors already do.

### Q08 — What constructor conveniences are essential?

Should a class without new receive an implicit constructor? Should a missing parameterless super.new be inserted? Are field defaults, overloads, named/default arguments and fallible constructors required?

**Recommended:** one explicit new per class, explicit super.new, no defaults or overloads in MVP, no early return from new, and no partially initialized receiver escaping. Use ordinary named factory functions returning Result for recoverable construction failure, with validation before calling the infallible constructor. These restrictions do not change new's public visibility.

### Q09 — Is the first compilation unit a closed application or an extensible library?

Must a separately compiled external crate be able to subclass a Clust class, or can the compiler see the application's entire Clust hierarchy when compiling it? Are cross-module inheritance and privacy required immediately?

**Recommended:** multiple modules inside one application compilation unit, with cross-module inheritance and visibility checks; defer external Clust inheritance ABI and separately distributed Clust libraries. Require valid Rust interop boundaries, but no stable generated ABI. Use this boundary explicitly when deciding whether closed-world optimizations are valid.

### Q10 — Which platform, toolchain and commands are required?

Is Apple Silicon macOS sufficient first? Must Linux and Windows work immediately? What does successful use look like from a clean checkout?

**Recommended:** aarch64 macOS first; declare and test a specific stable Rust minimum before release. Provide `clust check`, `clust build`, `clust run`, and `clust emit-rust`, with debug/release profiles and explicit artifact paths. Add a minimal project manifest and Cargo-backed dependency/build orchestration. A compiler test suite is required; a source-language `clust test` runner is deferred unless required.

## 3. Additional Decisions with Proposed Defaults

These can be adopted together; only exceptions need detailed answers.

| ID | Question | Recommended MVP default | Why it matters |
| --- | --- | --- | --- |
| Q11 | Which primitive types and literal inference rules? | Rust-width signed/unsigned integers, bool, char, f32/f64 and unit; contextual integer typing, otherwise i32; no implicit numeric narrowing or cross-type promotion. Owned String and a narrowly specified borrowed string view. | Determines type inference, arithmetic and adapters. |
| Q12 | Overflow, casts and indexing failures? | Checked integer arithmetic in both profiles, division/index errors trap, constant errors rejected at compile time. Explicit numeric casts have documented range checking; no silent truncation as the default source conversion. | Existing utilities expose wrapping/truncating choices that must not choose source policy implicitly. |
| Q13 | Control flow and expression results? | if/else, while, for over a defined iterator subset, break/continue, explicit return; bool-only conditions and short-circuit logical operators. No implicit trailing-expression return in MVP. | Defines CFG, initialization joins and ownership effects. |
| Q14 | Nullability and recoverable errors? | No implicit null; Option/Result and match. Defer try/catch exceptions; decide `?` separately, with explicit match sufficient initially. | Affects typing, early exits and cleanup. |
| Q15 | Panic behavior and cleanup? | Define runtime traps and explicit panic with unwinding through valid initialized state; abort on a second panic during cleanup. Construction failure uses its separate cleanup contract. | Requires tested paths; if abort-only is chosen instead, destructor guarantees on those paths must be narrowed explicitly. |
| Q16 | Destructor syntax? | `drop() { ... }` as a special implicit-receiver hook, never a normal manually invoked method and never inherited as an override; exact visibility and permitted effects specified before implementation. | User-level resource lifecycle needs syntax independent of Rust adapters. |
| Q17 | Privacy model and overriding details? | Module privacy following Rust boundaries; no protected, no overload resolution by argument types in MVP; public instance methods override implicitly with invariant signatures; no abstract/final classes yet. | Some restrictions are existing assistant-authored draft choices and should be explicitly accepted. |
| Q18 | Collections and minimum useful library? | Owned String, Vec with construction/push/index/length/iteration, Option/Result, print/println and assertions, through documented adapters. No networking, async I/O or collection catalogue yet. | The Animal collection acceptance scenario otherwise lacks a callable library surface. |
| Q19 | Rust library access in MVP? | Explicit small Rust adapter modules with declared type/access/effect contracts; no automatic arbitrary crate API importer. Cargo dependencies may be used behind adapters. | General Rust metadata and lifetime importing is a separate major compiler subsystem. |
| Q20 | Generic and interface boundaries? | Basic monomorphized generics with interface bounds; default interface methods; dynamic interfaces restricted to dispatchable signatures. No generic virtual methods, GATs, higher-ranked surface syntax or specialization. | Requires precise unsupported diagnostics rather than accidental base dispatch or invalid Rust traits. |
| Q21 | Lexical rules and formatting? | ASCII identifiers for MVP, UTF-8 string/char literals, line and nested block comments; documented operator precedence; whitespace otherwise insignificant, statements use semicolons. Explicitly define field boundaries. No string interpolation initially. | Most lexical machinery exists, but complete parser agreement does not. |
| Q22 | Required nonfunctional acceptance criteria? | Reproducible native build, deterministic source diagnostics, no compiler crashes on malformed fixtures, positive/negative semantic suites, differential execution cases, and a clean documented installation path. Record performance baselines without promising Rust parity for all programs. | Defines completion and prevents replacing a usable MVP with a large unvalidated feature list. |

## 4. Decisions I Can Make without Further Questions

Internal Rust crate layout, syntax-tree node structures, semantic IDs, CFG representation, fixed-point traversal order, diagnostic code organization, fixture layout, temporary artifact naming, and compatible parsing/dataflow reuse do not require individual approval.

I can also select ordinary implementation techniques within an approved behavior contract. If a technical experiment shows that the selected representation cannot preserve required semantics, I should report the concrete incompatibility and alternatives rather than silently change the language.

## 5. Suggested Answer Format

```text
Q01: Recommended synchronous MVP; first demo = ...
Q02: Required = ...; deferred = ...
Q03-Q06: Recommended defaults, except ...
Q07: Use the example syntax, except ...
Q08-Q10: Recommended defaults, except ...
Q11-Q22: Recommended defaults, except ...
Additional must-have program or constraint: ...
```

After these decisions, the next output should be an approved MVP matrix, reconciled syntax/semantic rules, and acceptance fixtures. Implementation should then follow the assessment's exit gates. No need to ask the user to design COLI's algorithm or Rust vtable layout; those are compiler engineering tasks once observable behavior is fixed.
