// Smoke-test a deployed Eplyx cloud workspace without credentials:
//   node scripts/cloud-smoke.mjs https://eplyx-cloud-production.up.railway.app
// Checks health, security headers, page shells and that every project API
// refuses anonymous access. It creates nothing and sends no secret.
const base = (process.argv[2] ?? '').replace(/\/+$/, '');
if (!/^https:\/\/|^http:\/\/127\.0\.0\.1/.test(base)) { console.error('usage: node scripts/cloud-smoke.mjs https://<origin>'); process.exit(2); }
const checks = [];
const check = (name, ok, detail = '') => { checks.push({ name, ok }); console.log(`${ok ? '✓' : '✗'} ${name}${detail ? ` — ${detail}` : ''}`); };

const health = await fetch(`${base}/healthz`);
const body = await health.json().catch(() => ({}));
check('health endpoint', health.ok && body.ok === true && body.database === 'reachable', JSON.stringify(body));
check('server declares it executes nothing', body.executes === false);

const page = await fetch(`${base}/`);
const csp = page.headers.get('content-security-policy') ?? '';
check('landing page', page.ok && (await page.text()).includes('/assets/cloud.js'));
check('content security policy', csp.includes("script-src 'self'") && csp.includes("frame-ancestors 'none'"));
check('frame and sniffing protection', page.headers.get('x-frame-options') === 'DENY' && page.headers.get('x-content-type-options') === 'nosniff');
check('HSTS over https', !base.startsWith('https://') || Boolean(page.headers.get('strict-transport-security')));
for (const asset of ['dashboard.js', 'pages.js', 'env.js', 'cloud.js', 'settings.js', 'dashboard.css', 'cloud.css']) {
 const response = await fetch(`${base}/assets/${asset}`);
 check(`asset ${asset}`, response.ok);
}
const shell = await (await fetch(`${base}/p/prj_00000000000000000000/runs`)).text();
check('project page shell points at the hosted view API', shell.includes('data-api="/api/v1/projects/prj_00000000000000000000/view"') && shell.includes('data-cloud="1"'));
for (const path of ['/api/v1/workspaces', '/api/v1/me', '/api/v1/projects/prj_00000000000000000000/view/project', '/api/v1/projects/prj_00000000000000000000/view/runs']) {
 const response = await fetch(base + path);
 check(`anonymous ${path} refused`, response.status === 401, String(response.status));
}
const post = await fetch(`${base}/api/v1/projects/prj_00000000000000000000/runs`, { method:'POST', headers:{ 'content-type':'application/json' }, body:'{}' });
check('anonymous sync refused', post.status === 401, String(post.status));
const forged = await fetch(`${base}/api/v1/workspaces`, { headers:{ authorization:'Bearer eplyx_u_forged-token' } });
check('forged token refused', forged.status === 401, String(forged.status));
const crossSite = await fetch(`${base}/api/v1/auth/login`, { method:'POST', headers:{ 'content-type':'application/json', origin:'https://evil.example' }, body:JSON.stringify({ email:'x@example.com', password:'x' }) });
check('cross-site sign-in refused', crossSite.status === 403, String(crossSite.status));
const failed = checks.filter(c => !c.ok).length;
console.log(`\n${checks.length - failed}/${checks.length} checks passed against ${base}`);
process.exit(failed ? 1 : 0);
