// Seven executable invariant mutations; each source is restored in finally.
import {readFile, writeFile} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {resolve} from 'node:path';

const root = resolve(import.meta.dirname, '..');
const invariantFile = resolve(root, 'engine/src/conversion/invariants.rs');
const packageFile = resolve(root, 'engine/src/conversion/package.rs');
const gateFile = resolve(root, 'engine/src/conversion/package_gate.rs');
const preflightFile = resolve(root, 'engine/src/conversion/package_preflight.rs');
const unit = name => ['test', '--locked', '-p', 'eplyx-lifecycle-impact', '--lib', name, '--', '--exact'];
const integration = name => ['test', '--locked', '-p', 'eplyx-lifecycle-impact', '--test', 'transition_package', name, '--', '--exact'];
const specs = [
  {id: 'indeterminate_as_satisfied', file: invariantFile,
    find: 'outcome(Status::Indeterminate, "Exact reconciled candidate execution is unavailable.")',
    replace: 'outcome(Status::Satisfied, "Exact reconciled candidate execution is unavailable.")',
    args: unit('conversion::invariants::tests::conversion_output_uses_reconciled_vm_evidence_only'),
    assertion: 'conversion_output_uses_reconciled_vm_evidence_only'},
  {id: 'sample_as_population', file: invariantFile,
    find: 'outcome(Status::Indeterminate, "No exhaustive account-bound transition or mobility path evidence exists for the observed positive-balance population.")',
    replace: 'outcome(Status::Satisfied, "No exhaustive account-bound transition or mobility path evidence exists for the observed positive-balance population.")',
    args: unit('conversion::invariants::tests::bounded_sample_never_satisfies_population_requirement'),
    assertion: 'bounded_sample_never_satisfies_population_requirement'},
  {id: 'replacement_as_official', file: invariantFile,
    find: 'evidence.conversion["official_transition"].as_str()',
    replace: 'evidence.conversion["status"].as_str()',
    args: unit('conversion::invariants::tests::replacement_proof_cannot_satisfy_official_path'),
    assertion: 'replacement_proof_cannot_satisfy_official_path'},
  {id: 'ignore_invariant_identity', file: packageFile,
    find: 'let canonical = crate::expansion::canonical(&manifest)?;',
    replace: 'let canonical = crate::expansion::canonical(&Manifest { invariants: None, ..manifest.clone() })?;',
    args: integration('invariant_identity_binds_type_severity_config_and_is_order_independent'),
    assertion: 'invariant_identity_binds_type_severity_config_and_is_order_independent'},
  {id: 'ignore_blocking_violation', file: gateFile,
    find: '(Severity::Blocking, Status::Violated) => blocked = true,',
    replace: '(Severity::Blocking, Status::Violated) => invariant_warning = true,',
    args: unit('conversion::package_gate::tests::invariant_severity_and_certainty_affect_gate_without_mutating_evidence'),
    assertion: 'invariant_severity_and_certainty_affect_gate_without_mutating_evidence'},
  {id: 'trust_serialized_invariants', file: preflightFile,
    find: 'let report = evaluate(&package, output_directory, &b)?;\n    ensure!(\n        crate::expansion::canonical(&report)?.as_bytes()',
    replace: 'let report: Value = serde_json::from_slice(&read(output_directory, "report.json", 128 * 1024 * 1024)?)?;\n    ensure!(\n        crate::expansion::canonical(&report)?.as_bytes()',
    args: ['build', '--locked', '-p', 'eplyx-lifecycle-impact'],
    assertion: 'serialized_invariant_evidence_ref_rejected', replayProbe: true},
  {id: 'ignore_existing_reconciliation', file: invariantFile,
    find: 'let reconciled = conversion["reconciliation"]["reconciled"].as_bool();',
    replace: 'let reconciled = Some(true);',
    args: unit('conversion::invariants::tests::conversion_output_uses_reconciled_vm_evidence_only'),
    assertion: 'conversion_output_uses_reconciled_vm_evidence_only'},
];

const results = [];
for (const spec of specs.filter(s => !process.env.EPLYX_M12_MUTATION || s.id === process.env.EPLYX_M12_MUTATION)) {
  const original = await readFile(spec.file, 'utf8');
  if (original.split(spec.find).length !== 2) throw Error(`non-unique mutation anchor: ${spec.id}`);
  let output = '';
  let status = null;
  try {
    await writeFile(spec.file, original.replace(spec.find, spec.replace));
    const run = spawnSync('cargo', spec.args, {cwd: root, encoding: 'utf8', timeout: 240000,
      maxBuffer: 8 * 1024 * 1024, env: {...process.env, RUST_TEST_THREADS: '1'}});
    output = `${run.stdout || ''}\n${run.stderr || ''}`;
    status = run.status;
    if (spec.replayProbe && status === 0) {
      const probe = spawnSync('node', ['scripts/test-milestone12-replay-tamper.mjs'], {
        cwd: root, encoding: 'utf8', timeout: 240000, maxBuffer: 4 * 1024 * 1024,
      });
      output += `\n${probe.stdout || ''}\n${probe.stderr || ''}`;
      status = probe.status;
    }
  } finally {
    await writeFile(spec.file, original);
  }
  const compilerError = output.includes('error: could not compile') || output.includes('error[E');
  const killed = status !== 0 && !compilerError && output.includes(spec.assertion);
  results.push({id: spec.id, killed, exit_code: status, compiler_error: compilerError,
    named_assertion: spec.assertion, ...(!killed ? {diagnostic: output.slice(-2200)} : {})});
  process.stdout.write(`${spec.id}: ${killed ? 'KILLED' : 'SURVIVED'}\n`);
}
await writeFile(resolve(root, 'reports/milestone12-mutations.json'), JSON.stringify({
  schema_version: 1, kind: 'milestone12-invariant-mutations', results, restored_sources: true,
}, null, 2) + '\n');
if (results.some(result => !result.killed)) process.exitCode = 1;
