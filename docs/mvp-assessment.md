# Compiler MVP Assessment

**Date:** November 19, 2026. **Status:** repository audit and proposed delivery boundary, not an approved expansion or reduction of SPEC.md.

This audit remains a historical implementation snapshot. Its proposed scope is superseded by [approved MVP decisions](mvp-decisions.md), including required closures and weak references.

See [decision questions](mvp-questions.md), [Stage 1](stage-1.md), and [implementation status](implementation-status.md).

## 1. Executive Assessment

Clust currently has a documented language direction and two tested infrastructure libraries. It cannot yet parse, type-check, compile, or execute a Clust program. Stage 1 is incomplete; early Stage 2 infrastructure exists, but there is no end-to-end compiler pipeline.

The most consequential remaining work is not tokenization. It is specifying and implementing the interaction between implicit ownership, class subtyping, actual-class dispatch, borrowed results, and complete-object destruction. A successful Rust build alone cannot establish that the translation preserves these semantics.

No defensible completion percentage or delivery date follows from the amount of existing code. The MVP feature boundary and ownership contract must be selected first.

## 2. Verified Repository State

| Area | Evidence | Assessment |
| --- | --- | --- |
| Workspace | Root Cargo.toml; two members, Rust 2021, declared Rust 1.86 minimum, no external dependencies. | Builds locally; minimum compiler version was not tested. |
| Source infrastructure | compiler/frontend/src/source.rs | Source IDs, byte spans, Unicode-aware character locations, basic diagnostics and notes. No source excerpt renderer or generated-to-source mapping pipeline. |
| Lexer | compiler/frontend/src/lexer.rs | Words, numbers, strings, characters, comments, symbols, newlines and EOF; reports malformed input. It does not establish syntactic validity. |
| Syntax tree and parser | No modules or implementation files. | Absent. |
| Name/module resolution | No implementation. | Absent. |
| Type and member checking | docs/semantics/types.md only. | Rules partially documented, no executable checker. |
| Constructor checking | docs/semantics/construction.md only. | Initialization model exists on paper; no control-flow analysis. |
| Inheritance and dispatch | SPEC.md and docs/semantics/inheritance.md. | Required behavior documented; no representation or lowering implemented. |
| COLI | docs/semantics/ownership.md. | Proof obligations and vocabulary, no inference algorithm or effect contracts. |
| HIR/MIR and verification | No implementation. | Absent. |
| Integer utilities | compiler/semantics/src/integer.rs | Typed integer operations, explicit checked/wrapping policies, casts and error values. Not a source-language expression evaluator. |
| Layout utilities | compiler/semantics/src/layout.rs | Explicit aggregate/enum arithmetic for an initial 64-bit model. Not a verified Rust ABI or polymorphic class representation. |
| Rust generation and build driver | No implementation. | Absent. |
| CLI, standard library adapters, runtime | No implementation. | Absent. |
| Examples and compiler acceptance suite | One lexer test reads a Markdown example. | No executable .cl fixtures or compile-pass/compile-fail harness. |
| CI and packaging | No repository CI workflow or packaging setup found. | Absent. |
| Git state | `git ls-files` was empty; project files appeared untracked. | Current work is local and not yet recorded in tracked repository history. No commit was made during this audit. |

The Rust source and test files total 1,599 physical lines: 997 library lines and 602 test lines. This is an inventory metric, not a maturity measure.

### Validation Performed

On host `aarch64-apple-darwin`, using Rust 1.100.0:

- `cargo test --workspace --offline`: passed all 20 integration tests.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.

Test distribution: five inherited lexer tests, one Clust example tokenization test, eight arithmetic tests, and six layout tests. Some arithmetic tests exhaustively exercise eight-bit inputs; lexer tests also check short UTF-8 input robustness. These checks provide useful component confidence, not compiler semantic coverage.

Not verified: Rust 1.86 compatibility, release-profile behavior in this audit, other targets, a full compiler-core test run, or any complete Clust compilation. Compiler-core observations below are source inspection, not a claim that its entire pipeline passed validation here.

## 3. Specification Maturity and Gaps

### Established Direction

- Rust implementation and mandatory Rust output; `.cl` source files.
- Classes instead of a separate user-facing struct construct.
- Single inheritance with actual-class dispatch: Dog and Cat can be used as Animal, including owned results and heterogeneous base-typed collections.
- Always-public `new`, implicit construction receiver, and `super.new` for base initialization.
- Interfaces with static generic semantics unless explicitly used through `dyn`.
- `self` versus `var self`; no required source borrow/move/lifetime annotations.
- No arbitrary implicit Clone, slicing, lifetime leaks, or unsafe ownership repair.

These do not need to be asked again as basic design choices. The missing question is which subset must ship in the MVP and what exact contracts govern the unsettled cases.

### Concrete Gaps and Reconciliation Work

