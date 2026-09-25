import { test, expect } from '@playwright/test';
import { readFileSync, existsSync } from 'node:fs';

const HEALTHY = 'run_20260924122823725_ce7b4d55310c';
const UNDERFUNDED = 'run_20260924123736483_ce7b4d55310c';
const RESERVE = 'cx_2ab4735232a3f228da1a29d1';
const seed = () => JSON.parse(readFileSync('test-results/cloud-seed.json', 'utf8'));

async function signIn(page) {
 const { email, password } = seed();
 await page.goto('/login');
 await page.fill('input[name=email]', email);
 await page.fill('input[name=password]', password);
 await page.click('button[type=submit]');
 await expect(page.locator('h1')).toHaveText('Workspaces');
}

test.beforeAll(async () => {
 for (let i = 0; i < 300 && !existsSync('test-results/cloud-seed.json'); i++) await new Promise(r => setTimeout(r, 100));
 expect(existsSync('test-results/cloud-seed.json')).toBeTruthy();
});

test.beforeEach(async ({ page }) => {
 page.on('pageerror', error => { throw error; });
 await page.addInitScript(() => localStorage.setItem('eplyx-detail', 'overview'));
});

test('signed-out visitors see that sync is optional and cannot read projects', async ({ page }) => {
 await page.goto('/');
 await expect(page.locator('h1')).toHaveText('Cloud sync is optional. Eplyx execution stays local.');
 await expect(page.getByText('What is synced?')).toBeVisible();
 await page.goto(`/p/${seed().project}`);
 await expect(page).toHaveURL(/\/login\?next=/);
});

test('workspace lists the synced project and the overview says it shows synced results', async ({ page }) => {
 await signIn(page);
 await page.locator('.project-row').first().click();
 await expect(page.locator('.synced-banner')).toContainText('Viewing them does not run RPC, execution or replay');
 const hero = page.locator('.hero-status');
 await expect(hero.locator('h1')).toHaveText('BLOCKED');
 await expect(hero).toContainText('Synced result');
 await expect(page.locator('.release-health')).toContainText('Latest CI run');
 await expect(page.locator('.release-health')).toContainText('74');
 await page.screenshot({ path:'test-results/cloud-overview.png', fullPage:true });
});

test('hosted comparison keeps the search-equivalence warning', async ({ page }) => {
 await signIn(page);
 await page.goto(`/p/${seed().project}/compare?left=${HEALTHY}&right=${UNDERFUNDED}`);
 await expect(page.locator('.equivalence--warn')).toContainText('Search domains differ — counterexample disappearance does not prove resolution.');
 await expect(page.getByText('Resolved (equivalent search)')).toHaveCount(0);
 await expect(page.locator('body')).not.toContainText(/\bfixed\b/i);
});

test('counterexample detail offers a local reproduce command and synced history', async ({ page }) => {
 await signIn(page);
 await page.goto(`/p/${seed().project}/counterexamples/${RESERVE}`);
 await expect(page.locator('.kind--derived')).toBeVisible();
 await expect(page.locator('.command code').first()).toHaveText(`eplyx reproduce ${RESERVE}`);
 await expect(page.getByText('Reproduction history')).toBeVisible();
 await expect(page.locator('body')).toContainText('the cloud never replays');
});

test('run evidence stays local and settings mint a CI token once', async ({ page }) => {
 await signIn(page);
 await page.goto(`/p/${seed().project}/runs/${HEALTHY}#evidence`);
 await expect(page.locator('#evidence')).toContainText('Artifacts stay on the machine that ran Eplyx');
 await expect(page.locator('#evidence a[href*="/artifacts/"]')).toHaveCount(0);
 await page.goto(`/p/${seed().project}/settings`);
 await page.fill('input[name=label]', 'browser test');
 await page.click('button:has-text("Create CI token")');
 await expect(page.locator('.secret')).toContainText('eplyx_ci_');
 await page.reload();
 await expect(page.locator('.secret')).toHaveCount(0);
 await expect(page.locator('table')).toContainText('browser test');
});

test('device approval requires the browser session and the matching code', async ({ page, request }) => {
 await page.emulateMedia({ reducedMotion:'reduce' });
 const started = await (await request.post('/api/v1/auth/device', { data:{ client:'browser spec CLI' } })).json();
 await signIn(page);
 await page.goto(`/device?code=${started.user_code}`);
 await expect(page.locator('.device-code')).toHaveText(started.user_code);
 await page.click('button[data-decide=true]');
 await expect(page.locator('.auth-card')).toContainText('approved');
 await expect(page.locator('.device-outcome--approved')).toHaveCSS('animation-duration','0.12s');
 const token = await request.post('/api/v1/auth/device/token', { data:{ device_code:started.device_code } });
 expect(token.ok()).toBeTruthy();
});

test('narrow screens keep the hosted workspace inside the viewport', async ({ page }) => {
 await page.setViewportSize({ width:390, height:844 });
 await signIn(page);
 await page.goto(`/p/${seed().project}/runs`);
 await expect(page.locator('tbody tr[data-run]')).toHaveCount(4);
 const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
 expect(overflow).toBeLessThanOrEqual(1);
});
