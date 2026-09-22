// Operator-side CI assembly. The engine still validates the completed package.
import {createHash} from 'node:crypto';
import {mkdir, readFile, realpath, writeFile} from 'node:fs/promises';
import {resolve, relative, dirname, sep} from 'node:path';

const [templateArg, programArg, outputArg] = process.argv.slice(2);
if (!templateArg || !programArg || !outputArg || process.argv.length !== 5) {
  throw Error('usage: node scripts/assemble-transition-package.mjs <template> <candidate.so> <new-output-directory>');
}
const root = await realpath(resolve(import.meta.dirname, '..'));
function withinRoot(path) {
  const full = resolve(path);
  const rel = relative(root, full);
  if (rel === '..' || rel.startsWith(`..${sep}`)) throw Error('package assembly path escapes repository');
  return full;
}
const template = withinRoot(templateArg);
const programPath = withinRoot(programArg);
const output = withinRoot(outputArg);
for (const input of [template, programPath]) {
  const actual = await realpath(input);
  if (!actual.startsWith(root + sep)) throw Error('package assembly input symlink escapes repository');
}
const outputParent = await realpath(dirname(output));
if (outputParent !== root && !outputParent.startsWith(root + sep)) {
  throw Error('package assembly output symlink escapes repository');
}
const actualTemplate = await realpath(template);
const actualManifest = await realpath(resolve(template, 'eplyx.json'));
if (!actualManifest.startsWith(actualTemplate + sep)) throw Error('template manifest symlink escapes package');
const manifest = JSON.parse(await readFile(resolve(template, 'eplyx.json'), 'utf8'));
if (typeof manifest.config !== 'string' || !manifest.config || manifest.config.includes('..') || resolve(template, manifest.config) === template) {
  throw Error('invalid template config path');
}
const configPath = resolve(template, manifest.config);
if (!configPath.startsWith(`${template}${sep}`)) throw Error('template config escapes package');
const actualConfig = await realpath(configPath);
if (!actualConfig.startsWith(actualTemplate + sep)) throw Error('template config symlink escapes package');
const config = await readFile(configPath);
const program = await readFile(programPath);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
manifest.config = 'config.json';
manifest.configSha256 = hash(config);
manifest.candidateProgram.artifact = 'program.so';
manifest.candidateProgram.sha256 = hash(program);
await mkdir(output);
await writeFile(resolve(output, 'eplyx.json'), JSON.stringify(manifest, null, 2) + '\n', {flag: 'wx'});
await writeFile(resolve(output, 'config.json'), config, {flag: 'wx'});
await writeFile(resolve(output, 'program.so'), program, {flag: 'wx'});
process.stdout.write(JSON.stringify({candidate_program_sha256: manifest.candidateProgram.sha256, output}) + '\n');
