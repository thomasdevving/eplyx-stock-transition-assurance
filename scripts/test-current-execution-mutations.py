#!/usr/bin/env python3
"""Seven executable current-evidence faults. Compilation errors never count as kills."""
import hashlib,json,pathlib,subprocess,sys
ROOT=pathlib.Path(__file__).resolve().parents[1]
OUT=ROOT/'reports/milestone4-validation/mutations'
def replacements(*pairs):
 def edit(text):
  for before,after in pairs:
   if text.count(before)!=1:raise RuntimeError('mutation anchor is not unique: '+before)
   text=text.replace(before,after)
  return text
 return edit
FAULTS=[
 ('historical_proof','engine/src/lifecycle/current.rs',replacements(('result["wallet_observation"] = serde_json::to_value(wallet)?;','result["wallet_observation"] = serde_json::to_value(wallet)?; result["paths"][3]["status"] = "Proven".into();')),'current_proof_requires_actual_execution_and_replays_identically','historical proof cannot enter fresh observations','current_execution'),
 ('focused_account','engine/src/probe/current.rs',replacements(('.find(|a| a["address"] == request.source)','.find(|_| true)'),('.find(|a| a["pubkey"] == request.source)','.find(|_| true)')),'exact_account_amount_run_and_fixture_bindings','focused account mismatch must be rejected','current_execution'),
 ('run_binding','engine/src/probe/current.rs',replacements(('&& c.run_id == run_id','&& !run_id.is_empty()')),'exact_account_amount_run_and_fixture_bindings','cross-run or cross-fixture proof must be rejected','current_execution'),
 ('withheld_fee','engine/src/probe/token_transfer.rs',replacements(('&& tokens[1].withheld_fee_change_raw == fee.to_string()','&& !tokens[1].withheld_fee_change_raw.is_empty()')),'probe::token_transfer::tests::missing_withheld_fee_fails_exact_reconciliation','missing_destination_withheld_fee_must_reject_proof',None),
 ('known_signer','engine/src/probe/current.rs',replacements(('"signer_possession_known":false','"signer_possession_known":true')),'current_proof_requires_actual_execution_and_replays_identically','assumption never proves possession','current_execution'),
 ('promote_indeterminate','engine/src/probe/current.rs',replacements(('result["status"] = serde_json::to_value(status)?;','result["status"] = serde_json::to_value(if status == PathStatus::Indeterminate { PathStatus::Proven } else { status })?;')),'missing_capture_never_promotes_to_proven','missing capture must not promote','current_execution'),
 ('global_exitability','engine/src/probe/current_market.rs',replacements(('"global_exitability_established":false','"global_exitability_established":true')),'current_market_executes_and_reconciles_one_exact_route_offline','one route never proves global exitability','current_execution'),
]
def main():
 if OUT.exists():raise SystemExit('Refusing to overwrite mutation evidence')
 originals={path:(ROOT/path).read_bytes() for _,path,*_ in FAULTS}
 for _,path,edit,*_ in FAULTS:edit(originals[path].decode())
 OUT.mkdir();results=[]
 try:
  for name,path,edit,test,assertion,target in FAULTS:
   source=ROOT/path;mutated=edit(originals[path].decode());source.write_text(mutated)
   try:
    cmd=['cargo','test','--locked','-p','eplyx-lifecycle-impact']+(['--test',target] if target else ['--lib'])+[test,'--','--exact']
    run=subprocess.run(cmd,cwd=ROOT,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    log=run.stdout;(OUT/f'{name}.txt').write_text(log)
    killed=run.returncode==101 and 'test result: FAILED' in log and test+' ... FAILED' in log and assertion in log
    results.append(dict(mutation=name,source=path,test=test,named_assertion=assertion,killed_by_assertion=killed,exit_code=run.returncode,command=cmd,mutated_source_sha256=hashlib.sha256(mutated.encode()).hexdigest(),log_sha256=hashlib.sha256(log.encode()).hexdigest()))
    print(f'{name}: '+('killed by named assertion' if killed else 'NOT killed'),flush=True)
   finally:source.write_bytes(originals[path])
   if not killed:print(log[-6000:]);break
 finally:
  for path,original in originals.items():(ROOT/path).write_bytes(original)
 report=dict(injected=len(results),killed=sum(r['killed_by_assertion'] for r in results),sources_restored=all((ROOT/p).read_bytes()==v for p,v in originals.items()),source_sha256={p:hashlib.sha256(v).hexdigest() for p,v in originals.items()},results=results)
 (OUT/'results.json').write_text(json.dumps(report,indent=2)+'\n')
 return 0 if report['injected']==report['killed']==7 and report['sources_restored'] else 1
if __name__=='__main__':sys.exit(main())
