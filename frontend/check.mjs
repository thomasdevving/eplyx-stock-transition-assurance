import { readdir } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { prepareEvidence } from './evidence.mjs';
for (const directory of ['src', 'dashboard']) for (const file of await readdir(new URL(`./${directory}/`, import.meta.url))) {
 if (!file.endsWith('.js')) continue;
 const result=spawnSync(process.execPath,['--check',fileURLToPath(new URL(`./${directory}/${file}`,import.meta.url))],{stdio:'inherit'});
 if (result.status!==0) process.exit(result.status || 1);
}
await prepareEvidence();
console.log('Frontend JavaScript syntax and published report digests verified.');
