import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';

async function scenario(mode) {
  const events={}, intervals=new Map(), timeouts=new Map(), calls=[];
  let id=0, pending;
  const context={
    TextEncoder, Event:class {}, location:{href:'tauri://localhost/'},
    addEventListener:(name,fn)=>events[name]=fn,
    dispatchEvent:()=>{},
    setInterval:fn=>{intervals.set(++id,fn);return id;},
    clearInterval:key=>intervals.delete(key),
    setTimeout:(fn,ms)=>{timeouts.set(++id,{fn,ms});return id;},
    clearTimeout:key=>timeouts.delete(key),
    getComputedStyle:()=>{if(mode==='css')throw Error('CSS');return {getPropertyValue:()=>'',font:'x'.repeat(10000)};},
    document:{body:{},querySelector:()=>null,styleSheets:[],getElementById:()=>({childElementCount:1})},
    window:{__TAURI_INTERNALS__:{invoke:(command,args)=>{
      calls.push({command,args});
      if(command==='native_qa_report'){
        if(mode==='reject')return Promise.reject(Error('IPC'));
        return new Promise(resolve=>pending=resolve);
      }
      return Promise.resolve();
    }}},
  };
  vm.runInNewContext(readFileSync('src-tauri/src/native_qa.js','utf8'),context);
  for(let i=0;i<1000;i++)events['native-qa-record']({detail:{command:'accept',theme:{roles:{accent:'#123456'}}}});
  events.DOMContentLoaded();
  const report=[...intervals.values()][0];
  const first=report();
  await report();
  if(mode==='stall'){
    assert.equal(calls.length,1);
    assert.equal(calls[0].args.value.records.length,64);
    assert.ok(Buffer.byteLength(JSON.stringify(calls[0].args.value))<=65536);
    assert.equal(calls[0].args.value.font.length,256);
  } else {
    await first;
    await report(); // exceptions release the busy guard
    if(mode==='reject')assert.equal(calls.length,2);
  }
  [...timeouts.values()].find(t=>t.ms===60000).fn();
  await Promise.resolve();await Promise.resolve();
  assert.equal(intervals.size,0);
  assert.equal(timeouts.size,0);
  assert.equal(calls.at(-1).command,'native_qa_close');
  if(pending){pending();await first;}
}
for(const mode of ['stall','reject','css'])await scenario(mode);
console.log('PASS bounded telemetry, no overlapping reports, failure recovery, independent deadline');

assert.match(readFileSync('src-tauri/Cargo.toml','utf8'), /default = \[\]/);
assert.match(readFileSync('src-tauri/src/main.rs','utf8'), /#\[cfg\(feature = "native-qa"\)\]\s*let builder = builder.append_invoke_initialization_script/);
console.log('PASS native QA disabled by default and injection feature gated');
