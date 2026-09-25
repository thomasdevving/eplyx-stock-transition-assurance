// Start the real `eplyx-cloud` server and seed it through the public API and
// the real `eplyx sync` CLI, then write the seed for the browser specs.
//   EPLYX_CLOUD_TEST_DATABASE_URL=postgres://… node frontend/tests/cloud/server.mjs <port>
// Each start uses fresh accounts, so the database may be shared across runs.
import { cp, mkdir, mkdtemp, rm, writeFile, readFile } from 'node:fs/promises';
import { spawn, execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const [port] = process.argv.slice(2);
const database = process.env.EPLYX_CLOUD_TEST_DATABASE_URL;
if (!database) { console.error('set EPLYX_CLOUD_TEST_DATABASE_URL; cloud browser tests never skip silently'); process.exit(2); }
const repo = fileURLToPath(new URL('../../../', import.meta.url));
const exe = name => join(repo, 'target/debug', process.platform === 'win32' ? `${name}.exe` : name);
const base = `http://127.0.0.1:${port}`;
// The health check turns green before seeding ends; specs wait for this file.
await rm(join(repo, 'test-results/cloud-seed.json'), { force:true });
const server = spawn(exe('eplyx-cloud'), [], { stdio:'inherit', env:{ ...process.env, DATABASE_URL:database, PORT:port, EPLYX_CLOUD_PUBLIC_URL:base } });
server.on('exit', code => process.exit(code ?? 1));
for (let i = 0; i < 100; i++) { try { if ((await fetch(`${base}/healthz`)).ok) break; } catch {} await new Promise(r => setTimeout(r, 100)); }

const stamp = Date.now();
const email = `alice+${stamp}@example.com`, password = 'correct horse battery';
const post = async (path, body, cookie = '') => {
 const response = await fetch(base + path, { method:'POST', headers:{ 'content-type':'application/json', origin:base, ...(cookie ? { cookie } : {}) }, body:JSON.stringify(body) });
 if (!response.ok) throw new Error(`${path}: ${response.status} ${await response.text()}`);
 return { body:await response.json(), cookie:response.headers.get('set-cookie')?.split(';')[0] ?? cookie };
};
const { body:account, cookie } = await post('/api/v1/auth/signup', { email, password, name:'Alice' });
const { body:{ project } } = await post(`/api/v1/workspaces/${account.workspace_id}/projects`, { name:`transition-acceptance-${stamp}` }, cookie);
const { body:ci } = await post(`/api/v1/projects/${project.id}/ci-tokens`, { label:'browser seed' }, cookie);
const root = await mkdtemp(join(tmpdir(), 'eplyx-cloud-browser-'));
await cp(join(repo, 'fixtures/dashboard/transition-acceptance'), root, { recursive:true });
for (const child of ['runs', 'counterexamples', 'reproductions', 'cache']) await mkdir(join(root, '.eplyx', child), { recursive:true });
await promisify(execFile)(exe('eplyx'), ['sync'], { cwd:root, env:{ ...process.env, EPLYX_TOKEN:ci.token, EPLYX_PROJECT_ID:project.id, EPLYX_CLOUD_URL:base, EPLYX_CONFIG_DIR:join(root, 'home') } });
await writeFile(join(repo, 'test-results/cloud-seed.json'), JSON.stringify({ base, email, password, project:project.id })).catch(async () => { await mkdir(join(repo, 'test-results'), { recursive:true }); await writeFile(join(repo, 'test-results/cloud-seed.json'), JSON.stringify({ base, email, password, project:project.id })); });
console.log(`seeded ${project.id}`);
const stop = async () => { server.kill(); await rm(root, { recursive:true, force:true }); process.exit(0); };
process.on('SIGINT', stop); process.on('SIGTERM', stop);
