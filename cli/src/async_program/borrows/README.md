# borrows

Infer value, shared and exclusive parameter modes from their uses and propagate contracts across calls. Rewrite signatures and arguments before rustc checks each future's actual lifetime.

Module entry: [borrows](../borrows.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [inference](inference.rs) | `lower` |
| [rewrite](rewrite.rs) | `call`, `bindings`, `Names`, `visit_pat_ident` |
