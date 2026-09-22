import {readFile, writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createHash, randomUUID} from 'node:crypto';
import {executeEngine, validRunId} from './analysis-service.mjs';
import {conversionMechanism} from './conversion-service.mjs';
const hash=b=>createHash('sha256').update(b).digest('hex');
const isDigest=v=>typeof v==='string'&&/^[a-f0-9]{64}$/.test(v);

/**
 * The browser asks for exactly one thing: stress-test the registered candidate
 * plan that this completed conversion already used. It supplies no budget, no
 * mint, no account, no amount, no program, no endpoint, no path and no status.
 * Everything below is server-controlled.
 */
export async function stressBudget(service){
 const {stdout}=await executeEngine(service.executable,['conversion-stress-budget'],{cwd:service.root,timeoutMs:10000});
 const value=JSON.parse(stdout);
 if(value.accepts_browser_supplied_budget!==false||value.historical_population_used!==false)throw new Error('EvidenceVerificationFailed');
 return value;
}

export async function createStress(service,parentId,key,owner){
 const parent=service.get(parentId);
 // The parent is a completed candidate conversion: it pins the exact plan.
 if(!parent||parent.owner!==owner||parent.status!=='Completed'||!parent.conversion_request||!parent.plan_sha256)throw new Error('InvalidParent');
 if(!validRunId(key))throw new Error('InvalidRequestKey');
 const scopedKey=`${owner}:${key}`;
 if(service.keys.has(scopedKey)){
  const old=service.jobs.get(service.keys.get(scopedKey));
  if(old.parent_run_id!==parentId||!old.stress_request)throw new Error('RequestKeyConflict');
  return old;
 }
 if(service.queue.length>=4||service.jobs.size>=200)throw new Error('QueueFull');
 await service.artifact(parentId); // Verify the immutable parent conversion bytes first.
 const budget=await stressBudget(service);
 const id=randomUUID();
 const job={id,owner,request_key:key,selection:parent.selection,parent_run_id:parentId,
  parent_conversion_sha256:parent.canonical_sha256,parent_plan_sha256:parent.plan_sha256,
  wallet_run_id:parent.parent_run_id,
  stress_request:{candidate_plan_sha256:parent.plan_sha256},
  budget:budget.budget,classifier_version:budget.classifier_version,selector_version:budget.selector_version,
  status:'Queued',created_at:new Date().toISOString(),
  operation:'Fresh bounded read-only current population discovery, a frozen deterministic test plan, and offline local execution of the same candidate conversion for each selected exact account',
  authorization:false,result:null,error:null};
 service.jobs.set(id,job);service.keys.set(scopedKey,id);await service.save(job);service.queue.push(job);void service.process();return job;
}

