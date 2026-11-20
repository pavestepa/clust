# Closures and Weak References

This document specifies the implemented synchronous subset. The single-owner and exclusive-access rules also apply to generated closure environments. Rejection of a program outside this subset is not a proof that the broader language forbids it.

## Closures

A closure uses `fn(parameters) -> Result { body }`. Parameters have explicit types; an omitted result means unit. Noncapturing closures are function values with type `(Parameter, ...) => Result`. Capturing closures are callable objects. A compatible, nongeneric, single-method interface whose method is named `call` can describe a capturing closure, including an owning `dyn` result.

```clust
interface Read { call(self) -> i32; }

fn make(value: i32) -> dyn Read {
    return fn() -> i32 { return value; };
}

fn main() -> i32 {
    let read = make(7);
    return read();
}
```

Capture analysis respects lexical bindings, including local and pattern shadowing. Copy values can enter the environment by value. Resource captures use ordinary ownership and storage inference: an environment can retain a checked read-only view or take ownership when no incompatible use remains. Escaping owned environments destroy their resources when the environment's owner is destroyed. There is no implicit cloning or reference counting.

A closure that mutates an owned capture needs a `var` binding and a mutable callable receiver. When a mutable local is also used outside the closure, the compiler creates a scoped exclusive capture. The borrow lasts through the last closure call, not necessarily through the end of the surrounding scope:

```clust
var count = 0;
var next = fn() -> i32 { count += 1; return count; };
assert(next() == 1);
assert(next() == 2);
assert(count == 2);
```

Mutating `count` before a later `next()` call is rejected. Scoped closures borrowing mutable locals currently support direct calls in the declaring scope; storing, returning or passing these closures as values is rejected. An owned mutable capture can escape through a compatible mutable callable interface. Capture types not recoverable by the current source inference need an explicit annotation on the captured binding. General FnOnce-style extraction and higher-order scoped mutable capture are not implemented.

## Weak Identity

Import `Weak` from `core.weak`. `Weak<T>.new(target)` observes an existing non-Copy class place without retaining its payload. A weak target must own its fields; arbitrary objects containing borrowed fields are rejected. A polymorphic descriptor may refer to an otherwise owning concrete target.

- `handle.is_alive() -> bool` reports whether destruction has started.
- `handle.with<R: Copy>(callback: (T) => R) -> Option<R>` checks expiration and invokes a read-only callback synchronously while the target remains valid.
- An expired access returns `Option.None` without invoking the callback.
- The callback must be a noncapturing function value and must not consume or mutate the target. Its result is restricted to scalars, tuples and arrays of scalars. A borrowed class result cannot escape this operation.

`Weak<Animal>` created from a `Cat` preserves the complete Cat identity and invokes Cat overrides. Moving an owner does not create a new target identity. A generation number distinguishes a new allocation reusing an old address. Weak access never resurrects a destroyed object.

Destruction invalidates weak identity before running the object's own destructor. Creating another weak handle inside that destructor produces an expired handle. Construction may observe a receiver only after all required fields have been initialized; partial construction still invalidates registered identity when unwinding.

A parent can own a child while the child holds `Weak<Parent>`. This is a valid weak back-edge, not a strong ownership cycle. See `examples/clust_weak_backedge/main.cl`. The implemented API is `Weak<T>`; a `weak` field modifier is not part of this subset.
