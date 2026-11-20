# CLI, Projects and Rust Integration

The default source profile uses `.cl`, `new`, `self`, methods without `fn`, and arrow return types. Fields and declarations are private unless marked `pub`; constructors are always callable when their class is accessible. A private `fn main()` is a valid entry point. Every class requires one explicit constructor. Constructors cannot return early, and Clust source cannot contain unsafe blocks or locally implemented unsafe functions.

## Commands

```sh
clust check src/main.cl
clust emit-rust src/main.cl
clust build src/main.cl -o application
clust run src/main.cl -- argument1 argument2
```

`check` performs the implemented type, ownership and cleanup checks. `emit-rust` writes generated Rust to stdout. `build` creates an executable. `run` inherits the working directory and standard streams, forwards arguments after `--`, and returns the executable's exit code. File-based runs remove their temporary build directories after execution or compilation failure. The library does not yet provide a program-argument access API.

Debug and release both check integer overflow by default in the Clust profile. `--release` selects optimization; it does not silently change arithmetic. `--overflow-checks=off` is an explicit experimental policy override.

`syntax` and bounded reference `eval` are development commands. Reference execution cannot run arbitrary imported Rust functions: it reports `ForeignFunction`, rather than substituting a fabricated result. Use native `run` for adapters.

## Project Manifest

Create `Clust.toml`:

```toml
[package]
name = "application"
entry = "src/main.cl"

[dependencies]
support = { path = "rust-support" }
```

`entry` defaults to `src/main.cl`. Dependencies use Cargo's dependency syntax; relative dependency paths are resolved against the Clust project directory. Dependencies are Rust implementation dependencies, not automatic Clust imports. Unknown manifest sections and package keys are diagnosed.

```sh
clust check
clust build --release
clust run --release
clust build path/to/project -o application
```

With no input, the CLI reads `Clust.toml` in the current directory. A directory or an explicit `Clust.toml` path can also be supplied. Cargo builds use `--offline`, generate Rust and a Cargo manifest under `target/clust/generated`, retain Cargo's lockfile, and put executables under `target/clust/artifacts/debug` or `release`. Registry dependencies must already be cached. `CLUST_RUSTC` and `CLUST_CARGO` select the build tools. Object emission currently takes a source-file input rather than a manifest.

## Explicit Rust adapters

Direct module and crate calls are documented in the [native Rust guide](rust-interop.md).

Declare an explicit Rust function path and its Clust signature. Clust paths use dots, including the `rust` attribute. The backend emits Rust path separators when generating the call.

```clust
#[rust("support.increment")]
#[safety("value must be nonnegative")]
extern "Rust" unsafe fn increment(value: i32) -> i32;

fn main() -> i32 {
    return increment(6);
}
```

The Rust implementation belongs to the `support` dependency. The caller uses ordinary syntax. Generated Rust contains the required unsafe call region and preserves the optional safety text. The declaration must match the Rust API; rustc checks the actual call signature. Unsafe preconditions remain the caller's responsibility. Safe imports use the same declaration without `unsafe` or `safety`.

The implemented boundary is deliberately limited to immutable scalar value parameters and scalar results: integers, floats, bool, char, unit and static `str`. Classes, references, mutable parameters and raw pointers are rejected at this boundary. No Clust owner or borrow may cross it implicitly. Adapter functions can be used as ordinary function values. Rust panics are caught and converted into the compiler's unwind path; initialized Clust resources are cleaned up, and a second panic during cleanup aborts.

For opaque owned objects or borrowed results use the native mixed profile.

## Available Library Surface

```clust
use core.option.{Option, is_some, unwrap_or};
use core.result.{Result, is_ok};
```

`Option<T>` has `None` and `Some(T)`. `Result<T, E>` has `Ok(T)` and `Err(E)`. Payload access uses exhaustive `match`; the normal ownership rules apply. These modules are bundled with the compiler and need no dependency entry.

`print(value)` and `println(value)` accept one scalar or string. `assert(condition)` accepts a bool and unwinds on false. Output errors also follow the unwind path. Owned String values also print their UTF-8 contents. Formatting arguments are not implemented.

Interfaces support default method bodies and static shorthand parameters such as `fn read(value: Read)`. Each shorthand parameter becomes an independent inferred generic parameter. Explicit implementations take precedence over defaults; conflicting defaults require an explicit implementation. Default bodies are checked against the interface contract rather than concrete implementation fields. Dynamic interfaces and class-polymorphic values retain actual payloads and dispatch through their runtime type. See the implementation status for generic boundaries.


