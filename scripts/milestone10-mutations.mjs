// Executable source mutations for the authority-resolution trust boundaries.
// Each source file is restored in finally before the next mutation is applied.
import {readFile, writeFile, mkdir} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {resolve} from 'node:path';

const root=resolve(import.meta.dirname,'..');
const authority=resolve(root,'engine/src/stress/authority.rs');
const dlmm=resolve(root,'engine/src/lifecycle/exposure/meteora_dlmm.rs');
const fixture='stress::authority::tests::frozen_plan_and_offline_resolution_preserve_non_wallet_boundaries';
const multisig='stress::authority::tests::exact_spl_multisig_resolution_never_fabricates_approvals';
const peers='stress::authority::tests::resolved_pool_never_proves_peer_accounts';
const refresh='stress::authority::tests::refreshed_population_rejects_prior_resolution_plan';
const specs=[
 {id:'off_curve_as_pda',file:authority,
  find:'ResolutionStatus::Unresolved,\n            PathStatus::Indeterminate,\n            "Account existence, curve status',
  replace:'ResolutionStatus::ResolvedPDA,\n            PathStatus::Indeterminate,\n            "Account existence, curve status',
  test:fixture,assertion:'off-curve alone must not resolve a PDA'},
 {id:'program_authority_wallet_signer',file:authority,
  find:'signer_assumed_locally: false,\n        reason,',
  replace:'signer_assumed_locally: true,\n        reason,',
  test:fixture,assertion:'non-wallet authority cannot inherit a direct signer or execution proof'},
 {id:'multisig_without_approvals',file:authority,
  find:'ResolutionStatus::ResolvedNeedsPrivateAuthorization,\n            PathStatus::Unsupported,',
  replace:'ResolutionStatus::ResolvedNeedsPrivateAuthorization,\n            PathStatus::Proven,',
  test:multisig,assertion:'multisig approvals are not conversion proof'},
 {id:'resolved_authority_proves_peers',file:authority,
  find:'let case = resolve_one(selection, entity, capture, &plan.mint)?;',
  replace:'let case = cases.last().cloned().unwrap_or(resolve_one(selection, entity, capture, &plan.mint)?);',
  test:peers,assertion:'one resolved pool cannot prove its peers'},
 {id:'ignore_controller_program',file:dlmm,
  find:'decode::account_bytes(raw, PROGRAM_ID)',
  replace:'decode::raw_account_bytes(raw)',
  test:fixture,assertion:'wrong controller program must be rejected'},
 {id:'inherit_resolution_after_refresh',file:authority,
  find:'*plan == self::plan(observation)?',replace:'true',
  test:refresh,assertion:'refreshed population cannot inherit authority resolution'},
 {id:'promote_unsupported_to_executable',file:authority,
  find:'execution_supported: false,\n        signer_assumed_locally:',
  replace:'execution_supported: true,\n        signer_assumed_locally:',
  test:fixture,assertion:'non-wallet authority cannot inherit a direct signer or execution proof'},
];
const results=[];
for(const spec of specs.filter(s=>!process.env.EPLYX_M10_MUTATION||s.id===process.env.EPLYX_M10_MUTATION)){
 const original=await readFile(spec.file,'utf8');
 if(original.split(spec.find).length!==2)throw new Error(`Non-unique mutation anchor: ${spec.id}`);
 let output='',status=null;
 try{
  await writeFile(spec.file,original.replace(spec.find,spec.replace));
  const run=spawnSync('cargo',['test','--locked','-p','eplyx-lifecycle-impact','--lib',spec.test,'--','--exact'],
   {cwd:root,encoding:'utf8',timeout:180000,maxBuffer:8*1024*1024,env:{...process.env,RUST_TEST_THREADS:'1'}});
  output=(run.stdout||'')+'\n'+(run.stderr||'');
  status=run.status;
 }finally{
  await writeFile(spec.file,original);
 }
 const compilerError=output.includes('error: could not compile')||output.includes('error[E');
 const killed=status!==0&&!compilerError&&output.includes(spec.test)&&output.includes(spec.assertion);
 results.push({id:spec.id,killed,exit_code:status,compiler_error:compilerError,named_assertion:spec.assertion,
  ...(!killed?{diagnostic:output.slice(-3000)}:{})});
 process.stdout.write(`${spec.id}: ${killed?'KILLED':'SURVIVED'}\n`);
}
const report={schema_version:1,kind:'milestone10-authority-mutations',results,
  inapplicable:['independent-banks-as-sequence: no selected executable sequential control path exists'],
  restored_sources:true};
const out=resolve(root,'reports/milestone10-mutations.json');
await mkdir(resolve(root,'reports'),{recursive:true});
await writeFile(out,JSON.stringify(report,null,2)+'\n');
if(results.some(r=>!r.killed))process.exitCode=1;
