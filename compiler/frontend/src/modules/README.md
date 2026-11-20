# modules

Load reachable files, check module visibility and resolve paths. Rewrite names throughout declarations, types, patterns and expressions before passing the combined AST to core.

Module entry: [modules](../modules.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [builtins](builtins.rs) | `builtin_requests`, `builtins` |
| [declarations](declarations.rs) | `declaration`, `generics`, `item`, `function`, `parameters` |
| [expressions](expressions.rs) | `block`, `expr`, `expression_path` |
| [linker](linker.rs) | `link` |
| [names](names.rs) | `item_name`, `path`, `name` |
| [patterns](patterns.rs) | `pattern` |
| [types](types.rs) | `ty`, `arguments` |
| [visibility](visibility.rs) | `visibility`, `accessible`, `resolve` |
