// Offline assertion over the local Milestone 12 acceptance capture.
import {cp, mkdtemp, readFile, rm, writeFile} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {join, resolve} from 'node:path';

const root = resolve(import.meta.dirname, '..');
const source = join(root, 'target/milestone12-acceptance-healthy-network');
const packageDirectory = join(root, 'target/milestone12-package-healthy');
const executable = join(root, 'target/debug/eplyx-lifecycle.exe');
const temporary = await mkdtemp(join(root, 'target/milestone12-replay-tamper-'));
try {
  const result = join(temporary, 'result');
  await cp(source, result, {recursive: true});
  const reportPath = join(result, 'report.json');
  const original = await readFile(reportPath, 'utf8');
  const anchor = /("evidence_refs": \[\r?\n\s*")/;
  if (!anchor.test(original)) throw Error('missing invariant evidence-ref array');
  await writeFile(reportPath, original.replace(anchor, '$1tampered-'));
  const replay = spawnSync(executable, ['replay-package-preflight', packageDirectory, '--result', result], {
    cwd: root, encoding: 'utf8', timeout: 180000, maxBuffer: 4 * 1024 * 1024,
    env: {...process.env, SOLANA_RPC_URL: ''},
  });
  if (replay.status === 0) {
    console.error('serialized_invariant_evidence_ref_rejected: replay accepted a forged evidence reference');
    process.exitCode = 1;
  } else if (replay.status !== 2 || !(replay.stderr || '').includes('saved report differs from offline replay')) {
    console.error('serialized_invariant_evidence_ref_rejected: replay failed for an unexpected reason');
    console.error((replay.stderr || '').slice(-1500));
    process.exitCode = 2;
  } else {
    console.log('serialized_invariant_evidence_ref_rejected');
  }
} finally {
  await rm(temporary, {recursive: true, force: true});
}
