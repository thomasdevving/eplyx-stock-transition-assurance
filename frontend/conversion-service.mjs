import {readFile,writeFile,unlink} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createHash,randomUUID} from 'node:crypto';
import {executeEngine,validRunId} from './analysis-service.mjs';
const hash=b=>createHash('sha256').update(b).digest('hex');
const FIELDS=['source','replacement_mint','amount_mode','amount_decimal','ratio_numerator','ratio_denominator','rounding','conversion_fee_bps','reserve_funded_replacement_raw'];
const digits=(value,max)=>typeof value==='string'&&value.length>0&&value.length<=max&&/^[0-9]+$/.test(value);
/** The browser chooses bounded terms only: never a program, instruction, account, path or endpoint. */
export function validateConversionFields(request){
 if(!request||typeof request!=='object'||Array.isArray(request)||Object.keys(request).some(k=>!FIELDS.includes(k)))throw new Error('InvalidConversion');
 const {source,replacement_mint,amount_mode,amount_decimal,ratio_numerator,ratio_denominator,rounding,conversion_fee_bps,reserve_funded_replacement_raw}=request;
 if(typeof source!=='string'||source.length<32||source.length>44)throw new Error('InvalidConversion');
 if(typeof replacement_mint!=='string'||replacement_mint.length<32||replacement_mint.length>44)throw new Error('InvalidConversion');
 if(!['Full','Custom'].includes(amount_mode))throw new Error('InvalidConversion');
 if(amount_mode==='Custom'?typeof amount_decimal!=='string'||amount_decimal.length<1||amount_decimal.length>280||!/^[0-9]+(\.[0-9]+)?$/.test(amount_decimal):amount_decimal!==null)throw new Error('InvalidConversion');
 if(!Number.isSafeInteger(ratio_numerator)||!Number.isSafeInteger(ratio_denominator)||ratio_numerator<1||ratio_denominator<1||ratio_numerator>1e15||ratio_denominator>1e15)throw new Error('InvalidConversion');
 if(!['Floor','Ceiling'].includes(rounding))throw new Error('InvalidConversion');
 if(!Number.isSafeInteger(conversion_fee_bps)||conversion_fee_bps<0||conversion_fee_bps>10000)throw new Error('InvalidConversion');
 if(!digits(reserve_funded_replacement_raw,20))throw new Error('InvalidConversion');
 return request;
}
/** The plan the engine executes. Mechanism, design and provenance are server-fixed. */
export function conversionPlan(request,sourceMint,id){
 return {schema_version:1,id,version:1,provenance:'OperatorSupplied',mechanism:'EplyxDemoCandidateConversion',
  adapter_id:'eplyx-demo-candidate-conversion/v1',mechanism_ref:'Eplyx Demo Candidate Conversion (registered repository candidate)',
  source_mint:sourceMint,replacement_mint:request.replacement_mint,source_account:request.source,
  amount_mode:request.amount_mode,...(request.amount_mode==='Custom'?{amount_decimal:request.amount_decimal}:{}),
  terms:{ratio_numerator:request.ratio_numerator,ratio_denominator:request.ratio_denominator,rounding:request.rounding,conversion_fee_bps:request.conversion_fee_bps},
  authority_model:{holder_signs:true,candidate_authority:'ProgramDerived'},
  source_consumption:'Burn',replacement_delivery:'ProposedReserveRelease',
  reserve:{funded_replacement_raw:request.reserve_funded_replacement_raw}};
}
export async function conversionMechanism(service){
 const {stdout}=await executeEngine(service.executable,['conversion-mechanism'],{cwd:service.root,timeoutMs:10000});
 const value=JSON.parse(stdout);
 if(value.deployed_on_mainnet!==false||value.issuer_mechanism!==false||value.accepts_uploaded_programs!==false)throw new Error('EvidenceVerificationFailed');
 return value;
}
export async function createConversion(service,parentId,request,key,owner){
 validateConversionFields(request);
 const parent=service.get(parentId);
 if(!parent||parent.owner!==owner||parent.status!=='Completed'||parent.selection.scope!=='wallet'||parent.check_request||parent.preflight_request||parent.conversion_request)throw new Error('InvalidParent');
 if(request.replacement_mint===parent.selection.mint)throw new Error('InvalidConversion');
 if(!validRunId(key))throw new Error('InvalidRequestKey');
 const scopedKey=`${owner}:${key}`;
 if(service.keys.has(scopedKey)){const old=service.jobs.get(service.keys.get(scopedKey));if(old.parent_run_id!==parentId||JSON.stringify(old.conversion_request)!==JSON.stringify(request))throw new Error('RequestKeyConflict');return old;}
 if(service.queue.length>=4||service.jobs.size>=200)throw new Error('QueueFull');
 await service.artifact(parentId,true); // Verify the immutable parent bytes first.
 const id=randomUUID(),plan=conversionPlan(request,parent.selection.mint,id);
 const planPath=resolve(service.directory,`${id}.plan.json`);
 await writeFile(planPath,JSON.stringify(plan),{flag:'wx'});
 let validated;
 try{const response=await executeEngine(service.executable,['validate-conversion-plan','--input',resolve(service.directory,`${parentId}.capture.json`),'--plan',planPath],{cwd:service.root,timeoutMs:15000});validated=JSON.parse(response.stdout);}
 catch{await unlink(planPath);throw new Error('InvalidConversion');}
 const job={id,owner,request_key:key,selection:parent.selection,parent_run_id:parentId,parent_capture_sha256:parent.capture_sha256,
  conversion_request:request,conversion_plan:plan,plan_sha256:validated.plan_sha256,validated_request:validated,
  status:'Queued',created_at:new Date().toISOString(),
  operation:'Fresh read-only revalidation followed by offline local execution of the operator-supplied candidate conversion plan',
  authorization:false,result:null,error:null};
 service.jobs.set(id,job);service.keys.set(scopedKey,id);await service.save(job);service.queue.push(job);void service.process();return job;
}
export async function executeConversion(service,job){
 const parent=service.get(job.parent_run_id);
 if(!parent||parent.capture_sha256!==job.parent_capture_sha256)throw new Error('EvidenceVerificationFailed');
 await service.artifact(parent.id,true);
 const mechanism=await conversionMechanism(service);
 job.candidate_program_sha256=mechanism.program_sha256;
 const capturePath=resolve(service.directory,`${job.id}.capture.json`);
 const options={cwd:service.root,timeoutMs:120000,onStage:stage=>{job.stage=stage;}};
 job.stage='Revalidating current state';
 await service.runner(service.executable,['capture-conversion-check','--input',resolve(service.directory,`${parent.id}.capture.json`),'--plan',resolve(service.directory,`${job.id}.plan.json`),'--run-id',parent.id,'--check-id',job.id,'--out',capturePath],{...options,fresh:true});
 if(hash(await readFile(service.executable))!==job.engine_sha256)throw new Error('EvidenceVerificationFailed');
 job.capture_sha256=hash(await readFile(capturePath));
 // This separate child receives PATH only: no RPC URL or provider secrets can enter the VM.
 const {code,stdout}=await service.runner(service.executable,['replay-conversion-check','--input',capturePath,'--run-id',parent.id,'--check-id',job.id,'--wallet-sha256',job.parent_capture_sha256,'--capture-sha256',job.capture_sha256,'--plan-sha256',job.plan_sha256,'--program-sha256',mechanism.program_sha256],{...options,fresh:false});
 if(hash(await readFile(service.executable))!==job.engine_sha256)throw new Error('EvidenceVerificationFailed');
 const result=JSON.parse(stdout);
 if(code!==0||result.kind!=='current-conversion'||result.run_id!==parent.id||result.check_id!==job.id||result.wallet_capture_sha256!==parent.capture_sha256||result.execution_capture_sha256!==job.capture_sha256||result.plan_sha256!==job.plan_sha256||result.provenance!=='OperatorSupplied'||result.mint!==parent.selection.mint||result.source!==job.conversion_request.source||result.replacement_mint!==job.conversion_request.replacement_mint||result.candidate_mechanism?.program_sha256!==mechanism.program_sha256||result.candidate_mechanism?.deployed_on_mainnet!==false||result.official_transition!=='NotTested'||result.issuer_binding_established!==false||result.signer_possession_known!==false||result.funds_moved!==false||result.readiness!==null||result.authorization!==false)throw new Error('InvalidEngineResult');
 job.result=result;job.engine_exit_code=code;job.canonical_sha256=hash(stdout);
 await writeFile(resolve(service.directory,`${job.id}.artifact`),stdout,{flag:'wx'});
 await writeFile(resolve(service.directory,`${job.id}.manifest.json`),JSON.stringify({schema_version:1,run_id:parent.id,check_id:job.id,parent_capture_sha256:job.parent_capture_sha256,capture_sha256:job.capture_sha256,plan_sha256:job.plan_sha256,candidate_program_sha256:mechanism.program_sha256,result_sha256:job.canonical_sha256,engine_sha256:job.engine_sha256,request:job.conversion_request}),{flag:'wx'});
}
