import {test,expect} from '@playwright/test';
import {createServer} from 'node:http';
import {spawn,execFileSync} from 'node:child_process';
import {once} from 'node:events';
import {mkdir,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const dir=process.env.EPLYX_REPORT_DIR||'reports/milestone3-validation';
const custom='So11111111111111111111111111111111111111112';
const owner='8qbHbw2BbbTHBW1sbeqakYXVKRQM8Ne7pLK7m6CVfeR'; // [2;32], public fixture authority
const zero='CktRuQ2mttgRGkXJtyksdKHjUdc2C4TgDzyB98oEzy8'; // [3;32]
const legacy='TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';
const token2022='TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb';
const decode58=s=>{let n=0n;for(const c of s)n=n*58n+BigInt('123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'.indexOf(c));return Buffer.from(n.toString(16).padStart(64,'0'),'hex');};
const raw=(b,program)=>({owner:program,executable:false,space:b.length,data:[b.toString('base64'),'base64']});
async function submit(page){const accepted=page.waitForResponse(r=>r.url().endsWith('/api/runs')&&r.request().method()==='POST');await page.locator('#run-analysis').click();const response=await accepted;expect(response.status()).toBe(202);const q=await response.json();await expect(page.locator(`.analysis-output[data-run="${q.id}"]`)).toBeVisible({timeout:100000});return(await page.request.get(new URL(`/api/runs/${q.id}`,page.url()).href)).json();}
async function retain(page,job,label){const base=new URL(page.url()).origin;const capture=await(await page.request.get(`${base}/api/runs/${job.id}/capture`)).body();expect(createHash('sha256').update(capture).digest('hex')).toBe(job.capture_sha256);const path=`${dir}/${label}.capture.json`;await writeFile(path,capture);const replay=execFileSync('target/debug/eplyx-lifecycle',['replay-current','--input',path]);expect(createHash('sha256').update(replay).digest('hex')).toBe(job.canonical_sha256);await writeFile(`${dir}/${label}.result.json`,replay);return JSON.parse(capture);}

test('deterministic wallet browser/service/engine: multiple accounts, precision, focus, zero, errors, refresh and isolation',async({page,browser})=>{
 test.setTimeout(180000);await mkdir(dir,{recursive:true});let lookup=0,fail=false,partial=false;const calls=[];
 const rpc=createServer(async(req,res)=>{let body='';for await(const c of req)body+=c;const {method,params}=JSON.parse(body);calls.push({method,params});let result;
 if(method==='getGenesisHash')result='5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d';
 else if(method==='getAccountInfo'){
  if(params[0]===owner||params[0]===zero)result={context:{slot:11},value:null};
  else if(params[0]==='11111111111111111111111111111111')result={context:{slot:11},value:{executable:true,owner:'11111111111111111111111111111111'}};
  else{const b=Buffer.alloc(82);b[44]=9;b[45]=1;result={context:{slot:10},value:raw(b,params[0]===custom?legacy:token2022)};}
 }else if(method==='getTokenAccountsByOwner'){
  lookup++;if(fail){res.writeHead(429);res.end();return;}
  const program=params[1].mint===custom?legacy:token2022;
  result={context:{slot:12},value:params[0]===zero?[]:[owner,zero].map((pubkey,index)=>{const b=Buffer.alloc(165);decode58(params[1].mint).copy(b);decode58(partial&&index===0?zero:params[0]).copy(b,32);b.writeBigUInt64LE(18446744073709551615n,64);b[108]=2;return{pubkey,account:raw(b,program)};})};
 }else throw new Error(method);
 res.setHeader('Content-Type','application/json');res.end(JSON.stringify({jsonrpc:'2.0',id:1,result}));
 });rpc.listen(0,'127.0.0.1');await once(rpc,'listening');
 const port=4197,base=`http://127.0.0.1:${port}`;
 const service=spawn(process.execPath,['frontend/serve.mjs'],{env:{...process.env,PORT:String(port),SOLANA_RPC_URL:`http://127.0.0.1:${rpc.address().port}/private-provider-key?secret=hidden`},stdio:['ignore','pipe','pipe']});
 try{
  await expect.poll(async()=>{try{return(await fetch(`${base}/api/health`)).status;}catch{return 0;}}).toBe(200);
  await page.goto(`${base}/analysis#analysis`);await expect(page.locator('[name="asset"] option')).toHaveCount(9);const cat=await(await page.request.get(`${base}/api/catalogue`)).json();const firstAsset=cat.entries.find(e=>e.assertions[0].symbol==='SPACEX'),secondAsset=cat.entries.find(e=>e.assertions[0].symbol==='OPENAI');
  await page.locator('[name="scope"]').selectOption('wallet');await page.locator('[name="asset"]').selectOption(firstAsset.mint);await page.locator('[name="public_owner"]').fill(owner);await expect(page.locator('#sample-choice')).toBeHidden();
  const first=await submit(page);expect(first.result.wallet_observation.account_count).toBe(2);expect(first.result.wallet_observation.public_balance_total_raw).toBe('36893488147419103230');expect(lookup).toBe(1);await expect(page.locator('#analysis-results')).toContainText('36,893,488,147.41910323');
  await page.locator('[name="focused_account"]').first().check();const focused=await page.locator('[name="focused_account"]:checked').inputValue();const before=lookup;
  await page.getByRole('button',{name:'Technical',exact:true}).click();await page.getByRole('button',{name:'Overview',exact:true}).click();await expect(page.locator('[name="focused_account"]:checked')).toHaveValue(focused);expect(lookup).toBe(before);
  await page.reload();await expect(page.locator('#analysis-results')).toContainText('Saved run');await expect(page.locator('[name="focused_account"]:checked')).toHaveValue(focused);expect(lookup).toBe(before);
  const capture=await retain(page,first,'fixture-multiple');expect(JSON.stringify(capture)).not.toContain('private-provider-key');expect(JSON.stringify(capture)).not.toContain('secret=');expect(capture.observations[3].params[0]).toBe(owner);expect(capture.observations[3].params[1].mint).toBe(firstAsset.mint);
  await page.locator('#analysis-results').scrollIntoViewIfNeeded();await page.screenshot({path:`${dir}/wallet-desktop.png`});await page.setViewportSize({width:390,height:844});await page.locator('#analysis-results').scrollIntoViewIfNeeded();await page.screenshot({path:`${dir}/wallet-mobile.png`});expect(await page.evaluate(()=>document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
  const refreshed=await submit(page);expect(refreshed.id).not.toBe(first.id);expect(lookup).toBe(2);expect(refreshed.result.acquisition.started_at).not.toBe(first.result.acquisition.started_at);expect((await page.request.get(`${base}/api/runs/${first.id}`)).status()).toBe(200);
  const stranger=await browser.newContext();expect((await stranger.request.get(`${base}/api/runs/${first.id}`)).status()).toBe(404);await stranger.close();
  await page.locator('[name="asset"]').selectOption(secondAsset.mint);await expect(page.locator('#analysis-results')).toBeEmpty();const other=await submit(page);expect(other.result.wallet_observation.selected_mint).toBe(secondAsset.mint);expect(JSON.stringify(other.result)).not.toContain(firstAsset.mint);
  partial=true;const gap=await submit(page);expect(gap.result.wallet_observation.status).toBe('Partial');await expect(page.locator('#analysis-results')).toContainText('Returned account rows');await expect(page.locator('#analysis-results')).toContainText('1 matching accounts were decoded');expect(gap.result.wallet_observation.public_balance_total_raw).toBeNull();partial=false;
  await page.locator('[name="public_owner"]').fill(zero);const empty=await submit(page);expect(empty.result.wallet_observation.account_count).toBe(0);await expect(page.locator('#analysis-results')).toContainText('No direct token holdings found');await expect(page.locator('#analysis-results')).toContainText('Protocol positions were not checked');
  fail=true;const failure=await submit(page);expect(failure.result.wallet_observation.account_count).toBeNull();await expect(page.locator('#analysis-results')).toContainText('Current holdings could not be established');expect(failure.result.wallet_observation.status).toBe('Unavailable');fail=false;
  await page.locator('[name="source"]').selectOption('custom');await page.locator('[name="mint"]').fill(custom);await page.locator('[name="public_owner"]').fill(owner);const customJob=await submit(page);expect(customJob.result.mint.token_program).toBe(legacy);expect(customJob.result.selection.reference).toBeNull();
  for(const job of [first,refreshed,other,empty,failure,customJob]){expect(job.result.lifecycle_event).toBeNull();expect(job.result.readiness).toBeNull();expect(job.result.execution_performed).toBe(false);expect(job.result.paths.every(p=>p.status==='NotTested')).toBe(true);}
  await page.locator('[name="public_owner"]').fill('11111111111111111111111111111111');const program=await submit(page);expect(program.result.wallet_observation.status).toBe('InvalidOwner');await expect(page.locator('#analysis-results')).toContainText('executable program rather than a wallet owner');
  const count=calls.length;await page.locator('[name="public_owner"]').fill('invalid wallet');const invalid=page.waitForResponse(r=>r.url().endsWith('/api/runs')&&r.request().method()==='POST');await page.locator('#run-analysis').click();expect((await invalid).status()).toBe(400);await expect(page.locator('#analysis-error')).toContainText('valid Solana public wallet');expect(calls.length).toBe(count);expect(calls.some(c=>c.method==='getTokenLargestAccounts')).toBe(false);
  await writeFile(`${dir}/fixture-browser-runs.json`,JSON.stringify([first,refreshed,other,empty,failure,customJob,program],null,2));
 }finally{service.kill();rpc.close();await once(service,'exit');}
});

test('live wallet browser acquisition with refresh, second asset, custom mint, zero target and offline replay',async({page})=>{
 test.skip(process.env.EPLYX_WALLET_LIVE!=='1','Opt-in bounded real mainnet acceptance');test.setTimeout(480000);await mkdir(dir,{recursive:true});
 await page.goto('/analysis#analysis');await expect(page.locator('[name="asset"] option')).toHaveCount(9);const cat=await(await page.request.get('/api/catalogue')).json();const spacex=cat.entries.find(e=>e.assertions[0].symbol==='SPACEX'),openai=cat.entries.find(e=>e.assertions[0].symbol==='OPENAI');
 // A known public authority is a query target only. No historical balance is used.
 const known=process.env.EPLYX_ACCEPTANCE_OWNER;expect(known,'Set the public owner query target').toBeTruthy();
 await page.locator('[name="scope"]').selectOption('wallet');await page.locator('[name="asset"]').selectOption(spacex.mint);await page.locator('[name="public_owner"]').fill(known);
 const jobs=[];for(const label of ['known','refresh','zero-target','second-asset','custom']){
  if(label==='zero-target')await page.locator('[name="public_owner"]').fill(zero);
  if(label==='second-asset'){await page.locator('[name="asset"]').selectOption(openai.mint);await page.locator('[name="public_owner"]').fill(known);}
  if(label==='custom'){await page.locator('[name="source"]').selectOption('custom');await page.locator('[name="mint"]').fill(custom);}
  const job=await submit(page);await retain(page,job,`live-${label}`);jobs.push({label,...job});await writeFile(`${dir}/live-browser-runs.json`,JSON.stringify(jobs,null,2)+'\n');expect(job.result.lifecycle_event).toBeNull();expect(job.result.readiness).toBeNull();expect(job.result.execution_performed).toBe(false);
  if(label==='known'){const id=job.id;await page.getByRole('button',{name:'Technical',exact:true}).click();await page.getByRole('button',{name:'Overview',exact:true}).click();await expect(page.locator('.wallet-output')).toHaveAttribute('data-run',id);await page.reload();await expect(page.locator('.wallet-output')).toHaveAttribute('data-run',id);}
 }
 expect(jobs[0].id).not.toBe(jobs[1].id);expect(jobs[0].result.acquisition.started_at).not.toBe(jobs[1].result.acquisition.started_at);
 await page.locator('#analysis-results').scrollIntoViewIfNeeded();await page.screenshot({path:`${dir}/live-wallet-desktop.png`});
 await page.locator('[name="public_owner"]').fill('invalid owner');const invalid=page.waitForResponse(r=>r.url().endsWith('/api/runs')&&r.request().method()==='POST');await page.locator('#run-analysis').click();expect((await invalid).status()).toBe(400);await expect(page.locator('#analysis-error')).toContainText('valid Solana public wallet');
 await page.goto('/evidence');await expect(page.locator('.consumer-only .report-top')).toContainText('Saved published result');
});