export async function executeStress(service,job){
 const parent=service.get(job.parent_run_id);
 if(!parent||parent.canonical_sha256!==job.parent_conversion_sha256||parent.plan_sha256!==job.parent_plan_sha256)throw new Error('EvidenceVerificationFailed');
 const mechanism=await conversionMechanism(service);
 job.candidate_program_sha256=mechanism.program_sha256;
 const file=name=>resolve(service.directory,`${job.id}.${name}`);
 const seconds=Number(job.budget?.population_timeout_seconds)||420;
 const options={cwd:service.root,onStage:stage=>{job.stage=stage;}};

 // 1. A new acquisition. No saved population is read or substituted anywhere.
 job.stage='Discovering current token accounts';
 await service.runner(service.executable,
  ['capture-conversion-stress-population','--mint',job.selection.mint,'--run-id',job.wallet_run_id,'--stress-id',job.id,'--out',file('population.json')],
  {...options,timeoutMs:(seconds+120)*1000,fresh:true});
 if(hash(await readFile(service.executable))!==job.engine_sha256)throw new Error('EvidenceVerificationFailed');
 job.population_capture_sha256=hash(await readFile(file('population.json')));

 // 2. Freeze the plan before any case is captured or executed.
 job.stage='Freezing the test plan';
 const planned=await service.runner(service.executable,
  ['plan-conversion-stress','--population',file('population.json'),'--plan',resolve(service.directory,`${parent.id}.plan.json`),'--out',file('stressplan.json')],
  {...options,timeoutMs:300000,fresh:false});
 const summary=JSON.parse(planned.stdout);
 job.stress_plan_sha256=hash(await readFile(file('stressplan.json')));
 if(summary.stress_plan_sha256!==job.stress_plan_sha256||summary.population_capture_sha256!==job.population_capture_sha256
  ||summary.candidate_plan_sha256!==job.parent_plan_sha256||summary.candidate_program_sha256!==mechanism.program_sha256
  ||summary.frozen_before_execution!==true)throw new Error('EvidenceVerificationFailed');
 job.plan_summary=summary;
 await service.save(job);

 // 3. Fresh per-case revalidation, in frozen plan order.
 job.stage='Revalidating current state per case';
 await service.runner(service.executable,
  ['capture-conversion-stress-cases','--population',file('population.json'),'--stress-plan',file('stressplan.json'),'--out',file('cases.json')],
  {...options,timeoutMs:(seconds+120)*1000,fresh:true});
 if(hash(await readFile(service.executable))!==job.engine_sha256)throw new Error('EvidenceVerificationFailed');
 job.cases_capture_sha256=hash(await readFile(file('cases.json')));
 // The frozen plan must be byte-identical to what was planned before capture.
 if(hash(await readFile(file('stressplan.json')))!==job.stress_plan_sha256)throw new Error('EvidenceVerificationFailed');

 // 4. Offline execution and aggregation. This child receives PATH only: no RPC
 //    URL or provider secret can reach the VM.
 job.stage='Running candidate conversions locally';
 const {code,stdout}=await service.runner(service.executable,
  ['replay-conversion-stress','--population',file('population.json'),'--stress-plan',file('stressplan.json'),'--cases',file('cases.json'),
   '--run-id',job.wallet_run_id,'--stress-id',job.id,'--population-sha256',job.population_capture_sha256,
   '--stress-plan-sha256',job.stress_plan_sha256,'--cases-sha256',job.cases_capture_sha256,'--program-sha256',mechanism.program_sha256],
  {...options,timeoutMs:900000,fresh:false});
 if(hash(await readFile(service.executable))!==job.engine_sha256)throw new Error('EvidenceVerificationFailed');
 const result=JSON.parse(stdout);
 verifyStressResult(result,job,parent,mechanism,code);
 job.result=result;job.engine_exit_code=code;job.canonical_sha256=hash(stdout);
 await writeFile(file('artifact'),stdout,{flag:'wx'});
 await writeFile(file('manifest.json'),JSON.stringify({schema_version:1,stress_id:job.id,run_id:job.wallet_run_id,
  parent_conversion_id:parent.id,parent_conversion_sha256:job.parent_conversion_sha256,
  population_capture_sha256:job.population_capture_sha256,stress_plan_sha256:job.stress_plan_sha256,
  cases_capture_sha256:job.cases_capture_sha256,candidate_plan_sha256:job.parent_plan_sha256,
  candidate_program_sha256:mechanism.program_sha256,result_sha256:job.canonical_sha256,engine_sha256:job.engine_sha256,
  replay_command:['replay-conversion-stress','--population',`${job.id}.population.json`,'--stress-plan',`${job.id}.stressplan.json`,
   '--cases',`${job.id}.cases.json`,'--run-id',job.wallet_run_id,'--stress-id',job.id,
   '--population-sha256',job.population_capture_sha256,'--stress-plan-sha256',job.stress_plan_sha256,
   '--cases-sha256',job.cases_capture_sha256,'--program-sha256',mechanism.program_sha256]}),{flag:'wx'});
}

/**
 * Structural checks the service will not take on trust from the engine. These
 * mirror the engine's own invariants: a serialized status is never evidence.
 */
