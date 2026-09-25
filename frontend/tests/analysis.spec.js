const reports=process.env.EPLYX_REPORT_DIR||'reports/milestone2-validation';
import { test, expect } from '@playwright/test';
import {engineExecutable} from '../analysis-service.mjs';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
const choose=async(page,review,check)=>{await page.locator('[name="review"]').selectOption(review);if(check)await page.locator('[name="check"]').selectOption(check);};

test('new visitor uses overview, existing hero toggle preserves selections without requests, CTA focuses real form',async({page})=>{
 let submissions=0;page.on('request',r=>{if(r.method()==='POST'&&r.url().endsWith('/api/runs'))submissions++;});
 await page.goto('/');await expect(page.locator('html')).toHaveAttribute('data-mode','overview');
 await expect(page.getByRole('button',{name:'Overview',exact:true})).toHaveAttribute('aria-pressed','true');
 await page.getByRole('link',{name:/^Use Eplyx/}).click();await expect(page.locator('#start')).toBeInViewport();await page.getByRole('link',{name:/^Try it in the browser/}).click();await expect(page.locator('#analysis')).toBeInViewport();await expect(page).toHaveURL(/\/analysis#analysis$/);
 await choose(page,'assumption','complete-exit');await page.locator('[name="stage"]').selectOption('after_deadline');
 await page.getByRole('button',{name:'Technical',exact:true}).click();await expect(page.locator('html')).toHaveAttribute('data-mode','technical');
 await page.getByRole('button',{name:'Overview',exact:true}).click();await expect(page.locator('[name="stage"]')).toHaveValue('after_deadline');await expect(page.locator('[name="check"]')).toHaveValue('complete-exit');expect(submissions).toBe(0);
 await expect(page.locator('.technical-only:visible')).toHaveCount(0);
 await expect(page.locator('#analysis')).toContainText('actual signing access was not verified');
 const text=await page.locator('body').innerText();for(const raw of ['NotTested','OfficialTransition','SHA-256','447865621','solana-token-account:','Phase 14'])expect(text).not.toContain(raw);
 await page.setViewportSize({width:1440,height:1000});await page.evaluate(()=>window.scrollTo(0,0));await page.screenshot({path:`${reports}/after-desktop.png`});
 await page.setViewportSize({width:390,height:844});await page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));await page.screenshot({path:`${reports}/after-mobile.png`});
 expect(await page.evaluate(()=>document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
});

test('real frontend runs four engine cases with typed outcomes, reload and mode changes never authorize rollout',async({page,request})=>{
 test.setTimeout(2500000);
 const engineHash=createHash('sha256').update(await readFile(engineExecutable)).digest('hex');
 await page.goto('/analysis#analysis');
 const rows=[['overview',null,'Incomplete',0,'after_transition'],['assumption','complete-exit','Incomplete',4,'after_transition'],['assumption','required-sale','Blocked',3,'after_transition'],['assumption','principal-removal','Ready',0,'after_deadline']];
 let submissions=0;page.on('request',r=>{if(r.method()==='POST'&&r.url().endsWith('/api/runs'))submissions++;});
 const observed=[];
 for(const [review,check,status,exit,stage] of rows){
  await choose(page,review,check);await page.locator('[name="stage"]').selectOption(stage);
  const accepted=page.waitForResponse(r=>r.url().endsWith('/api/runs')&&r.request().method()==='POST');
  await page.locator('#run-analysis').click();const response=await accepted;expect(response.status()).toBe(202);const queued=await response.json();
  await expect(page.locator('#run-analysis')).toBeDisabled();
  await page.getByRole('button',{name:'Technical',exact:true}).click();await page.getByRole('button',{name:'Overview',exact:true}).click();
  await expect(page.locator('[name="review"]')).toHaveValue(review);
  if(review==='overview'){await page.reload();await expect(page.locator('[name="review"]')).toHaveValue(review);}
  await expect(page.locator(`.analysis-output[data-run="${queued.id}"]`)).toBeVisible({timeout:620000});
  const job=await (await page.request.get(`/api/runs/${queued.id}`)).json();
  expect(job.status).toBe('Completed');expect(job.cached).toBe(false);expect(job.engine_sha256).toBe(engineHash);expect(job.engine_exit_code).toBe(exit);
  const readiness=job.result.assessment?.readiness||job.result.readiness;expect(readiness.overall_status).toBe(status);
  expect(job.selection).toEqual({asset:'spacex',review,stage,check});expect(job.authorization).toBe(false);expect(job.guarded_action_invoked).toBe(false);
  const canonical=await (await page.request.get(`/api/runs/${queued.id}/artifact`)).body();expect(createHash('sha256').update(canonical).digest('hex')).toBe(job.canonical_sha256);
  const output=page.locator(`.analysis-output[data-run="${queued.id}"]`);await expect(output).toContainText('Analysis completed');await expect(output).toContainText('does not repeat a swap or withdrawal');
  await expect(page.locator('#analysis-error')).toBeHidden();await expect(page.locator('.technical-only:visible')).toHaveCount(0);
  if(check==='complete-exit'){expect(job.result.assessment.candidate_acceptance).toBe('NotAccepted');await expect(output).toContainText('126,543');await expect(output).toContainText('position account remained');}
  if(check==='required-sale')await expect(output).toContainText('original optional-route policy is unchanged');
  if(check==='principal-removal'){expect(readiness.evaluated_scope).toBe('DemoEntityReadiness');expect(job.result.assessment.readiness.rollout_readiness).toBeNull();await expect(output).toContainText('does not establish complete-position exit');}
  observed.push({id:job.id,selection:job.selection,status:job.status,engine_exit_code:job.engine_exit_code,readiness:status,scope:readiness.evaluated_scope,engine_sha256:job.engine_sha256,canonical_sha256:job.canonical_sha256});
 }
 expect(submissions).toBe(4);
 await page.setViewportSize({width:1440,height:1000});await page.locator('#analysis-results').evaluate(el=>el.scrollIntoView({block:'start'}));await page.screenshot({path:`${reports}/result-desktop.png`});
 await page.getByRole('button',{name:'Technical',exact:true}).click();await expect(page.locator('.analysis-technical')).toBeVisible();await page.locator('.analysis-technical summary').click();await expect(page.locator('.analysis-technical')).toContainText('DemoEntityReadiness');
 await page.getByRole('button',{name:'Overview',exact:true}).click();await page.setViewportSize({width:390,height:844});await page.locator('#analysis-results').evaluate(el=>el.scrollIntoView({block:'start'}));await page.screenshot({path:`${reports}/result-mobile.png`});expect(await page.evaluate(()=>document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
 const {writeFile}=await import('node:fs/promises');await writeFile(`${reports}/saved-browser-observations.json`,JSON.stringify(observed,null,2)+'\n');
});

test('unavailable backend and saved reports are labeled honestly',async({page})=>{
 await page.route('**/api/**',route=>route.abort());await page.goto('/analysis#analysis');await expect(page.locator('#analysis-error')).toContainText('unavailable');
 await page.goto('/evidence');await expect(page.locator('.consumer-only .report-top')).toContainText('Saved published result');await expect(page.locator('.technical-only:visible')).toHaveCount(0);
});

test('API rejects unsupported caller commands, evidence and oversized bodies',async({request})=>{
 for(const selection of [{asset:'OTHER',review:'overview',stage:'after_transition',check:null},{asset:'spacex',review:'overview',stage:'after_transition',check:null,command:'compare-scenarios'},{asset:'spacex',review:'assumption',stage:'after_transition',check:'/tmp/plan.json'}]){
  const r=await request.post('/api/runs',{data:{selection,request_key:crypto.randomUUID()}});expect(r.status()).toBe(400);
 }
 expect((await request.post('/api/runs',{data:{selection:{},request_key:crypto.randomUUID()},headers:{Origin:'https://elsewhere.invalid'}})).status()).toBe(403);
 expect((await request.post('/api/runs',{data:{padding:'a'.repeat(3000)}})).status()).toBe(413);
});
