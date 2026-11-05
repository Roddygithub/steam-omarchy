// Execute the real initialization effect with isolated browser-state stubs.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
const source=readFileSync('src/main.tsx','utf8');
const effect=source.split('\n').find(line=>line.startsWith('useEffect(()=>{const h=()=>setRoute('));
assert.ok(effect,'Initialization effect exists');
for(const mode of ['valid','corrupt','denied']){
 const events=new Map(),routes=[],ready=[],messages=[];
 const context={useEffect:fn=>{context.cleanup=fn()},window:{addEventListener:(k,v)=>events.set(k,v),removeEventListener:(k,v)=>{assert.equal(events.get(k),v);events.delete(k)}},location:{hash:'#home'},scan:()=>{},desktop:false,validPrefs:v=>v,setPrefs:()=>{},setReady:v=>ready.push(v),setRoute:v=>routes.push(v),setMessage:v=>messages.push(v),localStorage:{getItem:()=>{if(mode==='denied')throw Error('Storage unavailable');return mode==='valid'?'{}':'{broken'}}};
 vm.createContext(context);vm.runInContext(effect,context);
 assert.equal(events.size,1);
 context.location.hash='#preferences';events.get('hashchange')();
 context.location.hash='#library';events.get('hashchange')();
 assert.deepEqual(routes,['#preferences','#library']);
 assert.deepEqual(ready,mode==='valid'?[true]:[],'Failed read must not enable autosave');
 assert.equal(messages.length,mode==='valid'?0:1);
 context.cleanup();assert.equal(events.size,0);
}
console.log('PASS valid/corrupt/denied demo storage: navigation, cleanup and save protection');
