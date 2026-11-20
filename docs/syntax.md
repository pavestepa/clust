# Clust Syntax

This document describes the implemented synchronous source profile. [Implementation status](implementation-status.md) lists unsupported combinations; [MVP decisions](mvp-decisions.md) records the approved scope.

## Lexical Forms

Source files use `.cl`, ASCII identifiers and UTF-8 string/character literals. Comments use `//` or nested `/* ... */`. Strings use double quotes; characters use single quotes. Numeric types are `i8` through `i128`, `u8` through `u128`, `isize`, `usize`, `f32` and `f64`. Other primitive types are `bool`, `char`, `str` and unit `()`.

Integer literals use contextual types, otherwise i32. Different numeric types require explicit `as Type` conversion. Integer overflow is checked by default in debug and release. Arithmetic, bounds and assertion failures unwind initialized resources. A borrowed `str` is currently a static string value; owned text uses the library `String` class.

Executable statements end with semicolons. Control-flow blocks need no trailing semicolon. Fields may end with a comma, semicolon or newline. Identifiers beginning with `$` are reserved for compiler-generated nodes and cannot be written in source.

## Classes, Constructors and Methods

```clust
class Animal {
    pub age: u32
    new() { self.age = 0; }
    pub read(self) -> u32 { return self.age; }
    pub advance(var self) { self.age += 1; }
}

class Cat: Animal {
    pub color: u8,
    new(color: u8) {
        super.new();
        self.color = color;
    }
    pub read(self) -> u32 { return super.read() + 1; }
    drop() { println(self.age); }
}

fn main() -> i32 {
    var cat = Cat.new(7);
    cat.advance();
    return cat.read() as i32;
}
```

Free functions require `fn`; methods omit it. Results use `-> T`; omitted results mean unit. Value returns are explicit. A read-only receiver is `self`, a mutable receiver is `var self`. Associated methods omit the receiver and are called through the class name. A mutable ordinary parameter is `var value: T`; call sites have no mutation marker.

Every class requires exactly one explicit `new`. It is always public, including when `pub` is omitted. Constructors have implicit initialization receivers/results and cannot declare a receiver, result annotation or early return. `super.new` initializes the direct base exactly once before accessing the derived receiver. Constructor overloads, synthesized constructors and field defaults are excluded. `drop()` is a special implicit-receiver hook, not a manually callable method.

Classes use single inheritance. Public instance methods override compatible base methods implicitly. Private fields/methods use lexical class access. Constructors and destructors are not virtual. Ordinary class subtyping needs no `dyn` keyword: Cat can be passed, stored or returned where Animal is expected.

## Bindings, Expressions and Control Flow

```clust
let answer: i32 = 42;
var total = 0;
while total < 3 { total += 1; }
if answer > total { println(answer); } else { println(total); }
```

`let` provides read access; `var` additionally requires write permission and exclusivity. Local mutable bindings can be reassigned. Tuple values use `(a, b)`, fixed arrays use `{a, b}` and array types use `{T; N}`. Indexing uses `value[index]`. An explicit local view pattern can use `ref name` or `ref var name`; ordinary class parameters usually infer their view without such syntax.

Supported control flow includes if/else, while, loop, break, continue, return and exhaustive match. Conditions are bool. `&&` and `||` short-circuit. `for item in collection { ... }` evaluates its source once and iterates read-only through `len()` and `at(index)`. Vec and String provide this protocol; String iteration yields UTF-8 bytes. The element binding has iteration scope. `continue` advances to the next element; `break` leaves the loop. Mutable iteration and ranges are not implemented.

Operator precedence, from low to high: assignment; `||`; `&&`; comparisons/equality; bitwise operations; shifts; addition/subtraction; multiplication/division/remainder; casts; unary operators; calls, field access and indexing. Parentheses explicitly group expressions. Consult the parser's precedence table for distinctions among individual comparison and bitwise operators.

## Generics, Interfaces and Dynamic Values

