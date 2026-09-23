// Executable coherence-boundary mutations. Each source is restored in finally.
import {readFile,writeFile} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {resolve} from 'node:path';

const root=resolve(import.meta.dirname,'..');
const coherence=resolve(root,'engine/src/conversion/coherence.rs');
const demo=resolve(root,'engine/src/conversion/demo.rs');
const current=resolve(root,'engine/src/conversion/current.rs');
const stress=resolve(root,'engine/src/stress/execute.rs');
const unit=(name)=>['test','--locked','-p','eplyx-lifecycle-impact','--lib',`conversion::coherence::tests::${name}`,'--','--exact'];
const integration=(suite,name)=>['test','--locked','-p','eplyx-lifecycle-impact','--test',suite,name,'--','--exact'];
const specs=[
 {id:'accept_later_clock',file:coherence,find:'let coherent = clock == slot;',replace:'let coherent = true;',
  args:unit('later_clock_and_earlier_account_context_never_grant_proof'),assertion:'later_clock_earlier_bank_rejected'},
 {id:'ignore_recapture_source_drift',file:coherence,
  find:'account_index != clock_index && previous[account_index] != values[account_index]',replace:'false',
  args:unit('changed_source_or_program_during_recapture_is_indeterminate'),assertion:'recapture_source_drift_rejected'},
 {id:'reuse_first_batch_instead_of_final',file:demo,
  find:'let values = evidence[final_record].result["value"]',replace:'let values = evidence[4].result["value"]',
  args:integration('candidate_conversion','coherent_retry_replays_the_exact_final_bank_and_not_discovery_bytes'),assertion:'final_recapture_bytes_used'},
 {id:'replace_frozen_selected_case',file:stress,
  find:'entity_id: case.entity_id.clone(),\n            token_account: case.token_account.clone(),\n            authority: case.authority.clone(),',
  replace:'entity_id: case.entity_id.clone(),\n            token_account: plan.selected[1].token_account.clone(),\n            authority: case.authority.clone(),',
  args:integration('conversion_stress','changed_selected_source_stays_selected_and_indeterminate'),assertion:'frozen_selected_case_never_replaced'},
 {id:'ignore_candidate_config_fee',file:demo,
  find:'data[146..148].copy_from_slice(&plan.terms.conversion_fee_bps.to_le_bytes());',
  replace:'data[146..148].copy_from_slice(&0u16.to_le_bytes());',
  args:integration('candidate_conversion','proposed_candidate_config_keeps_the_exact_operator_fee'),assertion:'candidate_config_fee_bound'},
 {id:'stabilization_failure_as_failed',file:current,
  find:'if text.contains("Unsupported: ") {\n                    PathStatus::Unsupported\n                } else {\n                    PathStatus::Indeterminate\n                },',
  replace:'if text.contains("Unsupported: ") {\n                    PathStatus::Unsupported\n                } else {\n                    PathStatus::Failed\n                },',
  args:integration('candidate_conversion','exhausted_coherence_capture_is_indeterminate_and_never_failed_execution'),assertion:'stabilization_never_becomes_failed_or_proven'},
 {id:'trust_serialized_coherence_flag',file:coherence,
  find:'let coherent = clock == slot;',replace:'let coherent = record.result["coherence_status"] == "Verified";',
  args:unit('serialized_success_or_rewritten_min_chain_cannot_bypass_replay'),assertion:'serialized_coherence_flag_rejected'},
];
const results=[];
for(const spec of specs.filter(s=>!process.env.EPLYX_M11_MUTATION||s.id===process.env.EPLYX_M11_MUTATION)){
 const original=await readFile(spec.file,'utf8');
 if(original.split(spec.find).length!==2)throw Error(`non-unique mutation anchor: ${spec.id}`);
 let output='',status=null;
 try{
  await writeFile(spec.file,original.replace(spec.find,spec.replace));
  const run=spawnSync('cargo',spec.args,{cwd:root,encoding:'utf8',timeout:180000,maxBuffer:8*1024*1024,
   env:{...process.env,RUST_TEST_THREADS:'1'}});
  output=(run.stdout||'')+'\n'+(run.stderr||'');status=run.status;
 }finally{await writeFile(spec.file,original);}
 const compilerError=output.includes('error: could not compile')||output.includes('error[E');
 const killed=status!==0&&!compilerError&&output.includes(spec.assertion);
 results.push({id:spec.id,killed,exit_code:status,compiler_error:compilerError,named_assertion:spec.assertion,
  ...(!killed?{diagnostic:output.slice(-2500)}:{})});
 process.stdout.write(`${spec.id}: ${killed?'KILLED':'SURVIVED'}\n`);
}
await writeFile(resolve(root,'reports/milestone11-mutations.json'),JSON.stringify({schema_version:1,
 kind:'milestone11-coherence-mutations',results,restored_sources:true},null,2)+'\n');
if(results.some(result=>!result.killed))process.exitCode=1;
