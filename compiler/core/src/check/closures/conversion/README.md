# conversion

Convert closure expressions inside a typed lexical scope. Capture analysis identifies free bindings. Environment construction materializes fields, a constructor and the call method for an escaping callable.

Module entry: [conversion](../conversion.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [capture](capture.rs) | `convert_closure` |
| [environment](environment.rs) | `Captures`, `environment` |
