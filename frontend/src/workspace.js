// The optional hosted Eplyx team workspace. Following this link sends nothing:
// runs reach the workspace only through an explicit `eplyx sync`, and all
// analysis still runs locally or in CI.
export const CLOUD_WORKSPACE_URL = 'https://eplyx-cloud-production.up.railway.app';

export const workspaceLink = (label, className = '') =>
  `<a href="${CLOUD_WORKSPACE_URL}/" class="${className}" rel="noopener" data-cloud-workspace>${label}</a>`;
