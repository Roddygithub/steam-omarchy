import assert from 'node:assert/strict';
import {collectionIds,collectionNameError,renameCollection,validPrefs} from '../src/preferences.ts';
const p=validPrefs(JSON.parse('{"favorites":["440"],"collections":{"First":["440","730"],"Empty":[],"__proto__":["570"]}}'));
assert.deepEqual(collectionIds(p.collections,'constructor'),[]);
assert.deepEqual(collectionIds(p.collections,'__proto__'),['570']);
assert.ok(collectionNameError(p.collections,'   '));
assert.ok(collectionNameError(p.collections,'x'.repeat(81)));
assert.equal(collectionNameError(p.collections,'x'.repeat(80)),'');
assert.ok(collectionNameError(p.collections,' Empty '));
assert.equal(renameCollection(p.collections,'First','Empty'),p.collections);
const renamed=renameCollection(p.collections,'First','  New  ');
assert.deepEqual(renamed.New,['440','730']);
assert.equal(Object.hasOwn(renamed,'First'),false);
assert.deepEqual(p.favorites,['440']);
for(const key of ['__proto__','constructor','toString']){
 const base={First:['440']};
 const next=renameCollection(base,'First',key);
 assert.equal(Object.hasOwn(next,key),true);
 assert.deepEqual(collectionIds(next,key),['440']);
 assert.equal(Object.getPrototypeOf(next),Object.prototype);
}
assert.deepEqual(validPrefs({collections:{Bad:3,Valid:['440',{},'no']},favorites:null}).collections,{Bad:[],Valid:['440']});
assert.deepEqual(validPrefs(null).favorites,[]);
assert.deepEqual(collectionIds({broken:undefined},'broken'),[]);
console.log('Collection validation, rename, own keys, IDs and malformed preferences passed');
