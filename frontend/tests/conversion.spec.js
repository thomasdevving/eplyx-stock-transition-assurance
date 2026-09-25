import {test,expect} from '@playwright/test';
import {engineExecutable} from '../analysis-service.mjs';
import {mkdir,writeFile,readFile} from 'node:fs/promises';
import {execFileSync,spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import {once} from 'node:events';
const dir=process.env.EPLYX_REPORT_DIR||'reports/milestone6-validation';
const hash=b=>createHash('sha256').update(b).digest('hex');
const spacex={symbol:'SPACEX',owner:'2wCvQzHiDHAHTvzwPeof9H3uEzq8Bzvg38DFbvZMGkuj',source:'741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs',recipient:'123aUGPWa93jiga876U3rLdBP86JNFSoz9tSQWCAskMc'};
const openai={symbol:'OPENAI',owner:'6GJbPKBtovsrMEEMcic5KMi5tswh9qSyT5ZYLMqEwNgt',source:'F3L4drnkirxAnqeFVpJiRgTUeNMdBFZnV4bwZKuyrSpy'};
const USDC='EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v';
const OPENAI_MINT='PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF';
const SPACEX_MINT='PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh';
async function get(page,path){return (await page.request.get(new URL(path,page.url()).href)).json();}
const local=d=>new Date(d.getTime()-d.getTimezoneOffset()*60000).toISOString().slice(0,16);

async function fetchWallet(page,asset){
 const catalogue=await get(page,'/api/catalogue');
 await page.locator('[name="scope"]').selectOption('wallet');
 await page.locator('[name="asset"]').selectOption(catalogue.entries.find(e=>e.assertions[0].symbol===asset.symbol).mint);
 await page.locator('[name="public_owner"]').fill(asset.owner);
 const response=page.waitForResponse(r=>r.url().endsWith('/api/runs')&&r.request().method()==='POST');
 await page.locator('#run-analysis').click();
 const accepted=await response;expect(accepted.status()).toBe(202);
 const submitted=await accepted.json();
 await expect(page.locator(`.wallet-output[data-run="${submitted.id}"]`)).toBeVisible({timeout:100000});
 await page.locator(`[name="focused_account"][value="${asset.source}"]`).check();
 return get(page,`/api/runs/${submitted.id}`);
}
/** Open the proposed replacement transition and declare the replacement asset. */
async function propose(page,successor){
 await page.locator('[name="proposed_change"]').selectOption('replacement');
 await page.locator('[name="successor_mint"]').fill(successor);
 await page.locator('[name="successor_mint"]').dispatchEvent('input');
 const future=new Date(Date.now()+30*86400000);
 await page.locator('[name="effective_at"]').fill(local(future));
 return future;
}
async function conversion(page,parent,{numerator='1',denominator='2',rounding='Floor',fee='0',reserve='1000000000000',amount='0.000001'}={}){
 await expect(page.locator('.conversion-form')).toBeVisible();
 await page.locator('[name="conversion_mode"]').selectOption('candidate');
 await expect(page.locator('[name="conversion_numerator"]')).toBeVisible();
 await page.locator('[name="conversion_numerator"]').fill(numerator);
 await page.locator('[name="conversion_denominator"]').fill(denominator);
 await page.locator('[name="conversion_rounding"]').selectOption(rounding);
 await page.locator('[name="conversion_fee"]').fill(fee);
 await page.locator('[name="conversion_reserve"]').fill(reserve);
 await page.locator('[name="conversion_amount"]').selectOption('Custom');
 await page.locator('[name="conversion_amount_decimal"]').fill(amount);
 const response=page.waitForResponse(r=>r.url().endsWith(`/api/runs/${parent.id}/conversions`)&&r.request().method()==='POST');
 await page.getByRole('button',{name:'Test conversion plan',exact:true}).click();
 const accepted=await response;expect(accepted.status()).toBe(202);
 let job=await accepted.json();
 await expect.poll(async()=>{job=await get(page,`/api/runs/${job.id}`);return job.status;},{timeout:180000}).toBe('Completed');
 await expect(page.locator(`[data-conversion="${job.id}"]`)).toBeVisible({timeout:15000});
 return job;
}
async function preflight(page,parent,label,conversionId){
 if(conversionId)await page.locator(`.preflight-form [name="conversion_check"][value="${conversionId}"]`).check();
 const accepted=page.waitForResponse(r=>r.url().endsWith(`/api/runs/${parent.id}/preflights`)&&r.request().method()==='POST');
 await page.getByRole('button',{name:'Run pre-flight',exact:true}).click();
 const response=await accepted;expect(response.status()).toBe(202);
 let job=await response.json();
 await expect.poll(async()=>{job=await get(page,`/api/runs/${job.id}`);return job.status;},{timeout:180000}).toBe('Completed');
 await expect(page.locator(`[data-preflight="${job.id}"]`)).toBeVisible({timeout:15000});
 await writeFile(`${dir}/${label}.job.json`,JSON.stringify(job,null,2));
 return job;
}

test('operator-supplied candidate conversion through browser, service and VM',async({page})=>{
 test.setTimeout(600000);
 await mkdir(dir,{recursive:true});
 const revive=(_key,value,context)=>typeof value==='number'&&!Number.isSafeInteger(value)?JSON.rawJSON(context.source):value;
 const market=JSON.parse(await readFile('reports/milestone4-validation/live-market.capture.json','utf8'),revive);
 const transfer=JSON.parse(await readFile('reports/milestone4-validation/live-transfer.capture.json','utf8'),revive);
 const secondTransfer=JSON.parse(await readFile('reports/milestone4-validation/live-second-transfer.capture.json','utf8'),revive);
 const openaiMint=JSON.parse(await readFile('reports/milestone4-validation/openai.capture.json','utf8'),revive);
 const raw=new Map();
 for(const c of [transfer,secondTransfer,market]){const batch=c.observations.at(-1);batch.params[0].forEach((a,i)=>{if(batch.result.value[i])raw.set(a,batch.result.value[i]);});}
 raw.set(OPENAI_MINT,openaiMint.observations[1].result.value);
 raw.set('11111111111111111111111111111111',{owner:'NativeLoader1111111111111111111111111111111',executable:true,lamports:1,space:0,data:['','base64']});
 const slot=market.observations.at(-1).result.context.slot;
 const owners={[spacex.owner]:spacex.source,[openai.owner]:openai.source};
 const rpc=createServer(async(req,res)=>{
  let text='';for await(const b of req)text+=b;
  const {method,params}=JSON.parse(text);let result;
  if(method==='getGenesisHash')result=market.observations[0].result;
  else{
   let value;
   if(method==='getAccountInfo')value=raw.get(params[0])??null;
   else if(method==='getMultipleAccounts')value=params[0].map(a=>raw.get(a)??null);
   else if(method==='getProgramAccounts')value=market.observations[1].result.value;
   else if(method==='getTokenAccountsByOwner'){const source=owners[params[0]];value=source?[{pubkey:source,account:raw.get(source)}]:[];}
   else throw new Error(method);
   result={context:{slot},value};
  }
  res.setHeader('Content-Type','application/json');res.end(JSON.stringify({jsonrpc:'2.0',id:1,result}));
 });
 rpc.listen(0,'127.0.0.1');await once(rpc,'listening');
 const port=4198,base=`http://127.0.0.1:${port}`;
 const service=spawn(process.execPath,['frontend/serve.mjs'],{env:{...process.env,PORT:String(port),SOLANA_RPC_URL:`http://127.0.0.1:${rpc.address().port}/secret`},stdio:'pipe'});
 try{
  await expect.poll(async()=>{try{return(await fetch(`${base}/api/health`)).status;}catch{return 0;}}).toBe(200);
  await page.goto(`${base}/analysis#analysis`);
  await page.addStyleTag({content:'html{scroll-behavior:auto!important}'});

  // CASE A — a proposed replacement transition with no conversion plan at all.
  const parent=await fetchWallet(page,spacex);
  await propose(page,USDC);
  await expect(page.locator('.conversion-form')).toContainText('No conversion plan');
  const none=await preflight(page,parent,'case-a-no-plan');
  expect(none.result.replacement_conversion.status).toBe('NotTested');
  expect(none.result.paths.find(p=>p.path_type==='OfficialTransition').status).toBe('NotTested');
  expect(none.result.views[1].readiness.candidate_plan.status).toBe('Incomplete');
  expect(none.result.views[1].readiness.full_transition.status).toBe('Incomplete');
  await expect(page.locator(`[data-preflight="${none.id}"] [data-conversion-summary]`)).toContainText('No conversion mechanism was supplied');

  // CASE B — the operator supplies the candidate plan they intend to deploy.
  const proven=await conversion(page,parent,{});
  expect(proven.result.status).toBe('Proven');
  expect(proven.result.provenance).toBe('OperatorSupplied');
  expect(proven.result.official_transition).toBe('NotTested');
  expect(proven.result.issuer_binding_established).toBe(false);
  expect(proven.result.candidate_mechanism.deployed_on_mainnet).toBe(false);
  expect(proven.result.reconciliation.source_debited_raw).toBe('1000');
  expect(proven.result.reconciliation.replacement_released_raw).toBe('500');
  expect(proven.result.funds_moved).toBe(false);
  const rendered=page.locator(`[data-conversion="${proven.id}"]`);
  await expect(rendered).toContainText('Passed in VM simulation');
  await expect(rendered.locator('[data-conversion-input]')).toContainText('0.000001');
  await expect(rendered.locator('[data-official-conversion]')).toContainText('not independently verified');
  // Program identities and account metas stay out of the Overview reading.
  await expect(rendered).not.toContainText('BPFLoader2111111111111111111111111111111111',{useInnerText:true});
  await expect(rendered).not.toContainText('account_plan',{useInnerText:true});
  const ready=await preflight(page,parent,'case-b-candidate-plan',proven.id);
  expect(ready.result.replacement_conversion.status).toBe('Proven');
  expect(ready.result.replacement_conversion.official_transition).toBe('NotTested');
  expect(ready.result.views[1].readiness.candidate_plan.status).toBe('Ready');
  expect(ready.result.views[1].readiness.full_transition.status).toBe('Incomplete');
  expect(ready.result.paths.find(p=>p.path_type==='OfficialTransition').status).toBe('NotTested');
  expect(ready.result.population_readiness).toBeNull();
  const panel=page.locator(`[data-preflight="${ready.id}"]`);
  await expect(panel.locator('[data-readiness="candidate"]')).toContainText('Selected requirements met');
  await expect(panel.locator('[data-readiness="full"]')).toContainText('More evidence needed');
  await expect(panel.locator('[data-official-conversion]')).toContainText('not independently verified');
  // Overview and Technical agree on every status.
  await page.getByRole('button',{name:'Technical',exact:true}).click();
  await expect(panel).toContainText(ready.result.replacement_conversion.plan_sha256);
  await expect(panel.locator('[data-readiness="candidate"]')).toContainText('Selected requirements met');
  await page.getByRole('button',{name:'Overview',exact:true}).click();
  await page.locator(`[data-conversion="${proven.id}"]`).evaluate(e=>e.scrollIntoView({block:'start'}));
  await page.screenshot({path:`${dir}/candidate-conversion-desktop.png`});
  await page.setViewportSize({width:390,height:844});
  await page.locator(`[data-conversion="${proven.id}"]`).evaluate(e=>e.scrollIntoView({block:'start'}));
  await page.screenshot({path:`${dir}/candidate-conversion-mobile.png`});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
  await page.setViewportSize({width:1280,height:900});

  // Offline replay reruns the actual candidate program and reproduces the result.
  const bytes=await(await page.request.get(`${base}/api/runs/${proven.id}/capture`)).body();
  const capturePath=`${dir}/candidate-conversion.capture.json`;
  await writeFile(capturePath,bytes);
  expect(hash(bytes)).toBe(proven.capture_sha256);
  const replay=execFileSync(engineExecutable,['replay-conversion-check','--input',capturePath,'--run-id',parent.id,'--check-id',proven.id,'--wallet-sha256',parent.capture_sha256,'--capture-sha256',proven.capture_sha256,'--plan-sha256',proven.plan_sha256,'--program-sha256',proven.candidate_program_sha256],{env:{PATH:process.env.PATH},maxBuffer:32*1024*1024});
  expect(hash(replay)).toBe(proven.canonical_sha256);
  await writeFile(`${dir}/candidate-conversion.result.json`,replay);

  // CASE C — a valid but incompatible candidate configuration.
  const failed=await conversion(page,parent,{reserve:'1'});
  expect(failed.result.status).toBe('Failed');
  expect(failed.result.execution.success).toBe(false);
  expect(failed.result.funds_moved).toBe(false);
  expect(failed.result.reconciliation.replacement_released_raw).toBe('0');
  await expect(page.locator(`[data-conversion="${failed.id}"]`)).toContainText('Failed in VM simulation');
  await expect(page.locator(`[data-conversion="${failed.id}"]`)).not.toContainText('the real issuer transition will fail');
  await writeFile(`${dir}/case-c-failed.job.json`,JSON.stringify(failed,null,2));

  // CASE D — a refreshed wallet run inherits nothing.
  const refreshed=await fetchWallet(page,spacex);
  expect(refreshed.id).not.toBe(parent.id);
  await propose(page,USDC);
  await expect(page.locator('.conversion-form')).toBeVisible();
  await expect(page.locator(`[data-conversion="${proven.id}"]`)).toHaveCount(0);
  const after=await preflight(page,refreshed,'case-d-refresh');
  expect(after.result.replacement_conversion.status).toBe('NotTested');
  expect(after.result.views[1].readiness.candidate_plan.status).toBe('Incomplete');
  const rejected=await page.request.post(`${base}/api/runs/${refreshed.id}/preflights`,{data:{request_key:crypto.randomUUID(),request:{...after.preflight_request,conversion_check_id:proven.id}}});
  expect(rejected.status()).toBe(400);

  // CASE E — a second freshly observed asset, same generic adapter.
  const second=await fetchWallet(page,openai);
  await propose(page,SPACEX_MINT);
  const secondPlan=await conversion(page,second,{numerator:'3',denominator:'4'});
  expect(secondPlan.result.status).toBe('Proven');
  expect(secondPlan.result.mint).toBe(OPENAI_MINT);
  expect(secondPlan.result.replacement_mint).toBe(SPACEX_MINT);
  expect(secondPlan.result.reconciliation.replacement_released_raw).toBe('750');
  expect(secondPlan.result.official_transition).toBe('NotTested');
  expect(secondPlan.result.issuer_binding_established).toBe(false);
  await writeFile(`${dir}/case-e-second-asset.job.json`,JSON.stringify(secondPlan,null,2));

  // No page can submit code, accounts, endpoints, paths or a claimed status.
  for(const injected of [{program:'He4VZWmVgtbXVmHJ3tRmbLKuNDo9WG3tw5Gr36KupJUf'},{instruction:'01'},{accounts:[]},
   {transaction:'AQ=='},{path:'/etc/passwd'},{rpc:'https://example.invalid'},{status:'Proven'},{provenance:'IssuerVerified'},
   {source_mint:USDC},{adapter_id:'other'}]){
   const response=await page.request.post(`${base}/api/runs/${second.id}/conversions`,{data:{request_key:crypto.randomUUID(),
    request:{source:openai.source,replacement_mint:SPACEX_MINT,amount_mode:'Full',amount_decimal:null,ratio_numerator:1,
     ratio_denominator:1,rounding:'Floor',conversion_fee_bps:0,reserve_funded_replacement_raw:'1000',...injected}}});
   expect(response.status(),JSON.stringify(injected)).toBe(400);
  }
  const mechanism=await get(page,'/api/conversion-mechanism');
  expect(mechanism.deployed_on_mainnet).toBe(false);
  expect(mechanism.accepts_uploaded_programs).toBe(false);
  await writeFile(`${dir}/browser-acceptance.json`,JSON.stringify({parent,none,proven,ready,failed,refreshed,after,second,secondPlan,mechanism},null,2));
 }finally{service.kill();rpc.close();await once(service,'exit');}
});
