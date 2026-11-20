# structured

Select scalar functions eligible for structured Rust emission, then lower their expressions and blocks. Eligibility must preserve arithmetic traps, identity and cleanup behavior.

Module entry: [structured](../structured.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [emission](emission.rs) | `scalar_name`, `block`, `emit` |
