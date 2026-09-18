import { cp, mkdir, rm } from 'node:fs/promises';
import { prepareVendorAssets } from './vendor.mjs';
import { prepareEvidence } from './evidence.mjs';
await prepareVendorAssets();
await prepareEvidence();
const out = new URL('../dist/', import.meta.url);
await rm(out, { recursive: true, force: true });
await mkdir(out, { recursive: true });
for (const path of ['index.html', 'src', 'public']) {
  await cp(new URL(path, import.meta.url), new URL(path, out), { recursive: true });
}
console.log('Built static Eplyx Stock Transition frontend in dist/');
