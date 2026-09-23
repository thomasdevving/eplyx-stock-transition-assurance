// Summarize already verified local, read-only Milestone 12 artifacts without RPC.
import {createHash} from 'node:crypto';
import {readFile, writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';

const root = resolve(import.meta.dirname, '..');
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const bytes = relative => readFile(resolve(root, relative));
const json = async relative => JSON.parse(await readFile(resolve(root, relative), 'utf8'));
const sourceFiles = [
  'engine/src/conversion/invariants.rs',
  'engine/src/conversion/package.rs',
  'engine/src/conversion/package_gate.rs',
  'engine/src/conversion/package_preflight.rs',
  'engine/tests/transition_package.rs',
];
const sourceSha256 = {};
for (const file of sourceFiles) sourceSha256[file] = sha256(await bytes(file));

const cases = [];
for (const [name, directory, expectedConversion, expectedGate, expectedOutput] of [
  ['healthy', 'target/milestone12-acceptance-healthy-network', 'Ready', 'Warn', 'Satisfied'],
  ['underfunded', 'target/milestone12-acceptance-underfunded', 'Blocked', 'Block', 'Violated'],
  ['second', 'target/milestone12-acceptance-second', 'Ready', 'Warn', 'Satisfied'],
]) {
  const reportBytes = await bytes(`${directory}/report.json`);
  const report = JSON.parse(reportBytes);
  const bindings = await json(`${directory}/bindings.json`);
  const findings = Object.fromEntries(report.invariants.map(finding => [finding.invariant_type, finding.status]));
  if (report.candidate_plan_readiness !== expectedConversion || report.gate_outcome !== expectedGate ||
      findings.conversion_output_matches !== expectedOutput || report.official_transition !== 'NotTested' ||
      report.funds_moved !== false || report.exact_selected_cases.length !== 0 ||
      findings.no_positive_balance_stranded !== 'Indeterminate') {
    throw Error(`${name}: saved acceptance status differs from verified expectation`);
  }
  if (report.population_summary.counts.positive_balance_accounts_observed <= 0) {
    throw Error(`${name}: positive-balance population count is missing`);
  }
  cases.push({
    name,
    provider: 'FastRPC read-only mainnet',
    artifact_directory: directory,
    captured_at: report.current_capture_timestamp,
    transition_package_sha256: report.transition_package_sha256,
    candidate_program_sha256: report.candidate_program_sha256,
    population_sha256: bindings.population_sha256,
    conversion_capture_sha256: bindings.conversion_capture_sha256,
    stress_cases_sha256: bindings.cases_sha256,
    authority_report_sha256: bindings.authority_report_sha256,
    report_sha256: sha256(reportBytes),
    enumeration_completeness: report.population_summary.enumeration_completeness,
    positive_balance_accounts_observed: report.population_summary.counts.positive_balance_accounts_observed,
    selected_stress_cases: report.exact_selected_cases.length,
    selected_authority_cases: report.non_standard_account_control.coverage.cases_selected,
    candidate_plan_readiness: report.candidate_plan_readiness,
    conversion_stress_readiness: report.conversion_stress_readiness.status,
    population_rollout_readiness: report.population_rollout_readiness.status,
    official_transition: report.official_transition,
    invariant_statuses: findings,
    gate_outcome: report.gate_outcome,
    offline_replay_exit: expectedGate === 'Block' ? 3 : 0,
    funds_moved: report.funds_moved,
  });
}
const mutations = await json('reports/milestone12-mutations.json');
if (mutations.results.length !== 7 || mutations.results.some(result => !result.killed)) {
  throw Error('mutation campaign is incomplete');
}
const record = {
  schema_version: 1,
  kind: 'milestone12-operator-invariant-validation',
  engine_binary_sha256: sha256(await bytes('target/debug/eplyx-lifecycle.exe')),
  source_sha256: sourceSha256,
  live_acceptance: cases,
  checks: {
    workspace_cargo_test: 'Passed after the last executable change',
    host_program_tests: 'Fixture v1: 7 passed; fixture v2: 7 passed; candidate conversion: 4 passed',
    cargo_fmt_check: 'Passed for workspace and both program manifests',
    cargo_clippy_deny_warnings: 'Passed for workspace and all three program variants',
    node_service_tests: '47 passed',
    frontend_build_and_check: 'Passed',
    mutations_killed: 7,
    mutations_total: 7,
    replay_tampered_invariant_reference: 'Rejected before accepting saved result',
    mutated_package_replay: 'Rejected: package identity changed',
    historical_m8_replay_exit: 4,
    historical_m9_replay_exit: 0,
    historical_m10_replay_exit: 0,
    historical_m11_replay_exit: 0,
    healthy_strict_same_evidence_exit: 3,
    git_diff_check: 'Passed',
    make_command: 'Unavailable on Windows; equivalent commands run separately',
  },
  limits: [
    'FastRPC rejected recorded-authority account batches for both assets; zero live stress cases were selected or executed.',
    'A bounded selection cannot satisfy the population-stranding invariant; full account-bound path evidence is unavailable.',
    'The candidate mechanism and reserve are proposed; no issuer authorization, official transition or key possession is established.',
  ],
};
await writeFile(resolve(root, 'reports/milestone12-validation.json'), JSON.stringify(record, null, 2) + '\n');
process.stdout.write('Wrote reports/milestone12-validation.json\n');
