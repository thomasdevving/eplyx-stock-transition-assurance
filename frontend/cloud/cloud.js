// Hosted Eplyx workspace pages outside a project: landing, sign-in, sign-up,
// CLI device approval and the workspace list. Project pages reuse the local
// dashboard modules. Nothing here runs analysis; it manages accounts only.
import { initializeMode } from './mode.js';
import { Mark } from './brand.js';
import { esc, panel, kv, commandLine, gatePill, count, empty, ago } from './ui.js';

initializeMode();
const app = document.querySelector('#app');

async function call(method, path, body) {
 const response = await fetch(path, { method, credentials:'same-origin', headers:{ Accept:'application/json', ...(body ? { 'Content-Type':'application/json' } : {}) }, body:body ? JSON.stringify(body) : undefined });
 const data = await response.json().catch(() => ({}));
 if (!response.ok) { const error = new Error(data.error || `Request failed (${response.status})`); error.status = response.status; throw error; }
 return data;
}
const me = () => call('GET', '/api/v1/me').catch(error => { if (error.status === 401) return null; throw error; });
const nextPath = () => { const next = new URLSearchParams(location.search).get('next'); return next && next.startsWith('/') && !next.startsWith('//') ? next : '/'; };

function frame(body, user) {
 return `<div class="cloud">
  <header class="cloud__top">
   <a class="brand" href="/" aria-label="Eplyx cloud">${Mark({ className:'brand__mark' })}<span class="brand__lockup"><span class="brand__word">Eplyx</span><span class="brand__sub">Cloud workspace</span></span></a>
   <span class="spacer"></span>
   ${user ? `<span class="cloud__user">${esc(user.email)}</span><button type="button" class="button button--ghost" data-sign-out>Sign out</button>` : '<a class="button button--ghost" href="/login">Sign in</a>'}
  </header>
  <main id="main" tabindex="-1">${body}</main>
 </div>`;
}

const WHAT = `<ul class="limits limits--tight">
 <li><strong>Synced by default:</strong> run metadata (commit, branch, source, version, hashes, gate policy and outcome), the engine report with its analytical statuses, evidence bindings, the package manifest and config (public addresses and terms), the search result, saved counterexamples and reproduction records — each as exact bytes with its SHA-256.</li>
 <li><strong>Never synced:</strong> source code, the candidate <code>.so</code>, population/stress/wallet captures, <code>eplyx.toml</code>, environment variables, RPC URLs or credentials, local absolute paths and Solana keys.</li>
 <li>Projects and runs are private to their workspace. A public repository never makes a project public.</li>
</ul>`;

async function landing() {
 const demo = await fetch('/api/v1/demo/view/project', { credentials:'omit' }).then(r => r.ok).catch(() => false);
 app.innerHTML = frame(`<section class="welcome">
  <span class="eyebrow">Optional team sync</span>
  <h1>Cloud sync is optional. Eplyx execution stays local.</h1>
  <p class="lead">Preflight, search, reproduction and gate evaluation run on your machine or in CI without an account. This workspace shows your team synced local and CI results, counterexamples, reproduction history and run comparisons.</p>
  <p><a class="button" href="/signup">Create an account</a> <a class="button button--ghost" href="/login">Sign in</a>${demo ? ' <a class="button button--ghost" href="/demo">View the public demo project</a>' : ''}</p>
 </section>
 ${panel({ title:'Local work and optional sync', body:`<div class="grid-2 grid-2--tight"><div><h3 class="subhead">Local only</h3>${commandLine('eplyx preflight')}${commandLine('eplyx search')}${commandLine('eplyx dashboard')}</div><div><h3 class="subhead">Optional team sync</h3>${commandLine('eplyx login')}${commandLine('eplyx link')}${commandLine('eplyx sync')}</div></div>` })}
 ${panel({ title:'What is synced?', body:WHAT })}`, null);
}

function authForm({ title, intro, fields, submit, footer, onSubmit }) {
 app.innerHTML = frame(`<section class="auth-card"><h1>${esc(title)}</h1><p class="muted">${intro}</p>
  <form class="form" data-form novalidate>${fields.map(([name, label, type, extra = '']) => `<label class="field"><span>${esc(label)}</span><input name="${name}" type="${type}" ${extra}></label>`).join('')}
   <p class="form-error" data-error role="alert"></p><button class="button" type="submit">${esc(submit)}</button></form>
  <p class="muted">${footer}</p></section>`, null);
 const form = app.querySelector('[data-form]');
 form.addEventListener('submit', async event => {
  event.preventDefault();
  const values = Object.fromEntries(new FormData(form));
  const button = form.querySelector('button'); button.disabled = true;
  try { await onSubmit(values); } catch (error) { form.querySelector('[data-error]').textContent = error.message; button.disabled = false; }
 });
 form.querySelector('input')?.focus();
}

