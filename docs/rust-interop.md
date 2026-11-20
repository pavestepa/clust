# Calling Rust from Clust

Place `math.rs` beside `main.cl`. A module can also live in `math/mod.rs`.
Each name must identify exactly one module. Rust resolves children of Rust
modules with its usual file layout.

```rust
// math.rs
pub fn twice(n: i32) -> i32 { n * 2 }
```

```clust
// main.cl
mod math;
fn main() -> i32 { return math.twice(21); }
```

Run `clust run main.cl`. The executable exits with status 42. To consume a
crate, add a Cargo dependency under `[dependencies]` in `Clust.toml` and run
`clust run .`. For example, `api = { package = "my-api", path = "../api" }`
makes `api.twice(21)` available if the library exports that function.
Cargo builds offline, so registry dependencies must already be cached.

## Native representations

| Rust API | Clust call behavior |
| --- | --- |
| `T` | Pass the Rust value directly. Copy types can be reused. Moving a non-Copy value transfers destruction responsibility. |
| `&T`, `&mut T` | Pass the expression as an argument. The compiler can insert a borrow. Exclusive access requires writable storage, such as a local declared with `var`. |
| Borrowed result | The Rust signature determines which input can own the referenced data. Mutation, moves and task escape are checked against the live borrow. |
| `String`, `Vec<T>`, `Option<T>`, `Result<T,E>` | These are the actual Rust types in native code. Use their methods and explicit constructors for conversions, such as `String.from("hello")`. |
| Private struct or enum | Its layout and destructor belong to the defining Rust crate. Use its public constructors, methods and constants. |
| Associated items and trait methods | Write `Type.item` for an associated item, `value.method()` for an instance method and `use api.TraitName;` to bring a trait into scope. |
| Async | Await a returned future with `.await`. A borrowed future may run within the owner's lifetime. Tokio spawn requires a Send, static future. |
| Attributes and macros | Use `#[derive(Clone, Debug)]` on a native data class. Attribute and function-like macros use dotted crate paths. Invocation payloads use Rust macro syntax. |

Explicit Clust imports `core.option.Option` and `core.result.Result` use the
same native enum representations. `core.vec.Vec` and `core.string.String` use
typed wrappers in the mixed profile to preserve Clust operations such as
`reserve(capacity)` and `String.new(text)`. Their backing storage belongs to a
Rust Vec or String. At a native call the adapter can borrow the storage or
transfer the container by value without copying its allocation. Use
`from_native` and `into_native` when converting explicitly, including typed
assignments. For non-Copy vector elements use a borrowed index or Rust's
`get` method. The mixed profile's value-returning `at` requires Copy.

See the [handle example](../examples/mixed_rust/main.cl) and its
[Rust implementation](../examples/mixed_rust/device.rs). The exit code is 7,
the string length after mutation plus one destructor invocation.

## A real Cargo derive

With the following dependencies in `Clust.toml`, Serde can implement both
serialization traits on the Clust data class.

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```clust
#[derive(serde.Serialize, serde.Deserialize)]
class Message {
    pub value: i32
    new(n: i32) { self.value = n; }
}

fn main() -> i32 {
    let message = Message.new(42);
    let json = serde_json.to_string(message).unwrap();
    println(json);
    let result = serde_json.from_str<Message>(json).unwrap();
    return result.value;
}
```

Running the project prints `{"value":42}` and exits with status 42. The first
call borrows the message. The second borrows the resulting string as a slice
and returns an owned Message inside a Rust Result.

## Compilation boundaries

Local modules exporting simple scalar functions can use the full synchronous
Clust frontend, including inheritance. Rich Rust APIs use the native mixed
profile. It supports data classes with field-initializing constructors,
methods, generic parameters and destructors. Combining rich imports with
Clust inheritance, dynamic interfaces or weak references requires further
compiler integration. Other representation changes require an explicit
conversion. Unsafe operations belong inside a Rust API with a
safe public contract.

`check`, `build`, `run` and `emit-rust` use Cargo for mixed applications.
Use `run` for execution. The reference interpreter cannot execute native Rust.
The CLI obtains expanded function signatures from rustdoc JSON. The rustdoc
subprocess enables its unstable JSON flag. The application uses the selected
Rust compiler normally. Rustdoc schema changes can require an importer update.
Some diagnostics refer to generated Rust. The LSP does not yet index these
expanded APIs for completion or navigation.
