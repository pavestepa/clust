# Clust for VS Code

Install the VSIX built for your operating system and architecture through **Extensions: Install from VSIX...** in the command palette. Open a Clust project folder and a `.cl` file. The extension starts its bundled language server. If another extension claims the same suffix, select **Clust** in the language mode picker.

Use F12 for a definition, Shift+F12 for references, F2 for rename, and **Trigger Suggest** for completions. Enable **Editor: Inlay Hints** to see available type and ownership information. Async type checking requires Cargo and Tokio 1.54.1 in the local Cargo cache.

## Build a package on the target machine

Run from the repository root with Rust, Cargo and Node.js 22 or newer:

```sh
npm ci --prefix lsp/vscode
npm run package --prefix lsp/vscode
```

The resulting platform-specific VSIX in this directory contains the optimized Rust binary, client dependencies and syntax grammar. Set `clust.serverPath` in User Settings to test a different server binary. For extension development, launch `code --extensionDevelopmentPath="$PWD/lsp/vscode" .` from the repository root. On macOS the launcher is also available at `/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code`.
