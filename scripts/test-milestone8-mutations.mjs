// Six fail-closed package/evidence mutations against the retained healthy run.
// No RPC is exposed to the child process and no source files are modified.
import {cpSync, mkdtempSync, mkdirSync, readFileSync, writeFileSync, readdirSync, linkSync, rmSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {resolve, join, sep} from 'node:path';
import {fileURLToPath} from 'node:url';
import {dirname} from 'node:path';

const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const target=resolve(root,'target');
const exe=resolve(target,'debug','eplyx-lifecycle.exe');
const source=resolve(root,'examples/transitions/demo-fixed-ratio');
const result=resolve(root,'reports/milestone8-healthy-worker');
const env={...process.env};delete env.SOLANA_RPC_URL;
const run=(...args)=>spawnSync(exe,args,{cwd:root,env,encoding:'utf8',timeout:120000});
function temp(label,body){
  const dir=mkdtempSync(join(target,`m8-${label}-`));
  if(!dir.startsWith(target+sep))throw Error('temporary directory escaped target');
  try{return body(dir);}finally{rmSync(dir,{recursive:true,force:true});}
}
function check(label,process,expected){
  if(process.status!==2||!process.stderr.includes(expected))throw Error(`${label}: exit ${process.status}: ${process.stderr}`);
  return {mutation:label,killed_by:expected,application_exit:process.status};
}
function packageMutation(label,mutate,command='validate-transition-package',expected='invalid package manifest'){
  return temp(label,dir=>{
    const copy=join(dir,'package');cpSync(source,copy,{recursive:true});
    const file=join(copy,'eplyx.json'),manifest=JSON.parse(readFileSync(file,'utf8'));
    mutate(manifest);writeFileSync(file,JSON.stringify(manifest));
    const args=command==='replay-package-preflight'?[command,copy,'--result',result]:[command,copy];
    return check(label,run(...args),expected);
  });
}
const baseline=run('replay-package-preflight',source,'--result',result);
if(baseline.status!==4||!baseline.stdout.includes('"declared_preflight_status":"Incomplete"'))
  throw Error(`baseline failed: ${baseline.status}: ${baseline.stderr}`);
const mutations=[
  packageMutation('ignore-program-hash',m=>m.candidateProgram.sha256='0'.repeat(64),
    'validate-transition-package','candidate program SHA-256 mismatch'),
  packageMutation('path-escape',m=>m.candidateProgram.artifact='../program.so',
    'validate-transition-package','package path must be a relative member without traversal'),
  packageMutation('trust-declared-success',m=>m.conversionStatus='Proven'),
  packageMutation('promote-official',m=>m.officialTransition='Proven'),
  packageMutation('reuse-after-terms-change',m=>m.terms.numerator='3',
    'replay-package-preflight','package identity changed'),
  temp('wrong-stress-binding',dir=>{
    const copy=join(dir,'result');
    mkdirSync(copy);
    for(const name of readdirSync(result)){
      const from=join(result,name),to=join(copy,name);
      if(name==='bindings.json'){
        const b=JSON.parse(readFileSync(from,'utf8'));b.transition_package_sha256='0'.repeat(64);
        writeFileSync(to,JSON.stringify(b));
      }else linkSync(from,to);
    }
    return check('wrong-stress-package-digest',run('replay-package-preflight',source,'--result',copy),'package identity changed');
  }),
];
const output={schema_version:1,baseline_exit:baseline.status,mutations,all_killed:mutations.length===6};
writeFileSync(resolve(root,'reports/milestone8-mutations.json'),JSON.stringify(output,null,2)+'\n');
process.stdout.write(JSON.stringify(output)+'\n');
