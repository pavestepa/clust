# lifecycle

Lower destructor declarations to checked function entries. Validate effects on the finalizing object and schedule the correct specialized body for cleanup.

Module entry: [lifecycle](../lifecycle.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [destructors](destructors.rs) | `is_destructor` |
| [scheduling](scheduling.rs) | `schedule_destructors` |
| [validation](validation.rs) | `validate` |
