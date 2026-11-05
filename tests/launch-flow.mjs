// Actual UI callback, synthetic IPC and instant timers; no native IPC or Steam.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import ts from 'typescript';
const source=readFileSync('src/main.tsx','utf8');
const callback=source.match(/async function launch\(g:Entry\).*\n/)[0];
for(const mode of ['observed','unobserved','handler-timeout','status-error']) {
 const messages=[],busy=[],calls=[];
 const context={desktop:true,setLaunching:v=>busy.push(v),setMessage:v=>messages.push(v),
 setTimeout:f=>f(),invoke:async(command,args)=>{
  calls.push([command,args]);
  if(mode==='handler-timeout')throw Error('Délai du gestionnaire');
  if(command==='request_launch')return 'Demande remise · non confirmée';
  if(mode==='status-error')throw Error('Observation impossible');
  return mode==='observed';
 }};
 vm.createContext(context);
 vm.runInContext(ts.transpile(callback+';globalThis.run=launch;'),context);
 await context.run({appId:'440'});
 assert.deepEqual(busy,[true,false]);
 assert.deepEqual(JSON.parse(JSON.stringify(calls[0])),['request_launch',{appId:'440'}]);
 const count=mode==='handler-timeout'?0:mode==='unobserved'?10:1;
 assert.equal(calls.length,1+count);
 for(const [command,args] of calls.slice(1)){
  assert.equal(command,'launch_status');
  assert.deepEqual(JSON.parse(JSON.stringify(args)),{appId:'440'});
 }
 if(mode!=='handler-timeout')assert.equal(messages[1],'Demande remise · non confirmée');
 if(mode==='status-error')assert.ok(messages.at(-1).includes('Observation impossible'));
 assert.ok(messages[0].includes('en cours'));
 if(mode==='observed')assert.ok(messages.at(-1).includes('nouvelle instance non confirmée'));
 if(mode==='unobserved')assert.ok(messages.at(-1).includes('aucun processus'));
 if(mode==='handler-timeout')assert.equal(calls.length,1);
}
console.log('PASS actual UI callback with synthetic IPC: observed, no process, timeout, status failure; button reset');
