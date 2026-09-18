#!/usr/bin/env python3
"""Inject eight executable counterfactual faults; require named assertion failures."""
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'engine/src/counterfactual/mod.rs'

def replace(before, after):
    def edit(source):
        if source.count(before) != 1:
            raise RuntimeError('nonunique mutation anchor: ' + before)
        return source.replace(before, after)
    return edit

faults = [
    ('Change captured holder balance only after the event',
     replace('raw_balance: direct.balance.raw.clone(),', 'raw_balance: if status == LifecycleStatus::Active { direct.balance.raw.clone() } else { "17622".into() },'),
     'balances_never_change'),
    ('Report lifecycle meaning change as an on-chain mutation',
     replace('onchain_state_changed: false,', 'onchain_state_changed: changed,'),
     'vault_semantics_change_without_byte_changes'),
    ('Keep selected holder Unaffected after transition',
     replace('impact: direct.impact_classification,', 'impact: Impact::Unaffected,'),
     'holder_requires_transition_after_event'),
    ('Classify public zero-balance accounts as transition exposure',
     replace('.entry(e.impact_classification)', '.entry(if status == LifecycleStatus::TransitionRequired && e.balance.raw == "0" { Impact::RequiresTransition } else { e.impact_classification })'),
     'zero_balance_never_becomes_public_exposure'),
    ('Promote proven token movement to official transition proof after deadline',
     replace('historical_status: row.status,\n                lifecycle_relevant: relevant\n                    && direct.balance.raw', 'historical_status: if status == LifecycleStatus::NoIssuerEntitlement && row.path_type == ExitPathType::OfficialTransition { PathStatus::Proven } else { row.status },\n                lifecycle_relevant: relevant\n                    && direct.balance.raw'),
     'official_transition_never_inherits_proof'),
    ('Change readiness without assurance-policy justification',
     replace('policy_result:relevant.then_some(self.readiness.overall_status)', 'policy_result:relevant.then_some(ReadinessStatus::Ready)'),
     'readiness_changes_only_with_policy_and_applicability'),
    ('Accept distinct production-state digests as identical worlds',
     replace('from.production_state_digest == to.production_state_digest,', 'true,'),
     'different_worlds_are_non_comparable'),
    ('Let scenario evaluation rewrite stored historical execution matrix',
     replace('historical_direct_paths:self.direct_paths.clone()', 'historical_direct_paths:{let mut paths=self.direct_paths.clone();paths[0]["status"]=serde_json::json!("Proven");paths}'),
     'historical_execution_evidence_is_immutable'),
]

def main():
    logs = ROOT / 'reports/phase13-mutations'
    output = ROOT / 'reports/spacex-phase13-mutation-results.json'
    if logs.exists() or output.exists():
        raise SystemExit('refusing to overwrite mutation evidence')
    logs.mkdir()
    original = SOURCE.read_bytes()
    results = []
    try:
        for i, (fault, edit, test) in enumerate(faults, 1):
            mutated = edit(original.decode())
            SOURCE.write_text(mutated)
            try:
                command = ['cargo', 'test', '--locked', '-p', 'eplyx-lifecycle-impact', '--test', 'lifecycle_counterfactual', test, '--', '--exact']
                run = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
                log = run.stdout
                log_path = logs / f'{i:02}.txt'
                log_path.write_text(log)
                caught = run.returncode != 0 and 'test result: FAILED' in log and test + ' ... FAILED' in log and ('assertion' in log or 'same_world_counterfactual_invariants' in log or 'zero_balance_has_no_public_economic_exposure' in log or 'different_production_digests_rejected' in log)
                results.append(dict(mutation=i, fault=fault, test=test, command=command, exit_code=run.returncode, caught_by_named_assertion=caught, mutated_source_sha256=hashlib.sha256(mutated.encode()).hexdigest(), log_file=str(log_path.relative_to(ROOT)), log_sha256=hashlib.sha256(log.encode()).hexdigest()))
                print(f'{i}/8 {test}: ' + ('caught' if caught else 'NOT caught'), flush=True)
            finally:
                SOURCE.write_bytes(original)
            if not caught:
                print(log[-6000:], file=sys.stderr)
                break
    finally:
        SOURCE.write_bytes(original)
    report = dict(schema_version=1, injected=len(results), caught=sum(r['caught_by_named_assertion'] for r in results), source_restored=SOURCE.read_bytes()==original, source_sha256=hashlib.sha256(original).hexdigest(), results=results)
    output.write_text(json.dumps(report, indent=2) + '\n')
    return 0 if report['injected']==8 and report['caught']==8 and report['source_restored'] else 1

if __name__ == '__main__':
    sys.exit(main())
