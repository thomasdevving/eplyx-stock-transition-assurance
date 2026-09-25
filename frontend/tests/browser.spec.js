import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { pinnedReports } from '../evidence.mjs';

test('desktop renders the sculpture, scoped stones, navigation and exact evidence', async ({page}) => {
 const errors=[];
 page.on('pageerror', e=>errors.push(e.message));
 await page.setViewportSize({width:1440,height:1000});
 await page.goto('/');
 await expect(page.getByRole('heading',{level:1})).toContainText('Rehearse the');
 await expect(page.locator('.logo-core')).toHaveClass(/logo-core--rendered/);
 await expect(page.locator('.orbit-body:visible')).toHaveCount(3);
 await page.getByRole('button',{name:'Technical',exact:true}).click();
 await expect(page.locator('.orbit-caption')).toContainText('Browser VM checks');
 await expect(page.locator('.orbit-body:visible')).toHaveCount(3);
 await page.keyboard.press('Tab');
 await page.locator('.orbit-body:visible').filter({hasText:'Evidence & workspace'}).focus();
 await expect(page.locator('.orbit-detail')).toHaveClass(/is-visible/);
 await expect(page.locator('.orbit-detail')).toContainText('OfficialTransition remains NotTested');
 await page.keyboard.press('Escape');
 expect(await page.locator('.hero').evaluate(el=>el.scrollLeft)).toBe(0);
 await page.evaluate(()=>window.scrollTo(0,0));
 await expect(page.locator('.hero__copy')).toHaveCSS('opacity','1');
 await page.screenshot({path:'test-results/desktop-hero.png'});
 await page.locator('#how').scrollIntoViewIfNeeded();
 await expect(page.locator('#how .section-heading')).toHaveCSS('opacity','1');
 await expect(page.locator('.system-flow:visible')).toHaveCSS('opacity','1');
 await page.screenshot({path:'test-results/desktop-method.png'});
 await page.locator('#result').scrollIntoViewIfNeeded();
 await page.locator('.result-window').scrollIntoViewIfNeeded();
 await expect(page.locator('.result-window')).toHaveCSS('opacity','1');
 await page.screenshot({path:'test-results/desktop-result.png'});
 await page.getByRole('link',{name:'Explore evidence'}).first().click();
 await expect(page).toHaveURL(/\/evidence$/);
 await expect(page.locator('.report-verdict')).toContainText('Incomplete');
 const direct=page.locator('#direct');
 await direct.getByText('Secondary Market Exit',{exact:true}).click();
 await expect(direct).toContainText('448018537');
 await expect(direct).toContainText('17,621');
 await expect(page.locator('#position')).toContainText('21,190,323');
 await expect(page.locator('#population')).toContainText('10,151');
 await page.screenshot({path:'test-results/desktop-evidence.png',fullPage:true});
 await page.goBack();
 await expect(page.locator('.orbit-body:visible')).toHaveCount(3);
 await page.goto('/evidence#notice');
 await expect(page.locator('#notice')).toBeInViewport();
 await page.goto('/#%5Bbad-selector');
 expect(errors).toEqual([]);
});

