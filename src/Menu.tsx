import {useEffect,useRef,useState} from 'react';
import type {ReactNode} from 'react';
// Native disclosure, with scoped keyboard navigation; no menu dependency.
export function Menu({label,children}:{label:string;children:ReactNode}){
 const root=useRef<HTMLDetailsElement>(null);
 const [opened,setOpened]=useState(false);
 useEffect(()=>{if(!opened)return;const outside=(e:PointerEvent)=>{if(!root.current?.contains(e.target as Node))root.current?.removeAttribute('open')};document.addEventListener('pointerdown',outside);return()=>document.removeEventListener('pointerdown',outside)},[opened]);
 return <details className="menu" ref={root} onToggle={e=>setOpened(e.currentTarget.open)} onKeyDown={e=>{
 const el=root.current!;const summary=el.querySelector('summary')!;
 if(e.key==='Escape'){e.preventDefault();e.stopPropagation();el.open=false;summary.focus();return}
 if((e.target as HTMLElement).tagName==='SELECT')return;
 if(e.key==='ArrowLeft'){e.preventDefault();e.stopPropagation();el.open=false;summary.focus();return}
 if(['ArrowDown','ArrowUp','Home','End','ArrowRight'].includes(e.key)){
 e.preventDefault();e.stopPropagation();el.open=true;
 const items=Array.from(el.querySelectorAll<HTMLElement>('button:not(:disabled), a, select, summary')).filter(x=>x===summary||x.closest('details')===el||(x.tagName==='SUMMARY'&&x.parentElement?.parentElement?.closest('details')===el));
 const i=items.indexOf(document.activeElement as HTMLElement);
 items[e.key==='Home'?1:e.key==='End'?items.length-1:e.key==='ArrowUp'?(i<=1?items.length-1:i-1):(i+1)%items.length]?.focus();
 }
 }}><summary>{label}</summary><div className="menu-panel" onClick={e=>{if((e.target as HTMLElement).closest('button,a')){const details=(e.target as HTMLElement).closest('details');if(details){let outer=details;while(outer.parentElement?.closest('details'))outer=outer.parentElement.closest('details')!;outer.open=false;outer.querySelector('summary')?.focus()}}}}>{children}</div></details>
}
