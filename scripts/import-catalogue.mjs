import { importCatalogue } from '../frontend/catalogue.mjs';
import { executeEngine } from '../frontend/analysis-service.mjs';
const root=new URL('../',import.meta.url).pathname;
const result=await importCatalogue(root,{validateMints:mints=>executeEngine(root+'target/debug/eplyx-lifecycle',['validate-address',...mints.flatMap(m=>['--mint',m])],{cwd:root,timeoutMs:5000})});
console.log(JSON.stringify(result,null,2));if(result.status!=='Completed')process.exitCode=1;