1. **No MVP boundary.** SPEC.md includes crates, advanced generics, closures, async, Tokio, channels, actors, mutexes and weak references. A synchronous executable subset is a proposal, not permission to silently omit these from the final language.
2. **Source grammar is partial.** Functions, bindings, ordinary method signatures, return syntax, module imports, operator precedence and most control flow are not fully defined. Earlier examples use `->`, whereas the inherited lexer currently produces separate `-` and `>` symbols. The parser/token contract needs an explicit decision, not an assumption that all examples already work.
3. **Ownership is described through obligations rather than rules.** It is not determined how a call that retains an argument affects subsequent aliases, how ownership joins branches, or which return dependencies are admitted.
4. **Constructor cleanup is incomplete.** Definite initialization has a useful starting model, but early exits, loops, implicit constructors, field defaults and partial cleanup order remain unsettled.
5. **Class polymorphism needs a representation proof.** Embedded base fields alone lose derived dispatch. Base method bodies, nested calls on self, base field access, interface conversion and owned destruction must preserve the complete receiver.
6. **Mutable base access needs two contracts.** Mutating a Dog through Animal is different from replacing an Animal-typed slot with Cat. A single unrestricted replacement reference cannot safely represent both.
7. **Generic virtual methods are an unresolved boundary.** Not every method contract maps directly to a Rust trait object. The MVP needs an explicit unsupported boundary or a working adaptation, not a fallback to base-only calls.
8. **Open inheritance complicates optimization.** A closed set of classes can use finite representations that do not generalize to separately compiled subclasses. The module and crate boundary must be selected before relying on that technique.
9. **Utilities retain experimental policy details.** Integer comments mention debug checked/release wrapping defaults although Clust has not chosen its policy. Layout code assumes 64-bit pointers, fixed primitive alignments, declaration-order fields, and explicit enum tags. These utilities must not silently become Clust ABI promises or drive raw Rust field offsets.
10. **The source example test is weak semantic evidence.** It checks that tokenization succeeds, selected words appear, spans are valid, and EOF exists. It would not catch many malformed class declarations or incorrect constructor semantics.
11. **Compiler unsafe lint does not constrain generated strings.** `unsafe_code = forbid` in the compiler crates does not prove emitted Rust or an imported runtime contains no unsafe operations.
12. **Foundation versus implementation status must stay explicit.** Documentation uses normative rules for future behavior. No checker currently enforces constructor public visibility, numeric argument ranges, subtype preservation, or exclusive access on source programs.

## 4. Compiler Core Integration

The workspace now includes parser/AST, module loading, HIR/MIR, type and ownership checking, interpreters, Rust emission, CLI and runtime source. See [implementation status](implementation-status.md) for the distinction between the integrated experimental profile and the approved Clust language.

The audit above predates this integration. Its implementation inventory and test counts are historical, not current readiness claims.

## 5. Proposed MVP Acceptance Boundary

**Proposal for user decision; not yet approved.** Deliver a synchronous native command-line compiler targeting Apple Silicon first, with inspectable generated Rust, basic Cargo orchestration, source-level diagnostics, and a documented supported subset.

Include:

- `.cl` input, `check`, `build`, `run`, and generated Rust inspection.
- Functions and methods, typed/inferred local bindings, arithmetic, conditions, loops and returns.
- Primitive values, owned strings, a small vector API and minimal output/assertion facilities.
- Classes, explicit constructors, single inheritance, implicit public overrides and real class-subtype dispatch.
- Borrowed and owned Animal values, heterogeneous Vec<Animal>, inherited fields, and complete destruction.
- A conservative static COLI core, definite initialization, and positive/negative safety tests.
- Small interfaces and basic generics only if selected in the scope decision; advanced combinations must have explicit unsupported diagnostics.

Defer by explicit agreement: automatic arbitrary Rust API importing, external inheritance ABI, closures, async, concurrency, weak cyclic graphs, advanced associated types, specialization, IDE integration, and package publishing. Deferral must be recorded as unsupported MVP features rather than removal from the foundation.

### Acceptance Scenarios

| Scenario | Required observable result |
| --- | --- |
| Build and run a simple main. | Correct output/exit status; real native artifact. |
| Construct Animal and Cat using the documented syntax. | Correct initialized fields; omitted pub on new does not restrict access. |
| Pass Dog and Cat as Animal. | Correct actual-class method result. |
| Call an override from an inherited base method. | Correct derived implementation; super selects the base body. |
| Return and store different subclasses as Animal. | No slicing; valid lifetimes and correct owned destruction. |
| Iterate a mixed Vec<Animal>. | Dispatch works for each actual class. |
| Partial constructor failure and normal scope exit. | Cleanup follows the selected deterministic contract. |
| Invalid initialization, aliasing, type and lifetime cases. | Stable Clust source diagnostics; no native artifact reported as successful. |
| Build generated Rust independently. | Supported generated programs compile and behave like the source model. |

## 6. Implementation Sequence and Exit Gates

1. **Decision baseline:** answer the priority questions; create an approved MVP feature matrix and executable source fixtures. Exit: no unspecified syntax or observable behavior in the acceptance examples.
2. **Parser/AST:** adapt the experimental frontend infrastructure. Exit: parse every approved fixture and reject malformed variants with Clust spans.
3. **Name and type checking:** modules, class hierarchy, signatures, constructors and initialization. Exit: type and initialization positive/negative suites pass.
4. **Polymorphism feasibility:** implement a representative Rust lowering for inherited state, actual-class dispatch, super, owned returns and mixed collections. Do this before bulk-importing the compiler core. Exit: all critical class scenarios execute with correct cleanup.
5. **COLI and MIR:** adapt dataflow, effects, loans, ownership transfer, lifetime validation and cleanup. Exit: path-sensitive safety cases are checked before code generation.
6. **Rust backend and CLI:** connect check/build/run, Cargo, source diagnostics and artifacts. Exit: the complete source-to-native acceptance suite passes.
7. **MVP hardening:** release/debug agreement with selected policies, deterministic diagnostics, reproducible builds, supported-target CI, parser robustness, and unsafe audits if applicable. Exit: documented limitations and repeatable installation/use instructions.

Stage 1 decisions and implementation probes may overlap. This does not justify coding an unspecified memory model and documenting its accidental behavior afterward.

## 7. Work That Does Not Require Further Product Decisions

I can independently choose internal Rust module organization, AST/HIR/MIR data structures, diagnostic plumbing, graph algorithms, test harness structure, generated temporary names, and safe implementation details consistent with approved semantics.

The user does need to select MVP scope and externally observable semantics. The accompanying questionnaire provides recommended defaults, concrete consequences and stable IDs so answers can be short.
