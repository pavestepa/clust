# dispatch

Discover dynamic interface implementations, construct a tagged payload per hierarchy root, and generate method dispatch over concrete variants. Field accessors share the generated dispatch rules.

Module entry: [dispatch](../dispatch.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [interfaces](interfaces.rs) | `discover` |
| [methods](methods.rs) | `emit` |
| [payload](payload.rs) | `payload` |
