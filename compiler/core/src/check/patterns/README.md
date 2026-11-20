# patterns

Check match arms and bind payloads with their inferred ownership permissions. Restrict guard effects and verify that all reachable variants are covered.

Module entry: [patterns](../patterns.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [binding](binding.rs) | `pattern` |
| [guards](guards.rs) | `readonly_guard`, `block`, `readonly_pattern` |
