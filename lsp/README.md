# Clust language server

This project owns the Rust stdio server, its VS Code client and native extension packaging. It is a Cargo workspace member. Install the server alone with `cargo install --path lsp --locked` from the repository root. [VS Code installation and packaging](vscode/README.md)

Opening or editing a document schedules background project analysis. The server reads open buffers before disk files and uses the compiler's manifest resolver, module loader, type checker and ownership checks. It publishes source ranges in UTF-16 and includes compiler notes as related locations. Changes arriving during analysis supersede its diagnostics. The VS Code client also watches Clust sources and manifests.

Hover, completion, argument help, definition and references use a source index that can operate on incomplete code. It handles named declarations, scoped locals, explicit parameter types and direct class construction. Resolution of aliases, inherited members, generic substitutions and chained call results is limited. Rename checks the proposed synchronous project with the compiler and returns versioned edits. Names with conflicts or an invalid resulting project are rejected.

Inlay hints expose inferred local types and ownership modes for checked calls. Enable inlay hints in the editor to see them. Quick fixes insert missing semicolons and suggest imports from reachable project modules. Formatting adjusts block indentation using the editor's indentation settings.

Async files use the CLI's async lowering followed by an isolated offline Cargo check of generated Rust. This requires Cargo and the project's pinned Tokio dependency in the local cache. Generated Rust errors currently appear at the start of the async document. Semantic navigation and hints have their best coverage in synchronous code.

## Development

```sh
cargo build --locked -p clust-lsp
npm ci --prefix lsp/vscode
node lsp/vscode/test/server.js target/debug/clust-lsp
```

The integration test starts a real client and server, changes an imported module in memory, checks navigation and edits, and sends an invalid async function through Cargo.
