#!/usr/bin/env python3
"""Inject eight executable notice faults and retain actual assertion failures."""
import argparse,hashlib,json,pathlib,subprocess,sys
ROOT=pathlib.Path(__file__).resolve().parents[1]
CORE=ROOT/'engine/src/notice/mod.rs'
ADAPTER=ROOT/'engine/src/notice/prestocks.rs'
WORKFLOW=ROOT/'engine/src/notice/workflow.rs'
def blind_identity(source):
 start=source.index('pub(super) fn verify_identity(')
 body=source.index(') -> Result<AssetIdentity> {',start)+len(') -> Result<AssetIdentity> {')
 end=source.index('\npub(super) fn check_compatibility(',body)
 return source[:body]+'''
 let _=base;
 Ok(AssetIdentity{asserted_mint:assertion.value.clone(),observed_mint:Some(assertion.value.clone()),status:IdentityStatus::Verified,slot:Some(mint.slot),observed_name:None,observed_symbol:None,provenance:vec![]})
}
'''+source[end:]
def replace(before,after):
 def edit(source):
  if source.count(before)!=1:raise RuntimeError('nonunique mutation anchor: '+before)
  return source.replace(before,after)
 return edit
faults=[
 (WORKFLOW,blind_identity,'notice::tests::missing_chain_artifact_cannot_grant_identity',False,'Trust successor address/metadata without raw on-chain verification'),
 (ADAPTER,replace('PathStatus::NotTested,','PathStatus::Proven,'),'notice::tests::issuer_wording_never_proves_official_transition',False,'Let issuer swap wording prove OfficialTransition'),
 (CORE,replace('classifications: vec![ProvenanceClass::DemoConfigured],','classifications: vec![if ptr=="/policy/effective_at" {ProvenanceClass::IssuerAsserted} else {ProvenanceClass::DemoConfigured}],'),'notice::tests::demo_time_never_becomes_issuer_assertion',False,'Relabel demo timestamp as issuer assertion'),
 (CORE,replace('sha256(self.raw_content.as_bytes()) == self.content_sha256','true'),'notice::tests::source_digest_mismatch_is_rejected',False,'Ignore source raw-content digest mismatch'),
 (WORKFLOW,replace('ensure!(\n            actual.event() == event,','let mut expected=actual.event().clone();expected.deadline=event.deadline.clone();\n        ensure!(\n            &expected == event,'),'notice::tests::altered_deadline_without_regeneration_is_rejected',False,'Accept altered normalized deadline without source regeneration'),
 (ADAPTER,replace('conversion_ratio: Field::new(None, ProvenanceClass::Unknown, banner_loc.clone()),','conversion_ratio: Field::new(Some("1:1".into()), ProvenanceClass::Derived, banner_loc.clone()),'),'notice::tests::unrelated_data_never_infers_conversion_ratio',False,'Infer invented conversion ratio from unrelated page data'),
 (ADAPTER,replace('MechanismType::Unknown,','MechanismType::Swap,'),'notice::tests::unknown_mechanism_stays_unknown',False,'Convert unknown mechanism into an executable Swap'),
 (WORKFLOW,replace('let readiness = readiness::evaluate(&policy, &verified)?;','let mut readiness = readiness::evaluate(&policy, &verified)?;readiness.overall_status=readiness::ReadinessStatus::Ready;'),'notice_cannot_bypass_existing_readiness_requirements',True,'Bypass unchanged required readiness findings'),
]
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--suffix',default='');args=parser.parse_args();suffix='-'+args.suffix if args.suffix else ''
 logs=ROOT/f'reports/phase12-mutations{suffix}';out=ROOT/f'reports/spacex-phase12-mutation-results{suffix}.json'
 if logs.exists() or out.exists():raise SystemExit('refusing to overwrite mutation evidence')
 logs.mkdir();originals={p:p.read_bytes()for p in [CORE,ADAPTER,WORKFLOW]};results=[]
 try:
  for i,(path,edit,test,integration,description)in enumerate(faults,1):
   mutated=edit(originals[path].decode());path.write_text(mutated)
   try:
    command=['cargo','test','--locked','-p','eplyx-lifecycle-impact']+(['--test','lifecycle_notice']if integration else['--lib'])+[test,'--','--exact']
    run=subprocess.run(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True);log=run.stdout
    (logs/f'{i:02}.txt').write_text(log)
    caught=run.returncode!=0 and 'test result: FAILED'in log and test+' ... FAILED'in log and ('assertion'in log or 'panicked at'in log)
    results.append(dict(mutation=i,fault=description,test=test,command=command,exit_code=run.returncode,caught_by_test_assertion=caught,mutated_source_sha256=hashlib.sha256(mutated.encode()).hexdigest(),log_file=f'{logs.name}/{i:02}.txt',log_sha256=hashlib.sha256(log.encode()).hexdigest()))
    print(f'{i}/8 {test}: '+('caught'if caught else'NOT caught'),flush=True)
   finally:path.write_bytes(originals[path])
   if not caught:print(log[-6000:],file=sys.stderr);break
 finally:
  for p,b in originals.items():p.write_bytes(b)
 report=dict(schema_version=1,injected=len(results),caught=sum(r['caught_by_test_assertion']for r in results),source_restored=all(p.read_bytes()==b for p,b in originals.items()),source_sha256={str(p.relative_to(ROOT)):hashlib.sha256(b).hexdigest()for p,b in originals.items()},results=results)
 out.write_text(json.dumps(report,indent=2)+'\n')
 return 0 if len(results)==8 and report['caught']==8 and report['source_restored']else 1
if __name__=='__main__':sys.exit(main())
