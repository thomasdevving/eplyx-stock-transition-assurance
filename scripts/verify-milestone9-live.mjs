// Bind three freshly produced local reports to offline replay results.
// Usage: node scripts/verify-milestone9-live.mjs <healthy-dir> <underfunded-dir> <second-asset-dir>
import {createHash} from 'node:crypto';
import {readFileSync, writeFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {resolve, relative, sep, dirname} from 'node:path';
import {fileURLToPath} from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const dirs = process.argv.slice(2);
if (dirs.length !== 3) throw Error('expected healthy, underfunded and second-asset result directories');
const exe = resolve(root, 'target/debug', `eplyx-lifecycle${process.platform === 'win32' ? '.exe' : ''}`);
const env = {...process.env};
delete env.SOLANA_RPC_URL;
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const expected = [
  ['demo-fixed-ratio', 'Ready', 'Incomplete', 'Warn', 0],
  ['demo-underfunded', 'Blocked', 'Blocked', 'Block', 3],
  ['demo-second-asset', 'Ready', 'Incomplete', 'Warn', 0],
];
const cases = [];
for (let i = 0; i < 3; i++) {
  const [name, candidate, stress, outcome, exit] = expected[i];
  const directory = resolve(dirs[i]);
  if (!directory.startsWith(root + sep)) throw Error('result directory escapes repository');
  const pkg = `examples/transitions/${name}`;
  const reportBytes = readFileSync(resolve(directory, 'report.json'));
  const report = JSON.parse(reportBytes);
  const replay = spawnSync(exe, ['replay-package-preflight', pkg, '--result', directory],
    {cwd: root, env, encoding: 'utf8', timeout: 180000});
  if (replay.status !== exit || report.candidate_plan_readiness !== candidate ||
      report.conversion_stress_readiness.status !== stress ||
      report.population_rollout_readiness.status !== 'Incomplete' ||
      report.gate_policy !== 'block-only' || report.gate_outcome !== outcome ||
      report.official_transition !== 'NotTested' || report.funds_moved !== false ||
      report.exact_selected_cases.length !== 10) {
    throw Error(`${name}: live report or offline replay mismatch: ${replay.status} ${replay.stderr}`);
  }
  cases.push({package: name, result_directory: relative(root, directory).split(sep).join('/'),
    current_capture_timestamp: report.current_capture_timestamp,
    source_mint: report.source_mint,
    candidate_plan_readiness: candidate,
    conversion_stress_readiness: stress,
    population_rollout_readiness: 'Incomplete',
    gate_policy: 'block-only', gate_outcome: outcome, gate_exit: exit,
    replay_exit: replay.status,
    selected_stress_counts: report.selected_stress_counts,
    transition_package_sha256: report.transition_package_sha256,
    candidate_program_sha256: report.candidate_program_sha256,
    report_sha256: sha(reportBytes)});
}
if (cases[0].source_mint === cases[2].source_mint) throw Error('second asset inherited first mint');
const strict = spawnSync(exe, ['replay-package-preflight', 'examples/transitions/demo-fixed-ratio',
  '--result', resolve(dirs[0]), '--gate', 'strict'], {cwd: root, env, encoding: 'utf8', timeout: 180000});
if (strict.status !== 3 || !strict.stdout.includes('"gate_outcome":"Block"') ||
    !strict.stdout.includes('"candidate_plan_readiness":"Ready"')) throw Error('strict policy did not block same healthy evidence');
const output = {schema_version: 1, kind: 'milestone9-live-validation',
  engine_binary_sha256: sha(readFileSync(exe)),
  cases, healthy_strict_same_evidence_exit: 3,
  rpc_environment_removed_for_replay: true,
  funds_moved: false};
writeFileSync(resolve(root, 'reports/milestone9-live-validation.json'), JSON.stringify(output, null, 2) + '\n');
process.stdout.write(JSON.stringify({cases: cases.length, strict_exit: 3, offline_replay: true}) + '\n');
