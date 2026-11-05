;
// Compiled only with native-qa. Observe real IPC; never synthesize theme events.
(() => {
 const records=[]; let latest=null, last='', busy=false, closed=false;
 const bounded = value => typeof value === 'string' ? value.slice(0,256) : null;
 addEventListener('pagehide',()=>close());
 let timer, stop, deadline, sentinel;
 const invoke=(command,args)=>window.__TAURI_INTERNALS__.invoke(command,args);
 const close=()=>{
   if(closed)return;
   closed=true;
   clearInterval(timer);clearInterval(stop);clearTimeout(deadline);clearTimeout(sentinel);
   Promise.resolve().then(()=>invoke('native_qa_close')).catch(()=>{});
 };
 deadline=setTimeout(close,60000);
 addEventListener('native-qa-record',e=>{
   if(closed || !e.detail || typeof e.detail.command!=='string')return;
   records.push({command:bounded(e.detail.command)});
   if(records.length>64)records.shift();
   if(e.detail.theme){
     try {
       const serialized=JSON.stringify(e.detail.theme);
       if(new TextEncoder().encode(serialized).length<=8192)latest=JSON.parse(serialized);
     } catch { /* Ignore malformed QA records. */ }
   }
 });
 addEventListener('DOMContentLoaded',()=>{
   if(closed)return;
   timer=setInterval(async()=>{
     if(busy || closed)return;
     busy=true;
     try {
     const css=getComputedStyle(document.body);
     const value={url:bounded(location.href),font:bounded(css.font),headerDisplay:document.querySelector('header')?getComputedStyle(document.querySelector('header')).display:null,sheets:[...document.styleSheets].slice(0,16).map(s=>({href:bounded(s.href),rules:s.cssRules.length})),theme:latest,roles:Object.fromEntries(['surface','foreground','accent','muted','border','focus'].map(k=>[k,bounded(css.getPropertyValue('--'+k).trim())])),background:bounded(css.backgroundColor),color:bounded(css.color),scheme:bounded(css.colorScheme),children:document.getElementById('root').childElementCount,records:records.slice()};
     const serialized=JSON.stringify(value);
     if(serialized!==last && new TextEncoder().encode(serialized).length<=65536){
       await invoke('native_qa_report',{value});last=serialized;
     }
     } catch { /* Deadline remains active after CSS or IPC failure. */ }
     finally { busy=false; }
   },250);
   // Harness requests unmount through a fixture palette with a unique accent.
   stop=setInterval(()=>{
     if(latest?.roles.accent==='#654321'){
       clearInterval(stop);dispatchEvent(new Event('native-qa-unmount'));
       sentinel=setTimeout(close,1000);
     }
   },100);
 });
})();
