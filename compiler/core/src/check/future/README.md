# future

Scan later statements for overlapping uses before deciding whether a place may move. This is a look-ahead analysis of the synchronous checker.

Module entry: [future](../future.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [addresses](addresses.rs) | `address`, `block`, `statements` |
| [queries](queries.rs) | `call_view_needed`, `ownership_view_needed` |
| [reads](reads.rs) | `read` |
