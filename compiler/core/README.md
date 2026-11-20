# core

The core consumes a resolved frontend module. Semantic checking produces a typed program. MIR lowering makes ownership, control flow and cleanup explicit. The Rust emitter and the two reference interpreters consume these representations.

See the [implementation guide](src/README.md).