function login() {
 authForm({ title:'Sign in', intro:'Sign in to your Eplyx cloud workspace. Your password is entered here only — never in the CLI.', submit:'Sign in',
  fields:[['email', 'Email', 'email', 'autocomplete="username" required'], ['password', 'Password', 'password', 'autocomplete="current-password" required']],
  footer:`No account? <a href="/signup${location.search}">Create one</a>.`,
  onSubmit:async v => { await call('POST', '/api/v1/auth/login', { email:v.email, password:v.password }); location.assign(nextPath()); } });
}

function signup() {
 authForm({ title:'Create an account', intro:'An account gives you a private workspace. Nothing is synced until you run <code>eplyx link</code> and <code>eplyx sync</code>.', submit:'Create account',
  fields:[['name', 'Name', 'text', 'autocomplete="name" required maxlength="80"'], ['email', 'Email', 'email', 'autocomplete="username" required'], ['password', 'Password (10+ characters)', 'password', 'autocomplete="new-password" required minlength="10"'], ['signup_code', 'Sign-up code (if this server requires one)', 'text', 'autocomplete="off"']],
  footer:`Already registered? <a href="/login${location.search}">Sign in</a>.`,
  onSubmit:async v => { await call('POST', '/api/v1/auth/signup', { name:v.name, email:v.email, password:v.password, ...(v.signup_code ? { signup_code:v.signup_code } : {}) }); location.assign(nextPath()); } });
}

async function device(user) {
 if (!user) { location.assign(`/login?next=${encodeURIComponent(location.pathname + location.search)}`); return; }
 const initial = new URLSearchParams(location.search).get('code') ?? '';
 const render = async (code, revealApproval=false) => {
  let info = null, problem = '';
  if (code) info = await call('GET', `/api/v1/auth/device/lookup?code=${encodeURIComponent(code)}`).catch(error => { problem = error.message; return null; });
  const body = info
   ? info.state === 'pending'
    ? `<p>A CLI is asking to sign in as <strong>${esc(user.email)}</strong>:</p><p><code>${esc(info.client)}</code> · requested ${esc(ago(info.created_at))}</p><div class="device-code">${esc(info.user_code)}</div>
       <p class="muted">Approve only if this code matches the one shown in your terminal. The CLI receives a scoped Eplyx token that can link projects and sync run results; it never sees your password.</p>
       <p><button class="button" data-decide="true">Approve</button> <button class="button button--ghost" data-decide="false">Deny</button></p>`
    : `<div class="device-outcome${revealApproval&&info.state==='approved'?' device-outcome--approved':''}"><p>This code is <strong>${esc(info.state)}</strong>.</p>${info.state === 'approved' ? '<p class="muted">Return to your terminal; <code>eplyx login</code> finishes on its own.</p>' : ''}</div>`
   : `<form class="form" data-code-form><label class="field"><span>Code shown by <code>eplyx login</code></span><input name="code" value="${esc(code)}" autocomplete="off" required></label><p class="form-error" role="alert">${esc(problem)}</p><button class="button" type="submit">Continue</button></form>`;
  app.innerHTML = frame(`<section class="auth-card"><h1>Approve CLI sign-in</h1>${body}</section>`, user);
  app.querySelector('[data-code-form]')?.addEventListener('submit', event => { event.preventDefault(); render(new FormData(event.target).get('code')); });
  app.querySelectorAll('[data-decide]').forEach(button => button.addEventListener('click', async () => {
   const approve=button.dataset.decide==='true';
   button.disabled=true;
   try {
    await call('POST', '/api/v1/auth/device/approve', { user_code:info.user_code, approve });
    await render(info.user_code, approve);
   } catch(error) { alert(error.message); button.disabled=false; }
  }));
 };
 await render(initial);
}

