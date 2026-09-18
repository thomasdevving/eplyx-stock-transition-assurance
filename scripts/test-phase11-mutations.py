#!/usr/bin/env python3
"""Inject ten executable gate faults, catch named assertions, restore exact source."""
import argparse,hashlib,json,pathlib,subprocess,sys
ROOT=pathlib.Path(__file__).resolve().parents[1]
CORE=ROOT/'engine/src/readiness/mod.rs'
PREFIX='readiness::tests::'
faults=[
('satisfied |= accepted_path(f, c);','satisfied |= accepted_path(f, c) || f.status == PathStatus::NotTested;','not_tested_never_becomes_proven_or_failed','Treat NotTested as Proven'),
('&& f.path_type == c.path_type','&& (f.path_type == c.path_type || f.path_type == ExitPathType::Transfer)','transfer_never_satisfies_official_transition','Transfer substitutes for OfficialTransition'),
('&& f.path_type == c.path_type','&& (f.path_type == c.path_type || f.path_type == ExitPathType::Withdrawal)','withdrawal_never_satisfies_official_transition','Withdrawal substitutes for OfficialTransition'),
('f.principal_unwind == PathStatus::Proven\n                    && f.fee_collection == PathStatus::Proven\n                    && f.position_closure == PathStatus::Proven\n                    && !residual','f.principal_unwind==PathStatus::Proven','principal_is_not_complete_position_exit','Principal unwind substitutes for complete position exit'),
('if n==e.population.positive_balance_entities','if n>0','sampled_entity_never_proves_population','Inherit one sampled holder to population'),
('    f == c\n','    let mut other=f.clone(); other.entity_id=c.entity_id.clone(); other==*c\n','evidence_cannot_inherit_between_entities','Ignore evidence entity mismatch'),
('    f == c\n','    let mut other=f.clone(); other.venue=c.venue.clone(); other.context_id=c.context_id.clone(); other.captured_slot=c.captured_slot; other.clock=c.clock.clone(); other.fixture_sha256=c.fixture_sha256.clone(); other==*c\n','venue_context_bank_and_fixture_are_exact','Ignore venue/context/bank mismatch'),
('} else if v.contains(&FindingEffect::IncompleteEvidence) {\n        ReadinessStatus::Incomplete','} else if v.contains(&FindingEffect::IncompleteEvidence) {\n        ReadinessStatus::Ready','missing_evidence_is_incomplete_not_ready','Missing evidence becomes Ready'),
('} else if v.contains(&FindingEffect::IncompleteEvidence) {\n        ReadinessStatus::Incomplete','} else if v.contains(&FindingEffect::IncompleteEvidence) {\n        ReadinessStatus::Blocked','not_tested_never_becomes_proven_or_failed','Incomplete becomes Blocked without policy justification'),
('    if v.contains(&FindingEffect::Blocking) {','    if v.contains(&FindingEffect::Satisfied) { ReadinessStatus::Ready } else if v.contains(&FindingEffect::Blocking) {','unrelated_success_cannot_erase_required_failure','Unrelated success erases a required failure'),
]
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--suffix',default='');args=parser.parse_args();suffix='-'+args.suffix if args.suffix else '';out=ROOT/f'reports/spacex-phase11-mutation-results{suffix}.json';logs=ROOT/f'reports/phase11-mutations{suffix}'
 if out.exists()or logs.exists():raise SystemExit('refusing to overwrite mutation evidence')
 logs.mkdir();original=CORE.read_bytes();results=[]
 try:
  for i,(before,after,test,description)in enumerate(faults,1):
   source=original.decode()
   if source.count(before)!=1:raise RuntimeError(f'mutation {i} source anchor not unique: {before}')
   CORE.write_text(source.replace(before,after))
   try:
    command=['cargo','test','--locked','-p','eplyx-lifecycle-impact','--lib',PREFIX+test]
    run=subprocess.run(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True);log=run.stdout
    (logs/f'{i:02}.txt').write_text(log)
    caught=run.returncode!=0 and 'test result: FAILED'in log and PREFIX+test+' ... FAILED'in log
    results.append(dict(mutation=i,fault=description,test=PREFIX+test,exit_code=run.returncode,caught_by_test_assertion=caught,log_file=f'{logs.name}/{i:02}.txt',log_sha256=hashlib.sha256(log.encode()).hexdigest()))
    print(f'{i}/10 {test}: '+('caught'if caught else'NOT caught'),flush=True)
   finally:CORE.write_bytes(original)
   if not caught:print(log[-6000:],file=sys.stderr);break
 finally:CORE.write_bytes(original)
 report=dict(schema_version=1,injected=len(results),caught=sum(r['caught_by_test_assertion']for r in results),source_restored=CORE.read_bytes()==original,source_sha256={str(CORE.relative_to(ROOT)):hashlib.sha256(original).hexdigest()},results=results)
 out.write_text(json.dumps(report,indent=2)+'\n')
 return 0 if len(results)==10 and report['caught']==10 and report['source_restored']else 1
if __name__=='__main__':sys.exit(main())
