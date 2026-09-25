import {createPreflight,executePreflight} from './preflight-service.mjs';
import {createConversion,executeConversion,conversionMechanism} from './conversion-service.mjs';
import {createStress,executeStress,stressBudget} from './stress-service.mjs';
import { CatalogueStore, CLUSTER, isDigest } from './catalogue.mjs';
import { spawn } from 'node:child_process';
import { randomUUID, createHash } from 'node:crypto';
import { readFile, writeFile, mkdir, readdir, access, rename, unlink } from 'node:fs/promises';
import { resolve } from 'node:path';
import { AccessGate } from './access-gate.mjs';

/** The built engine. Windows needs the .exe suffix; other platforms do not. */
export const engineExecutable = process.env.EPLYX_ENGINE || (process.platform === 'win32'
 ? 'target/debug/eplyx-lifecycle.exe'
 : 'target/debug/eplyx-lifecycle');
export const stages = ['before_transition', 'after_transition', 'after_deadline'];
export const checks = Object.freeze({
 'complete-exit': 'lp-complete-exit.json',
 'required-sale': 'required-failed-route.json',
 'transfer-conversion': 'transfer-official-conversion.json',
 'principal-removal': 'principal-removal-positive-control.json',
});
export function validateSelection(value, {legacy=false}={}) {
 if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('InvalidSelection');
 const { asset, review, stage, check } = value;
 if(review==='current' && asset!=='spacex') {
  if(typeof value.mint!=='string'||value.mint.length<32||value.mint.length>44)throw new Error('InvalidMint');
  if(Object.keys(value).some(k=>!['asset','review','stage','check','cluster','mint','catalogue_version','sample_accounts','scope','public_owner'].includes(k)) || asset!=='mint' || stage!==null || check!==null || value.cluster!==CLUSTER || typeof value.mint!=='string' || value.mint.length<32 || value.mint.length>44 || typeof value.sample_accounts!=='boolean' || (value.catalogue_version!==null&&!isDigest(value.catalogue_version)))throw new Error('InvalidSelection');
  if(value.scope!==undefined&&!['token','wallet'].includes(value.scope))throw new Error('InvalidSelection');
  if(value.scope==='wallet' ? typeof value.public_owner!=='string'||value.public_owner.length>100||value.sample_accounts!==false : value.public_owner!==undefined)throw new Error('InvalidOwner');
  return {asset,review,stage,check,cluster:value.cluster,mint:value.mint,catalogue_version:value.catalogue_version,sample_accounts:value.sample_accounts,...(value.scope?{scope:value.scope}:{}),...(value.scope==='wallet'?{public_owner:value.public_owner}:{})};
 }
 if(Object.keys(value).some(k=>!['asset','review','stage','check'].includes(k)) || asset!=='spacex' || !['overview','holding','position','assumption',...(legacy?['current']:[])].includes(review) || (review==='current'?stage!==null:!stages.includes(stage)))throw new Error('InvalidSelection');
 if (review === 'assumption' ? typeof check!=='string'||!Object.hasOwn(checks, check) : check !== null) throw new Error('InvalidSelection');
 return { asset, review, stage, check };
}
const idPattern = /^[a-f0-9]{8}-[a-f0-9]{4}-4[a-f0-9]{3}-[89ab][a-f0-9]{3}-[a-f0-9]{12}$/;
export const validRunId = id => typeof id === 'string' && idPattern.test(id);
const canonical=v=>JSON.stringify(v,(_,item)=>item&&typeof item==='object'&&!Array.isArray(item)?Object.fromEntries(Object.entries(item).sort(([a],[b])=>a.localeCompare(b))):item);
const hash = value => createHash('sha256').update(value).digest('hex');
export function engineArguments(selection, capturePath, referencePath) {
 if(selection.review==='current') { if(!capturePath||!referencePath)throw new Error('MissingCapturePath'); return ['inspect-current','--mint',selection.mint,'--selection',referencePath,'--out',capturePath]; }
 return selection.review === 'assumption'
  ? ['evaluate-rollout', '--plan', `probes/phase14-plans/${checks[selection.check]}`, '--target-view', selection.stage, '--format', 'json']
  : ['compare-scenarios', '--format', 'json'];
}
export function parseEngineResult(selection, code, stdout, pinnedSelection) {
 const artifact = JSON.parse(stdout);
 if(selection.review==='current') {
  const allowed=['schema_version','kind','asset','identity_provenance','acquisition','mint','mint_slot','accounts','discovery','paths','lifecycle_event','readiness','authorization','funds_moved','local_execution_performed','signer_assumed_locally','signer_possession_known','limitations','selection','execution_performed','inspection','account_observation','onchain_metadata','identity_mismatches','wallet_observation'];
  if(Object.keys(artifact).some(k=>!allowed.includes(k)) || artifact.paths?.map(p=>p.path).join(',')!=='OfficialTransition,Redemption,SecondaryMarketExit,Transfer,Withdrawal')throw new Error('InvalidEngineResult');
  if(code!==0 || artifact.kind!=='current-inspection' || artifact.schema_version!==(selection.scope==='wallet'?3:2) || artifact.asset?.mint!==selection.mint || artifact.selection?.mint!==selection.mint || artifact.selection?.cluster!==selection.cluster || artifact.selection?.sample_accounts!==selection.sample_accounts || (selection.catalogue_version!==null&&artifact.selection?.reference?.version!==selection.catalogue_version) || (pinnedSelection&&canonical(artifact.selection)!==canonical(pinnedSelection)) || artifact.lifecycle_event!==null || artifact.execution_performed!==false || artifact.funds_moved!==false || artifact.signer_assumed_locally!==false || artifact.signer_possession_known!==false || !['Completed','Partial','Unsupported','Unavailable'].includes(artifact.inspection?.status) || artifact.readiness!==null || artifact.authorization!==false || artifact.local_execution_performed!==false || artifact.paths?.length!==5 || artifact.paths.some(p=>p.status!=='NotTested'))throw new Error('InvalidEngineResult');
  if(selection.scope==='wallet') {
   const w=artifact.wallet_observation;
   if(artifact.selection.public_owner!==selection.public_owner||w?.submitted_owner!==selection.public_owner||w?.selected_mint!==selection.mint||!['Completed','Partial','Unavailable','InvalidOwner'].includes(w.status)||!Array.isArray(w.token_accounts)||w.token_accounts.some(a=>a.state?.owner!==selection.public_owner||a.state?.mint!==selection.mint))throw new Error('InvalidEngineResult');
  } else if(artifact.wallet_observation!==undefined||artifact.selection.public_owner!==undefined)throw new Error('InvalidEngineResult');
  return artifact;
 }
 if (selection.review === 'assumption') {
  if (![0,3,4,5].includes(code) || artifact.evaluation_command_exit_code !== code || artifact.candidate.target_view.id !== selection.stage || !['Ready','Blocked','Incomplete'].includes(artifact.readiness.overall_status)) throw new Error('InvalidEngineResult');
  return { kind:'assumption', assessment:artifact, evidence_bindings:artifact.production_world };
 }
 if (code !== 0 || artifact.schema_version !== 1) throw new Error('InvalidEngineResult');
 const view = artifact.scenarios.find(v => v.scenario.id === selection.stage);
 if (!view) throw new Error('InvalidEngineResult');
 const { entity_impacts, ...selectedView } = view;
 return { kind:'transition', view:selectedView, readiness:artifact.frozen_readiness,
  direct_paths:artifact.historical_direct_paths, position_paths:artifact.historical_position_paths,
  position:artifact.frozen_position, evidence_bindings:artifact.production_world };
}
export function executeEngine(executable, args, { cwd, timeoutMs=600000, fresh=false, onStage } = {}) {
 return new Promise((resolvePromise, reject) => {
  const env={PATH:process.env.PATH}; if(fresh && process.env.SOLANA_RPC_URL)env.SOLANA_RPC_URL=process.env.SOLANA_RPC_URL;
  const child=spawn(executable,args,{cwd,env,shell:false,stdio:['ignore','pipe','pipe']});
  let stdout='', stderr='', exceeded=false, timedOut=false;
  const timer=setTimeout(()=>{timedOut=true;child.kill('SIGKILL');},timeoutMs);
  child.stdout.on('data',buffer=>{stdout+=buffer; if(Buffer.byteLength(stdout)>32*1024*1024){exceeded=true;child.kill('SIGKILL');}});
  child.stderr.on('data',buffer=>{if(stderr.length<1024*1024)stderr+=buffer; for(const line of String(buffer).split('\n'))if(line.startsWith('CURRENT_STAGE:'))onStage?.(line.slice(14).trim());});
  child.on('error',()=>{clearTimeout(timer);reject(new Error('BackendUnavailable'));});
  child.on('close',code=>{
   clearTimeout(timer);
   if(timedOut)reject(new Error('AnalysisTimeout'));
   else if(exceeded)reject(new Error('ResultTooLarge'));
   else if(![0,3,4,5].includes(code))reject(new Error(code===2?'EvidenceVerificationFailed':'EngineFailure'));
   else resolvePromise({code,stdout});
  });
 });
}
export class AnalysisService {
 constructor({ root, directory=resolve(root,'.analysis-runs'), runner=executeEngine, timeoutMs=600000 }={}) {
  this.root=root;this.catalogue=new CatalogueStore(root);this.directory=directory;this.executable=resolve(root,engineExecutable);
  this.runner=runner;this.timeoutMs=timeoutMs;this.jobs=new Map();this.keys=new Map();this.queue=[];this.running=false;
 }
 async initialize() {
  await mkdir(this.directory,{recursive:true});
  for(const file of (await readdir(this.directory)).filter(f=>validRunId(f.replace(/\.json$/,'')) && f.endsWith('.json'))) {
   try {
    const job=JSON.parse(await readFile(resolve(this.directory,file),'utf8'));
    if(!validRunId(job.id))continue;
    validateSelection(job.selection,{legacy:true});
    if(['Queued','Running'].includes(job.status)){job.status='Error';job.error={code:'AnalysisInterrupted',message:'The local service restarted. Please run the analysis again.'};await this.save(job);}
    this.jobs.set(job.id,job);if(validRunId(job.request_key))this.keys.set(`${job.owner||'local'}:${job.request_key}`,job.id);
   } catch { /* An invalid local run record is not an evidence artifact. */ }
  }
 }
 async save(job) { const destination=resolve(this.directory,`${job.id}.json`);await writeFile(`${destination}.tmp`,JSON.stringify(job));await rename(`${destination}.tmp`,destination); }
 async available() { try { await access(this.executable);return true; } catch { return false; } }
 async submit(selection, key, owner='local') {
  const request=(this.submitting || Promise.resolve()).then(()=>this.createJob(selection,key,owner));
  this.submitting=request.catch(()=>{});return request;
 }
 async createJob(selection, key, owner) {
  selection=validateSelection(selection);
  if(!validRunId(key))throw new Error('InvalidRequestKey');
  const scopedKey=`${owner}:${key}`;
  if(this.keys.has(scopedKey)) {
   const existing=this.jobs.get(this.keys.get(scopedKey));
   if(JSON.stringify(existing.selection)!==JSON.stringify(selection))throw new Error('RequestKeyConflict');
   return existing;
  }
  if(!await this.available())throw new Error('BackendUnavailable');
  let pinnedSelection=null;
  if(selection.review==='current'){
   try{await executeEngine(this.executable,['validate-address','--mint',selection.mint],{cwd:this.root,timeoutMs:5000});}catch{throw new Error('InvalidMint');}
   if(selection.scope==='wallet'){try{await executeEngine(this.executable,['validate-address','--mint',selection.public_owner],{cwd:this.root,timeoutMs:5000});}catch{throw new Error('InvalidOwner');}}
   const reference=selection.catalogue_version ? await this.catalogue.reference(selection.catalogue_version,selection.mint) : null;
   pinnedSelection={cluster:selection.cluster,mint:selection.mint,reference,sample_accounts:selection.sample_accounts,...(selection.scope==='wallet'?{public_owner:selection.public_owner}:{})};
  }
  if(this.queue.length>=4 || this.jobs.size>=200)throw new Error('QueueFull');
  const job={id:randomUUID(),owner,request_key:key,selection,status:'Queued',created_at:new Date().toISOString(),
   resolved:{asset:'SPACEX PreStocks',scope:selection.review,entity:selection.review==='holding'?'solana-token-account:741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs':selection.review==='position'?'solana-program-position:BpTBNQ7vNaBujkwhgyBoiYyvEc6KrUUNTGiQDsrwTNxN':null,policy:selection.check==='principal-removal'?'stocklana-phase14-principal-removal-only-v1':selection.check==='required-sale'?'stocklana-phase14-required-exact-route-v1':'stocklana-spacex-preflight-v1',trusted_bundle:'stocklana-frozen-production',candidate:selection.review==='assumption'?checks[selection.check].replace('.json',''):null,target_view:selection.stage},
   operation:selection.review==='assumption'?'Existing offline rollout assertion and readiness evaluation':'Existing offline counterfactual evaluation with frozen readiness findings',
   cached:false, authorization:false, guarded_action_invoked:false, result:null,error:null};
  if(selection.review==='current'){job.pinned_selection=pinnedSelection;job.resolved={asset:pinnedSelection.reference?.assertions[0].name||'Custom token address',cluster:selection.cluster,mint:selection.mint,scope:selection.scope==='wallet'?'Public owner and selected mint token accounts':'Current mint inspection; optional bounded token-account sample',public_owner:selection.public_owner||null,policy:null,trusted_bundle:null};job.operation='Fresh read-only acquisition and current-state inspection';}
  this.jobs.set(job.id,job);this.keys.set(scopedKey,job.id);await this.save(job);this.queue.push(job);void this.process();return job;
 }
 async submitPreflight(parentId,request,key,owner) {
  const operation=(this.submitting||Promise.resolve()).then(()=>createPreflight(this,parentId,request,key,owner));
  this.submitting=operation.catch(()=>{});return operation;
 }
 async submitConversion(parentId, request, key, owner) {
  const operation=(this.submitting||Promise.resolve()).then(()=>createConversion(this,parentId,request,key,owner));
  this.submitting=operation.catch(()=>{});return operation;
 }
 async submitStress(parentId, key, owner) {
  const operation=(this.submitting||Promise.resolve()).then(()=>createStress(this,parentId,key,owner));
  this.submitting=operation.catch(()=>{});return operation;
 }
 async mechanism() { return conversionMechanism(this); }
 async stressBudget() { return stressBudget(this); }
 async submitCheck(parentId, request, key, owner) {
  const operation=(this.submitting||Promise.resolve()).then(()=>this.createCheck(parentId,request,key,owner));
  this.submitting=operation.catch(()=>{});return operation;
 }
 async createCheck(parentId, request, key, owner) {
  const parent=this.get(parentId);
  if(!parent||parent.owner!==owner||parent.status!=='Completed'||parent.selection.scope!=='wallet'||parent.check_request||parent.preflight_request||parent.conversion_request)throw new Error('InvalidParent');
  if(!request||typeof request!=='object'||Array.isArray(request)||Object.keys(request).some(k=>!['path','source','amount_mode','amount_decimal','recipient','output_mint','minimum_output_decimal'].includes(k))||!['Transfer','SecondaryMarketExit'].includes(request.path)||typeof request.source!=='string'||typeof request.recipient!=='string'||request.source.length>44||request.recipient.length>44||!['Full','Custom'].includes(request.amount_mode)||(request.amount_decimal!==null&&typeof request.amount_decimal!=='string')||(request.amount_decimal?.length||0)>280)throw new Error('InvalidCheck');
  if(request.path==='SecondaryMarketExit'&&(request.recipient!==''||typeof request.output_mint!=='string'||request.output_mint.length>44||typeof request.minimum_output_decimal!=='string'||request.minimum_output_decimal.length>280))throw new Error('InvalidCheck');
  if(!validRunId(key))throw new Error('InvalidRequestKey');
  const scopedKey=`${owner}:${key}`;
  if(this.keys.has(scopedKey)){const existing=this.jobs.get(this.keys.get(scopedKey));if(existing.parent_run_id!==parentId||canonical(existing.check_request)!==canonical(request))throw new Error('RequestKeyConflict');return existing;}
  if(this.queue.length>=4||this.jobs.size>=200)throw new Error('QueueFull');
  await this.artifact(parentId,true); // Verify the immutable parent bytes before validating any request.
  const id=randomUUID(),requestPath=resolve(this.directory,`${id}.request.json`);
  await writeFile(requestPath,JSON.stringify(request),{flag:'wx'});
  let validated;
  try {const response=await executeEngine(this.executable,['validate-current-check','--input',resolve(this.directory,`${parentId}.capture.json`),'--request',requestPath],{cwd:this.root,timeoutMs:10000});validated=JSON.parse(response.stdout);}
  catch{await unlink(requestPath);throw new Error('InvalidCheck');}
  const job={id,owner,request_key:key,selection:parent.selection,parent_run_id:parentId,parent_capture_sha256:parent.capture_sha256,check_request:request,validated_request:validated,status:'Queued',created_at:new Date().toISOString(),operation:'Fresh read-only execution capture followed by offline local simulation',authorization:false,result:null,error:null};
  this.jobs.set(id,job);this.keys.set(scopedKey,id);await this.save(job);this.queue.push(job);void this.process();return job;
 }
 async capabilities(id) {
  const parent=this.get(id);if(parent?.status!=='Completed'||parent.selection.scope!=='wallet'||parent.check_request||parent.preflight_request||parent.conversion_request)throw new Error('InvalidParent');
  await this.artifact(id,true);
  const {stdout}=await executeEngine(this.executable,['current-check-capabilities','--input',resolve(this.directory,`${id}.capture.json`)],{cwd:this.root,timeoutMs:10000});
  const value=JSON.parse(stdout);if(value.wallet_capture_sha256!==parent.capture_sha256)throw new Error('EvidenceVerificationFailed');return value;
 }
 async executeCheck(job) {
  const parent=this.get(job.parent_run_id);
  if(!parent||parent.capture_sha256!==job.parent_capture_sha256)throw new Error('EvidenceVerificationFailed');
  await this.artifact(parent.id,true);
  const capturePath=resolve(this.directory,`${job.id}.capture.json`);
  const options={cwd:this.root,timeoutMs:120000,onStage:stage=>{job.stage=stage;}};
  job.stage='Preparing current state';
  await this.runner(this.executable,['capture-current-check','--input',resolve(this.directory,`${parent.id}.capture.json`),'--request',resolve(this.directory,`${job.id}.request.json`),'--run-id',parent.id,'--check-id',job.id,'--out',capturePath],{...options,fresh:true});
  if(hash(await readFile(this.executable))!==job.engine_sha256)throw new Error('EvidenceVerificationFailed');
  job.capture_sha256=hash(await readFile(capturePath));
  // This separate child receives PATH only: no RPC URL or provider secrets can enter the VM.
  const {code,stdout}=await this.runner(this.executable,['replay-current-check','--input',capturePath,'--run-id',parent.id,'--check-id',job.id,'--wallet-sha256',job.parent_capture_sha256,'--capture-sha256',job.capture_sha256],{...options,fresh:false});
  if(hash(await readFile(this.executable))!==job.engine_sha256)throw new Error('EvidenceVerificationFailed');
  const result=JSON.parse(stdout);
  if(code!==0||result.kind!=='current-execution'||result.run_id!==parent.id||result.check_id!==job.id||result.wallet_capture_sha256!==parent.capture_sha256||result.execution_capture_sha256!==job.capture_sha256||canonical(result.request)!==canonical(job.check_request)||result.mint!==parent.selection.mint||result.source!==job.check_request.source||result.signer_possession_known!==false||result.funds_moved!==false||result.readiness!==null||result.authorization!==false)throw new Error('InvalidEngineResult');
  job.result=result;job.engine_exit_code=code;job.canonical_sha256=hash(stdout);
  await writeFile(resolve(this.directory,`${job.id}.artifact`),stdout,{flag:'wx'});
  await writeFile(resolve(this.directory,`${job.id}.manifest.json`),JSON.stringify({schema_version:1,run_id:parent.id,check_id:job.id,parent_capture_sha256:job.parent_capture_sha256,capture_sha256:job.capture_sha256,result_sha256:job.canonical_sha256,engine_sha256:job.engine_sha256,request:job.check_request}),{flag:'wx'});
 }
 async process() {
  if(this.running)return; this.running=true;
  try {
   while(this.queue.length) {
    const job=this.queue.shift();job.status='Running';job.started_at=new Date().toISOString();await this.save(job);
    try {
     job.engine_sha256=hash(await readFile(this.executable));
     if(job.stress_request){await executeStress(this,job);}else if(job.preflight_request){await executePreflight(this,job);}else if(job.conversion_request){await executeConversion(this,job);}else if(job.check_request){await this.executeCheck(job);}else{
     const referencePath=resolve(this.directory,`${job.id}.selection.json`);
     if(job.selection.review==='current')await writeFile(referencePath,JSON.stringify(job.pinned_selection),{flag:'wx'});
     const {code,stdout}=await this.runner(this.executable,engineArguments(job.selection,resolve(this.directory,`${job.id}.capture.json`),referencePath),{cwd:this.root,timeoutMs:job.selection.review==='current'?Math.min(this.timeoutMs,90000):this.timeoutMs,fresh:job.selection.review==='current',onStage:stage=>{job.stage=stage;}});
     job.result=parseEngineResult(job.selection,code,stdout,job.pinned_selection);job.engine_exit_code=code;
     job.canonical_sha256=hash(stdout);await writeFile(resolve(this.directory,`${job.id}.artifact`),stdout);
     if(job.selection.review==='current'){
      job.capture_sha256=hash(await readFile(resolve(this.directory,`${job.id}.capture.json`)));
      await writeFile(resolve(this.directory,`${job.id}.manifest.json`),JSON.stringify({schema_version:1,run_id:job.id,selection:job.selection,pinned_selection:job.pinned_selection,engine_sha256:job.engine_sha256,result_sha256:job.canonical_sha256,capture_sha256:job.capture_sha256}));
     }
     }
     job.status='Completed';
    } catch(error) {
     const known=['BackendUnavailable','AnalysisTimeout','ResultTooLarge','EvidenceVerificationFailed','EngineFailure'];
     job.status='Error';job.result=null;job.error={code:known.includes(error.message)?error.message:'InvalidEngineResult',message:job.stress_request?'The production-state stress test could not complete. The earlier candidate conversion result is still available. No stress evidence, coverage or readiness was granted and no funds moved.':job.preflight_request?'The pre-flight preparation or evidence verification could not complete. The original wallet and local checks remain available; no readiness was granted.':job.conversion_request?'The candidate conversion check could not complete. The wallet observation is still available. No conversion proof was granted and no funds moved.':job.check_request?'The local execution check could not complete. The wallet observation is still available. No execution proof was granted.':job.selection.review==='current'?'Current mainnet acquisition or decoding could not finish. No saved example was substituted. Retry with a new capture; the server operator can check the configured RPC provider.':error.message==='AnalysisTimeout'?'The analysis timed out. You can try again.':'The saved inputs could not be verified or the local engine could not finish. No readiness result was granted.'};
    }
    job.completed_at=new Date().toISOString();await this.save(job);
   }
  } finally {this.running=false;}
 }
 get(id) {return validRunId(id)?this.jobs.get(id):undefined;}
 async artifact(id, capture=false) {const job=this.get(id);if(job?.status!=='Completed'||(capture&&!job.capture_sha256))return null;const bytes=await readFile(resolve(this.directory,capture?`${id}.capture.json`:`${id}.artifact`));if(hash(bytes)!==(capture?job.capture_sha256:job.canonical_sha256))throw new Error('EvidenceVerificationFailed');return bytes;}
}
// `access` is an optional AccessGate; without one (local use) every request
// is allowed exactly as before. `secure` adds the Secure flag to cookies.
export async function handleAnalysisAPI(service,request,response,origin,{access=new AccessGate(),secure=false}={}) {
 const pathname=new URL(request.url,origin).pathname;
 if(!pathname.startsWith('/api/'))return false;
 const json=(status,value)=>{response.writeHead(status,{'Content-Type':'application/json','Cache-Control':'no-store','X-Content-Type-Options':'nosniff'});response.end(JSON.stringify(value));};
 if(request.headers.host!==new URL(origin).host || (request.headers.origin && request.headers.origin!==origin) || request.headers['sec-fetch-site']==='cross-site'){json(403,{error:{code:'OriginRejected',message:'Use the local application origin.'}});return true;}
 if(pathname==='/api/catalogue' && request.method==='GET'){json(200,await service.catalogue.current());return true;}
 if(pathname==='/api/health' && request.method==='GET'){json(200,{available:await service.available()});return true;}
 if(pathname==='/api/conversion-stress-budget' && request.method==='GET'){
  try{json(200,await service.stressBudget());}catch{json(503,{error:{code:'BudgetUnavailable',message:'The local engine is not built.'}});}
  return true;
 }
 if(pathname==='/api/conversion-mechanism' && request.method==='GET'){
  try{json(200,await service.mechanism());}catch{json(503,{error:{code:'MechanismUnavailable',message:'The registered candidate conversion mechanism is not built. Run ./scripts/build-programs.sh.'}});}
  return true;
 }
 let session=request.headers.cookie?.match(/(?:^|; )eplyx_session=([a-f0-9]{64})(?:;|$)/)?.[1];
 if(!session){session=hash(randomUUID()+randomUUID());response.setHeader('Set-Cookie',`eplyx_session=${session}; HttpOnly; SameSite=Strict; Path=/; Max-Age=2592000${secure?'; Secure':''}`);}
 if(pathname==='/api/access'){
  if(request.method==='GET'){json(200,{required:access.required,unlocked:access.unlocked(request,session)});return true;}
  if(request.method==='POST'){
   let code;
   try{if(!String(request.headers['content-type']).startsWith('application/json'))throw new Error();let body='';for await(const chunk of request){body+=chunk;if(Buffer.byteLength(body)>1024)throw new Error();}code=JSON.parse(body).code;}catch{json(400,{error:{code:'InvalidAccessRequest'}});return true;}
   const outcome=access.check(access.client(request),code);
   if(outcome==='blocked'){json(429,{error:{code:'AccessRateLimited',message:'Too many incorrect access codes. Wait 15 minutes.'}});return true;}
   if(outcome!=='ok'){json(401,{error:{code:'AccessCodeInvalid',message:'That access code is not correct.'}});return true;}
   const existing=response.getHeader('Set-Cookie');
   if(access.required)response.setHeader('Set-Cookie',[...(existing?[].concat(existing):[]),access.cookie(session)]);
   json(200,{required:access.required,unlocked:true});return true;
  }
 }
 if(!access.unlocked(request,session)){json(401,{error:{code:'AccessCodeRequired',message:'Enter the analysis access code to run analyses on this hosted demo. Saved results stay available without it.'}});return true;}
 const owner=hash(session);
 const capabilityMatch=pathname.match(/^\/api\/runs\/([a-f0-9-]+)\/capabilities$/);
 if(capabilityMatch&&request.method==='GET'){
  const parent=service.get(capabilityMatch[1]);if(!parent||parent.owner!==owner){json(404,{error:{code:'RunNotFound'}});return true;}
  try{json(200,await service.capabilities(parent.id));}catch{json(409,{error:{code:'CapabilitiesUnavailable'}});}return true;
 }
 const preflightMatch=pathname.match(/^\/api\/runs\/([a-f0-9-]+)\/preflights$/);
 if(preflightMatch){
  const parent=service.get(preflightMatch[1]);if(!parent||parent.owner!==owner){json(404,{error:{code:'RunNotFound'}});return true;}
  if(request.method==='GET'){json(200,[...service.jobs.values()].filter(j=>j.parent_run_id===parent.id&&j.owner===owner&&j.preflight_request));return true;}
  if(request.method==='POST'){
   try{
    if(!String(request.headers['content-type']).startsWith('application/json'))throw new Error('InvalidPreflight');
    let body='';for await(const chunk of request){body+=chunk;if(Buffer.byteLength(body)>4096)throw new Error('RequestTooLarge');}
    const value=JSON.parse(body);if(!value||Object.keys(value).some(k=>!['request','request_key'].includes(k)))throw new Error('InvalidPreflight');
    json(202,await service.submitPreflight(parent.id,value.request,value.request_key,owner));
   }catch(error){json(error.message==='QueueFull'?503:400,{error:{code:error.message==='QueueFull'?'QueueFull':'InvalidPreflight',message:'Choose a discovered account, a future effective time, an optional later deadline and valid replacement mint address. Select at most four completed checks from this exact run and account.'}});}
   return true;
  }
 }
 const conversionMatch=pathname.match(/^\/api\/runs\/([a-f0-9-]+)\/conversions$/);
 if(conversionMatch){
  const parent=service.get(conversionMatch[1]);if(!parent||parent.owner!==owner){json(404,{error:{code:'RunNotFound'}});return true;}
  if(request.method==='GET'){json(200,[...service.jobs.values()].filter(j=>j.parent_run_id===parent.id&&j.owner===owner&&j.conversion_request));return true;}
  if(request.method==='POST'){
   try{
    if(!String(request.headers['content-type']).startsWith('application/json'))throw new Error('InvalidConversion');
    let body='';for await(const chunk of request){body+=chunk;if(Buffer.byteLength(body)>2048)throw new Error('RequestTooLarge');}
    const value=JSON.parse(body);if(!value||Object.keys(value).some(k=>!['request','request_key'].includes(k)))throw new Error('InvalidConversion');
    json(202,await service.submitConversion(parent.id,value.request,value.request_key,owner));
   }catch(error){json(error.message==='QueueFull'?503:400,{error:{code:error.message==='QueueFull'?'QueueFull':'InvalidConversion',message:'Choose an account discovered in this wallet run, a different existing replacement mint, a positive whole-number ratio, a supported rounding rule, a conversion fee between 0 and 10000 bps and a proposed reserve amount. Only the registered demonstration mechanism runs; program code, instructions and accounts cannot be supplied.'}});}
   return true;
  }
 }
 const stressMatch=pathname.match(/^\/api\/runs\/([a-f0-9-]+)\/stress$/);
 if(stressMatch){
  const parent=service.get(stressMatch[1]);if(!parent||parent.owner!==owner){json(404,{error:{code:'RunNotFound'}});return true;}
  if(request.method==='GET'){json(200,[...service.jobs.values()].filter(j=>j.parent_run_id===parent.id&&j.owner===owner&&j.stress_request));return true;}
  if(request.method==='POST'){
   try{
    if(!String(request.headers['content-type']).startsWith('application/json'))throw new Error('InvalidStress');
    let body='';for await(const chunk of request){body+=chunk;if(Buffer.byteLength(body)>512)throw new Error('RequestTooLarge');}
    const value=JSON.parse(body);
    // The browser supplies a request key and nothing else: no budget, mint,
    // account, amount, program, endpoint, path or claimed status.
    if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).some(k=>k!=='request_key'))throw new Error('InvalidStress');
    json(202,await service.submitStress(parent.id,value.request_key,owner));
   }catch(error){json(error.message==='QueueFull'?503:400,{error:{code:error.message==='QueueFull'?'QueueFull':'InvalidStress',message:'Stress-testing needs a completed candidate conversion for this run. The plan, population discovery, selection and execution budget are all controlled by this local server.'}});}
   return true;
  }
 }
 const checkMatch=pathname.match(/^\/api\/runs\/([a-f0-9-]+)\/checks$/);
 if(checkMatch) {
  const parent=service.get(checkMatch[1]);
  if(!parent||parent.owner!==owner){json(404,{error:{code:'RunNotFound'}});return true;}
  if(request.method==='GET'){json(200,[...service.jobs.values()].filter(j=>j.parent_run_id===parent.id&&j.owner===owner&&j.check_request));return true;}
  if(request.method==='POST'){
   try{
    if(!String(request.headers['content-type']).startsWith('application/json'))throw new Error('InvalidCheck');
    let body='';for await(const chunk of request){body+=chunk;if(Buffer.byteLength(body)>2048)throw new Error('RequestTooLarge');}
    const value=JSON.parse(body);if(!value||Object.keys(value).some(k=>!['request','request_key'].includes(k)))throw new Error('InvalidCheck');
    json(202,await service.submitCheck(parent.id,value.request,value.request_key,owner));
   }catch(error){json(error.message==='QueueFull'?503:400,{error:{code:error.message==='QueueFull'?'QueueFull':'InvalidCheck',message:'Choose an account discovered in this wallet run, a positive amount within its public balance, and valid parameters: a different existing recipient token account for Transfer, or a paired mint and explicit positive minimum output for a market check.'}});}
   return true;
  }
 }
 const match=pathname.match(/^\/api\/runs\/([a-f0-9-]+)(\/(?:artifact|capture))?$/);
 if(match && request.method==='GET') {
  const job=service.get(match[1]);
  if(!job || job.owner!==owner){json(404,{error:{code:'RunNotFound',message:'This local run is no longer available. Run it again.'}});return true;}
  if(match[2]){const bytes=await service.artifact(job.id,match[2]==='/capture');if(!bytes)json(409,{error:{code:'ResultNotAvailable'}});else{response.writeHead(200,{'Content-Type':'application/json','Content-Disposition':`attachment; filename="analysis-${job.id}.json"`,'Cache-Control':'no-store'});response.end(bytes);}return true;}
  const execution_checks=job.selection.scope==='wallet'&&!job.check_request&&!job.preflight_request&&!job.conversion_request?[...service.jobs.values()].filter(c=>c.parent_run_id===job.id&&c.owner===owner&&c.check_request&&c.status==='Completed').map(c=>({check_id:c.id,capture_sha256:c.capture_sha256,result_sha256:c.canonical_sha256,engine_sha256:c.engine_sha256,evidence:c.result})):undefined;
  json(200,execution_checks?{...job,execution_checks}:job);return true;
 }
 if(pathname==='/api/runs' && request.method==='POST') {
  try {
   if(!String(request.headers['content-type']).startsWith('application/json'))throw new Error('InvalidSelection');
   let body='';for await(const chunk of request){body+=chunk;if(Buffer.byteLength(body)>2048)throw new Error('RequestTooLarge');}
   const value=JSON.parse(body);
   if(!value || Object.keys(value).some(k=>!['selection','request_key'].includes(k)))throw new Error('InvalidSelection');
   json(202,await service.submit(value.selection,value.request_key,owner));
  } catch(error) {
   const unavailable=['BackendUnavailable','QueueFull'].includes(error.message), large=error.message==='RequestTooLarge';
   json(unavailable?503:large?413:400,{error:{code:unavailable||large?error.message:'InvalidSelection',message:unavailable?'The local analysis engine is unavailable or busy. Saved reports are still available.':error.message==='InvalidOwner'?'Enter a valid Solana public wallet / owner address. No wallet connection is required.':error.message==='InvalidMint'?'Enter a valid Solana token mint address. Address case is significant.':error.message==='CatalogueMintMismatch'?'The selected mint does not match this catalogue version. Select the asset again.':'Choose a supported asset and inspection. The catalogue reference may be unavailable; custom token entry remains available.'}});
  }
  return true;
 }
 json(404,{error:{code:'NotFound'}});return true;
}
