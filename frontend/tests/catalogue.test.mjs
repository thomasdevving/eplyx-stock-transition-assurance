import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile,mkdtemp,mkdir,writeFile,rm } from 'node:fs/promises';
import {resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {randomUUID,createHash} from 'node:crypto';
import {CatalogueStore,parseProducts,catalogueFromCapture,SOURCE_URL} from '../catalogue.mjs';
import {AnalysisService,validateSelection,parseEngineResult,executeEngine, engineExecutable} from '../analysis-service.mjs';
import {CurrentResult} from '../src/analysis.js';
const root=resolve('.'),store=new CatalogueStore(root),catalogue=await store.current();
const retained=JSON.parse(await readFile(resolve(store.directory,`${catalogue.version}.json`),'utf8'));
const finish=async service=>{while(service.running||service.queue.length)await new Promise(r=>setTimeout(r,10));};
const html=products=>'<script>self.__next_f.push('+JSON.stringify([1,'0:'+JSON.stringify({products})+'\n'])+')</script>';
const one={splMint:catalogue.entries[0].mint,name:'Duplicate',symbol:'DUP',decimals:9};
const request=entry=>({asset:'mint',review:'current',stage:null,check:null,cluster:'solana-mainnet',mint:entry.mint,catalogue_version:catalogue.version,sample_accounts:false});
const result=(selection,pin)=>({schema_version:2,kind:'current-inspection',asset:{mint:selection.mint},selection:pin,inspection:{status:'Completed'},lifecycle_event:null,readiness:null,execution_performed:false,local_execution_performed:false,authorization:false,funds_moved:false,signer_assumed_locally:false,signer_possession_known:false,paths:['OfficialTransition','Redemption','SecondaryMarketExit','Transfer','Withdrawal'].map(path=>({path,status:'NotTested'}))});
test('retained first-party flight JSON provides nine real canonical mint identities, no script execution',()=>{
 assert.equal(catalogueFromCapture(retained).entries.length,9);assert.equal(catalogue.source_url,SOURCE_URL);assert.equal(catalogue.source_status,'SavedSource');
 assert.equal(new Set(catalogue.entries.map(e=>e.cluster+':'+e.mint)).size,9);
 assert.throws(()=>parseProducts('<script>globalThis.bad=true</script>'));assert.equal(globalThis.bad,undefined);
 assert.throws(()=>catalogueFromCapture({...retained,content:retained.content+'tampered'}));
});
test('parser rejects missing and malformed fields; duplicate names remain distinct; conflicting assertions retained',()=>{
 for(const field of ['splMint','name','symbol','decimals']){const bad={...one};delete bad[field];assert.throws(()=>parseProducts(html([bad])));}
 for(const decimals of [-1,256,'9'])assert.throws(()=>parseProducts(html([{...one,decimals}])));
 const entries=parseProducts(html([one,{...one,splMint:catalogue.entries[1].mint},{...one,symbol:'OTHER'},one]));
 assert.equal(entries.length,2);assert.equal(entries[0].assertions.length,2);assert.equal(entries[1].assertions[0].name,'Duplicate');
});
test('catalogue binding rejects a reviewed version combined with another mint',async()=>{
 await assert.rejects(store.reference(catalogue.version,'So11111111111111111111111111111111111111112'),/CatalogueMintMismatch/);
 await assert.rejects(store.reference('../arbitrary',one.splMint),/InvalidCatalogue/);
 for(const bad of [{...request(catalogue.entries[0]),cluster:'devnet'},{...request(catalogue.entries[0]),mint:'bad'},{...request(catalogue.entries[0]),source_url:'https://attacker'},{...request(catalogue.entries[0]),sample_accounts:'true'}])assert.throws(()=>validateSelection(bad));
});
test('real Solana parser accepts custom and off-curve addresses and rejects malformed addresses',async()=>{
 const {stdout}=await executeEngine(resolve(root,engineExecutable),['validate-address','--mint','So11111111111111111111111111111111111111112','--mint','11111111111111111111111111111111'],{cwd:root});assert.equal(JSON.parse(stdout).length,2);
 await assert.rejects(executeEngine(resolve(root,engineExecutable),['validate-address','--mint','O'.repeat(44)],{cwd:root}));
});
test('same mint and immutable source reference flow into structured arguments, captures, result, refresh and restart',async()=>{
 const directory=await mkdtemp(resolve(tmpdir(),'eplyx-m2-service-'));let calls=0;
 const service=new AnalysisService({root,directory,runner:async(_exe,args)=>{calls++;const pin=JSON.parse(await readFile(args[4],'utf8'));assert.equal(args[2],pin.mint);await writeFile(args[6],JSON.stringify({selection:pin,n:calls}));return {code:0,stdout:JSON.stringify(result(request(catalogue.entries[1]),pin))};}});await service.initialize();
 try{const selection=request(catalogue.entries[1]);const first=await service.submit(selection,randomUUID());await finish(service);assert.equal(first.status,'Completed');const bytes=await service.artifact(first.id,true);const next=await service.submit(selection,randomUUID());await finish(service);assert.equal(next.status,'Completed');assert.notEqual(first.id,next.id);assert.deepEqual(await service.artifact(first.id,true),bytes);assert.deepEqual(first.pinned_selection.reference,await store.reference(catalogue.version,selection.mint));const restarted=new AnalysisService({root,directory});await restarted.initialize();assert.equal(restarted.get(first.id).selection.mint,selection.mint);}
 finally{await rm(directory,{recursive:true,force:true});}
});
test('changed catalogue versions cannot rewrite earlier pinned references',async()=>{
 const directory=await mkdtemp(resolve(tmpdir(),'eplyx-m2-catalogue-'));const local=new CatalogueStore(directory);await mkdir(local.directory,{recursive:true});
 try{const changed={...retained,retrieved_at:'2026-09-20T00:00:00Z'};const fresh=catalogueFromCapture(changed);assert.notEqual(fresh.version,catalogue.version);for(const [version,capture] of [[catalogue.version,retained],[fresh.version,changed]])await writeFile(resolve(local.directory,version+'.json'),JSON.stringify(capture));const pinned=await local.reference(catalogue.version,one.splMint);await writeFile(resolve(local.directory,'current.json'),JSON.stringify({version:fresh.version}));assert.equal((await local.current()).version,fresh.version);assert.deepEqual(await local.reference(catalogue.version,one.splMint),pinned);}
 finally{await rm(directory,{recursive:true,force:true});}
});
test('cross-asset results, source mismatches, proof and lifecycle inheritance are rejected',async()=>{
 const selection=request(catalogue.entries[0]),pin={cluster:selection.cluster,mint:selection.mint,reference:await store.reference(catalogue.version,selection.mint),sample_accounts:false};
 for(const mutate of [r=>r.asset.mint=catalogue.entries[1].mint,r=>r.selection.reference.version='a'.repeat(64),r=>r.selection.reference.assertions[0].symbol='changed',r=>r.lifecycle_event={deadline:'saved'},r=>r.readiness={},r=>r.execution_performed=true,r=>r.paths[0].status='Proven']){const r=result(selection,structuredClone(pin));mutate(r);assert.throws(()=>parseEngineResult(selection,0,JSON.stringify(r),pin));}
});
test('consumer rendering escapes names and metadata and retains exact amounts without external fetch markup',()=>{
 const attack='<img src=x onerror=alert(1)>';const markup=CurrentResult({id:randomUUID(),result:{asset:{name:attack},mint:{is_initialized:true,decimal_supply:'0.000000001',decimals:9},acquisition:{completed_at:'2026-09-19T00:00:00Z'},onchain_metadata:{name:attack,symbol:attack,uri:'https://attacker.invalid'},discovery:{status:'Unavailable',gaps:[]},accounts:[]}});
 assert(!markup.includes('<img'));assert(markup.includes('&lt;img'));assert(markup.includes('0.000000001'));assert(markup.includes('Stock / issuer association unconfirmed'));assert(markup.includes('could not be loaded'));assert(!markup.includes('zero holders'));
});
test('failed catalogue import is retained and a dated saved catalogue remains usable',async()=>{
 const {importCatalogue}=await import('../catalogue.mjs');const directory=await mkdtemp(resolve(tmpdir(),'eplyx-m2-import-'));const local=new CatalogueStore(directory);await mkdir(local.directory,{recursive:true});const originalFetch=globalThis.fetch;
 try{await writeFile(resolve(local.directory,catalogue.version+'.json'),JSON.stringify(retained));await writeFile(resolve(local.directory,'current.json'),JSON.stringify({version:catalogue.version}));globalThis.fetch=async(url,options)=>{assert.equal(url,SOURCE_URL);assert.equal(options.redirect,'error');return new Response('',{status:429});};const attempt=await importCatalogue(directory,{validateMints:()=>{throw new Error('not called');}});assert.equal(attempt.status,'Unavailable');assert.equal(attempt.http_status,429);const saved=await local.current();assert.equal(saved.version,catalogue.version);assert.equal(saved.retrieved_at,catalogue.retrieved_at);assert.equal(saved.latest_attempt.status,'Unavailable');}
 finally{globalThis.fetch=originalFetch;await rm(directory,{recursive:true,force:true});}
});
