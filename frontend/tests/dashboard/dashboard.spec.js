import { test, expect } from '@playwright/test';

const HEALTHY = 'run_20260924122823725_ce7b4d55310c';
const UNDERFUNDED = 'run_20260924123736483_ce7b4d55310c';
const RESERVE = 'cx_2ab4735232a3f228da1a29d1';
const SECOND = 'http://127.0.0.1:4186';
const EMPTY = 'http://127.0.0.1:4187';

const tile = (page, label) => page.locator('.tile').filter({ has:page.locator('.tile__label', { hasText:new RegExp(`^${label}$`, 'i') }) });

async function noHorizontalScroll(page) {
 const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
 expect(overflow).toBeLessThanOrEqual(1);
}

test.beforeEach(async ({ page }) => {
 page.on('pageerror', error => { throw error; });
 await page.addInitScript(() => { if (!sessionStorage.getItem('seeded')) { localStorage.setItem('eplyx-detail', 'overview'); sessionStorage.setItem('seeded', '1'); } });
});

test('overview answers the release question from the latest run', async ({ page }) => {
 await page.goto('/');
 const hero = page.locator('.hero-status');
 await expect(hero.locator('h1')).toHaveText('BLOCKED');
 await expect(hero).toContainText('Run #4');
 await expect(tile(page, 'Stress')).toContainText('0 / 10');
 await expect(tile(page, 'Counterexamples')).toContainText('35 observed · 2 derived');
 await expect(tile(page, 'Production')).toContainText('17,838');
 await expect(page.getByText('What changed recently')).toBeVisible();
 await expect(page.locator('.panel', { hasText:'What changed recently' })).toContainText('1,000,000,000,000 → 0');
 await expect(page.getByText('Current known limitations')).toBeVisible();
 await page.screenshot({ path:'test-results/dashboard-overview.png', fullPage:true });
});

test('runs list is ordered, filterable and feeds comparison', async ({ page }) => {
 await page.goto('/runs');
 const rows = page.locator('tbody tr[data-run]');
 await expect(rows).toHaveCount(4);
 await expect(rows.first()).toHaveAttribute('data-run', UNDERFUNDED);
 await page.getByRole('button', { name:'WARN', exact:true }).click();
 await expect(rows).toHaveCount(2);
 await page.getByRole('button', { name:'Has counterexamples' }).click();
 await expect(rows).toHaveCount(2);
 await page.getByRole('button', { name:'All', exact:true }).click();
 await page.getByLabel('Select run #3 for comparison').check();
 await page.getByLabel('Select run #4 for comparison').check();
 await page.getByRole('button', { name:'Compare selected' }).click();
 await expect(page).toHaveURL(new RegExp(`left=${HEALTHY}&right=${UNDERFUNDED}`));
 await page.goto('/runs');
 await page.screenshot({ path:'test-results/dashboard-runs.png' });
});

test('blocked run detail renders engine statuses without recalculating them', async ({ page }) => {
 await page.goto(`/runs/${UNDERFUNDED}`);
 await expect(page.locator('h1')).toHaveText('BLOCKED');
 for (const section of ['Release candidate', 'Current production state', 'Candidate execution', 'Stress test', 'Counterexample search', 'Invariants', 'Deployment gate', 'Evidence & replay']) {
  await expect(page.getByRole('heading', { name:section, exact:true })).toBeVisible();
 }
 const api = await (await page.request.get(`/api/runs/${UNDERFUNDED}`)).json();
 const shown = await page.locator('#invariants .inv .pill').allTextContents();
 expect(shown.map(s => s.replace(/^\W+/, ''))).toEqual(api.invariant_results.map(i => i.status));
 await expect(page.locator('#gate')).toContainText('Saved gate matches the engine gate');
 await expect(page.locator('#evidence')).toContainText(`eplyx show ${UNDERFUNDED}`);
 await page.screenshot({ path:'test-results/dashboard-blocked-run.png', fullPage:true });
});

test('the UI shows whatever the engine recorded, even an unusual gate', async ({ page }) => {
 await page.route(`**/api/runs/${UNDERFUNDED}`, async route => {
  const body = await (await route.fetch()).json();
  body.gate.outcome = 'Pass';
  body.gate_detail.saved.outcome = 'Pass';
  body.invariant_results[0].status = 'Indeterminate';
  await route.fulfill({ json:body });
 });
 await page.goto(`/runs/${UNDERFUNDED}`);
 await expect(page.locator('h1')).toHaveText('PASS');
 await expect(page.locator('#invariants .inv').first()).toContainText('Indeterminate');
});

test('counterexample detail tells the derived boundary story', async ({ page, context }) => {
 await context.grantPermissions(['clipboard-read', 'clipboard-write']);
 await page.goto(`/counterexamples/${RESERVE}`);
 await expect(page.locator('.kind')).toContainText('DERIVED COUNTEREXAMPLE');
 await expect(page.locator('.kind')).toContainText('Eplyx derived this failing variant from an observed production state.');
 await expect(page.locator('.ladder')).toContainText('747,775,404,621');
 await expect(page.locator('.ladder')).toContainText('747,775,404,622');
 await expect(page.locator('.ladder__row.is-derived')).toContainText('FAIL');
 await expect(page.getByText('fails below this replacement-reserve boundary')).toBeVisible();
 const command = page.locator('.command', { hasText:`eplyx reproduce ${RESERVE}` });
 await command.getByRole('button', { name:/Copy/ }).click();
 await expect(command.getByRole('button')).toHaveText('Copied');
 expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(`eplyx reproduce ${RESERVE}`);
 await expect(page.getByText('Reproduction history')).toBeVisible();
 await expect(page.locator('main')).toContainText('Reproduced 1 time');
 await page.screenshot({ path:'test-results/dashboard-counterexample.png', fullPage:true });
 await page.goto('/counterexamples');
 await page.getByRole('button', { name:/^Observed/ }).click();
 await page.locator('.cx-row').first().click();
 await expect(page.locator('.kind')).toContainText('OBSERVED COUNTEREXAMPLE');
 await expect(page.locator('.kind')).toContainText('This exact captured production state failed.');
});

