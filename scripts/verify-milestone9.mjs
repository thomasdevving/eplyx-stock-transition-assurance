// Offline acceptance over retained real production captures. No RPC is exposed.
import {createHash} from 'node:crypto';
import {mkdtempSync, readFileSync, readdirSync, linkSync, writeFileSync, cpSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {resolve, join, dirname, relative, sep} from 'node:path';
import {fileURLToPath} from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const target = resolve(root, 'target');
const exe = resolve(target, 'debug', `eplyx-lifecycle${process.platform === 'win32' ? '.exe' : ''}`);
const env = {...process.env};
delete env.SOLANA_RPC_URL;
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const fileSha = path => sha(readFileSync(resolve(root, path)));
function run(args) {
  return spawnSync(exe, args, {cwd: root, env, encoding: 'utf8', timeout: 180000});
}
function expectRun(args, exit) {
  const result = run(args);
  if (result.status !== exit) throw Error(`${args.join(' ')}: exit ${result.status}; ${result.stderr}`);
  return result;
}
const cases = [
  {name: 'healthy-block-only', pkg: 'demo-fixed-ratio', source: 'milestone8-healthy-worker', policy: 'block-only', exit: 0, outcome: 'Warn', candidate: 'Ready', stress: 'Incomplete'},
  {name: 'healthy-strict', pkg: 'demo-fixed-ratio', source: 'milestone8-healthy-worker', policy: 'strict', exit: 3, outcome: 'Block', candidate: 'Ready', stress: 'Incomplete'},
  {name: 'underfunded-block-only', pkg: 'demo-underfunded', source: 'milestone8-underfunded', policy: 'block-only', exit: 3, outcome: 'Block', candidate: 'Blocked', stress: 'Blocked'},
  {name: 'underfunded-strict', pkg: 'demo-underfunded', source: 'milestone8-underfunded', policy: 'strict', exit: 3, outcome: 'Block', candidate: 'Blocked', stress: 'Blocked'},
  {name: 'second-asset-block-only', pkg: 'demo-second-asset', source: 'milestone8-second-asset', policy: 'block-only', exit: 0, outcome: 'Warn', candidate: 'Ready', stress: 'Incomplete'},
];
const output = [];
const caseDirs = new Map();
for (const c of cases) {
  const pkg = `examples/transitions/${c.pkg}`;
  const source = resolve(root, 'reports', c.source);
  const out = mkdtempSync(join(target, `milestone9-${c.name}-`));
  if (!out.startsWith(target + sep)) throw Error('result path escaped target');
  caseDirs.set(c.name, out);
  for (const name of readdirSync(source)) {
    if (name === 'report.json' || name === 'report.md') continue;
    if (name === 'bindings.json') {
      const bindings = JSON.parse(readFileSync(join(source, name), 'utf8'));
      bindings.gate_policy = c.policy;
      writeFileSync(join(out, name), JSON.stringify(bindings));
    } else linkSync(join(source, name), join(out, name));
  }
  expectRun(['finish-package-preflight', pkg, '--result', out], 0);
  const replay = expectRun(['replay-package-preflight', pkg, '--result', out], c.exit);
  const report = JSON.parse(readFileSync(join(out, 'report.json'), 'utf8'));
  const old = JSON.parse(readFileSync(join(source, 'report.json'), 'utf8'));
  for (const field of ['transition_package_sha256', 'candidate_program_sha256', 'candidate_plan_readiness', 'conversion_stress_readiness', 'population_rollout_readiness', 'official_transition', 'exact_selected_cases', 'stress_results', 'funds_moved']) {
    if (JSON.stringify(report[field]) !== JSON.stringify(old[field])) throw Error(`${c.name}: analytical ${field} changed`);
  }
  if (report.gate_policy !== c.policy || report.gate_outcome !== c.outcome ||
      report.candidate_plan_readiness !== c.candidate || report.conversion_stress_readiness.status !== c.stress ||
      report.official_transition !== 'NotTested' || report.population_rollout_readiness.status !== 'Incomplete' ||
      report.funds_moved !== false || !replay.stdout.includes(`"gate_outcome":"${c.outcome}"`)) {
    throw Error(`${c.name}: policy or evidence mismatch`);
  }
  output.push({case: c.name, policy: c.policy, outcome: c.outcome, exit: c.exit,
    result_directory: relative(root, out).split(sep).join('/'),
    transition_package_sha256: report.transition_package_sha256,
    candidate_program_sha256: report.candidate_program_sha256,
    source_mint: report.source_mint,
    selected_cases: report.exact_selected_cases.length,
    selected_stress_counts: report.selected_stress_counts,
    report_sha256: sha(readFileSync(join(out, 'report.json'))),
    capture_timestamp: report.current_capture_timestamp});
}
if (output[0].source_mint === output[4].source_mint) throw Error('second asset inherited first mint');

// A new program digest and package digest cannot be applied to old execution evidence.
const originalPackage = resolve(root, 'examples/transitions/demo-fixed-ratio');
const mutated = mkdtempSync(join(target, 'milestone9-mutated-package-'));
if (!mutated.startsWith(target + sep)) throw Error('mutated package path escaped target');
cpSync(originalPackage, mutated, {recursive: true});
const bytes = readFileSync(join(mutated, 'program.so'));
bytes[64] ^= 1;
writeFileSync(join(mutated, 'program.so'), bytes);
const manifest = JSON.parse(readFileSync(join(mutated, 'eplyx.json'), 'utf8'));
manifest.candidateProgram.sha256 = sha(bytes);
writeFileSync(join(mutated, 'eplyx.json'), JSON.stringify(manifest));
const validated = expectRun(['validate-transition-package', mutated], 0);
const changed = JSON.parse(validated.stdout);
if (changed.candidate_program_sha256 === output[0].candidate_program_sha256 ||
    changed.transition_package_sha256 === output[0].transition_package_sha256) throw Error('binary mutation retained old identity');
const oldEvidence = expectRun(['replay-package-preflight', mutated, '--result', caseDirs.get('healthy-block-only')], 2);
if (!oldEvidence.stderr.includes('package identity changed')) throw Error('old evidence was not rejected by package binding');

const tamperedResult = mkdtempSync(join(target, 'milestone9-tampered-gate-'));
if (!tamperedResult.startsWith(target + sep)) throw Error('tampered report path escaped target');
const baseline = caseDirs.get('healthy-block-only');
for (const name of readdirSync(baseline)) {
  if (name === 'report.json') {
    const edited = JSON.parse(readFileSync(join(baseline, name), 'utf8'));
    edited.gate_outcome = 'Pass';
    writeFileSync(join(tamperedResult, name), JSON.stringify(edited));
  } else linkSync(join(baseline, name), join(tamperedResult, name));
}
const changedGate = expectRun(['replay-package-preflight', 'examples/transitions/demo-fixed-ratio', '--result', tamperedResult], 2);
if (!changedGate.stderr.includes('saved report differs from offline replay')) throw Error('tampered gate outcome was accepted');

const mutations = JSON.parse(readFileSync(resolve(root, 'reports/milestone9-mutations.json'), 'utf8'));
if (!mutations.all_killed || mutations.mutations.length !== 6 || mutations.mutations.some(m => !m.compiled)) throw Error('mutation campaign incomplete');
const historical = spawnSync('git', ['diff', '--name-only', 'HEAD', '--', 'evidence', 'probes', 'snapshots', 'assets', 'fixtures', 'policies', 'scenarios', 'reports'], {cwd: root, encoding: 'utf8'});
if (historical.status !== 0 || historical.stdout.trim()) throw Error('historical tracked artifacts changed');
const report = {schema_version: 1, kind: 'milestone9-ci-gate-validation',
  engine_binary_sha256: fileSha(`target/debug/eplyx-lifecycle${process.platform === 'win32' ? '.exe' : ''}`),
  source_sha256: Object.fromEntries(['engine/src/conversion/package.rs', 'engine/src/conversion/package_gate.rs', 'engine/src/conversion/package_preflight.rs', 'engine/src/conversion/demo.rs', 'engine/src/conversion/current.rs', 'engine/src/stress/execute.rs', 'engine/src/main.rs'].map(p => [p, fileSha(p)])),
  cases: output, program_mutation: {candidate_program_sha256: changed.candidate_program_sha256, transition_package_sha256: changed.transition_package_sha256, old_evidence_exit: 2},
  gate_outcome_tamper_rejected: true,
  mutation_campaign_sha256: fileSha('reports/milestone9-mutations.json'), mutations_killed: 6,
  rpc_environment_removed_for_replay: true, historical_tracked_artifacts_unchanged: true};
writeFileSync(resolve(root, 'reports/milestone9-validation.json'), JSON.stringify(report, null, 2) + '\n');
process.stdout.write(JSON.stringify({cases: output.length, mutations_killed: 6, historical_tracked_artifacts_unchanged: true}) + '\n');
