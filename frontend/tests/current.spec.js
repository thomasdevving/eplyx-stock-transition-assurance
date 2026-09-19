import { test, expect } from '@playwright/test';
import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';

test('failed new submission cannot display a prior result as its outcome',async({page})=>{
 // UI failure injection only; this is not a live integration test.
 const fresh={id:'11111111-1111-4111-8111-111111111111',status:'Completed',selection:{asset:'mint',review:'current',stage:null,check:null,cluster:'solana-mainnet',mint:'So11111111111111111111111111111111111111112',catalogue_version:null,sample_accounts:false},result:{kind:'current-inspection',mint:{is_initialized:true,decimal_supply:'1',extensions:[]},accounts:[],acquisition:{completed_at:'2026-09-19T00:00:00Z'},discovery:{status:'Unavailable',gaps:[]}}};
 await page.addInitScript(job=>localStorage.setItem('eplyx-analysis',JSON.stringify({selection:job.selection,current:{id:job.id,status:'Completed'},previous:{id:job.id,status:'Completed'}})),fresh);
 await page.route(`**/api/runs/${fresh.id}`,route=>route.fulfill({json:fresh}));
 await page.route('**/api/runs',route=>{expect(Object.keys(route.request().postDataJSON()).sort()).toEqual(['request_key','selection']);return route.fulfill({status:503,json:{error:{message:'Current acquisition unavailable. No saved result substituted.'}}});});
 await page.goto('/analysis#analysis');await expect(page.locator('#analysis-results')).toContainText('Saved run');
 await page.locator('#run-analysis').click();await expect(page.locator('#analysis-error')).toContainText('unavailable');
 await expect(page.locator('#analysis-results')).toContainText('Previous capture');
 await expect(page.locator('#analysis-results')).not.toContainText('Newly fetched observations');
});
