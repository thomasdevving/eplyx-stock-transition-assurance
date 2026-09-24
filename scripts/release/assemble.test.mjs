import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { PLATFORMS, artifactName, parseSums, validateManifest } from './assemble.mjs';

const script = fileURLToPath(new URL('./assemble.mjs', import.meta.url));
const version = /^version = "(.+)"$/m.exec(await readFile(new URL('../../Cargo.toml', import.meta.url), 'utf8'))[1];
const assemble = (dir, commit = 'abc123') => spawnSync(process.execPath, [script, dir, commit], { encoding:'utf8' });

test('artifact names are fixed per supported platform', () => {
 assert.equal(artifactName('0.1.0', 'darwin-arm64'), 'eplyx-v0.1.0-darwin-arm64.tar.gz');
 assert.equal(artifactName('0.1.0', 'linux-x86_64'), 'eplyx-v0.1.0-linux-x86_64.tar.gz');
 assert.equal(artifactName('0.1.0', 'windows-x86_64'), 'eplyx-v0.1.0-windows-x86_64.zip');
 assert.throws(() => artifactName('0.1.0', 'linux-arm64'), /unsupported platform/);
 assert.throws(() => artifactName('0.1', 'darwin-arm64'), /invalid version/);
 assert.deepEqual(Object.keys(PLATFORMS).sort(), ['darwin-arm64', 'linux-x86_64', 'windows-x86_64']);
});

test('SHA256SUMS parsing is strict', () => {
 assert.deepEqual(parseSums(`${'a'.repeat(64)}  eplyx-v0.1.0-darwin-arm64.tar.gz\n`), [{ sha256:'a'.repeat(64), file:'eplyx-v0.1.0-darwin-arm64.tar.gz' }]);
 for (const line of [`${'a'.repeat(63)}  x.tar.gz`, `${'A'.repeat(64)}  x.tar.gz`, `${'a'.repeat(64)} x.tar.gz`, `${'a'.repeat(64)}  ../x.tar.gz`, `${'a'.repeat(64)}  dir/x.tar.gz`]) {
  assert.throws(() => parseSums(line), /malformed/, line);
 }
});

test('assembly hashes the exact archives and writes a valid manifest', async () => {
 const dir = await mkdtemp(join(tmpdir(), 'eplyx-assemble-'));
 try {
  for (const platform of Object.keys(PLATFORMS)) await writeFile(join(dir, artifactName(version, platform)), `archive ${platform}`);
  const result = assemble(dir);
  assert.equal(result.status, 0, result.stderr);
  const sums = parseSums(await readFile(join(dir, 'SHA256SUMS'), 'utf8'));
  assert.deepEqual(sums.map(s => s.file).sort(), [...Object.keys(PLATFORMS).map(p => artifactName(version, p)), 'install.ps1', 'install.sh'].sort());
  for (const { file, sha256 } of sums) assert.equal(sha256, createHash('sha256').update(await readFile(join(dir, file))).digest('hex'), file);
  const manifest = validateManifest(JSON.parse(await readFile(join(dir, 'eplyx-release.json'), 'utf8')));
  assert.equal(manifest.tag, `v${version}`);
  assert.equal(manifest.commit, 'abc123');
  assert.equal(manifest.artifacts.find(a => a.platform === 'windows-x86_64').binary, 'eplyx.exe');
  assert.throws(() => validateManifest({ ...manifest, artifacts:manifest.artifacts.slice(1) }), /platforms differ/);
  assert.throws(() => validateManifest({ ...manifest, artifacts:manifest.artifacts.map(a => ({ ...a, sha256:'x' })) }), /invalid manifest artifact/);
  assert.throws(() => validateManifest({ ...manifest, schema_version:2 }), /unsupported release manifest/);
 } finally { await rm(dir, { recursive:true, force:true }); }
});

test('assembly refuses a missing platform or an unexpected archive', async () => {
 const dir = await mkdtemp(join(tmpdir(), 'eplyx-assemble-'));
 try {
  await writeFile(join(dir, artifactName(version, 'darwin-arm64')), 'x');
  await writeFile(join(dir, artifactName(version, 'linux-x86_64')), 'x');
  let result = assemble(dir);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /missing release archive eplyx-v.*-windows-x86_64\.zip/);
  await writeFile(join(dir, artifactName(version, 'windows-x86_64')), 'x');
  await writeFile(join(dir, `eplyx-v${version}-linux-arm64.tar.gz`), 'x');
  result = assemble(dir);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /unexpected archives/);
 } finally { await rm(dir, { recursive:true, force:true }); }
});
