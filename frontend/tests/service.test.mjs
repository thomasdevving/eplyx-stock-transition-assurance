import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { randomUUID } from 'node:crypto';
import { AnalysisService, validateSelection, engineArguments, parseEngineResult, executeEngine } from '../analysis-service.mjs';
import { tokenAmount } from '../src/presentation.js';
const root=resolve('.');
const selection={asset:'spacex',review:'assumption',stage:'after_transition',check:'principal-removal'};
const published=JSON.parse(await readFile('reports/spacex-rollout-assumptions.json','utf8'));
const positive=published.cases.find(c=>c.assessment.candidate_acceptance==='Accepted').assessment;
const make=async runner=>{const directory=await mkdtemp(resolve(tmpdir(),'eplyx-phase15-'));const service=new AnalysisService({root,directory,runner});await service.initialize();return{service,directory};};
const finished=async service=>{while(service.running||service.queue.length)await new Promise(r=>setTimeout(r,10));};
test('allowlist rejects commands paths addresses endpoints unknown assets and decorative selections',()=>{
 for(const bad of [{...selection,asset:'OTHER'},{...selection,check:['principal-removal']},{...selection,stage:'today'},{...selection,check:'../../x'},{...selection,review:'wallet'}, {...selection,command:'rm'}, {...selection,path:'/tmp/x'},{...selection,rpc:'https://x'}, {...selection,Proven:true}, {...selection,review:'overview'}])assert.throws(()=>validateSelection(bad));
 assert.deepEqual(validateSelection(selection),selection);
 assert.deepEqual(engineArguments(selection),['evaluate-rollout','--plan','probes/phase14-plans/principal-removal-positive-control.json','--target-view','after_transition','--format','json']);
 assert(!engineArguments(selection).includes('guard-rollout'));
});
test('completed policy outcomes 0 3 4 5 stay separate from processing errors',()=>{
 for(const c of published.cases){const a=c.assessment;const check=c.assessment.candidate.id.includes('positive')?'principal-removal':c.assessment.candidate.id.includes('failed-route')?'required-sale':c.assessment.candidate.id.includes('transfer')?'transfer-conversion':'complete-exit';const r=parseEngineResult({...selection,check},a.evaluation_command_exit_code,JSON.stringify(a));assert.equal(r.assessment.readiness.overall_status,a.readiness.overall_status);}
 const rejected={...positive,candidate_acceptance:'NotAccepted',evaluation_command_exit_code:5};assert.equal(parseEngineResult(selection,5,JSON.stringify(rejected)).assessment.candidate_acceptance,'NotAccepted');
 assert.throws(()=>parseEngineResult(selection,2,JSON.stringify(positive)));
 assert.throws(()=>parseEngineResult({...selection,stage:'before_transition'},0,JSON.stringify(positive)));
});
test('duplicate concurrent requests invoke one job and serialized jobs stay bounded',async()=>{
 let calls=0,release;const latch=new Promise(r=>release=r);
 const {service,directory}=await make(async()=>{calls++;await latch;return {code:0,stdout:JSON.stringify(positive)};});
 try{const key=randomUUID();const [a,b]=await Promise.all([service.submit(selection,key),service.submit(selection,key)]);assert.equal(a.id,b.id);assert.equal(calls<=1,true);for(let i=0;i<4;i++)await service.submit(selection,randomUUID());await assert.rejects(service.submit(selection,randomUUID()),/QueueFull/);release();await finished(service);assert.equal(calls,5);assert.equal(service.get(a.id).status,'Completed');assert.equal(service.get(a.id).cached,false);const restored=new AnalysisService({root,directory});await restored.initialize();assert.equal(restored.get(a.id).result.assessment.readiness.evaluated_scope,'DemoEntityReadiness');assert.equal((await restored.artifact(a.id)).toString(),JSON.stringify(positive));}finally{release();await rm(directory,{recursive:true,force:true});}
});
test('timeout verification failure and missing executable yield honest errors without result',async()=>{
 for(const code of ['AnalysisTimeout','EvidenceVerificationFailed']){const {service,directory}=await make(async()=>{throw new Error(code);});try{const job=await service.submit(selection,randomUUID());await finished(service);assert.equal(job.status,'Error');assert.equal(job.error.code,code);assert.equal(job.result,null);}finally{await rm(directory,{recursive:true,force:true});}}
 const {service,directory}=await make(()=>{});try{service.executable=resolve(directory,'missing');await assert.rejects(service.submit(selection,randomUUID()),/BackendUnavailable/);}finally{await rm(directory,{recursive:true,force:true});}
});
test('actual child timeout and nonzero verification exit are errors',async()=>{
 await assert.rejects(executeEngine(process.execPath,['-e','setInterval(()=>{},1000)'],{cwd:root,timeoutMs:30}),/AnalysisTimeout/);
 await assert.rejects(executeEngine(process.execPath,['-e','process.exit(2)'],{cwd:root}),/EvidenceVerificationFailed/);
});
test('restart makes interrupted job an error rather than silently restarting it',async()=>{
 const directory=await mkdtemp(resolve(tmpdir(),'eplyx-phase15-restart-'));try{const id=randomUUID();await writeFile(resolve(directory,`${id}.json`),JSON.stringify({id,selection,status:'Running'}));const s=new AnalysisService({root,directory});await s.initialize();assert.equal(s.get(id).status,'Error');assert.equal(s.get(id).error.code,'AnalysisInterrupted');}finally{await rm(directory,{recursive:true,force:true});}
});
test('integer formatting never loses large values or rounds small positives to zero',()=>{
 assert.equal(tokenAmount('9007199254740993123456',6),'9,007,199,254,740,993.123456');assert.equal(tokenAmount('1',9),'0.000000001');assert.equal(tokenAmount('1000000',6),'1');
});
test('real engine rejects a tampered isolated trust binding server-side',async()=>{
 const directory=await mkdtemp(resolve(tmpdir(),'eplyx-phase15-tamper-'));
 try {
  const binding=JSON.parse(await readFile('probes/phase14-evidence-binding.json','utf8'));binding.parent_policy.sha256='tampered';binding.parent_policy.file=resolve(root,'policies/stocklana-spacex-preflight-v1.json');binding.counterfactual.file=resolve(root,'reports/spacex-counterfactual-lifecycle.json');
  const file=resolve(directory,'binding.json');await writeFile(file,JSON.stringify(binding));
  const s=new AnalysisService({root,directory,runner:(executable,args,options)=>executeEngine(executable,[...args,'--binding',file],options)});await s.initialize();
  const job=await s.submit(selection,randomUUID());await finished(s);assert.equal(job.status,'Error');assert.equal(job.error.code,'EvidenceVerificationFailed');assert.equal(job.result,null);
 } finally {await rm(directory,{recursive:true,force:true});}
});
test('all scenario dates and example scopes select the actual engine view without changing proof',async()=>{
 const artifact=await readFile('reports/spacex-counterfactual-lifecycle.json','utf8');
 const outputs=[];
 for(const stage of ['before_transition','after_transition','after_deadline'])for(const review of ['overview','holding','position']){
  const s={asset:'spacex',review,stage,check:null};const r=parseEngineResult(validateSelection(s),0,artifact);assert.equal(r.view.scenario.id,stage);assert.equal(r.view.direct_holder.raw_balance,'17621');assert.equal(r.readiness.overall_status,'Incomplete');outputs.push(r.view.lifecycle_status);
 }
 assert(outputs.includes('Active'));assert(outputs.includes('TransitionRequired'));assert(outputs.includes('NoIssuerEntitlement'));
});