test('comparison states differences and never claims a fix', async ({ page }) => {
 await page.goto(`/compare?left=${HEALTHY}&right=${UNDERFUNDED}`);
 const story = page.locator('.story');
 await expect(story).toContainText('Run #3 → Run #4');
 await expect(story.locator('.story__cell', { hasText:'Gate' })).toContainText('WARN');
 await expect(story.locator('.story__cell', { hasText:'Gate' })).toContainText('BLOCK');
 await expect(story.locator('.story__cell', { hasText:'Candidate binary' })).toContainText('unchanged');
 await expect(page.locator('.panel', { hasText:'Input differences' })).toContainText('Proposed replacement reserve');
 await expect(page.getByText('Search conditions are not directly equivalent.')).toBeVisible();
 const banner = page.locator('.equivalence--warn');
 await expect(banner).toContainText('Search domains differ — counterexample disappearance does not prove resolution.');
 expect((await banner.boundingBox()).y).toBeLessThan((await story.boundingBox()).y);
 await expect(story.locator('.badge--warn')).toHaveText('Search domains differ');
 await page.screenshot({ path:'test-results/dashboard-compare.png', fullPage:true });
 await page.goto(`/compare?left=${UNDERFUNDED}&right=${HEALTHY}`);
 // Wait for the rendered comparison before asserting what it must not say.
 await expect(page.locator('main')).toContainText('Re-executed without failure');
 await page.locator('main details').evaluateAll(all => all.forEach(d => { d.open = true; }));
 const text = await page.locator('main').innerText();
 expect(text).toContain('Search domains differ — counterexample disappearance does not prove resolution.');
 expect(text).not.toMatch(/Resolved \(equivalent search\)/);
 expect(text.toLowerCase()).not.toMatch(/\bfixed\b/);
});

test('technical mode reveals exact values and persists', async ({ page }) => {
 await page.goto(`/runs/${UNDERFUNDED}`);
 const full = 'ce7b4d55310ca188e652951dd3102eea1af9c2469dd389a12ef11d6319ccf36d';
 await expect(page.locator('#release code.tech-only', { hasText:full }).first()).toBeHidden();
 await page.getByRole('button', { name:'Technical' }).click();
 await expect(page.locator('#release code.tech-only', { hasText:full }).first()).toBeVisible();
 await expect(page.locator('#evidence')).toContainText('replay-package-preflight');
 await page.reload();
 await expect(page.getByRole('button', { name:'Technical' })).toHaveAttribute('aria-pressed', 'true');
});

test('secondary pages summarize without dumping captures', async ({ page }) => {
 for (const [path, text] of [['/production', 'Positive balances by recorded authority model'], ['/invariants', 'Conversion output matches'], ['/gate', 'Gate history'], ['/project', 'Offline reproductions']]) {
  await page.goto(path);
  await expect(page.locator('main')).toContainText(text);
 }
 await page.goto('/project');
 await expect(page.locator('main')).toContainText('missing');
 await expect(tile(page, 'Offline reproductions')).toContainText('2 reproduced · 0 failed');
 await expect(page.locator('main')).toContainText('4 not recorded');
 await page.goto('/');
 await expect(page.locator('.panel', { hasText:'What changed recently' }).locator('.badge--warn')).toHaveText('Search domains differ');
 await page.goto('/runs');
 await expect(page.locator('tbody tr[data-run]').first()).toContainText('Not recorded');
 const body = await page.content();
 expect(body).not.toContain('BROWSER-RPC-SECRET');
});

test('responsive layouts keep the workspace inside the viewport', async ({ page }) => {
 for (const [width, height, name] of [[390, 844, 'mobile'], [834, 1112, 'tablet']]) {
  await page.setViewportSize({ width, height });
  for (const path of ['/', `/runs/${UNDERFUNDED}`, `/compare?left=${HEALTHY}&right=${UNDERFUNDED}`, `/counterexamples/${RESERVE}`]) {
   await page.goto(path);
   await expect(page.locator('main h1, main h2').first()).toBeVisible();
   await noHorizontalScroll(page);
  }
  await page.goto('/');
  await expect(page.locator('.hero-status')).toBeVisible();
  await page.screenshot({ path:`test-results/dashboard-${name}.png`, fullPage:true });
 }
});

test('a second asset uses generic asset display', async ({ page }) => {
 await page.goto(`${SECOND}/`);
 await expect(page.locator('.hero-status h1')).toHaveText('PASS WITH WARNINGS');
 await expect(tile(page, 'Counterexamples')).toContainText('0');
 await page.goto(`${SECOND}/runs/run_20260924122418513_ce7b4d55310c`);
 await expect(page.locator('#release')).toContainText('Prew…rpgF');
 expect((await page.locator('body').innerText()).toUpperCase()).not.toContain('SPACEX');
});

test('empty project explains the next CLI step', async ({ page }) => {
 await page.goto(`${EMPTY}/`);
 await expect(page.locator('main')).toContainText('Run eplyx preflight to create your first assurance run.');
 await expect(page.locator('main')).toContainText('eplyx init');
 await page.goto(`${EMPTY}/counterexamples`);
 await expect(page.locator('main')).toContainText('No counterexamples have been found in recorded searches.');
 await expect(page.locator('main')).not.toContainText('No counterexamples exist');
});
