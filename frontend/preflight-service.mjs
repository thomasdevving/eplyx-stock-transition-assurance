import {readFile,writeFile,unlink} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createHash,randomUUID} from 'node:crypto';
import {executeEngine,validRunId} from './analysis-service.mjs';
const hash=b=>createHash('sha256').update(b).digest('hex');
export function validatePreflightFields(request){
 if(!request||typeof request!=='object'||Array.isArray(request)||Object.keys(request).some(k=>!['source','successor_mint','effective_at','deadline','post_deadline','assurance','check_ids'].includes(k))||typeof request.source!=='string'||request.source.length>44||(request.successor_mint!==null&&(typeof request.successor_mint!=='string'||request.successor_mint.length>44))||typeof request.effective_at!=='string'||request.effective_at.length>40||(request.deadline!==null&&(typeof request.deadline!=='string'||request.deadline.length>40))||![null,'TransitionStillRequired'].includes(request.post_deadline)||!['Mobility','FullTransition'].includes(request.assurance)||!Array.isArray(request.check_ids)||request.check_ids.length>4||request.check_ids.some(id=>!validRunId(id))||new Set(request.check_ids).size!==request.check_ids.length)throw new Error('InvalidPreflight');
 return request;
}
export async function createPreflight(service,parentId,request,key,owner){
 validatePreflightFields(request);
 request={...request,check_ids:[...request.check_ids].sort()};
 const parent=service.get(parentId);
 if(!parent||parent.owner!==owner||parent.status!=='Completed'||parent.selection.scope!=='wallet'||parent.check_request||parent.preflight_request)throw new Error('InvalidParent');
 if(!validRunId(key))throw new Error('InvalidRequestKey');
 const scopedKey=`${owner}:${key}`;
 if(service.keys.has(scopedKey)){const old=service.jobs.get(service.keys.get(scopedKey));if(old.parent_run_id!==parentId||JSON.stringify(old.preflight_request)!==JSON.stringify(request))throw new Error('RequestKeyConflict');return old;}
 if(service.queue.length>=4||service.jobs.size>=200)throw new Error('QueueFull');
 const wallet=await service.artifact(parentId,true),checks=[];
 for(const id of request.check_ids){
  const check=service.get(id);
  if(!check||check.owner!==owner||check.parent_run_id!==parentId||check.status!=='Completed'||!check.check_request||check.check_request.source!==request.source||check.parent_capture_sha256!==parent.capture_sha256)throw new Error('InvalidCheckSelection');
  await service.artifact(id);
  checks.push({id,capture_sha256:check.capture_sha256,result_sha256:check.canonical_sha256,engine_sha256:check.engine_sha256,capture:(await service.artifact(id,true)).toString()});
 }
 checks.sort((a,b)=>a.id.localeCompare(b.id));request={...request,check_ids:[...request.check_ids].sort()};
 const id=randomUUID(),created_at=new Date().toISOString();
 let successorCapture=null;
 if(request.successor_mint){
  const previous=[...service.jobs.values()].reverse().find(j=>j.parent_run_id===parentId&&j.owner===owner&&j.status==='Completed'&&j.preflight_request?.source===request.source&&j.preflight_request?.successor_mint===request.successor_mint&&j.result?.successor_verification.status==='MintObserved');
  if(previous){const prior=JSON.parse((await service.artifact(previous.id,true)).toString());successorCapture=prior.inputs.successor_capture;}
 }
 const input={run_id:parentId,preflight_id:id,created_at,engine_sha256:hash(await readFile(service.executable)),wallet_capture:wallet.toString(),wallet_sha256:parent.capture_sha256,request,successor_capture:successorCapture,checks};
 const path=resolve(service.directory,`${id}.preflight-input.json`);await writeFile(path,JSON.stringify(input),{flag:'wx'});
 try{await executeEngine(service.executable,['validate-current-preflight','--input',path],{cwd:service.root,timeoutMs:10000});}catch{await unlink(path);throw new Error('InvalidPreflight');}
 const job={id,owner,request_key:key,selection:parent.selection,parent_run_id:parentId,parent_capture_sha256:parent.capture_sha256,preflight_request:request,status:'Queued',created_at,operation:'User-proposed prospective pre-flight over unchanged current observations',authorization:false,result:null,error:null};
 service.jobs.set(id,job);service.keys.set(scopedKey,id);await service.save(job);service.queue.push(job);void service.process();return job;
}
export async function executePreflight(service,job){
 const parent=service.get(job.parent_run_id);if(!parent||parent.capture_sha256!==job.parent_capture_sha256)throw new Error('EvidenceVerificationFailed');
 await service.artifact(parent.id,true);
 const input=JSON.parse(await readFile(resolve(service.directory,`${job.id}.preflight-input.json`),'utf8'));
 input.engine_sha256=job.engine_sha256;
 const options={cwd:service.root,timeoutMs:120000,onStage:stage=>{job.stage=stage;}};
 if(job.preflight_request.successor_mint&&!input.successor_capture){
  job.stage='Inspecting proposed replacement mint';
  const path=resolve(service.directory,`${job.id}.successor.capture.json`);
  await service.runner(service.executable,['capture-preflight-successor','--mint',job.preflight_request.successor_mint,'--out',path],{...options,fresh:true});
  input.successor_capture=await readFile(path,'utf8');
 }
 if(hash(await readFile(service.executable))!==job.engine_sha256)throw new Error('EvidenceVerificationFailed');
 job.stage='Preparing hypothetical transition';
 const prepared=resolve(service.directory,`${job.id}.prepared-input.json`),capture=resolve(service.directory,`${job.id}.capture.json`);
 await writeFile(prepared,JSON.stringify(input),{flag:'wx'});
 const {stdout:scenarioJSON}=await service.runner(service.executable,['prepare-current-preflight','--input',prepared,'--out',capture],{...options,fresh:false});
 const scenario=JSON.parse(scenarioJSON);job.scenario_sha256=scenario.scenario_sha256;
 await writeFile(resolve(service.directory,`${job.id}.scenario.json`),JSON.stringify(scenario.scenario),{flag:'wx'});
 job.capture_sha256=hash(await readFile(capture));job.stage='Replaying selected checks and evaluating pre-flight';
 const {code,stdout}=await service.runner(service.executable,['replay-current-preflight','--input',capture,'--run-id',parent.id,'--preflight-id',job.id,'--wallet-sha256',parent.capture_sha256,'--scenario-sha256',job.scenario_sha256,'--capture-sha256',job.capture_sha256],{...options,fresh:false});
 if(hash(await readFile(service.executable))!==job.engine_sha256)throw new Error('EvidenceVerificationFailed');
 const result=JSON.parse(stdout);
 if(code!==0||result.kind!=='current-preflight'||result.run_id!==parent.id||result.preflight_id!==job.id||result.wallet_capture_sha256!==parent.capture_sha256||result.scenario_sha256!==job.scenario_sha256||result.bundle_sha256!==job.capture_sha256||result.engine_sha256!==job.engine_sha256||result.entity.account!==job.preflight_request.source||result.entity.mint!==parent.selection.mint||result.proposed_change.source!=='UserProposed'||result.authorization!==false||result.funds_moved!==false||result.population_readiness!==null)throw new Error('InvalidEngineResult');
 job.result=result;job.engine_exit_code=code;job.canonical_sha256=hash(stdout);
 await writeFile(resolve(service.directory,`${job.id}.artifact`),stdout,{flag:'wx'});
 await writeFile(resolve(service.directory,`${job.id}.manifest.json`),JSON.stringify({schema_version:1,run_id:parent.id,preflight_id:job.id,wallet_sha256:parent.capture_sha256,scenario_sha256:job.scenario_sha256,capture_sha256:job.capture_sha256,result_sha256:job.canonical_sha256,engine_sha256:job.engine_sha256,request:job.preflight_request}),{flag:'wx'});
}
