# testing

Build explicit backend fixtures for integration tests. Production parsing stays separate from the compiler-tests feature.

Module entry: [testing](../testing.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [record](record.rs) | Build inline nominal IR directly. No parsed class or constructor is rewritten. |
| [records](records.rs) | Backend-only record definitions grouped by the suite that exercises them. |

Further reading: [records](records/README.md).