Use `use core.vec.Vec;`, `use core.string.String;` and `use core.weak.Weak;` to access the bundled collection, text and weak-reference APIs.

- `Vec<T>`: `new`, `len`, `capacity`, `reserve`, `push`, `pop` (Option), `at`, `at_mut`, `set` and `clear`. Index reads use `at`; indexed assignment uses `set` and destroys the replaced element. Growth moves elements without cloning. Live element borrows prevent invalidating growth/replacement.
- `String`: `new(str)`, byte `len`, `char_count`, `byte_at`/`at`, `append(str)`, `append_string(String)`, `push(char)`, `equals(String)`, `is_empty`, `clear` and `write`. The byte buffer remains private to preserve valid UTF-8.
- `for value in collection` performs read-only iteration with `len` and `at`. String elements are bytes.
- `Weak<T>`: `new(target)`, `is_alive` and `with<R>(callback)`. See [weak semantics](semantics/closures-and-weak.md).

Runnable fixtures include `clust_animals`, `clust_dynamic`, `clust_generic_inheritance`, `clust_generic_dynamic`, `clust_string`, `clust_iteration`, `clust_closures`, `clust_captures`, `clust_weak`, `clust_weak_identity` and `clust_weak_backedge`. Their nonzero exit codes are expected return values, not compiler failures.

## Async applications

An entry file containing an async function selects the scalar async backend. Use
`async fn main()` with a unit or i32 result. Call an async function to create a
future and use `.await` to resume when it completes. The CLI supports syntax,
check, emit-rust, build and run for these applications. Native execution uses
Tokio's multithreaded scheduler. Cargo needs the Tokio dependency cached for
offline builds. See `examples/clust_async/main.cl` for a complete program.

- `spawn(future)` schedules an owned Send future. `task.join().await` consumes
  its handle and returns its result or propagates task failure as a panic.
  Dropping the handle requests cancellation. `task.cancel().await` waits for
  termination. Cancellation is cooperative at suspension points.
- `channel(capacity)` returns a cloneable sender and one receiver. Capacity must
  be positive. Sending awaits capacity and returns a Result. Receiving through
  a mutable receiver returns an Option. Closing the receiver rejects new sends
  and permits draining buffered messages.
- `actor(initial, capacity, handler)` starts a task with private state and a
  bounded mailbox. Each synchronous handler consumes a state and a message and
  returns the next state. Senders enqueue messages using `actor.sender()`.
  Release external senders before awaiting `actor.finish()`, which drains the
  mailbox and returns the final state. Dropping the actor requests cancellation.
- `mutex(initial)` returns a cloneable handle. For Copy state, `read().await`
  takes a snapshot and `update(message, handler).await` applies a synchronous
  state transition under exclusive access. The guard stays inside the runtime.

The async source subset supports top-level functions in the entry file, scalar
values, tuples, and owned runtime handles. The Rust backend checks task transfers
and lifetimes. Classes, Clust modules, COLI borrowing, destructors and Rust
adapters currently require the synchronous pipeline. Async check diagnostics from
Cargo refer to generated Rust. Reference eval and object emission are synchronous
only.


### Parameters and pending futures

The async profile infers parameter contracts across direct function calls. An
ordinary parameter used only for borrowing operations (including forwarding to
another read-only parameter) borrows its handle. Passing a handle to a consuming
operation such as `drop`, returning it by value, or moving it into a local
binding uses an owned parameter. Scalar Copy parameters pass by value.
A `var` parameter borrows exclusively and requires writable storage at the call.

For example, `async fn read(value: Mutex<i32>) -> i32 { return value.read().await; }`
allows repeated `read(value).await` calls with the same mutex. Calling
`async fn next(var inbox: Receiver<i32>) -> i32` as `next(inbox)` borrows a
mutable receiver from future creation until completion or drop. A second
overlapping receive is rejected, including when neither future has been polled.

A direct `spawn(read(value))` moves the handle into a task which borrows it
internally. Use an explicit clone to retain a shareable handle in the caller.
A previously created borrowing future retains its original contract when passed
to spawn, which requires a Send, static future. A local exclusive borrow cannot
escape into a spawned task. Rust checks the generated references and their
lifetimes. This inference applies to the async profile's supported types.
