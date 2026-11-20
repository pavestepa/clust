const vscode = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let client;

async function activate(context) {
    const path = require('node:path');
    const bundled = context.asAbsolutePath(path.join('bin', process.platform === 'win32' ? 'clust-lsp.exe' : 'clust-lsp'));
    const command = vscode.workspace.getConfiguration('clust').get('serverPath') ||
        (require('node:fs').existsSync(bundled) ? bundled : 'clust-lsp');
    const watcher = vscode.workspace.createFileSystemWatcher('**/{*.cl,Clust.toml}');
    context.subscriptions.push(watcher);
    client = new LanguageClient('clust', 'Clust', { command, transport: TransportKind.stdio }, {
        synchronize: { fileEvents: watcher },
        documentSelector: [{ language: 'clust', scheme: 'file' }, { language: 'clust', scheme: 'untitled' }]
    });
    await client.start();
}

async function deactivate() {
    if (client && client.isRunning()) await client.stop();
}

module.exports = { activate, deactivate };