const freshSelection={asset:'mint',review:'current',stage:null,check:null,cluster:'solana-mainnet',mint:'So11111111111111111111111111111111111111112',catalogue_version:null,sample_accounts:true};
const currentArtifact=()=>({schema_version:2,kind:'current-inspection',asset:{mint:freshSelection.mint},selection:{cluster:freshSelection.cluster,mint:freshSelection.mint,reference:null,sample_accounts:true},inspection:{status:'Completed'},lifecycle_event:null,execution_performed:false,funds_moved:false,signer_assumed_locally:false,signer_possession_known:false,readiness:null,authorization:false,local_execution_performed:false,paths:['OfficialTransition','Redemption','SecondaryMarketExit','Transfer','Withdrawal'].map(path=>({path,status:'NotTested'}))});
test('current requests accept no saved scenario, endpoint, evidence, or filesystem scope',()=>{
 assert.deepEqual(validateSelection(freshSelection),freshSelection);
 for(const bad of [{...freshSelection,stage:'after_transition'},{...freshSelection,check:'principal-removal'},{...freshSelection,rpc:'http://attacker'},{...freshSelection,out:'/tmp/arbitrary'},{...freshSelection,evidence:{status:'Proven'}}])assert.throws(()=>validateSelection(bad));
 assert.deepEqual(engineArguments(freshSelection,'/server/run.capture.json','/server/run.selection.json'),['inspect-current','--mint',freshSelection.mint,'--selection','/server/run.selection.json','--out','/server/run.capture.json']);
});
test('new captures never accept historical statuses or readiness grants',()=>{
 for(const mutate of [v=>v.paths[0].status='Proven',v=>v.readiness={overall_status:'Ready'},v=>v.asset.mint='other',v=>v.local_execution_performed=true,v=>v.authorization=true]) {
  const value=currentArtifact();mutate(value);assert.throws(()=>parseEngineResult(freshSelection,0,JSON.stringify(value)));
 }
 assert.throws(()=>parseEngineResult(freshSelection,0,JSON.stringify(positive)));
});
test('refresh invokes acquisition again and keeps scoped capture digests, sessions and prior run',async()=>{
 let calls=0;
 const {service,directory}=await make(async(_exe,args,options)=>{
  calls++;assert.equal(options.fresh,true);assert.equal(args[0],'inspect-current');
  options.onStage('Fetching current mint state');
  await writeFile(args[6],JSON.stringify({test_capture:calls}));
  return {code:0,stdout:JSON.stringify(currentArtifact())};
 });
 try {
  const key=randomUUID();const first=await service.submit(freshSelection,key,'owner-a');await finished(service);
  const second=await service.submit(freshSelection,randomUUID(),'owner-a');await finished(service);
  const other=await service.submit(freshSelection,key,'owner-b');await finished(service);
  assert.equal(calls,3);assert.notEqual(first.id,second.id);assert.notEqual(first.id,other.id);
  assert.notEqual(first.capture_sha256,second.capture_sha256);assert.equal(first.status,'Completed');
  const manifest=JSON.parse(await readFile(resolve(directory,`${second.id}.manifest.json`)));assert.equal(manifest.capture_sha256,second.capture_sha256);
  const restarted=new AnalysisService({root,directory});await restarted.initialize();assert.equal(restarted.get(first.id).status,'Completed');
  await writeFile(resolve(directory,`${first.id}.capture.json`),'tampered');await assert.rejects(()=>service.artifact(first.id,true),/EvidenceVerificationFailed/);
 } finally {await rm(directory,{recursive:true,force:true});}
});
test('fresh acquisition failure never runs the historical engine or substitutes a saved result',async()=>{
 const argsSeen=[];const {service,directory}=await make(async(_exe,args)=>{argsSeen.push(args);throw new Error('EngineFailure');});
 try {const job=await service.submit(freshSelection,randomUUID());await finished(service);assert.equal(job.status,'Error');assert.equal(job.result,null);assert.equal(job.authorization,false);assert.equal(argsSeen.length,1);assert.equal(argsSeen[0][0],'inspect-current');assert.match(job.error.message,/No saved example was substituted/);}finally{await rm(directory,{recursive:true,force:true});}
});

