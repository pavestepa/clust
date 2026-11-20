# pipeline

A pass collects declarations and signatures, checks queued bodies and builds a typed program. Return and storage contracts may request another pass. A ready result has also passed initialization, ownership, lifecycle and MIR verification.

Module entry: [pipeline](../pipeline.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [contracts](contracts.rs) | `verify` |
| [signatures](signatures.rs) | `Declarations`, `collect` |
