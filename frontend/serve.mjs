import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { extname, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { prepareVendorAssets } from './vendor.mjs';
import { prepareEvidence } from './evidence.mjs';
import { AnalysisService, handleAnalysisAPI } from './analysis-service.mjs';
const production = process.argv.includes('--production');
if (!production) { await prepareVendorAssets(); await prepareEvidence(); }
const root = fileURLToPath(new URL(production ? '../dist/' : './', import.meta.url));
const types = { '.html':'text/html; charset=utf-8', '.js':'text/javascript; charset=utf-8', '.css':'text/css; charset=utf-8', '.svg':'image/svg+xml', '.png':'image/png', '.jpg':'image/jpeg', '.json':'application/json', '.md':'text/plain; charset=utf-8', '.woff2':'font/woff2' };
const port = Number(process.env.PORT || 4173);
const analysis = new AnalysisService({root:fileURLToPath(new URL('../',import.meta.url)),directory:process.env.EPLYX_RUN_DIRECTORY||undefined});
await analysis.initialize();
createServer(async (request, response) => {
  try {
    if(await handleAnalysisAPI(analysis,request,response,`http://127.0.0.1:${port}`))return;
    if (!['GET', 'HEAD'].includes(request.method)) { response.writeHead(405); response.end(); return; }
    const path = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
    const file = resolve(root, path === '/' || path === '/evidence' || path === '/evidence/' || path === '/analysis' || path === '/analysis/' ? 'index.html' : `.${path}`);
    if (!file.startsWith(resolve(root) + sep)) { response.writeHead(403); response.end(); return; }
    if (!(await stat(file)).isFile()) throw new Error('Not a file');
    const body = await readFile(file);
    response.writeHead(200, { 'Content-Type': types[extname(file)] || 'application/octet-stream', 'Cache-Control':'no-cache', 'X-Content-Type-Options':'nosniff' });
    response.end(request.method === 'HEAD' ? undefined : body);
  } catch {
    response.writeHead(404, { 'Content-Type':'text/plain' }); response.end('Not found');
  }
}).listen(port, '127.0.0.1', () => console.log(`Eplyx Stock Transition: http://localhost:${port}`));