test('wallet scope requires exact public owner and rejects injected settings and historical proof',async()=>{
 const s={...freshSelection,scope:'wallet',public_owner:'8qbHbw2BbbTHBW1sbeqakYXVKRQM8Ne7pLK7m6CVfeR',sample_accounts:false};
 assert.deepEqual(validateSelection(s),s);
 for(const bad of [{...s,scope:'position'},{...s,sample_accounts:true},{...s,public_owner:null},{...s,rpc_url:'https://evil'},{...s,focused_account:'arbitrary'},{...s,execution_performed:true}])assert.throws(()=>validateSelection(bad));
 const a=currentArtifact();a.schema_version=3;a.selection.sample_accounts=false;a.selection.public_owner=s.public_owner;a.wallet_observation={submitted_owner:s.public_owner,selected_mint:s.mint,status:'Completed',token_accounts:[]};
 assert.equal(parseEngineResult(s,0,JSON.stringify(a)).wallet_observation.submitted_owner,s.public_owner);
 for(const mutate of [v=>v.wallet_observation.submitted_owner='other',v=>v.wallet_observation.selected_mint='other',v=>v.selection.public_owner='other',v=>v.paths[0].status='Proven',v=>v.wallet_observation.token_accounts=[{state:{owner:'other',mint:s.mint}}],v=>v.readiness={overall_status:'Ready'}]){const b=structuredClone(a);mutate(b);assert.throws(()=>parseEngineResult(s,0,JSON.stringify(b)));}
 let calls=0;const {service,directory}=await make(async()=>{calls++;throw new Error('ShouldNotRun');});
 try{await assert.rejects(service.submit({...s,public_owner:'invalid wallet'},randomUUID()),/InvalidOwner/);assert.equal(calls,0);assert.equal(service.jobs.size,0);}finally{await rm(directory,{recursive:true,force:true});}
});
test('decimal formatting preserves u8 mint precision without floating point',()=>{
 assert.equal(tokenAmount('1',255),'0.'+'0'.repeat(254)+'1');
 assert.equal(tokenAmount('36893488147419103230',9),'36,893,488,147.41910323');
});

