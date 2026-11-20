# runtime

Implement native allocation, scope cleanup and the Tokio concurrency API. Allocation and destruction preserve the caller's layout and ownership invariants. Task, channel, actor and mutex behavior lives in tokio_support.

See the [implementation guide](src/README.md).
