const reports=process.env.EPLYX_REPORT_DIR||'reports/milestone2-validation';
import {test,expect} from '@playwright/test';
import {engineExecutable} from '../analysis-service.mjs';
import {mkdir,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
const custom='So11111111111111111111111111111111111111112';

test('catalogue search, custom draft, unavailable source and original toggle preserve selection without submission',async({page})=>{
 let posts=0;page.on('request',r=>{if(r.method()==='POST'&&r.url().endsWith('/api/runs'))posts++;});
 await page.goto('/analysis#analysis');await expect(page.locator('[name="asset"] option')).toHaveCount(9);
 const catalogue=await(await page.request.get('/api/catalogue')).json();const other=catalogue.entries.find(e=>e.assertions[0].symbol==='OPENAI');
 await page.locator('[name="search"]').fill('OpenAI');await page.locator('[name="asset"]').selectOption(other.mint);
 await page.getByRole('button',{name:'Technical',exact:true}).click();await page.getByRole('button',{name:'Overview',exact:true}).click();await expect(page.locator('[name="asset"]')).toHaveValue(other.mint);expect(posts).toBe(0);
 await page.locator('[name="source"]').selectOption('custom');await page.locator('[name="mint"]').fill(custom);
 await page.getByRole('button',{name:'Technical',exact:true}).click();await page.getByRole('button',{name:'Overview',exact:true}).click();await expect(page.locator('[name="mint"]')).toHaveValue(custom);expect(posts).toBe(0);
 await page.reload();await expect(page.locator('[name="mint"]')).toHaveValue(custom);await expect(page.locator('[name="source"]')).toHaveValue('custom');
 await page.route('**/api/catalogue',route=>route.fulfill({json:{source_status:'Unavailable',entries:[]}}));await page.reload();await expect(page.locator('[name="mint"]')).toBeVisible();
 await page.setViewportSize({width:390,height:844});await page.locator('#analysis-form').scrollIntoViewIfNeeded();await page.screenshot({path:`${reports}/form-mobile.png`});expect(await page.evaluate(()=>document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
});

test('live generic browser-to-engine inspection: two catalogue stocks, custom mint, nonmint, invalid input, refresh and offline replay',async({page,browser})=>{
 test.skip(process.env.EPLYX_LIVE_TEST!=='1','Explicit bounded live mainnet acceptance');test.setTimeout(480000);
 await mkdir(reports,{recursive:true});await page.goto('/analysis#analysis');
 await expect(page.locator('[name="asset"] option')).toHaveCount(9);
 const catalogue=await(await page.request.get('/api/catalogue')).json();const spacex=catalogue.entries.find(e=>e.assertions[0].symbol==='SPACEX'),other=catalogue.entries.find(e=>e.assertions[0].symbol==='OPENAI');expect(other).toBeTruthy();
 let posts=0;page.on('request',r=>{if(r.method()==='POST'&&r.url().endsWith('/api/runs'))posts++;});
 const jobs=[];
 async function run(label,mint){
  const accepted=page.waitForResponse(r=>r.url().endsWith('/api/runs')&&r.request().method()==='POST');await page.locator('#run-analysis').click();const response=await accepted;expect(response.status()).toBe(202);const queued=await response.json();
  await page.getByRole('button',{name:'Technical',exact:true}).click();await page.getByRole('button',{name:'Overview',exact:true}).click();
  await expect(page.locator(`.analysis-output[data-run="${queued.id}"]`)).toBeVisible({timeout:100000});
  const job=await(await page.request.get(`/api/runs/${queued.id}`)).json();expect(job.status).toBe('Completed');expect(job.selection.mint).toBe(mint);expect(job.pinned_selection.mint).toBe(mint);expect(job.result.asset.mint).toBe(mint);expect(job.result.selection.mint).toBe(mint);
  expect(job.result.lifecycle_event).toBeNull();expect(job.result.readiness).toBeNull();expect(job.result.execution_performed).toBe(false);expect(job.result.authorization).toBe(false);expect(job.result.paths.every(p=>p.status==='NotTested')).toBe(true);expect(job.resolved.policy).toBeNull();expect(job.resolved.trusted_bundle).toBeNull();
  const capture=await(await page.request.get(`/api/runs/${job.id}/capture`)).body();expect(createHash('sha256').update(capture).digest('hex')).toBe(job.capture_sha256);const parsed=JSON.parse(capture);expect(parsed.observations[1].params[0]).toBe(mint);expect(parsed.asset.mint).toBe(mint);
  const path=`${reports}/${label}.capture.json`;await writeFile(path,capture);const replay=execFileSync(engineExecutable,['replay-current','--input',path],{encoding:'utf8'});expect(createHash('sha256').update(replay).digest('hex')).toBe(job.canonical_sha256);await writeFile(`${reports}/${label}.result.json`,replay);
  jobs.push({label,...job});await writeFile(`${reports}/live-browser-runs.json`,JSON.stringify(jobs,null,2)+'\n');return job;
 }
 await page.locator('[name="asset"]').selectOption(spacex.mint);await page.locator('[name="sample"]').selectOption('yes');const first=await run('spacex',spacex.mint);expect(first.result.inspection.status).toBe('Completed');expect(first.result.selection.reference.version).toBe(catalogue.version);
 await page.locator('[name="asset"]').selectOption(other.mint);await expect(page.locator('#analysis-results')).toBeEmpty();const second=await run('openai',other.mint);expect(second.result.inspection.status).toBe('Completed');expect(second.result.selection.reference.assertions[0].symbol).toBe('OPENAI');expect(JSON.stringify(second.result)).not.toContain(spacex.mint);
 const refreshed=await run('openai-refresh',other.mint);expect(refreshed.id).not.toBe(second.id);expect(refreshed.result.acquisition.started_at).not.toBe(second.result.acquisition.started_at);expect((await page.request.get(`/api/runs/${second.id}`)).status()).toBe(200);
 await page.locator('[name="source"]').selectOption('custom');await page.locator('[name="mint"]').fill(custom);await page.locator('[name="sample"]').selectOption('no');const unfamiliar=await run('custom',custom);expect(unfamiliar.result.inspection.status).toBe('Completed');expect(unfamiliar.result.selection.reference).toBeNull();expect(unfamiliar.result.discovery.status).toBe('NotRequested');await expect(page.locator('#analysis-results')).toContainText('association unconfirmed');
 await page.reload();await expect(page.locator('#analysis-results')).toContainText('Saved run');await expect(page.locator('[name="mint"]')).toHaveValue(custom);expect(posts).toBe(4);
 await page.locator('#analysis-results').scrollIntoViewIfNeeded();await page.screenshot({path:`${reports}/current-result-desktop.png`});
 await page.setViewportSize({width:390,height:844});await page.locator('#analysis-results').scrollIntoViewIfNeeded();await page.screenshot({path:`${reports}/current-result-mobile.png`});expect(await page.evaluate(()=>document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
 await page.locator('[name="mint"]').fill('11111111111111111111111111111111');const nonmint=await run('nonmint','11111111111111111111111111111111');expect(nonmint.result.mint).toBeNull();expect(nonmint.result.inspection.account_type).toBe('Program');await expect(page.locator('#analysis-results')).toContainText('not a token mint');
 const stranger=await browser.newContext();expect((await stranger.request.get(`/api/runs/${nonmint.id}`)).status()).toBe(404);await stranger.close();
 await page.locator('[name="mint"]').fill('invalid token');const invalid=page.waitForResponse(r=>r.url().endsWith('/api/runs')&&r.request().method()==='POST');await page.locator('#run-analysis').click();expect((await invalid).status()).toBe(400);await expect(page.locator('#analysis-error')).toContainText('valid Solana token');await expect(page.locator('#analysis-results')).toBeEmpty();expect(posts).toBe(6);
 for(const job of [first,second,refreshed]){expect(['Sampled','Unavailable']).toContain(job.result.discovery.status);if(job.result.discovery.status==='Unavailable'){expect(job.result.discovery.sample_count).toBeNull();expect(job.result.mint).not.toBeNull();}}
 await page.goto('/evidence');await expect(page.locator('.consumer-only .report-top')).toContainText('Saved published result');
});