test('current execution API rejects caller-supplied proof programs instructions and cross-session parent',async()=>{
 const {service,directory}=await make(async()=>{throw new Error('must not execute');});
 try {
  const parent={id:randomUUID(),owner:'owner-a',status:'Completed',selection:{...freshSelection,scope:'wallet'},capture_sha256:'fixture-not-used'};service.jobs.set(parent.id,parent);
  const request={path:'Transfer',source:'741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs',recipient:'123aUGPWa93jiga876U3rLdBP86JNFSoz9tSQWCAskMc',amount_mode:'Full',amount_decimal:null};
  for(const key of ['program_id','account_metas','instructions','status','evidence_hash','rpc_url','executable','signature'])await assert.rejects(service.submitCheck(parent.id,{...request,[key]:'Proven'},randomUUID(),'owner-a'),/InvalidCheck/);
  await assert.rejects(service.submitCheck(parent.id,request,randomUUID(),'owner-b'),/InvalidParent/);
  assert.equal(parent.status,'Completed');assert.equal(service.queue.length,0);
 }finally{await rm(directory,{recursive:true,force:true});}
});
test('offline execution child receives no provider or unrelated secret environment',async()=>{
 const oldRpc=process.env.SOLANA_RPC_URL,oldSecret=process.env.EPLYX_TEST_SECRET;
 try{
  process.env.SOLANA_RPC_URL='https://fixture.invalid/test-provider-key';process.env.EPLYX_TEST_SECRET='synthetic-test-secret';
  const code='console.log(JSON.stringify({rpc:!!process.env.SOLANA_RPC_URL,secret:!!process.env.EPLYX_TEST_SECRET,path:!!process.env.PATH}))';
  const offline=await executeEngine(process.execPath,['-e',code],{cwd:root,fresh:false});assert.deepEqual(JSON.parse(offline.stdout),{rpc:false,secret:false,path:true});
  const capture=await executeEngine(process.execPath,['-e',code],{cwd:root,fresh:true});assert.deepEqual(JSON.parse(capture.stdout),{rpc:true,secret:false,path:true});
 }finally{if(oldRpc===undefined)delete process.env.SOLANA_RPC_URL;else process.env.SOLANA_RPC_URL=oldRpc;if(oldSecret===undefined)delete process.env.EPLYX_TEST_SECRET;else process.env.EPLYX_TEST_SECRET=oldSecret;}
});
