export function collectionIds(collections:Record<string,string[]>, key:string):string[]{return Object.hasOwn(collections,key)&&Array.isArray(collections[key])?collections[key]:[]}
