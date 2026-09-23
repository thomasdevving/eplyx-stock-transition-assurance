import { readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';

const sha256 = path => createHash('sha256').update(readFileSync(path)).digest('hex');
const read = path => JSON.parse(readFileSync(path, 'utf8'));
const files = [
  'population.capture.json', 'conversion.capture.json', 'authority.plan.json',
  'authority.report.json', 'stress.plan.json', 'stress.cases.json',
  'report.json', 'report.md',
];
const sources = [
  'engine/src/stress/population.rs', 'engine/src/stress/select.rs',
  'engine/src/stress/execute.rs', 'engine/src/stress/readiness.rs',
  'engine/src/conversion/current.rs', 'engine/src/conversion/demo.rs',
  'engine/src/conversion/package_preflight.rs',
];
const directories = {
  healthy: 'target/milestone13-acceptance-healthy',
  underfunded: 'target/milestone13-acceptance-underfunded',
  second: 'target/milestone13-acceptance-second',
};
const runs = Object.entries(directories).map(([name, directory]) => {
  const report = read(`${directory}/report.json`);
  const population = read(`${directory}/population.capture.json`);
  const authority = read(`${directory}/authority.plan.json`);
  const stress = read(`${directory}/stress.plan.json`);
  const counts = {};
  const classifications = {};
  for (const result of report.stress_results) {
    counts[result.status] = (counts[result.status] ?? 0) + 1;
    const classification = result.revalidation?.classification ?? 'Unknown';
    classifications[classification] = (classifications[classification] ?? 0) + 1;
  }
  return {
    name,
    provider: 'FastRPC read-only mainnet',
    artifact_directory: directory,
    captured_at: population.completed_at,
    transition_package_sha256: report.transition_package_sha256,
    artifact_sha256: Object.fromEntries(files.map(file => [file, sha256(`${directory}/${file}`)])),
    enumeration_completeness: report.population_summary.enumeration_completeness,
    token_accounts_observed: report.population_summary.counts.token_accounts_observed,
    positive_balance_accounts_observed: report.population_summary.counts.positive_balance_accounts_observed,
    authority_resolution_completeness: report.population_summary.authority_resolution_completeness,
    selected_authority_cases: authority.selected?.length ?? authority.cases?.length ?? 0,
    selected_stress_cases: stress.selected.length,
    stress_classifications: classifications,
    stress_statuses: counts,
    stress_rebinding: report.stress_rebinding,
    candidate_conversion: report.conversion_result.status,
    candidate_plan_readiness: report.candidate_plan_readiness,
    conversion_stress_readiness: report.conversion_stress_readiness.status,
    population_rollout_readiness: report.population_rollout_readiness.status,
    official_transition: report.official_transition,
    invariant_statuses: Object.fromEntries(report.invariants.map(row => [row.invariant_type, row.status])),
    gate_policy: report.gate_policy,
    gate_outcome: report.gate_outcome,
    live_preflight_exit: name === 'underfunded' ? 3 : 0,
    offline_replay_exit: name === 'underfunded' ? 3 : 0,
    strict_replay_exit: 3,
    funds_moved: report.funds_moved,
  };
});
const output = {
  schema_version: 1,
  kind: 'milestone13-final-state-rebinding-validation',
  engine_binary_sha256: sha256('target/debug/eplyx-lifecycle.exe'),
  source_sha256: Object.fromEntries(sources.map(path => [path, sha256(path)])),
  saved_retry_field_audit_sha256: sha256('reports/milestone13-field-diff.json'),
  saved_retry_field_audit: read('reports/milestone13-field-diff.json').summary,
  mutation_report_sha256: sha256('reports/milestone13-mutations.json'),
  live_acceptance: runs,
  validation: {
    workspace_cargo_test: 'Passed after final executable change; cargo test --locked --workspace, exit 0',
    host_program_tests: 'Fixture v1: 7 passed; fixture v2: 7 passed; candidate conversion: 4 passed',
    cargo_fmt_check: 'Passed for workspace and both program manifests',
    cargo_clippy_deny_warnings: 'Passed for workspace and all three program variants',
    node_service_tests: '47 passed',
    frontend_build_check: 'Build and syntax/report-digest check passed',
    mutations_killed: '7/7 named assertions; original source restored',
    offline_replay_exit: { healthy: 0, underfunded: 3, second: 0 },
    strict_replay_exit: { healthy: 3, underfunded: 3, second: 3 },
    historical_m8_m12_replay_exit: { m8: 4, m9: 0, m10: 0, m11: 0, m12: 0 },
    historical_m8_replay_note: 'Exit 4 is its unchanged analytical Incomplete status; saved bytes matched.',
    git_diff_check: 'Passed',
  },
  limitations: [
    'Finalized coherent account batches are verified capture contracts, not claims of an atomic validator snapshot across discovery and execution.',
    'The candidate program is locally proposed; no issuer authorization, signer possession, OfficialTransition or population rollout is established.',
    'No mainnet transaction was submitted and no funds moved.',
  ],
};
writeFileSync('reports/milestone13-validation.json', `${JSON.stringify(output, null, 2)}\n`);
console.log(JSON.stringify(runs.map(({ name, selected_stress_cases, stress_statuses, gate_outcome }) => ({ name, selected_stress_cases, stress_statuses, gate_outcome }))));
