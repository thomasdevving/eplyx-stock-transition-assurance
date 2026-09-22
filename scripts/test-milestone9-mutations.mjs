// Targeted source mutations. Each must compile, fail its named assertion,
// and be restored before final validation. This script never performs RPC.
import {readFileSync, writeFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {resolve, dirname} from 'node:path';
import {fileURLToPath} from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const gate = 'engine/src/conversion/package_gate.rs';
const preflight = 'engine/src/conversion/package_preflight.rs';
const pkg = 'engine/src/conversion/package.rs';
const demo = 'engine/src/conversion/demo.rs';
const mutations = [
  {name: 'incomplete-blocks-under-block-only', file: gate,
    from: 'else if incomplete {\n        Outcome::Warn', to: 'else if incomplete {\n        Outcome::Block',
    test: ['--lib', 'block_only_warns_on_incomplete_and_blocks_explicit_failure'], assertion: 'block_only_warns_on_incomplete_and_blocks_explicit_failure'},
  {name: 'blocked-incorrectly-passes', file: gate,
    from: 'if blocked || (policy == Policy::Strict && incomplete)',
    to: 'if false || (policy == Policy::Strict && incomplete)',
    test: ['--lib', 'block_only_warns_on_incomplete_and_blocks_explicit_failure'], assertion: 'block_only_warns_on_incomplete_and_blocks_explicit_failure'},
  {name: 'gate-mutates-analytical-readiness', file: preflight,
    from: 'let gate = package_gate::evaluate(&report, policy)?;',
    to: 'report["population_rollout_readiness"]["status"] = "Ready".into();\n    let gate = package_gate::evaluate(&report, policy)?;',
    test: ['--lib', 'gate_policy_preserves_analytical_evidence_and_official_boundary'], assertion: 'gate_policy_preserves_analytical_evidence_and_official_boundary'},
  {name: 'old-evidence-reused-after-package-change', file: pkg,
    from: 'let transition_package_sha256 = sha256(identity.as_bytes());',
    to: 'let transition_package_sha256 = "0".repeat(64);',
    test: ['--test', 'transition_package', 'changing_terms_program_or_config_changes_or_invalidates_identity'], assertion: 'changing_terms_program_or_config_changes_or_invalidates_identity'},
  {name: 'validated-package-executes-repository-fallback', file: demo,
    from: 'bytes: program.to_vec(),', to: 'bytes: program_bytes()?,',
    test: ['--test', 'transition_package', 'packaged_candidate_bytes_are_the_only_vm_program'], assertion: 'packaged_candidate_bytes_are_the_only_vm_program'},
  {name: 'different-candidate-digest-accepted-before-vm', file: demo,
    from: ['sha256(expected_bytes) == expected_sha256,', '&& sha256(&candidate[0].bytes) == expected_sha256,'],
    to: ['true,', '&& true,'],
    test: ['--test', 'transition_package', 'candidate_program_digest_mismatch_is_rejected_before_vm'], assertion: 'candidate_program_digest_mismatch_is_rejected_before_vm'},
];
const results = [];
for (const mutation of mutations) {
  const path = resolve(root, mutation.file);
  const original = readFileSync(path, 'utf8');
  let changed = original;
  const from = [].concat(mutation.from), to = [].concat(mutation.to);
  for (let i = 0; i < from.length; i++) {
    if (!changed.includes(from[i])) throw Error(`${mutation.name}: source anchor missing`);
    changed = changed.replace(from[i], to[i]);
  }
  try {
    writeFileSync(path, changed);
    const run = spawnSync('cargo', ['test', '--locked', '-p', 'eplyx-lifecycle-impact', ...mutation.test, '--', '--nocapture'],
      {cwd: root, encoding: 'utf8', timeout: 180000});
    const output = `${run.stdout ?? ''}\n${run.stderr ?? ''}`;
    if (run.status !== 101 || !output.includes(`${mutation.assertion} ... FAILED`) ||
        output.includes('could not compile') || output.includes('error[E')) {
      throw Error(`${mutation.name}: mutation was not killed by ${mutation.assertion}; exit=${run.status}\n${output.slice(-2500)}`);
    }
    results.push({mutation: mutation.name, killed_by: mutation.assertion, compiled: true});
    process.stdout.write(`${mutation.name}: killed by ${mutation.assertion}\n`);
  } finally {
    writeFileSync(path, original);
  }
}
writeFileSync(resolve(root, 'reports/milestone9-mutations.json'), JSON.stringify({schema_version: 1, all_killed: true, mutations: results}, null, 2) + '\n');