```clust
interface Read<T: Copy> {
    read(self) -> T;
}

class Box<T: Copy>: Read<T> {
    value: T
    new(value: T) { self.value = value; }
    pub read(self) -> T { return self.value; }
}

fn identity<T>(value: T) -> T { return value; }
```

Generic parameters can have interface bounds, defaults and const parameters such as `const N: usize`. Type arguments use `<...>`. The compiler monomorphizes functions and nominal types. An interface can contain default method bodies; explicit implementations win and ambiguous defaults require an explicit method.

An interface in a parameter position supplies static generic semantics. `dyn Interface` requests an owning or borrowed dynamic interface value. Dynamic methods must have dispatchable nongeneric instance signatures. Parameterized interfaces can use concrete arguments, for example `dyn Read<i32>`.

Uniform generic inheritance forwards every class parameter, in order, to the base: `class Cat<U: Copy>: Animal<U>`. Renaming parameters does not change their meaning. Specialized or reordered inheritance arguments are outside the current lowering.

## Enums and Matching

```clust
use core.option.{Option};

fn read(value: Option<i32>) -> i32 {
    return match value {
        Option.Some(number) => number,
        Option.None => 0,
    };
}
```

Enums use `enum Name<T> { None, Some(T) }`, including named payload forms. Patterns support bindings, wildcards, literals, aggregates and variants; guards use `if`. Ownership and view rules apply to matched payloads. There is no implicit null or `?` propagation; use Option/Result and explicit match.

## Closures and Weak References

A closure uses `fn(value: i32) -> i32 { return value + offset; }`. Noncapturing function types use `(i32) => i32`. Capturing objects can implement a compatible single-method `call` interface. Mutable closures require `var` where invocation changes captures. See [capture semantics](semantics/closures-and-weak.md) for lifetime and higher-order restrictions.

Use `Weak<T>` from `core.weak`, `Weak<T>.new(target)`, `is_alive()` and `with<R>(callback)`. Access returns Option and cannot yield an escaping class borrow. There is no source unsafe block and no implemented weak field modifier.

## Modules and Rust Calls

Modules follow Rust's declaration and name-resolution model. Clust source uses `.` as the path separator.

```clust
mod animals;
use crate.animals.{Animal, Cat};
use core.vec.Vec;

mod helpers {
    pub(super) fn initial_age() -> u32 { return 0; }
}
```

`mod animals;` declares a child module and loads `animals.cl` or `animals/mod.cl`. Defining both files is an error. A child of `animals.cl` is loaded from `animals/child.cl` or `animals/child/mod.cl`. Inline declarations use `mod name { ... }` and may contain further declarations. A file is part of the compilation unit only when a `mod` declaration reaches it. The compilation entry file is the crate root.

`use` introduces a name into the current module's scope. Paths may begin with `crate`, `self` or one or more `super` segments. An unqualified path begins in the current module. Examples include `use crate.animals.Cat as Pet;`, `use crate.animals.{self as pets, Animal};` and `use crate.animals.*;`. Groups can be nested. Explicit names take precedence over glob imports; using an ambiguous glob name is an error. Qualified paths also work in expressions, type annotations and patterns.

Module-level declarations default to private visibility within their declaring module and its descendants. `pub` exposes a declaration, subject to the visibility of its enclosing modules. `pub(crate)`, `pub(super)`, `pub(self)` and `pub(in crate.parent)` restrict module-level visibility to the indicated ancestor and its descendants. `pub use` re-exports an accessible declaration without widening its declared visibility. Class fields and methods retain the class access rules described above.

Bundled library modules are available through `core`, for example `use core.option.Option;`. Rust implementation dependencies remain explicit scalar adapters and Cargo dependencies. Adapter paths use the same dot notation.

```clust
#[rust("std.cmp.max")]
extern "Rust" fn maximum(left: i32, right: i32) -> i32;
```

An imported Rust unsafe function is called using ordinary syntax. Unsafe implementation and emitted call blocks remain in Rust. Declared safety preconditions remain the caller's responsibility. `Clust.toml` and adapter signatures are described in [CLI and adapters](cli.md).
