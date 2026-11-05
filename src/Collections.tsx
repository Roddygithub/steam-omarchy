import {useLayoutEffect,useRef,useState} from 'react';
import {collectionIds,collectionNameError,renameCollection} from './preferences';
type Props={collections:Record<string,string[]>;ready:boolean;onChange:(collections:Record<string,string[]>,previous?:string,next?:string)=>void};
export function Collections({collections,ready,onChange}:Props){
 const [editing,setEditing]=useState<string|null>(null),[deleting,setDeleting]=useState<string|null>(null),[name,setName]=useState(''),[error,setError]=useState('');
 const heading=useRef<HTMLHeadingElement>(null),trigger=useRef<HTMLElement|null>(null);
 const restore=useRef<{label:string|null;removed:boolean}|null>(null);
 useLayoutEffect(()=>{if(!restore.current)return;const {label,removed}=restore.current;restore.current=null;const target=!removed&&label?heading.current?.parentElement?.querySelector<HTMLElement>('[aria-label="'+CSS.escape(label)+'"]'):null;(target||heading.current)?.focus()},[editing,deleting]);
 function close(removed=false){restore.current={label:trigger.current?.getAttribute('aria-label')||null,removed};setEditing(null);setDeleting(null);setError('')}

 return <section className="collections"><h2 tabIndex={-1} ref={heading}>Collections locales</h2><div className="panel"><p>Organisez vos jeux dans cette application. Supprimer une collection ne supprime aucun jeu ni fichier Steam.</p>{!ready&&<p role="status">Lecture seule · préférences indisponibles, sauvegarde désactivée.</p>}<AddCollection collections={collections} ready={ready} onChange={onChange}/>
 {Object.keys(collections).length===0&&<p>Aucune collection locale.</p>}
 {Object.keys(collections).map(key=><div className="collection-row" key={key}>
 {editing===key?<form onSubmit={e=>{e.preventDefault();if(!ready)return;const problem=collectionNameError(collections,name,key);setError(problem);if(problem)return;onChange(renameCollection(collections,key,name),key,name.trim());close(true)}} onKeyDown={e=>{if(e.key==='Escape'){e.stopPropagation();close()}}}>
 <label>Nouveau nom de « {key} »<input autoFocus value={name} disabled={!ready} aria-invalid={!!error} aria-describedby="collection-error" onChange={e=>setName(e.target.value)}/></label>{error&&<p id="collection-error" role="alert">{error}</p>}<button disabled={!ready}>Enregistrer le nom</button> <button type="button" onClick={()=>close()}>Annuler</button></form>
 :deleting===key?<div onKeyDown={e=>{if(e.key==='Escape'){e.stopPropagation();close()}}}><p>Supprimer la collection « {key} » et ses associations locales ? Les jeux et les favoris sont conservés.</p><button autoFocus onClick={()=>close()}>Annuler</button> <button disabled={!ready} onClick={()=>{if(!ready)return;onChange(Object.fromEntries(Object.entries(collections).filter(([n])=>n!==key)),key);close(true)}}>Confirmer la suppression</button></div>
 :<><strong>{key}</strong><span>{collectionIds(collections,key).length} jeu(x)</span><button disabled={!ready} aria-label={'Renommer '+key} onClick={e=>{trigger.current=e.currentTarget;setEditing(key);setDeleting(null);setName(key);setError('')}}>Renommer</button><button disabled={!ready} aria-label={'Supprimer '+key} onClick={e=>{trigger.current=e.currentTarget;setDeleting(key);setEditing(null);setError('')}}>Supprimer</button></>}
 </div>)}</div></section>
}
export function AddCollection({collections,ready,onChange,appId}:Props&{appId?:string}){
 const [name,setName]=useState(''),[error,setError]=useState('');
 const input=useRef<HTMLInputElement>(null);
 return <form className="collection-add" onSubmit={e=>{e.preventDefault();if(!ready)return;const key=name.trim();const problem=appId&&Object.hasOwn(collections,key)?'':collectionNameError(collections,name);setError(problem);if(problem)return;onChange({...collections,[key]:appId?[...new Set([...collectionIds(collections,key),appId])]:[]});setName('');input.current?.focus()}} onKeyDown={e=>{if(e.key==='Escape'){e.stopPropagation();setName('');setError('')}}}>
 <label>{appId?'Collection locale · nom existant ou nouveau':'Créer une collection locale'}<input ref={input} disabled={!ready} value={name} aria-invalid={!!error} onChange={e=>setName(e.target.value)}/></label><button disabled={!ready}>{appId?'Ajouter le jeu':'Créer la collection'}</button>{error&&<p role="alert">{error}</p>}{appId&&<a href="#preferences">Gérer les collections</a>}
 </form>
}
