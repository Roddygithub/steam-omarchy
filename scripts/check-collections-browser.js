// Evaluate in T3's hidden web preview at /?demo#preferences. Uses only demo storage.
(async()=>{
 if(!new URLSearchParams(location.search).has('demo'))throw Error('Explicit web demo required');
 const check=(v,n)=>{if(!v)throw Error(n)};
 const wait=()=>new Promise(r=>setTimeout(r,100));
 const click=text=>{const b=[...document.querySelectorAll('button')].find(b=>b.textContent===text);check(b,text);b.click();return b};
 const fill=(input,value)=>{Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(input,value);input.dispatchEvent(new Event('input',{bubbles:true}))};
 const submit=async(form,value)=>{fill(form.querySelector('input'),value);await wait();form.requestSubmit();await wait()};
 const form=()=>document.querySelector('.collections .collection-add');
 await submit(form(),'QA empty');
 check(document.querySelector('[aria-label="Renommer QA empty"]'),'empty collection visible');
 document.querySelector('[aria-label="Renommer QA empty"]').click();await wait();
 await submit(document.querySelector('.collection-row form'),'   ');
 check(document.querySelector('[role=alert]'),'empty name error');
 await submit(document.querySelector('.collection-row form'),'QA renamed');
 check(document.querySelector('[aria-label="Supprimer QA renamed"]'),'rename applied');
 check(document.activeElement.textContent==='Collections locales','rename focus fallback');
 document.querySelector('[aria-label="Supprimer QA renamed"]').click();await wait();
 check(document.body.textContent.includes('Les jeux et les favoris sont conservés'),'explicit deletion explanation');
 click('Annuler');await wait();
 check(document.activeElement.getAttribute('aria-label')==='Supprimer QA renamed','cancel focus');
 document.querySelector('[aria-label="Renommer QA renamed"]').click();await wait();
 document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}));await wait();
 check(!document.querySelector('.collection-row form'),'Escape cancels');
 document.querySelector('[aria-label="Supprimer QA renamed"]').click();await wait();click('Confirmer la suppression');await wait();
 check(!document.querySelector('[aria-label="Supprimer QA renamed"]'),'deleted');
 await submit(form(),'QA retained');
 await wait();
 return {passed:true,width:innerWidth,overflow:document.documentElement.scrollWidth>innerWidth};
})()
