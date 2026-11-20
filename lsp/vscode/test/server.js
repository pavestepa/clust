// Protocol regression tests against the same JSON-RPC client used by the extension.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const {pathToFileURL} = require('node:url');
const {spawn} = require('node:child_process');
const rpc = require('vscode-jsonrpc/node');
const root = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), 'clust language '));
const child = spawn(process.argv[2]);
const client = rpc.createMessageConnection(new rpc.StreamMessageReader(child.stdout),new rpc.StreamMessageWriter(child.stdin));
const diagnostics = new Map();
client.onNotification('textDocument/publishDiagnostics', p => diagnostics.set(p.uri,p));
client.listen();
const uri = pathToFileURL(path.join(root,'main.cl')).href;
const other = pathToFileURL(path.join(root,'math.cl')).href;
const source = 'mod math;\nuse crate.math.twice;\nfn main()->i32 {\nlet result = twice(2);\nreturn result;\n}';
const moduleText = 'pub fn twice(x:i32)->i32 { return x+x; }';
function open(u,text){return client.sendNotification('textDocument/didOpen',{textDocument:{uri:u,text,version:1,languageId:'clust'}})}
function change(u,text,version){return client.sendNotification('textDocument/didChange',{textDocument:{uri:u,version},contentChanges:[{text}]})}
function query(method,line,character,extra={}){
    return client.sendRequest('textDocument/'+method,{textDocument:{uri},position:{line,character},...extra});
}
async function wait(check){
    for(let i=0;i<1200;i++){if(check())return;await new Promise(r=>setTimeout(r,25))}
    throw Error('Timed out: '+JSON.stringify([...diagnostics]));
}
async function run(){
    fs.writeFileSync(path.join(root,'main.cl'),source);
    fs.writeFileSync(path.join(root,'math.cl'),moduleText);
    fs.writeFileSync(path.join(root,'Clust.toml'),'[package]\nname="sample"\nentry="main.cl"\n');
    await client.sendRequest('initialize',{capabilities:{}});await client.sendNotification('initialized',{});
    await open(uri,source);await wait(()=>diagnostics.has(uri));assert.deepEqual(diagnostics.get(uri).diagnostics,[]);
    const definition=await query('definition',3,14);assert.equal(definition.uri,other);
    assert.match((await query('hover',3,14)).contents.value,/twice\(x:i32\)/);
    assert.equal((await query('signatureHelp',3,19)).signatures[0].parameters.length,1);
    assert.equal((await query('references',3,14,{context:{includeDeclaration:true}})).length,3);
    const rename=await query('rename',3,14,{newName:'double'});assert.equal(rename.documentChanges.length,2);
    await assert.rejects(query('rename',3,14,{newName:'main'}));
    assert((await query('formatting',0,0,{options:{tabSize:4,insertSpaces:true}})).length>0);
    const full={start:{line:0,character:0},end:{line:5,character:1}};
    assert((await query('inlayHint',0,0,{range:full})).some(h=>h.label.includes('i32')));
    await open(other,moduleText);await change(other,moduleText.replace('x:i32','x:bool'),2);
    await wait(()=>diagnostics.get(other)?.version===2);
    assert([...diagnostics.values()].some(d=>d.diagnostics.length>0));
    await change(other,moduleText,3);await wait(()=>diagnostics.get(other)?.version===3);
    await change(uri,source.replace('use crate.math.twice;',''),2);
    await wait(()=>diagnostics.get(uri)?.version===2);
    const fixes=await query('codeAction',0,0,{range:full,context:{diagnostics:diagnostics.get(uri).diagnostics}});
    assert(fixes.some(a=>a.title==='Import crate.math.twice'));
    await change(uri,'async fn main()->i32 {return false;}',3);
    await wait(()=>diagnostics.get(uri)?.version===3);assert.match(diagnostics.get(uri).diagnostics[0].message,/mismatched types/);
    const counter = fs.readFileSync(path.join(__dirname,'../../../examples/clust_counter/main.cl'),'utf8');
    await change(uri, counter+'{',4);
    await change(uri, counter,5);
    await wait(()=>diagnostics.get(uri)?.version===5);assert.deepEqual(diagnostics.get(uri).diagnostics,[]);
    const lines=counter.split('\n');
    const call=lines.findIndex(l=>l.includes('counter.add('));
    const members=await query('completion',call,lines[call].indexOf('.')+1);
    assert(members.some(m=>m.label==='add'));assert(!members.some(m=>m.label==='value'));
    const noSemi=counter.replace('return counter.read();','return counter.read()');
    await change(uri,noSemi,6);await wait(()=>diagnostics.get(uri)?.version===6);
    const missing=await query('codeAction',0,0,{range:full,context:{diagnostics:diagnostics.get(uri).diagnostics}});
    assert(missing.some(a=>a.title==='Insert missing semicolon'));
    await change(uri,source,7);await wait(()=>diagnostics.get(uri)?.version===7);
    await client.sendNotification('textDocument/didClose',{textDocument:{uri:other}});
    fs.renameSync(path.join(root,'math.cl'),path.join(root,'moved.cl'));
    await client.sendNotification('workspace/didChangeWatchedFiles',{changes:[{uri:other,type:3}]});
    await wait(()=>diagnostics.get(uri)?.diagnostics.some(e=>e.message.includes('module file not found')));
    await client.sendRequest('shutdown');await client.sendNotification('exit');
}
run().then(()=>console.log('Clust LSP integration passed')).catch(e=>{console.error(e);process.exitCode=1}).finally(()=>{client.dispose();child.kill();fs.rmSync(root,{recursive:true,force:true})});