test('mobile navigation, page width and keyboard path details work', async ({page}) => {
 await page.setViewportSize({width:390,height:844});
 await page.goto('/');
 await page.getByRole('button',{name:'Open navigation'}).click();
 await expect(page.getByRole('navigation',{name:'Primary navigation'})).toBeVisible();
 await page.getByRole('link',{name:'Capabilities',exact:true}).click();
 await expect(page.getByRole('button',{name:'Open navigation'})).toHaveAttribute('aria-expanded','false');
 await page.goto('/');
 await expect(page.locator('.hero__copy')).toHaveCSS('opacity','1');
 await page.screenshot({path:'test-results/mobile-hero.png'});
 await page.locator('#how').scrollIntoViewIfNeeded();
 await expect(page.locator('#how .section-heading')).toHaveCSS('opacity','1');
 await page.screenshot({path:'test-results/mobile-method.png'});
 await page.screenshot({path:'test-results/mobile-home.png',fullPage:true});
 expect(await page.evaluate(()=>document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
 await page.getByRole('button',{name:'Technical',exact:true}).click();
 await page.goto('/evidence');
 await expect(page.getByRole('navigation',{name:'Evidence sections'})).toBeVisible();
 const summary=page.locator('#direct details').first().locator('summary');
 await summary.focus(); await page.keyboard.press('Enter');
 await expect(page.locator('#direct details').first()).toHaveAttribute('open','');
 expect(await page.evaluate(()=>document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
 await page.screenshot({path:'test-results/mobile-evidence.png',fullPage:true});
});

test('headline and controls respect visibility and motion preference', async ({page}) => {
 await page.addInitScript(() => sessionStorage.setItem('eplyx-stock-intro-seen', '1'));
 await page.emulateMedia({ reducedMotion:'no-preference' });
 await page.goto('/');
 const headline=page.locator('.transition-roll');
 await expect.poll(() => headline.evaluate(slot => slot.querySelector('.transition-roll__track').getAnimations()[0]?.playState)).toBe('running');
 const measurements=await headline.evaluate(slot => ({
  height:slot.getBoundingClientRect().height,
  tallest:Math.max(...[...slot.querySelectorAll('.transition-roll__term')].map(term => term.getBoundingClientRect().height)),
  slotAnimations:slot.getAnimations({subtree:false}).length,
  properties:slot.querySelector('.transition-roll__track').getAnimations()[0].effect.getKeyframes().flatMap(frame => Object.keys(frame)),
 }));
 expect(measurements.height).toBeCloseTo(measurements.tallest, 0);
 expect(measurements.slotAnimations).toBe(0);
 expect(measurements.properties).not.toContain('height');
 await expect(page.locator('.orbit-switch__thumb')).toHaveCSS('transition-duration','0.2s');
 await expect(page.locator('.hero__copy')).toHaveCSS('opacity','1');
 await page.screenshot({path:'test-results/motion-hero-desktop.png'});
 const scene=page.locator('.core-scene');
 const bounds=await scene.boundingBox();
 await page.mouse.move(bounds.x+bounds.width*.75,bounds.y+bounds.height*.65);
 await expect.poll(() => page.locator('.orbit-plane--front').evaluate(el => el.style.transform)).toMatch(/^translate\([1-9-]/);
 expect(await scene.evaluate(el => el.style.getPropertyValue('--px'))).toBe('');
 await page.locator('#how').scrollIntoViewIfNeeded();
 await expect.poll(() => headline.evaluate(slot => slot.querySelector('.transition-roll__track').getAnimations()[0]?.playState)).toBe('paused');
 await page.emulateMedia({ reducedMotion:'reduce' });
 await expect.poll(() => headline.evaluate(slot => slot.querySelector('.transition-roll__track').getAnimations().length)).toBe(0);
 await expect.poll(() => page.locator('.orbit-plane--front').evaluate(el => el.style.transform)).toBe('translate(0px, 0px)');
 await page.goto('/evidence');
 const sign=page.locator('#direct .expand-sign').first();
 await expect(sign).toHaveCSS('transition-property','color');
});

test('a fresh completed analysis fades once while restored results stay still', async ({page}) => {
 const id='11111111-1111-4111-8111-111111111111';
 const mint='So11111111111111111111111111111111111111112';
 let selection;
 await page.addInitScript(() => {
  window.resultRevealStarts=0;
  document.addEventListener('animationstart', event => {
   if(event.animationName==='analysisResultReveal')window.resultRevealStarts++;
  });
 });
 await page.route('**/api/catalogue', route => route.fulfill({json:{entries:[]}}));
 await page.route('**/api/runs', route => {
  selection=route.request().postDataJSON().selection;
  return route.fulfill({json:{id,status:'Queued',selection}});
 });
 await page.route(`**/api/runs/${id}`, route => route.fulfill({json:{
  id,status:'Completed',selection,
  result:{kind:'current-inspection',asset:{name:'Test token'},inspection:{message:'Current token information retrieved'},
   mint:{is_initialized:true,decimal_supply:'1',decimals:9},accounts:[],
   acquisition:{completed_at:'2026-09-19T00:00:00Z'},discovery:{status:'NotRequested',gaps:[]}},
 }}));
 await page.goto('/analysis#analysis');
 await page.locator('[name="source"]').selectOption('custom');
 await page.locator('[name="mint"]').fill(mint);
 await page.locator('#run-analysis').click();
 const output=page.locator(`.analysis-output[data-run="${id}"]`);
 await expect(output).toBeVisible();
 await expect.poll(() => page.evaluate(() => window.resultRevealStarts)).toBe(1);
 await page.locator('[name="mint"]').dispatchEvent('input');
 expect(await page.evaluate(() => window.resultRevealStarts)).toBe(1);
 await page.reload();
 await expect(output).toContainText('Saved run');
 expect(await page.evaluate(() => window.resultRevealStarts)).toBe(0);
});

test('CLI approval fades only after an approval action', async ({page}) => {
 await page.emulateMedia({reducedMotion:'reduce'});
 const assets={
  'cloud.js':'../cloud/cloud.js', 'cloud.css':'../cloud/cloud.css',
  'dashboard.css':'../dashboard/dashboard.css', 'ui.js':'../dashboard/ui.js',
  'mode.js':'../src/mode.js', 'brand.js':'../src/brand.js', 'format.js':'../src/format.js',
  'logo.svg':'../public/logo.svg',
 };
 await page.route('**/device?code=ABCD', async route => route.fulfill({
  body:await readFile(new URL('../cloud/index.html',import.meta.url)),contentType:'text/html',
 }));
 await page.route('**/assets/*', async route => {
  const name=new URL(route.request().url()).pathname.split('/').at(-1);
  const file=assets[name];
  if(!file)return route.abort();
  await route.fulfill({body:await readFile(new URL(file,import.meta.url)),contentType:name.endsWith('.css')?'text/css':name.endsWith('.svg')?'image/svg+xml':'text/javascript'});
 });
 let state='pending';
 await page.route('**/api/v1/me', route => route.fulfill({json:{user:{email:'reviewer@example.com'}}}));
 await page.route('**/api/v1/auth/device/lookup?code=ABCD', route => route.fulfill({json:{state,client:'test CLI',user_code:'ABCD',created_at:'2026-09-25T00:00:00Z'}}));
 await page.route('**/api/v1/auth/device/approve', route => {
  state=route.request().postDataJSON().approve?'approved':'denied';
  return route.fulfill({json:{state}});
 });
 await page.goto('/device?code=ABCD');
 await expect(page.locator('.device-code')).toHaveText('ABCD');
 await expect(page.locator('.device-outcome--approved')).toHaveCount(0);
 await page.getByRole('button',{name:'Approve',exact:true}).click();
 await expect(page.locator('.device-outcome--approved')).toHaveCSS('animation-duration','0.12s');
 await page.reload();
 await expect(page.locator('.device-outcome')).toContainText('approved');
 await expect(page.locator('.device-outcome--approved')).toHaveCount(0);
});

test('downloads are the exact pinned source bytes and missing assets fail clearly', async ({request}) => {
 for (const [file,hash] of Object.values(pinnedReports)) {
  const response=await request.get(`/public/evidence/${file.split('/').at(-1)}`);
  expect(response.ok()).toBeTruthy();
  const bytes=await response.body();
  expect(createHash('sha256').update(bytes).digest('hex')).toBe(hash);
  expect(bytes.equals(await readFile(file))).toBeTruthy();
 }
 expect((await request.get('/public/missing.js')).status()).toBe(404);
 expect((await request.get('/evidence')).status()).toBe(200);
 expect((await request.get('/.git/config')).status()).toBe(404);
});

test('WebGL failure retains the original vector mark', async ({page}) => {
 await page.addInitScript(()=>{
  const original=HTMLCanvasElement.prototype.getContext;
  HTMLCanvasElement.prototype.getContext=function(type,...args) {
   return type.includes('webgl') ? null : original.call(this,type,...args);
  };
 });
 await page.goto('/');
 await expect(page.locator('.sculpture-fallback')).toBeVisible();
 await expect(page.locator('.orbit-body:visible')).toHaveCount(3);
 await page.getByRole('button',{name:'Technical',exact:true}).click();
 await page.getByRole('link',{name:'Explore evidence'}).first().click();
 await expect(page.locator('.report-verdict')).toContainText('Incomplete');
});

test('tablet layout keeps navigation and content within the viewport', async ({page}) => {
 for (const width of [768,1024]) {
  await page.setViewportSize({width,height:1024});
  for (const path of ['/','/evidence']) {
   await page.goto(path);
   const header=page.locator('.site-header:visible');
   const brand=await header.locator('.logo-link').boundingBox();
   const nav=await header.locator('nav').boundingBox();
   if (nav) expect(nav.x).toBeGreaterThanOrEqual(brand.x+brand.width);
   expect(await page.evaluate(()=>document.documentElement.scrollWidth)).toBeLessThanOrEqual(width);
  }
 }
});

test('published rollout cases keep assertions, policy and local guard observations separate', async ({page}) => {
 await page.goto('/');
 await page.getByRole('button',{name:'Technical',exact:true}).click();
 await page.goto('/evidence#rollout');
 const panel=page.locator('#rollout');
 await expect(panel).toBeVisible();
 await expect(panel.locator('details[data-plan]')).toHaveCount(4);
 for (const [id,status,scope,marker] of [
  ['lp-complete-exit-claim','Incomplete','PopulationRolloutReadiness','Not created'],
  ['required-failed-route-claim','Blocked','PopulationRolloutReadiness','Not created'],
  ['transfer-official-conversion-claim','Incomplete','PopulationRolloutReadiness','Not created'],
  ['principal-removal-positive-control','Ready','DemoEntityReadiness','Created for this local demonstration only'],
 ]) {
  const row=panel.locator(`[data-plan="${id}"]`);
  await row.locator(':scope > summary').click();
  await expect(row.locator(':scope > .scope-table')).toContainText(status);
  await expect(row.locator(':scope > .scope-table')).toContainText(scope);
  await expect(row.locator(':scope > .scope-table')).toContainText(marker);
  if (id==='lp-complete-exit-claim') {
   await expect(row.locator('.rollout-assertions')).toContainText('Protocol Accrued Fees Remain');
   await expect(row.locator('.rollout-assertions')).toContainText('Supported');
   await expect(row.locator('.rollout-assertions')).toContainText('Contradicted');
  }
  if (id==='transfer-official-conversion-claim') await expect(row).toContainText('OfficialTransition remains NotTested');
  await row.locator(':scope > summary').click();
 }
 await expect(page.locator('.report-verdict')).toContainText('Incomplete');
 await panel.screenshot({path:'test-results/phase14-rollout-cases.png'});
 await panel.locator('[data-plan="lp-complete-exit-claim"]').locator(':scope > summary').click();
 await panel.screenshot({path:'test-results/phase14-lp-claim.png'});
 expect(await page.evaluate(()=>document.documentElement.scrollWidth)).toBeLessThanOrEqual(1280);
});
