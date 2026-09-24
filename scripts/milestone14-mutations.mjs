// Eight bounded source mutations. Every edit is restored before the next run.
import {readFile, writeFile} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {resolve} from 'node:path';

const root = resolve(import.meta.dirname, '..');
const file = resolve(root, 'engine/src/conversion/search.rs');
const unit = name => ['test', '--locked', '--offline', '-p', 'eplyx-lifecycle-impact',
  '--lib', `conversion::search::tests::${name}`, '--', '--exact'];
const specs = [
  {id:'derived_as_observed', find:'Self::Derived { .. } => "Derived from observed production state",',
    replace:'Self::Derived { .. } => "Observed production state",',
    test:'provenance_variants_cannot_deserialize_as_each_other', assertion:'derived_must_not_be_labeled_observed'},
  {id:'replace_frozen_case', find:'&& capture.token_account == token_account',
    replace:'&& true',
    test:'frozen_wave_binding_rejects_replacement_and_replay_rejects_tampering', assertion:'failed_case_cannot_be_replaced_in_frozen_wave'},
  {id:'exceed_budget', find:'pub const MAX_BOUNDARY: usize = 20;',
    replace:'pub const MAX_BOUNDARY: usize = 21;',
    test:'budget_is_server_fixed_and_enforced', assertion:'boundary_budget_is_server_fixed'},
  {id:'binary_without_monotonicity', find:'if !monotonic_established {',
    replace:'if false {',
    test:'monotonic_boundary_is_integer_exact_and_otherwise_uses_ordered_probe', assertion:'unproven_monotonicity_forces_ordered_probe'},
  {id:'claim_nonminimal', find:'boundary == minimum\n        || witnesses.iter().any',
    replace:'true\n        || witnesses.iter().any',
    test:'minimum_claim_requires_adjacent_pass_and_same_failure_signature', assertion:'nonminimal_claim_must_fail'},
  {id:'ignore_signature_change', find:'if parent_signature.is_some_and(|original| original != signature) {',
    replace:'if false {',
    test:'minimum_claim_requires_adjacent_pass_and_same_failure_signature', assertion:'changed_failure_signature_must_be_reported'},
  {id:'trust_serialized_search', find:'saved == canonical(recomputed)?.as_bytes()',
    replace:'true',
    test:'frozen_wave_binding_rejects_replacement_and_replay_rejects_tampering', assertion:'serialized_search_outcome_is_not_replay_evidence'},
  {id:'claim_absence', find:'pub const NO_FINDING: &str = "No counterexample found within this search domain and budget.";',
    replace:'pub const NO_FINDING: &str = "No counterexample exists.";',
    test:'provenance_variants_cannot_deserialize_as_each_other', assertion:'bounded_no_finding_wording_is_exact'},
];

const results=[];
for (const spec of specs.filter(s => !process.env.EPLYX_M14_MUTATION || s.id===process.env.EPLYX_M14_MUTATION)) {
  const original=await readFile(file,'utf8');
  if (original.split(spec.find).length!==2) throw Error(`non-unique mutation anchor: ${spec.id}`);
  let output='', status=null;
  try {
    await writeFile(file, original.replace(spec.find,spec.replace));
    const run=spawnSync('cargo',unit(spec.test),{cwd:root,encoding:'utf8',timeout:240000,
      maxBuffer:8*1024*1024,env:{...process.env,RUST_TEST_THREADS:'1'}});
    output=`${run.stdout||''}\n${run.stderr||''}`;
    status=run.status;
  } finally { await writeFile(file,original); }
  const compilerError=output.includes('error: could not compile')||output.includes('error[E');
  const killed=status!==0&&!compilerError&&output.includes(spec.assertion);
  results.push({id:spec.id,killed,exit_code:status,compiler_error:compilerError,
    named_assertion:spec.assertion,...(!killed?{diagnostic:output.slice(-2200)}:{})});
  process.stdout.write(`${spec.id}: ${killed?'KILLED':'SURVIVED'}\n`);
}
await writeFile(resolve(root,'reports/milestone14-mutations.json'),JSON.stringify({
  schema_version:1,kind:'milestone14-counterexample-mutations',results,restored_sources:true,
},null,2)+'\n');
if(results.some(r=>!r.killed)) process.exitCode=1;
