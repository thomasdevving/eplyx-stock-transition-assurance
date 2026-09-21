#!/usr/bin/env python3
"""Executable candidate-conversion faults. Only named assertion failures count; restore every byte."""
import pathlib,subprocess,hashlib,json
ROOT=pathlib.Path(__file__).resolve().parents[1]
OUT=ROOT/'reports/milestone6-validation/mutations'
P='engine/src/preflight.rs'
C='engine/src/conversion/current.rs'
D='engine/src/conversion/demo.rs'
M='engine/src/conversion/mod.rs'
R='engine/src/readiness/mod.rs'
FAULTS=[
 ('proven_without_execution',[(C,'''    eprintln!("CURRENT_STAGE:Running candidate conversion locally");
    let execution = executor::execute_probe_message(''','''    if true {
        let mut forged = result.clone();
        forged["status"] = serde_json::to_value(PathStatus::Proven)?;
        forged["reconciliation"] = json!({"reconciled":true});
        return Ok(VerifiedReplacementConversion::new(forged));
    }
    eprintln!("CURRENT_STAGE:Running candidate conversion locally");
    let execution = executor::execute_probe_message(''')],
  'candidate_conversion','candidate_conversion_executes_the_actual_program_and_reconciles_exactly','proven_requires_an_actual_successful_vm_transaction'),
 ('proposed_marked_observed',[(D,'            origin: AccountOrigin::Proposed,\n            role: role.into(),','            origin: AccountOrigin::Observed,\n            role: role.into(),'),
   (M,'''pub fn validate_origins(accounts: &[FixtureAccount]) -> Result<()> {
    for a in accounts {''','''pub fn validate_origins(accounts: &[FixtureAccount]) -> Result<()> {
    if !accounts.is_empty() { return Ok(()); }
    for a in accounts {''')],
  'candidate_conversion','proposed_overlay_is_never_labelled_observed','config, authority, reserve and candidate program are proposed'),
 ('ignored_replacement_credit',[(D,'            && credited == released - replacement_fee','            && credited <= released')],
  'candidate_conversion','a_short_replacement_credit_fails_exact_reconciliation','short_replacement_credit_must_reject_proof'),
 ('ignored_ratio_rounding',[(D,'            && released.to_string() == expected.replacement_gross_raw','            && released > 0')],
  'candidate_conversion','a_release_that_ignores_the_terms_fails_reconciliation','release_outside_the_declared_terms_must_reject_proof'),
 ('operator_promoted_to_official',[(P,'''            "official_transition": PathStatus::NotTested, "issuer_binding_established": false,
            "reason": v["reason"],''','''            "official_transition": status, "issuer_binding_established": true,
            "reason": v["reason"],''')],
  'current_preflight','candidate_plan::a_proven_candidate_plan_makes_candidate_readiness_ready_and_nothing_else','operator_supplied_proof_must_not_report_an_official_transition'),
 ('inherited_across_refresh',[(C,'''            && c.run_id == run_id
            && c.check_id == check_id''','''            && !run_id.is_empty()
            && c.check_id == check_id'''),
   (P,'''        ensure!(
            sha256(format!("{v}\\n").as_bytes()) == evidence.result_sha256,
            "original candidate conversion result digest mismatch"
        );''','')],
  'current_preflight','candidate_plan::a_refreshed_run_cannot_inherit_candidate_conversion_proof','refresh_must_not_inherit_candidate_conversion_proof'),
 ('market_exit_satisfies_conversion',[(P,'''    let mut conversion_facts: Vec<ConversionFact> = vec![];''','''    let mut conversion_facts: Vec<ConversionFact> = vec![];
    for row in paths.rows() {
        for context in &row.contexts {
            for a in &context.attempts {
                if a.status == PathStatus::Proven {
                    conversion_facts.push(ConversionFact {
                        scope: readiness::EvidenceScope{asset_mint:state.mint.clone(),entity_id:format!("current:{run}:{}",r.source),state_shape:readiness::StateShape::DirectTokenAccount,authority:state.owner.clone(),exact_amount_raw:None,range:None,bps_to_remove:None,venue:None,context_id:None,captured_slot:None,clock:None,capture_context:None,captured_state_sha256:Some(wallet_hash.into()),fixture_sha256:None,source_before_raw:None,scenario_sha256:scenario_hash.into()},
                        status: PathStatus::Proven, provenance: conversion::PlanProvenance::OperatorSupplied,
                        plan_sha256: "0".repeat(64), program_sha256: "0".repeat(64),
                        replacement_mint: "market".into(), destination: String::new(),
                        execution_attempted: true, reconciled: true, rollback_verified: None,
                        holder_signer: SignerAssumption{authority:state.owner.clone(),signer_possession_known:false,signer_assumed_locally:true,wording:String::new()},
                        candidate_authority_assumed_locally: true, issuer_binding_established: false,
                        evidence_ids: vec![a.case_id.clone()], reason: String::new(),
                    });
                }
            }
        }
    }''')],
  'current_preflight','candidate_plan::mobility_evidence_can_never_stand_in_for_a_candidate_conversion','mobility_proof_must_not_satisfy_candidate_conversion'),
 ('ignored_plan_and_program_digests',[(R,'''                    && f.plan_sha256 == *plan_sha256
                    && f.program_sha256 == *program_sha256
                    && f.replacement_mint == *replacement_mint;''','''                    && !plan_sha256.is_empty()
                    && !program_sha256.is_empty()
                    && !replacement_mint.is_empty();'''),
   (C,'''    ensure!(
        sha256(program) == program_hash,
        "candidate program digest mismatch"
    );''',''''''),
   (C,'''    ensure!(
        c.plan_sha256 == s.plan_sha256 && c.plan_sha256 == plan_hash,
        "conversion plan digest mismatch"
    );''','''    ensure!(c.plan_sha256 == s.plan_sha256, "conversion plan digest mismatch");'''),
   (P,'''        ensure!(
            sha256(&program) == evidence.program_sha256,
            "candidate mechanism build differs from the recorded candidate program digest"
        );''',''''''),
   (P,'''                && v["plan_sha256"] == evidence.plan_sha256
''','''''')],
  'current_preflight','candidate_plan::candidate_evidence_is_exact_to_its_run_plan_program_and_result','exact_candidate_binding_must_be_required'),
]
def main():
 if OUT.exists():raise SystemExit('Refusing to overwrite mutation evidence')
 originals={p:(ROOT/p).read_bytes() for _,edits,*_ in FAULTS for p,_,_ in edits}
 for name,edits,*_ in FAULTS:
  for p,before,_ in edits:
   if originals[p].decode().count(before)!=1:raise RuntimeError(f'Nonunique mutation anchor in {name}: '+before[:60])
 OUT.mkdir(parents=True);rows=[]
 try:
  for name,edits,suite,test,assertion in FAULTS:
   for p,before,after in edits:(ROOT/p).write_text((ROOT/p).read_text().replace(before,after))
   cmd=['cargo','test','--locked','-p','eplyx-lifecycle-impact','--test',suite,test,'--','--exact']
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
