// Re-run the three retained package proofs without RPC and bind the delivery.
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {resolve,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';

const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const exe=resolve(root,'target/debug/eplyx-lifecycle.exe');
const env={...process.env};delete env.SOLANA_RPC_URL;
const sha=file=>createHash('sha256').update(readFileSync(resolve(root,file))).digest('hex');
const run=(program,args)=>spawnSync(program,args,{cwd:root,env,encoding:'utf8',timeout:120000});
const cases=[
  ['demo-fixed-ratio','milestone8-healthy-worker',4,'Ready','Incomplete'],
  ['demo-underfunded','milestone8-underfunded',3,'Blocked','Blocked'],
  ['demo-second-asset','milestone8-second-asset',4,'Ready','Incomplete'],
];
const results=[];
for(const [pkg,dir,exit,candidate,stress] of cases){
  const packagePath=`examples/transitions/${pkg}`;
  const resultPath=`reports/${dir}`;
  const process=run(exe,['replay-package-preflight',packagePath,'--result',resultPath]);
  if(process.status!==exit)throw Error(`${pkg}: offline exit ${process.status}: ${process.stderr}`);
  const report=JSON.parse(readFileSync(resolve(root,resultPath,'report.json')));
  const bindings=JSON.parse(readFileSync(resolve(root,resultPath,'bindings.json')));
  const population=JSON.parse(readFileSync(resolve(root,resultPath,'population.capture.json')));
  const manifest=JSON.parse(readFileSync(resolve(root,packagePath,'eplyx.json')));
  if(report.transition_package_sha256!==bindings.transition_package_sha256
    ||report.candidate_program_sha256!==manifest.candidateProgram.sha256
    ||report.candidate_plan_readiness!==candidate
    ||report.conversion_stress_readiness.status!==stress
    ||report.official_transition!=='NotTested'
    ||report.funds_moved!==false
    ||report.exact_selected_cases.length!==10
    ||report.current_capture_timestamp!==population.completed_at)
    throw Error(`${pkg}: report scope or provenance mismatch`);
  results.push({package:pkg,result_directory:resultPath,
    transition_package_sha256:report.transition_package_sha256,
    candidate_program_sha256:report.candidate_program_sha256,
    source_mint:report.source_mint,replacement_mint:report.replacement_mint,
    current_capture_timestamp:report.current_capture_timestamp,
    selected_cases:report.exact_selected_cases.length,
    outcome_counts:Object.fromEntries([...new Set(report.stress_results.map(r=>r.status))].map(status=>
      [status,report.stress_results.filter(r=>r.status===status).length])),
    candidate_plan_readiness:candidate,conversion_stress_readiness:stress,
    population_rollout_readiness:report.population_rollout_readiness.status,
    official_transition:'NotTested',funds_moved:false,
    offline_exit:exit,report_sha256:sha(`${resultPath}/report.json`)});
}
if(results[0].source_mint===results[2].source_mint)throw Error('second asset leaked first source mint');
const mutations=JSON.parse(readFileSync(resolve(root,'reports/milestone8-mutations.json')));
if(!mutations.all_killed||mutations.mutations.length!==6)throw Error('mutation campaign incomplete');
const historical=run('git',['diff','--name-only','HEAD','--','evidence','probes','snapshots','assets','fixtures','policies','scenarios','reports']);
const oldChanges=historical.stdout.trim().split(/\r?\n/).filter(file=>file&&!file.startsWith('reports/milestone8-'));
if(historical.status!==0||oldChanges.length)throw Error('tracked historical artifacts changed');
const output={schema_version:1,kind:'milestone8-validation',
  engine_binary_sha256:sha('target/debug/eplyx-lifecycle.exe'),
  source_sha256:Object.fromEntries([
    'engine/src/conversion/package.rs','engine/src/conversion/package_preflight.rs',
    'engine/src/conversion/mod.rs','engine/src/main.rs','engine/src/stress/select.rs',
    'engine/src/stress/execute.rs'].map(file=>[file,sha(file)])),
  cases:results,mutation_campaign_sha256:sha('reports/milestone8-mutations.json'),
  mutations_killed:6,historical_tracked_artifacts_unchanged:true,
  rpc_environment_removed_for_replay:true};
writeFileSync(resolve(root,'reports/milestone8-validation.json'),JSON.stringify(output,null,2)+'\n');
process.stdout.write(JSON.stringify({cases:results.length,mutations_killed:6,
  historical_tracked_artifacts_unchanged:true,engine_binary_sha256:output.engine_binary_sha256})+'\n');
