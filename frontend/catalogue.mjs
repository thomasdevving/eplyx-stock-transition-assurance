// Narrow, data-only adapter for the official PreStocks product page's JSON flight payload.
import { createHash, randomUUID } from 'node:crypto';
import { readFile, writeFile, mkdir, rename } from 'node:fs/promises';
import { resolve } from 'node:path';
export const SOURCE_URL='https://prestocks.com/products';
export const CLUSTER='solana-mainnet';
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
export const isDigest=v=>typeof v==='string'&&/^[a-f0-9]{64}$/.test(v);
const textField=(v,label)=>{if(typeof v!=='string'||!v.trim()||v.length>160)throw new Error(`Invalid catalogue ${label}`);return v;};
export function parseProducts(html) {
 const products=[];
 const walk=v=>{if(Array.isArray(v))v.forEach(walk);else if(v&&typeof v==='object'){if(Object.hasOwn(v,'products')){if(!Array.isArray(v.products))throw new Error('Malformed products');products.push(...v.products);}else Object.values(v).forEach(walk);}};
 // JSON.parse only: never evaluate downloaded JavaScript. Ignore unrelated flight records.
 for(const match of html.matchAll(/self\.__next_f\.push\((\[.*?\])\)<\/script>/gs)) {
  let record;try{record=JSON.parse(match[1]);}catch{continue;}
  if(!Array.isArray(record)||record[0]!==1||typeof record[1]!=='string')continue;
  for(const line of record[1].split('\n')){const colon=line.indexOf(':');if(colon<0)continue;let value;try{value=JSON.parse(line.slice(colon+1));}catch{continue;}walk(value);}
 }
 if(!products.length||products.length>100)throw new Error('Official product payload absent or exceeds budget');
 const entries=new Map();
 for(const p of products){
  const mint=textField(p.splMint,'mint'),name=textField(p.name,'name'),symbol=textField(p.symbol,'symbol');
  if(mint.length>44||mint.length<32||!Number.isInteger(p.decimals)||p.decimals<0||p.decimals>255)throw new Error('Malformed product identity fields');
  const assertion={mint,name,symbol,decimals:p.decimals};
  const entry=entries.get(mint)||{cluster:CLUSTER,mint,assertions:[]};
  if(!entry.assertions.some(a=>JSON.stringify(a)===JSON.stringify(assertion)))entry.assertions.push(assertion);
  entries.set(mint,entry);
 }
 return [...entries.values()];
}
export function catalogueFromCapture(capture) {
 if(capture.source_url!==SOURCE_URL||!Number.isFinite(Date.parse(capture.retrieved_at))||typeof capture.content!=='string'||Buffer.byteLength(capture.content)>2*1024*1024||digest(capture.content)!==capture.content_sha256)throw new Error('Invalid catalogue capture');
 const version=digest(JSON.stringify({source_url:capture.source_url,retrieved_at:capture.retrieved_at,content_sha256:capture.content_sha256}));
 return {version,source_url:SOURCE_URL,retrieved_at:capture.retrieved_at,content_sha256:capture.content_sha256,attribution:'PreStocks product source',source_status:'SavedSource',identity_fields:['splMint','name','symbol','decimals'],cluster_basis:'Solana SPL mint reference; mainnet genesis checked independently for each run',entries:parseProducts(capture.content)};
}
export class CatalogueStore {
 constructor(root){this.directory=resolve(root,'evidence/catalogue');}
 async version(version){if(!isDigest(version))throw new Error('InvalidCatalogue');const c=catalogueFromCapture(JSON.parse(await readFile(resolve(this.directory,`${version}.json`),'utf8')));if(c.version!==version)throw new Error('InvalidCatalogue');return c;}
 async current(){let latest_attempt=null;try{latest_attempt=JSON.parse(await readFile(resolve(this.directory,'latest-attempt.json'),'utf8'));}catch{}try{const pointer=JSON.parse(await readFile(resolve(this.directory,'current.json'),'utf8'));return {...await this.version(pointer.version),latest_attempt};}catch{return {source_status:'Unavailable',entries:[],latest_attempt};}}
 async reference(version,mint){const c=await this.version(version);const entry=c.entries.find(e=>e.mint===mint);if(!entry)throw new Error('CatalogueMintMismatch');return {version:c.version,source_url:c.source_url,retrieved_at:c.retrieved_at,content_sha256:c.content_sha256,assertions:entry.assertions};}
}
export async function importCatalogue(root, {validateMints}={}) {
 const store=new CatalogueStore(root);await mkdir(store.directory,{recursive:true});
 const attempt={id:randomUUID(),source_url:SOURCE_URL,started_at:new Date().toISOString()};
 try {
  const response=await fetch(SOURCE_URL,{redirect:'error',signal:AbortSignal.timeout(15000),headers:{Accept:'text/html'}});
  attempt.http_status=response.status;if(!response.ok)throw new Error(`HTTP ${response.status}`);
  let length=0;const chunks=[];for await(const chunk of response.body){length+=chunk.length;if(length>2*1024*1024)throw new Error('Response exceeds 2 MiB');chunks.push(chunk);}
  const content=Buffer.concat(chunks).toString('utf8'),retrieved_at=new Date().toISOString();
  const capture={source_url:SOURCE_URL,retrieved_at,content_sha256:digest(content),content};
  const catalogue=catalogueFromCapture(capture);
  if(!validateMints)throw new Error("Solana address validator required");
  await validateMints(catalogue.entries.map(e=>e.mint));
  await writeFile(resolve(store.directory,`${catalogue.version}.json`),JSON.stringify(capture),{flag:'wx'});
  await writeFile(resolve(store.directory,'current.json.tmp'),JSON.stringify({version:catalogue.version}));await rename(resolve(store.directory,'current.json.tmp'),resolve(store.directory,'current.json'));
  attempt.status='Completed';attempt.version=catalogue.version;attempt.entries=catalogue.entries.length;
 }catch(error){attempt.status='Unavailable';attempt.error=String(error.message).slice(0,300);}
 attempt.completed_at=new Date().toISOString();await writeFile(resolve(store.directory,`attempt-${attempt.id}.json`),JSON.stringify(attempt));await writeFile(resolve(store.directory,'latest-attempt.json'),JSON.stringify(attempt));return attempt;
}
