// Stage the build context for a manual deploy of the hosted Stock Transition
// frontend and analysis service (frontend/Dockerfile). A GitHub-connected
// Railway service builds the same thing from the repository itself.
//   node scripts/railway/stage-frontend.mjs <empty-output-dir>
//   npx @railway/cli up <output-dir> --path-as-root --service eplyx-stock --detach
// Only tracked (or new, not ignored) files are copied. The Dockerfile builds
// the candidate program itself, so local artifacts/, keypairs, run stores,
// build output and Milestone validation records never enter the context.
import { execFileSync } from 'node:child_process';
import { cp, mkdir, readdir, stat } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(fileURLToPath(new URL('../../', import.meta.url)));
const out = process.argv[2] && resolve(process.argv[2]);
if (!out) { console.error('usage: node scripts/railway/stage-frontend.mjs <empty-output-dir>'); process.exit(2); }
await mkdir(out, { recursive: true });
if ((await readdir(out)).length) { console.error(`${out} must be empty`); process.exit(2); }

const include = ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'package.json', 'package-lock.json',
 '.dockerignore', 'interface', 'engine', 'cloud', 'frontend', 'programs/eplyx-demo-conversion', 'docs',
 'reports', 'snapshots', 'evidence', 'probes', 'scenarios', 'policies', 'fixtures', 'examples'];
const tracked = execFileSync('git', ['-C', root, 'ls-files', '-z', '--cached', '--others', '--exclude-standard', '--', ...include], { encoding: 'utf8' })
 .split('\0').filter(Boolean)
 .filter(file => !file.startsWith('reports/milestone'));
let bytes = 0;
for (const file of tracked) {
 if (/keypair[^/]*\.json$|(^|\/)\.env|(^|\/)credentials\.json$/i.test(file)) throw new Error(`refusing to stage ${file}`);
 const source = join(root, file);
 const info = await stat(source).catch(() => null);
 if (!info?.isFile()) continue;
 await mkdir(dirname(join(out, file)), { recursive: true });
 await cp(source, join(out, file));
 bytes += info.size;
}
// The same service configuration a GitHub-connected deploy reads.
await cp(join(root, 'frontend/railway.json'), join(out, 'railway.json'));
console.log(`staged ${tracked.length} files (${(bytes / 1048576).toFixed(0)} MB) in ${out}`);
