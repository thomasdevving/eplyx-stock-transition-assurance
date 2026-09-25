// Stage the build context for the hosted Stock Transition frontend and
// analysis service (frontend/Dockerfile):
//   node scripts/railway/stage-frontend.mjs <empty-output-dir>
//   npx @railway/cli up <output-dir> --service eplyx-stock --detach
// Only tracked (or new, not ignored) sources and evidence data are copied, plus the three built
// .so programs the analysis engine loads. Keypairs, local run stores,
// analysis runs, build output, Milestone validation records and anything
// ignored by Git never enter the context.
import { execFileSync } from 'node:child_process';
import { cp, mkdir, readdir, stat, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(fileURLToPath(new URL('../../', import.meta.url)));
const out = process.argv[2] && resolve(process.argv[2]);
if (!out) { console.error('usage: node scripts/railway/stage-frontend.mjs <empty-output-dir>'); process.exit(2); }
await mkdir(out, { recursive: true });
if ((await readdir(out)).length) { console.error(`${out} must be empty`); process.exit(2); }

const include = ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'package.json', 'package-lock.json',
 'interface', 'engine', 'cloud', 'frontend', 'docs', 'reports', 'snapshots', 'evidence', 'probes',
 'scenarios', 'policies', 'fixtures', 'examples'];
const tracked = execFileSync('git', ['-C', root, 'ls-files', '-z', '--cached', '--others', '--exclude-standard', '--', ...include], { encoding: 'utf8' })
 .split('\0').filter(Boolean)
 .filter(file => !file.startsWith('reports/milestone'));
const programs = ['artifacts/eplyx_demo_conversion.so', 'artifacts/fixture_lending_v1.so', 'artifacts/fixture_lending_v2.so'];
let bytes = 0;
for (const file of [...tracked, ...programs]) {
 if (/keypair[^/]*\.json$|(^|\/)\.env|(^|\/)credentials\.json$/i.test(file)) throw new Error(`refusing to stage ${file}`);
 const source = join(root, file);
 const info = await stat(source).catch(() => null);
 if (!info?.isFile()) { if (programs.includes(file)) throw new Error(`missing ${file}; run ./scripts/build-programs.sh`); continue; }
 await mkdir(dirname(join(out, file)), { recursive: true });
 await cp(source, join(out, file));
 bytes += info.size;
}
await writeFile(join(out, 'railway.json'), JSON.stringify({
 $schema: 'https://railway.com/railway.schema.json',
 build: { builder: 'DOCKERFILE', dockerfilePath: 'frontend/Dockerfile' },
 deploy: { healthcheckPath: '/', healthcheckTimeout: 120, restartPolicyType: 'ON_FAILURE' },
}, null, 2));
await writeFile(join(out, '.dockerignore'), 'target\nnode_modules\ndist\n.analysis-runs\n');
console.log(`staged ${tracked.length + programs.length} files (${(bytes / 1048576).toFixed(0)} MB) in ${out}`);
