// Seven executable rebinding faults. Each edited source is restored in finally.
import {readFile, writeFile} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {resolve} from 'node:path';

const root = resolve(import.meta.dirname, '..');
const source = resolve(root, 'engine/src/stress/execute.rs');
const test = name => ['test', '--locked', '-p', 'eplyx-lifecycle-impact', '--test',
  'conversion_stress', name, '--', '--exact'];
const specs = [
  {id: 'peer_replacement',
    find: '&& capture.token_account == case.token_account',
    replace: '&& true',
    name: 'peer_capture_cannot_replace_a_frozen_selected_identity',
    assertion: 'peer_replacement_rejected'},
  {id: 'execute_stale_population_bytes',
    find: 'let execution = executor::execute_probe_message(\n        &p.accounts,',
    replace: `let mut stale_accounts = p.accounts.clone();
    if rebinding {
        let population = original_capture.context("missing population")?;
        let stale_raw = population.observations[entity.token_account_evidence.rpc_id]
            .result.as_ref().context("missing response")?
            .pointer(&entity.token_account_evidence.pointer).context("missing source")?;
        let source = stale_accounts.iter_mut().find(|account| account.address == case.token_account)
            .context("selected source missing")?;
        source.account.data = crate::lifecycle::decode::raw_account_bytes(stale_raw)?;
    }
    let execution = executor::execute_probe_message(
        &stale_accounts,`,
    name: 'full_at_final_capture_resolves_drifted_amount_before_vm_without_replacing_identity',
    assertion: 'final_current_amount_must_execute'},
  {id: 'reuse_discovery_amount',
    find: 'amount = final_amount;',
    replace: 'amount = case.selected_amount_raw.parse()?;',
    name: 'full_at_final_capture_resolves_drifted_amount_before_vm_without_replacing_identity',
    assertion: 'final_current_amount_must_execute'},
  {id: 'vm_outcome_rewrites_execution_plan',
    find: 'detail["execution"] = serde_json::to_value(&execution)?;',
    replace: 'detail["execution"] = serde_json::to_value(&execution)?;\n    if rebinding { detail["execution_plan"]["vm_result"] = serde_json::to_value(&execution)?; }',
    name: 'full_at_final_capture_resolves_drifted_amount_before_vm_without_replacing_identity',
    assertion: 'vm_result_cannot_rewrite_frozen_execution_plan'},
  {id: 'hide_bucket_drift',
    find: 'let bucket_preserved = final_bucket == Some(case.balance_bucket);',
    replace: 'let bucket_preserved = true;',
    name: 'final_bucket_drift_is_disclosed_without_rewriting_discovery_selection',
    assertion: 'bucket_drift_cannot_silently_satisfy_original_coverage'},
  {id: 'unsupported_authority_as_executable',
    find: 'final_state.owner != case.authority || final_amount == 0 || !eligible',
    replace: 'final_amount == 0 || !eligible',
    name: 'zero_frozen_authority_and_token_program_drift_never_execute_a_peer',
    assertion: 'unsupported_authority_drift_never_executes'},
  {id: 'claim_stale_discovery_proof',
    find: 'Only the frozen selected account identity and its verified final coherent state, exact final positive amount',
    replace: 'Only the frozen selected account identity and its stale population state, exact final positive amount',
    name: 'full_at_final_capture_resolves_drifted_amount_before_vm_without_replacing_identity',
    assertion: 'proof_scope_binds_final_not_stale_discovery_state'},
];

const results = [];
for (const spec of specs.filter(s => !process.env.EPLYX_M13_MUTATION || s.id === process.env.EPLYX_M13_MUTATION)) {
  const original = await readFile(source, 'utf8');
  if (original.split(spec.find).length !== 2) throw Error(`non-unique mutation anchor: ${spec.id}`);
  let output = '', status = null;
  try {
    await writeFile(source, original.replace(spec.find, spec.replace));
    const run = spawnSync('cargo', test(spec.name), {cwd: root, encoding: 'utf8', timeout: 240000,
      maxBuffer: 8 * 1024 * 1024, env: {...process.env, RUST_TEST_THREADS: '1'}});
    output = `${run.stdout || ''}\n${run.stderr || ''}`;
    status = run.status;
  } finally {
    await writeFile(source, original);
  }
  const compilerError = output.includes('error: could not compile') || output.includes('error[E');
  const killed = status !== 0 && !compilerError && output.includes(spec.assertion);
  results.push({id: spec.id, killed, exit_code: status, compiler_error: compilerError,
    named_assertion: spec.assertion, ...(!killed ? {diagnostic: output.slice(-2200)} : {})});
  process.stdout.write(`${spec.id}: ${killed ? 'KILLED' : 'SURVIVED'}\n`);
}
await writeFile(resolve(root, 'reports/milestone13-mutations.json'), JSON.stringify({
  schema_version: 1, kind: 'milestone13-rebinding-mutations', results, restored_sources: true,
}, null, 2) + '\n');
if (results.some(result => !result.killed)) process.exitCode = 1;
