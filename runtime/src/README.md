# runtime

Implement native allocation, scope cleanup and the Tokio concurrency API. Allocation and destruction preserve the caller's layout and ownership invariants. Task, channel, actor and mutex behavior lives in tokio_support.

| Source | Responsibilities or entry points |
| --- | --- |
| [allocation](allocation.rs) | `allocate`, `deallocate`, `insert`, `remove`, `count` |
| [lib](lib.rs) | Trusted Rust primitives for compiler-owned slots. These do not infer ownership. All linked records must stay at stable addresses until unlinked. |
| [tokio_support](tokio_support.rs) | `channel`, `Task`, `drop`, `join`, `cancel` |

Further reading: [allocation](allocation/README.md), [tokio_support](tokio_support/README.md).
