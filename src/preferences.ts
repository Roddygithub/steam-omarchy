export function collectionIds(collections:Record<string,string[]>, key:string):string[]{return Object.hasOwn(collections,key)&&Array.isArray(collections[key])?collections[key]:[]}

export type Prefs={palette:string;accent:string;favorites:string[];collections:Record<string,string[]>};
export const defaults:Prefs={palette:'system',accent:'#b4a5ef',favorites:[],collections:{}};
export function validPrefs(value:unknown):Prefs {
 const p=value&&typeof value==='object'?value as Record<string,unknown>:{};
 const ids=(v:unknown)=>Array.isArray(v)?v.filter((x):x is string=>typeof x==='string'&&/^[1-9][0-9]{0,9}$/.test(x)):[];
 const collections:Record<string,string[]>={};
 if(p.collections&&typeof p.collections==='object'&&!Array.isArray(p.collections))for(const [name,v] of Object.entries(p.collections)){if(name.length<=80)Object.defineProperty(collections,name,{value:ids(v),enumerable:true,configurable:true,writable:true})}
 return {palette:typeof p.palette==='string'&&['system','cold','warm','light','custom'].includes(p.palette)?p.palette:'system',accent:typeof p.accent==='string'&&/^#[0-9a-f]{6}$/i.test(p.accent)?p.accent:defaults.accent,favorites:ids(p.favorites),collections};
}

export function collectionNameError(collections:Record<string,string[]>, value:string, previous?:string):string {
 const name=value.trim();
 if(!name||name.length>80)return 'Saisissez un nom de 1 à 80 caractères.';
 if(name!==previous&&Object.hasOwn(collections,name))return 'Une collection porte déjà ce nom.';
 return '';
}
export function renameCollection(collections:Record<string,string[]>, previous:string, value:string):Record<string,string[]> {
 const name=value.trim();
 if(!Object.hasOwn(collections,previous)||collectionNameError(collections,value,previous))return collections;
 return Object.fromEntries(Object.entries(collections).map(([key,ids])=>[key===previous?name:key,ids]));
}
