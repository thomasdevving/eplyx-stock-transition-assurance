// Assemble release assets: SHA256SUMS and eplyx-release.json over the exact
// archives in <dir>, plus the two installers. Fails unless every supported
// platform is present exactly once.
//   node scripts/release/assemble.mjs <dir> <commit>
import { createHash } from 'node:crypto';
import { copyFile, readdir, readFile, stat, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

export const PLATFORMS = Object.freeze({
  'darwin-arm64': { target:'aarch64-apple-darwin', ext:'tar.gz', binary:'eplyx' },
  'linux-x86_64': { target:'x86_64-unknown-linux-gnu', ext:'tar.gz', binary:'eplyx' },
  'windows-x86_64': { target:'x86_64-pc-windows-msvc', ext:'zip', binary:'eplyx.exe' },
});

export function artifactName(version, platform) {
  const spec = PLATFORMS[platform];
  if (!spec) throw new Error(`unsupported platform ${platform}`);
  if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.]+)?$/.test(version)) throw new Error(`invalid version ${version}`);
  return `eplyx-v${version}-${platform}.${spec.ext}`;
}

export function parseSums(text) {
  return text.split('\n').filter(Boolean).map(line => {
    const match = /^([0-9a-f]{64})  ([A-Za-z0-9._-]+)$/.exec(line);
    if (!match) throw new Error(`malformed SHA256SUMS line: ${line}`);
    return { sha256:match[1], file:match[2] };
  });
}

export function validateManifest(manifest) {
  if (manifest.schema_version !== 1 || manifest.name !== 'eplyx') throw new Error('unsupported release manifest');
  const platforms = manifest.artifacts.map(a => a.platform).sort();
  if (JSON.stringify(platforms) !== JSON.stringify(Object.keys(PLATFORMS).sort())) throw new Error('manifest platforms differ from the supported set');
  for (const a of manifest.artifacts) {
    if (a.file !== artifactName(manifest.version, a.platform) || !/^[0-9a-f]{64}$/.test(a.sha256)) throw new Error(`invalid manifest artifact ${a.file}`);
  }
  return manifest;
}

async function main() {
  const [dir, commit = 'unknown'] = process.argv.slice(2);
  const root = fileURLToPath(new URL('../../', import.meta.url));
  const version = /^version = "(.+)"$/m.exec(await readFile(join(root, 'Cargo.toml'), 'utf8'))[1];
  const present = new Set(await readdir(dir));
  const artifacts = [];
  for (const [platform, spec] of Object.entries(PLATFORMS)) {
    const file = artifactName(version, platform);
    if (!present.has(file)) throw new Error(`missing release archive ${file}`);
    const bytes = await readFile(join(dir, file));
    artifacts.push({ platform, target:spec.target, file, binary:spec.binary, size:(await stat(join(dir, file))).size, sha256:createHash('sha256').update(bytes).digest('hex') });
  }
  const unexpected = [...present].filter(f => /^eplyx-v/.test(f) && !artifacts.some(a => a.file === f));
  if (unexpected.length) throw new Error(`unexpected archives: ${unexpected.join(', ')}`);
  for (const installer of ['install.sh', 'install.ps1']) await copyFile(join(root, 'scripts/install', installer), join(dir, installer));
  const sums = [...artifacts.map(a => [a.sha256, a.file])];
  for (const installer of ['install.sh', 'install.ps1']) sums.push([createHash('sha256').update(await readFile(join(dir, installer))).digest('hex'), installer]);
  await writeFile(join(dir, 'SHA256SUMS'), sums.map(([h, f]) => `${h}  ${f}`).join('\n') + '\n');
  const manifest = validateManifest({ schema_version:1, name:'eplyx', version, tag:`v${version}`, commit, artifacts });
  await writeFile(join(dir, 'eplyx-release.json'), JSON.stringify(manifest, null, 2) + '\n');
  console.log(`Assembled ${artifacts.length} archives for v${version}`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) await main();