export function verifyStressResult(result,job,parent,mechanism,code){
 const c=result?.coverage_summary,plan=result?.selection_plan;
 if(code!==0||result?.kind!=='current-conversion-stress'||result.stress_id!==job.id||result.run_id!==job.wallet_run_id
  ||result.asset_mint!==job.selection.mint
  ||result.population_capture?.capture_sha256!==job.population_capture_sha256
  ||result.population_capture?.historical_population_used!==false
  ||result.candidate_plan_sha256!==job.parent_plan_sha256
  ||result.candidate_mechanism?.program_sha256!==mechanism.program_sha256
  ||result.candidate_mechanism?.deployed_on_mainnet!==false
  ||result.candidate_mechanism?.issuer_mechanism!==false
  ||plan?.stress_plan_sha256!==job.stress_plan_sha256
  ||plan?.frozen_before_execution!==true
  ||result.official_transition!=='NotTested'
  ||result.funds_moved!==false||result.authorization!==false)throw new Error('InvalidEngineResult');
 if(!Array.isArray(result.results)||!Array.isArray(result.selected_cases)
  ||result.results.length!==result.selected_cases.length
  ||c?.exact_accounts_selected!==result.selected_cases.length)throw new Error('InvalidEngineResult');
 // No case may claim issuer binding, key possession, fund movement or an
 // official transition, and every result stays bound to its frozen case.
 for(const [i,r] of result.results.entries()){
  const selected=result.selected_cases[i];
  if(r.case_id!==selected.case_id||r.entity_id!==selected.entity_id||r.token_account!==selected.token_account
   ||r.selected_amount_raw!==selected.selected_amount_raw||r.case_plan_sha256!==selected.case_plan_sha256
   ||r.official_transition!=='NotTested'||r.issuer_binding_established!==false
   ||r.signer_possession_known!==false||r.funds_moved!==false
   ||r.candidate_program_sha256!==mechanism.program_sha256
   ||!['Proven','Failed','Indeterminate','Unsupported'].includes(r.status))throw new Error('InvalidEngineResult');
  if(r.status==='Proven'&&(r.execution_performed!==true||r.local_execution_performed!==true))throw new Error('InvalidEngineResult');
 }
 // Sampled evidence must never be reported as population readiness.
 const proven=result.results.filter(r=>r.status==='Proven').length;
 if(c.exact_accounts_proven!==proven)throw new Error('InvalidEngineResult');
 if(proven>c.positive_balance_accounts_observed)throw new Error('InvalidEngineResult');
 if(!['Ready','Blocked','Incomplete'].includes(result.readiness?.status))throw new Error('InvalidEngineResult');
 if(result.readiness?.scope!=='ConversionStressReadiness'||result.readiness?.population_readiness!==null)throw new Error('InvalidEngineResult');
 if(result.population_rollout_readiness?.scope!=='PopulationRolloutReadiness')throw new Error('InvalidEngineResult');
 if(c.exact_accounts_proven<c.positive_balance_accounts_observed
  &&result.population_rollout_readiness?.status==='Ready')throw new Error('InvalidEngineResult');
 // State-shape coverage may never be reported as entity coverage. Members of a
 // shape are not tested entities, so the totals across shapes can never exceed
 // the exact cases that were actually selected and actually executed.
 let shapeSelected=0,shapeExecuted=0;
 for(const shape of result.shape_coverage||[]){
  if(shape.entities_executed>shape.entities_selected||shape.entities_selected>shape.entities_in_shape
   ||shape.executed_entity_ids.length!==shape.entities_executed
   ||shape.entities_untested!==shape.entities_in_shape-shape.entities_executed)throw new Error('InvalidEngineResult');
  shapeSelected+=shape.entities_selected;shapeExecuted+=shape.entities_executed;
 }
 if(shapeSelected>result.selected_cases.length
  ||shapeExecuted>result.results.filter(r=>r.execution_performed).length)throw new Error('InvalidEngineResult');
 if(!isDigest(job.population_capture_sha256)||!isDigest(job.stress_plan_sha256)||!isDigest(job.cases_capture_sha256))throw new Error('InvalidEngineResult');
 if(parent.selection.mint!==result.asset_mint)throw new Error('InvalidEngineResult');
}
