import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile,writeFile,mkdtemp,rm} from 'node:fs/promises';
import {resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {createHash,randomUUID} from 'node:crypto';
import {createServer} from 'node:http';
import {AnalysisService} from '../analysis-service.mjs';
import {validateConversionFields,conversionPlan} from '../conversion-service.mjs';
const root=resolve('.'),hash=b=>createHash('sha256').update(b).digest('hex');
const SOURCE='741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs';
const OPENAI='PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF';
const request=()=>({source:SOURCE,replacement_mint:OPENAI,amount_mode:'Custom',amount_decimal:'0.000001',
 ratio_numerator:1,ratio_denominator:2,rounding:'Floor',conversion_fee_bps:0,reserve_funded_replacement_raw:'1000000000000'});

test('the browser can choose bounded terms only, never code, accounts, paths, endpoints or a status',()=>{
 assert.deepEqual(validateConversionFields(request()),request());
 // Nothing executable or authoritative may cross the API boundary.
 for(const field of ['program','program_id','program_bytes','instruction','instruction_data','accounts','account_metas','transaction','transaction_bytes','path','file','rpc','rpc_url','endpoint','status','provenance','mechanism','adapter_id','source_mint','engine','authorization'])
  assert.throws(()=>validateConversionFields({...request(),[field]:'injected'}),/InvalidConversion/,field);
 for(const patch of [{ratio_numerator:0},{ratio_denominator:0},{ratio_numerator:1.5},{conversion_fee_bps:10001},{conversion_fee_bps:-1},
  {rounding:'Bankers'},{amount_mode:'Everything'},{amount_decimal:'1e9'},{amount_decimal:'-1'},{reserve_funded_replacement_raw:'1.5'},
  {reserve_funded_replacement_raw:''},{replacement_mint:'x'},{source:'x'},{replacement_mint:123}])
  assert.throws(()=>validateConversionFields({...request(),...patch}),/InvalidConversion/,JSON.stringify(patch));
});
test('the server fixes the mechanism, provenance and design of every plan',()=>{
 const plan=conversionPlan(request(),'PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh','plan-1');
 assert.equal(plan.provenance,'OperatorSupplied');
 assert.equal(plan.mechanism,'EplyxDemoCandidateConversion');
 assert.equal(plan.adapter_id,'eplyx-demo-candidate-conversion/v1');
 assert.equal(plan.source_consumption,'Burn');
 assert.equal(plan.replacement_delivery,'ProposedReserveRelease');
 assert.equal(plan.authority_model.candidate_authority,'ProgramDerived');
 assert.equal(plan.source_mint,'PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh');
 assert.equal(plan.terms.ratio_denominator,2);
});

/** A local stub that serves the captured mainnet bytes. No network request leaves the test. */
async function stubRpc(){
 const load=async name=>JSON.parse(await readFile(`reports/milestone4-validation/${name}`,'utf8'));
 const [transfer,market,openai]=await Promise.all([load('live-transfer.capture.json'),load('live-market.capture.json'),load('openai.capture.json')]);
 const accounts=new Map(),slot=448573222;
 const absorb=observation=>{const addresses=observation.params[0],values=observation.result.value;
  addresses.forEach((address,index)=>{if(values[index]&&!accounts.has(address))accounts.set(address,values[index]);});};
 absorb(transfer.observations[3]);absorb(market.observations[6]);
 accounts.set(OPENAI,openai.observations[1].result.value);
 // The captured Clock must agree with the slot this stub reports.
 const clockKey='SysvarC1ock11111111111111111111111111111111',clock={...accounts.get(clockKey)};
 const bytes=Buffer.from(clock.data[0],'base64');bytes.writeBigUInt64LE(BigInt(slot),0);
 clock.data=[bytes.toString('base64'),'base64'];accounts.set(clockKey,clock);
 const genesis=JSON.parse(transfer.wallet_capture).observations[0].result;
 const server=createServer((req,res)=>{let body='';req.on('data',c=>{body+=c;});req.on('end',()=>{
  const {method,params}=JSON.parse(body);let result=null;
  if(method==='getGenesisHash')result=genesis;
  else if(method==='getAccountInfo')result={context:{slot},value:accounts.get(params[0])??null};
  else if(method==='getMultipleAccounts')result={context:{slot},value:params[0].map(a=>accounts.get(a)??null)};
  // Node's JSON numbers cannot hold a u64 rent epoch; restore the captured value
  // exactly. The real path never round-trips provider bytes through JavaScript.
  const payload=JSON.stringify({jsonrpc:'2.0',id:1,result}).replaceAll('"rentEpoch":18446744073709552000','"rentEpoch":18446744073709551615');
  res.writeHead(200,{'Content-Type':'application/json'});res.end(payload);
 });});
 await new Promise(r=>server.listen(0,'127.0.0.1',r));
 return {server,url:`http://127.0.0.1:${server.address().port}`,transfer};
}
async function setup(transfer){
 const directory=await mkdtemp(resolve(tmpdir(),'eplyx-conversion-'));
 const service=new AnalysisService({root,directory});await service.initialize();
 const capture=transfer.wallet_capture,id=randomUUID(),c=JSON.parse(capture);
 const parent={id,owner:'session-a',status:'Completed',capture_sha256:hash(capture),
  selection:{asset:'mint',review:'current',stage:null,check:null,cluster:'solana-mainnet',mint:c.asset.mint,
   catalogue_version:null,sample_accounts:false,scope:'wallet',public_owner:c.selection.public_owner}};
 service.jobs.set(id,parent);await writeFile(resolve(directory,`${id}.capture.json`),capture);
 return {directory,service,parent};
}
const finish=async service=>{for(let i=0;i<600&&(service.running||service.queue.length);i++)await new Promise(r=>setTimeout(r,50));assert.equal(service.running,false);};

test('a candidate conversion plan really executes, reconciles and stays separate from an official transition',async()=>{
 const {server,url,transfer}=await stubRpc();
 const previous=process.env.SOLANA_RPC_URL;process.env.SOLANA_RPC_URL=url;
 const {directory,service,parent}=await setup(transfer);
 try{
  const job=await service.submitConversion(parent.id,request(),randomUUID(),'session-a');
  await finish(service);
  assert.equal(job.status,'Completed',JSON.stringify(job.error));
  const r=job.result;
  assert.equal(r.status,'Proven',r.reason);
  assert.equal(r.provenance,'OperatorSupplied');
  assert.equal(r.official_transition,'NotTested');
  assert.equal(r.issuer_binding_established,false);
  assert.equal(r.candidate_mechanism.deployed_on_mainnet,false);
  assert.equal(r.funds_moved,false);
  assert.equal(r.reconciliation.reconciled,true);
  assert.equal(r.reconciliation.source_debited_raw,'1000');
  assert.equal(r.reconciliation.replacement_released_raw,'500');
  assert.equal(r.readiness,null);
  assert.equal(r.authorization,false);

  // The pre-flight consumes it without ever promoting the official path.
  const preflight={source:SOURCE,successor_mint:OPENAI,effective_at:'2035-01-01T00:00:00Z',deadline:null,
   post_deadline:null,assurance:'FullTransition',check_ids:[],conversion_check_id:job.id};
  const evaluated=await service.submitPreflight(parent.id,preflight,randomUUID(),'session-a');
  await finish(service);
  assert.equal(evaluated.status,'Completed',JSON.stringify(evaluated.error));
  const result=evaluated.result;
  assert.equal(result.replacement_conversion.status,'Proven');
  assert.equal(result.replacement_conversion.official_transition,'NotTested');
  const paths=Object.fromEntries(result.paths.map(p=>[p.path_type,p.status]));
  assert.equal(paths.OfficialTransition,'NotTested','a candidate plan never enters the official path');
  const active=result.views.find(v=>v.id==='ProposedActive');
  assert.equal(active.readiness.candidate_plan.status,'Ready');
  assert.equal(active.readiness.candidate_plan.scope,'CandidateConversionPlanReadiness');
  assert.equal(active.readiness.candidate_plan.official_transition_established,false);
  assert.equal(active.readiness.full_transition.status,'Incomplete','candidate readiness cannot become full transition readiness');
  assert.equal(active.readiness.mobility.status,'Incomplete','no mobility check was selected');
  assert.equal(result.population_readiness,null,'entity readiness never becomes population readiness');

  // A refreshed wallet run cannot inherit this candidate proof.
  const refreshed=await setup(transfer);
  try{
   refreshed.service.jobs.set(job.id,job);
   await assert.rejects(()=>refreshed.service.submitPreflight(refreshed.parent.id,preflight,randomUUID(),'session-a'),/InvalidCheckSelection/);
  }finally{await rm(refreshed.directory,{recursive:true,force:true});}
 }finally{
  server.close();if(previous===undefined)delete process.env.SOLANA_RPC_URL;else process.env.SOLANA_RPC_URL=previous;
  await rm(directory,{recursive:true,force:true});
 }
});
test('a candidate plan is rejected across sessions, parents and identities before any acquisition',async()=>{
 const {server,transfer}=await stubRpc();
 const {directory,service,parent}=await setup(transfer);
 try{
  await assert.rejects(()=>service.submitConversion(parent.id,request(),randomUUID(),'other-session'),/InvalidParent/);
  await assert.rejects(()=>service.submitConversion(parent.id,{...request(),replacement_mint:parent.selection.mint},randomUUID(),'session-a'),/InvalidConversion/);
  await assert.rejects(()=>service.submitConversion(parent.id,{...request(),source:'123aUGPWa93jiga876U3rLdBP86JNFSoz9tSQWCAskMc'},randomUUID(),'session-a'),/InvalidConversion/);
  await assert.rejects(()=>service.submitConversion(parent.id,{...request(),amount_decimal:'99999999'},randomUUID(),'session-a'),/InvalidConversion/);
  assert.equal(service.queue.length,0,'nothing was queued and no acquisition ran');
 }finally{server.close();await rm(directory,{recursive:true,force:true});}
});
