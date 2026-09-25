// Hosted analysis access code. When the server answers an analysis request
// with AccessCodeRequired, ask once for the code, unlock this browser session
// and retry the request. Locally no code is configured and nothing here runs.
import { esc } from './format.js';

const nativeFetch = window.fetch.bind(window);
let pending = null;

function prompt() {
 pending ??= new Promise(resolve => {
  const dialog = document.createElement('dialog');
  dialog.className = 'access-dialog';
  dialog.innerHTML = `<form method="dialog" class="access-dialog__form">
   <h2>Analysis access code</h2>
   <p>Fresh analyses on this hosted demo read live Solana state, so they need an access code. Saved results stay available without one. Eplyx never moves funds.</p>
   <label><span>Access code</span><input name="code" type="password" autocomplete="off" required maxlength="256"></label>
   <p class="access-dialog__error" role="alert"></p>
   <div class="access-dialog__actions"><button type="button" class="button button--text" data-cancel>Cancel</button><button type="submit" class="button button--primary">Unlock analysis</button></div>
  </form>`;
  document.body.append(dialog);
  const finish = ok => { dialog.close(); dialog.remove(); pending = null; resolve(ok); };
  dialog.querySelector('[data-cancel]').addEventListener('click', () => finish(false));
  dialog.addEventListener('cancel', event => { event.preventDefault(); finish(false); });
  dialog.querySelector('form').addEventListener('submit', async event => {
   event.preventDefault();
   const code = new FormData(event.target).get('code');
   const response = await nativeFetch('/api/access', { method:'POST', headers:{ 'Content-Type':'application/json' }, body:JSON.stringify({ code }) }).catch(() => null);
   if (response?.ok) return finish(true);
   const body = await response?.json().catch(() => ({}));
   dialog.querySelector('.access-dialog__error').innerHTML = esc(body?.error?.message ?? 'The analysis service could not check the code.');
  });
  dialog.showModal();
  dialog.querySelector('input').focus();
 });
 return pending;
}

window.fetch = async (input, init) => {
 const response = await nativeFetch(input, init);
 const url = typeof input === 'string' ? input : input?.url ?? '';
 if (response.status !== 401 || !url.startsWith('/api/') || url === '/api/access') return response;
 const body = await response.clone().json().catch(() => ({}));
 if (body?.error?.code !== 'AccessCodeRequired') return response;
 return (await prompt()) ? nativeFetch(input, init) : response;
};
