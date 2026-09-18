#!/usr/bin/env python3
"""Inject five executable rollout faults; compiler errors never count as detection."""
import hashlib,json,pathlib,subprocess,sys
ROOT=pathlib.Path(__file__).resolve().parents[1]
SOURCE=ROOT/'engine/src/rollout/mod.rs'
def replace(before,after):
    def edit(source):
        if source.count(before)!=1: raise RuntimeError('nonunique mutation anchor: '+before)
        return source.replace(before,after)
    return edit
def promote_complete_exit(source):
    anchor='                        Claim::CompletePositionExit => {'
    if source.count(anchor)!=1: raise RuntimeError('nonunique complete-exit arm')
    start=source.index('result.assessment =',source.index(anchor));end=source.index(';',start)+1
    return source[:start]+'result.assessment = EvidenceAssessment::Supported;'+source[end:]
FAULTS=[
 ('Promote principal withdrawal to complete position exit',promote_complete_exit,'principal_removal_never_establishes_complete_exit','principal_withdrawal_is_not_complete_exit'),
 ('Promote movement proof to official conversion',replace('result.assessment = EvidenceAssessment::EvidenceSubstitution;','result.assessment = EvidenceAssessment::Supported;'),'transfer_never_establishes_official_conversion','transfer_is_not_official_conversion'),
 ('Ignore explicit optional-to-required policy variant',replace('let readiness = readiness::evaluate(&policy, self.world.verified_evidence())?;','let readiness = readiness::evaluate(&self.counterfactual.frozen_readiness.policy, self.world.verified_evidence())?;'),'exact_required_failed_route_is_blocked','required_exact_failure_blocks'),
 ('Permit Incomplete assurance through the guard',replace('ReadinessStatus::Blocked | ReadinessStatus::Incomplete => {\n            GuardDisposition::RefusedReadiness\n        }','ReadinessStatus::Blocked => GuardDisposition::RefusedReadiness,\n        ReadinessStatus::Incomplete => GuardDisposition::Permitted'),'incomplete_assurance_never_runs_the_stub','incomplete_never_authorizes_stub'),
 ('Treat generic analysis exit zero as assurance completion',replace('if !matches!(completion,GateCommandCompletion::AssuranceEvaluation{exit_code} if exit_code==status.exit_code())','if !matches!(completion,GateCommandCompletion::AssuranceEvaluation{exit_code} | GateCommandCompletion::AnalysisCompleted{exit_code} if exit_code==status.exit_code())'),'analysis_completion_zero_never_authorizes_the_stub','analysis_success_is_not_assurance_approval'),
]
def main():
    logs=ROOT/'reports/phase14-mutations';out=ROOT/'reports/spacex-phase14-mutation-results.json'
    if logs.exists() or out.exists(): raise SystemExit('refusing to overwrite mutation evidence')
    original=SOURCE.read_bytes()
    # Validate every anchor before the first injection.
    for _,edit,_,_ in FAULTS: edit(original.decode())
    logs.mkdir();results=[]
    try:
        for i,(fault,edit,test,assertion) in enumerate(FAULTS,1):
            mutated=edit(original.decode());SOURCE.write_text(mutated)
            try:
                command=['cargo','test','--locked','-p','eplyx-lifecycle-impact','--test','rollout_assumptions',test,'--','--exact']
                run=subprocess.run(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
                log=run.stdout;p=logs/f'{i:02}.txt';p.write_text(log)
                caught=run.returncode==101 and 'test result: FAILED' in log and test+' ... FAILED' in log and assertion in log
                results.append(dict(mutation=i,fault=fault,test=test,named_assertion=assertion,command=command,exit_code=run.returncode,caught_by_named_assertion=caught,mutated_source_sha256=hashlib.sha256(mutated.encode()).hexdigest(),log_file=str(p.relative_to(ROOT)),log_sha256=hashlib.sha256(log.encode()).hexdigest()))
                print(f'{i}/5 {test}: '+('caught' if caught else 'NOT caught'),flush=True)
            finally: SOURCE.write_bytes(original)
            if not caught:
                print(log[-6000:],file=sys.stderr);break
    finally: SOURCE.write_bytes(original)
    report=dict(schema_version=1,injected=len(results),caught=sum(r['caught_by_named_assertion'] for r in results),source_restored=SOURCE.read_bytes()==original,source_sha256=hashlib.sha256(original).hexdigest(),results=results)
    out.write_text(json.dumps(report,indent=2)+'\n')
    return 0 if report['injected']==report['caught']==5 and report['source_restored'] else 1
if __name__=='__main__':sys.exit(main())
