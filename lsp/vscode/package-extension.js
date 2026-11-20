const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '../..');
const suffix = process.platform === 'win32' ? '.exe' : '';
const binary = 'clust-lsp' + suffix;
const output = execFileSync('cargo', ['build', '--locked', '--release', '-p', 'clust-lsp', '--message-format=json'], { cwd: root, encoding: 'utf8' });
const artifact = output.trim().split('\n').map(line => JSON.parse(line)).find(event => event.target?.name === 'clust-lsp' && event.executable);
fs.mkdirSync(path.join(__dirname, 'bin'), { recursive: true });
fs.copyFileSync(artifact.executable, path.join(__dirname, 'bin', binary));
const target = process.platform + '-' + process.arch;
execFileSync(process.execPath, [require.resolve('@vscode/vsce/vsce'), 'package', '--target', target, '--allow-missing-repository', '--skip-license'], {cwd: __dirname, stdio: 'inherit'});
