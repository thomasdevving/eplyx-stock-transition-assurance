// Hosted analysis access code: without a code nothing changes; with one, only
// an unlocked browser session may start or read analyses.
import test from 'node:test';
import assert from 'node:assert/strict';
import { Readable } from 'node:stream';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { AnalysisService, handleAnalysisAPI } from '../analysis-service.mjs';
import { AccessGate } from '../access-gate.mjs';

const ORIGIN = 'https://eplyx.example';
const SESSION = 'b'.repeat(64);

async function withService(run) {
 const directory = await mkdtemp(join(tmpdir(), 'eplyx-access-'));
 const service = new AnalysisService({ root: process.cwd(), directory, runner: async () => { throw new Error('no engine in this test'); } });
 await service.initialize();
 try { await run(service); } finally { await rm(directory, { recursive: true, force: true }); }
}

async function call(service, access, { method = 'GET', path, body, cookie = `eplyx_session=${SESSION}`, ip = '203.0.113.7' }) {
 let status = 0, payload = null;
 const headers = {};
 const request = Readable.from(body === undefined ? [] : [Buffer.from(JSON.stringify(body))]);
 Object.assign(request, { url: path, method, socket: { remoteAddress: '10.0.0.1' },
  headers: { host: 'eplyx.example', origin: ORIGIN, 'content-type': 'application/json', 'x-forwarded-for': ip, ...(cookie ? { cookie } : {}) } });
 const response = { writeHead: s => { status = s; }, end: v => { payload = v ? JSON.parse(v) : null; },
  setHeader: (name, value) => { headers[name.toLowerCase()] = value; }, getHeader: name => headers[name.toLowerCase()] };
 await handleAnalysisAPI(service, request, response, ORIGIN, { access, secure: true });
 return { status, payload, cookies: [].concat(headers['set-cookie'] ?? []) };
}

test('without a configured code every analysis route behaves as before', async () => {
 await withService(async service => {
  const access = new AccessGate(undefined);
  const state = await call(service, access, { path: '/api/access' });
  assert.deepEqual(state.payload, { required: false, unlocked: true });
  const runs = await call(service, access, { method: 'POST', path: '/api/runs', body: { selection: {}, request_key: 'x' } });
  assert.notEqual(runs.status, 401);
 });
});

test('a configured code gates analyses until this session unlocks it', async () => {
 await withService(async service => {
  const access = new AccessGate('judge-demo-code', { secure: true });
  for (const [method, path] of [['POST', '/api/runs'], ['GET', '/api/runs/00000000-0000-4000-8000-000000000000']]) {
   const locked = await call(service, access, { method, path, body: method === 'POST' ? { selection: {}, request_key: 'x' } : undefined });
   assert.equal(locked.status, 401, path);
   assert.equal(locked.payload.error.code, 'AccessCodeRequired');
  }
  // Read-only service information stays public.
  assert.equal((await call(service, access, { path: '/api/health' })).status, 200);
  assert.equal((await call(service, access, { path: '/api/catalogue' })).status, 200);
  const wrong = await call(service, access, { method: 'POST', path: '/api/access', body: { code: 'guess' } });
  assert.equal(wrong.status, 401);
  assert.equal(wrong.cookies.length, 0);
  const right = await call(service, access, { method: 'POST', path: '/api/access', body: { code: 'judge-demo-code' } });
  assert.equal(right.status, 200);
  const cookie = right.cookies.find(c => c.startsWith('eplyx_access='));
  assert.match(cookie, /HttpOnly; SameSite=Strict; .*Secure/);
  const unlocked = `eplyx_session=${SESSION}; ${cookie.split(';')[0]}`;
  const after = await call(service, access, { method: 'POST', path: '/api/runs', body: { selection: {}, request_key: 'x' }, cookie: unlocked });
  assert.notEqual(after.status, 401, 'an unlocked session reaches the analysis API');
  // The unlock is bound to its session: another session with the same cookie stays locked.
  const stolen = await call(service, access, { method: 'POST', path: '/api/runs', body: { selection: {}, request_key: 'x' }, cookie: `eplyx_session=${'c'.repeat(64)}; ${cookie.split(';')[0]}` });
  assert.equal(stolen.status, 401);
  // The code itself never appears in a cookie or response.
  assert.ok(!JSON.stringify(right).includes('judge-demo-code'));
 });
});

test('repeated wrong codes are rate limited per client', async () => {
 await withService(async service => {
  const access = new AccessGate('judge-demo-code');
  for (let i = 0; i < 10; i++) assert.equal((await call(service, access, { method: 'POST', path: '/api/access', body: { code: `wrong-${i}` } })).status, 401);
  const blocked = await call(service, access, { method: 'POST', path: '/api/access', body: { code: 'judge-demo-code' } });
  assert.equal(blocked.status, 429, 'even the right code waits out the window');
  const other = await call(service, access, { method: 'POST', path: '/api/access', body: { code: 'judge-demo-code' }, ip: '198.51.100.9' });
  assert.equal(other.status, 200);
 });
});

test('cross-origin requests are refused before any access check', async () => {
 await withService(async service => {
  const access = new AccessGate('judge-demo-code');
  let status = 0;
  const request = Readable.from([]);
  Object.assign(request, { url: '/api/access', method: 'GET', headers: { host: 'eplyx.example', origin: 'https://evil.example' } });
  await handleAnalysisAPI(service, request, { writeHead: s => { status = s; }, end: () => {}, setHeader: () => {}, getHeader: () => undefined }, ORIGIN, { access });
  assert.equal(status, 403);
 });
});
