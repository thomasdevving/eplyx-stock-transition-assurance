// The optional hosted Eplyx team workspace. Following this link does not upload
// run data: only an explicit `eplyx sync` sends selected local/CI CLI artifacts.
// Browser analysis uses its separate same-origin job API.
export const CLOUD_WORKSPACE_URL = 'https://eplyx-cloud-production.up.railway.app';

export const workspaceLink = (label, className = '') =>
  `<a href="${CLOUD_WORKSPACE_URL}/" class="${className}" rel="noopener" data-cloud-workspace>${label}</a>`;
