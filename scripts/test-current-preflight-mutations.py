#!/usr/bin/env python3
"""Executable prospective faults. Only named assertion failures count; restore every byte."""
import pathlib,subprocess,hashlib,json
ROOT=pathlib.Path(__file__).resolve().parents[1]
OUT=ROOT/'reports/milestone5-validation/mutations'
P='engine/src/preflight.rs'
R='engine/src/resolution/current.rs'
anchor='    let paths = resolution::current::resolve(&scope, &verified)?;'
FAULTS=[
 ('historical_phase7_proof',[(P,anchor,anchor+'''\n    let mut paths=paths;
    let historical:serde_json::Value=serde_json::from_str(include_str!("../../reports/spacex-lifecycle-path-resolution.json"))?;
    paths.rows[3].status=serde_json::from_value(historical["paths"][3]["status"].clone())?;''')], 'saved_phase7_proof_cannot_enter_missing_current_checks','historical proof cannot satisfy current path'),
 ('mobility_as_conversion',[(P,anchor,anchor+'\n    let mut paths=paths; paths.rows[0].status=paths.rows[3].status;')], 'current_mobility_never_proves_official_conversion','mobility must not prove official conversion'),
 ('issuer_escalation',[(P,'"source":"UserProposed"','"source":"IssuerAsserted"')], 'proposed_provenance_never_becomes_issuer_assertion','user proposal cannot become issuer assertion'),
 ('future_balance',[(P,'"balance_raw":state.raw_balance,"account_sha256":digest(&raw)?,"technical_state":"Unchanged"','"balance_raw":if active {"0"}else{state.raw_balance.as_str()},"account_sha256":digest(&raw)?,"technical_state":"Unchanged"')], 'unchanged_current_bytes_at_all_proposed_times','proposed time cannot change current balance'),
 ('refresh_inheritance',[(P,'check.capture.as_bytes(),\n            run,','check.capture.as_bytes(),\n            serde_json::from_str::<serde_json::Value>(&check.capture)?["run_id"].as_str().unwrap(),'),(R,'v["run_id"] == run && v["wallet_capture_sha256"] == wallet_hash','!run.is_empty() && v["wallet_capture_sha256"] == wallet_hash')], 'refreshed_run_and_other_entity_reject_existing_proof','refresh cannot inherit earlier run proof'),
 ('scenario_binding',[(P,'bundle.scenario_sha256 == scenario_hash\n            && digest(&bundle.scenario)? == scenario_hash','!scenario_hash.is_empty()')], 'scenario_digest_cannot_reuse_readiness','different scenario digest must reject readiness'),
 ('entity_to_population',[(P,'"population_readiness":null','"population_readiness":{"status":"Ready"}')], 'entity_ready_never_becomes_population_ready','selected entity must not grant population readiness'),
]
def main():
 if OUT.exists():raise SystemExit('Refusing to overwrite mutation evidence')
 originals={p:(ROOT/p).read_bytes() for _,edits,*_ in FAULTS for p,_,_ in edits}
 for _,edits,*_ in FAULTS:
  for p,before,_ in edits:
   if originals[p].decode().count(before)!=1:raise RuntimeError('Nonunique mutation anchor: '+before)
 OUT.mkdir();rows=[]
 try:
  for name,edits,test,assertion in FAULTS:
   for p,before,after in edits:(ROOT/p).write_text((ROOT/p).read_text().replace(before,after))
   cmd=['cargo','test','--locked','-p','eplyx-lifecycle-impact','--test','current_preflight',test,'--','--exact']
   try:
    run=subprocess.run(cmd,cwd=ROOT,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    log=run.stdout;(OUT/f'{name}.txt').write_text(log)
    killed=run.returncode==101 and 'test result: FAILED' in log and test+' ... FAILED' in log and assertion in log
    rows.append(dict(mutation=name,command=cmd,named_assertion=assertion,killed_by_assertion=killed,exit_code=run.returncode,edits=[dict(file=p,before=b,after=a,mutated_sha256=hashlib.sha256((ROOT/p).read_bytes()).hexdigest()) for p,b,a in edits],log_sha256=hashlib.sha256(log.encode()).hexdigest()))
    print(name+(': killed' if killed else ': NOT killed'),flush=True)
   finally:
    for p,_,_ in edits:(ROOT/p).write_bytes(originals[p])
   if not killed:print(log[-4000:]);break
 finally:
  for p,data in originals.items():(ROOT/p).write_bytes(data)
 report=dict(injected=len(rows),killed=sum(r['killed_by_assertion'] for r in rows),sources_restored=all((ROOT/p).read_bytes()==data for p,data in originals.items()),source_sha256={p:hashlib.sha256(data).hexdigest() for p,data in originals.items()},results=rows)
 (OUT/'results.json').write_text(json.dumps(report,indent=2)+'\n')
 if report['killed']!=len(FAULTS):raise SystemExit(1)
if __name__=='__main__':main()