async function home(user) {
 const { workspaces } = await call('GET', '/api/v1/workspaces');
 const members = await Promise.all(workspaces.map(ws => call('GET', `/api/v1/workspaces/${ws.id}/members`).catch(() => ({ members:[] }))));
 const section = (ws, n) => panel({ cls:'workspace', eyebrow:ws.role === 'owner' ? 'Workspace · owner' : 'Workspace · member', title:ws.name, body:`
  ${ws.projects.length ? `<div class="project-list">${ws.projects.map(p => `<a class="project-row" href="/p/${esc(p.id)}"><strong>${esc(p.name)}</strong><span class="muted">${count(p.runs)} synced runs</span>${p.latest_gate ? gatePill(p.latest_gate) : ''}<code class="tech-only">${esc(p.id)}</code></a>`).join('')}</div>` : empty('No projects yet. Create one here, or run `eplyx link --create <name>` in your project.')}
  <form class="form-row" data-new-project="${esc(ws.id)}"><label class="field"><span>New project</span><input name="name" required maxlength="80" placeholder="my-transition"></label><button class="button button--ghost" type="submit">Create project</button></form>
  <h3 class="subhead">Members</h3>${kv(members[n].members.map(m => [m.email, `${esc(m.role)}${ws.role === 'owner' && m.role !== 'owner' ? ` <button type="button" class="button button--ghost" data-remove="${esc(ws.id)}:${esc(m.id)}">Remove</button>` : ''}`]))}
  ${ws.role === 'owner' ? `<form class="form-row" data-add-member="${esc(ws.id)}"><label class="field"><span>Add a member by account email</span><input name="email" type="email" required></label><button class="button button--ghost" type="submit">Add member</button></form>` : ''}
  <p class="form-error" data-ws-error="${esc(ws.id)}" role="alert"></p>` });
 app.innerHTML = frame(`<div class="page-head"><h1>Workspaces</h1><p class="muted">Projects are private to their workspace. Owners manage members and CI tokens; members view and sync.</p></div>
  ${workspaces.map(section).join('')}
  ${panel({ title:'New workspace', body:'<form class="form-row" data-new-workspace><label class="field"><span>Name</span><input name="name" required maxlength="80"></label><button class="button button--ghost" type="submit">Create workspace</button></form>' })}
  ${panel({ title:'Connect a local project', body:`${commandLine('eplyx login')}${commandLine('eplyx link')}${commandLine('eplyx sync')}<p class="muted">Cloud sync is optional. Eplyx execution stays local.</p>` })}
  ${panel({ title:'What is synced?', body:WHAT })}`, user);
 const fail = (ws, error) => { const slot = app.querySelector(`[data-ws-error="${ws}"]`); if (slot) slot.textContent = error.message; else alert(error.message); };
 app.querySelectorAll('[data-new-project]').forEach(form => form.addEventListener('submit', async event => { event.preventDefault(); const ws = form.dataset.newProject;
  try { const { project } = await call('POST', `/api/v1/workspaces/${ws}/projects`, { name:new FormData(form).get('name') }); location.assign(`/p/${project.id}`); } catch (error) { fail(ws, error); } }));
 app.querySelectorAll('[data-add-member]').forEach(form => form.addEventListener('submit', async event => { event.preventDefault(); const ws = form.dataset.addMember;
  try { await call('POST', `/api/v1/workspaces/${ws}/members`, { email:new FormData(form).get('email') }); home(user); } catch (error) { fail(ws, error); } }));
 app.querySelectorAll('[data-remove]').forEach(button => button.addEventListener('click', async () => { const [ws, member] = button.dataset.remove.split(':');
  try { await call('DELETE', `/api/v1/workspaces/${ws}/members/${member}`); home(user); } catch (error) { fail(ws, error); } }));
 app.querySelector('[data-new-workspace]').addEventListener('submit', async event => { event.preventDefault();
  try { await call('POST', '/api/v1/workspaces', { name:new FormData(event.target).get('name') }); home(user); } catch (error) { alert(error.message); } });
}

document.addEventListener('click', event => {
 if (!event.target.closest('[data-sign-out]')) return;
 call('POST', '/api/v1/auth/logout').finally(() => location.assign('/'));
});

async function route() {
 const path = location.pathname.replace(/\/+$/, '') || '/';
 try {
  if (path === '/login') return login();
  if (path === '/signup') return signup();
  const user = (await me())?.user ?? null;
  if (path === '/device') return device(user);
  if (path === '/' || path === '/workspaces') return user ? home(user) : landing();
  app.innerHTML = frame(`<div class="page-head"><h1>Not found</h1></div>${empty('This page does not exist in the Eplyx cloud workspace.')}`, user);
 } catch (error) {
  app.innerHTML = frame(`<div class="page-head"><h1>Unavailable</h1></div>${empty(error.message)}`, null);
 }
}
route();
