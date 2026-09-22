#!/usr/bin/env python3
"""Executable conversion-stress faults.

Each fault makes sampled evidence pretend to be broader than it is. Only a named
assertion failure counts as a kill; a compiler error does not. Every byte of
every mutated source is restored, and the restoration is verified.
"""
import pathlib, subprocess, hashlib, json

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / 'reports/milestone7-validation/mutations'
E = 'engine/src/stress/execute.rs'
M = 'engine/src/stress/mod.rs'
P = 'engine/src/stress/population.rs'
R = 'engine/src/readiness/mod.rs'
SUITE = 'conversion_stress'

FAULTS = [
 # 1. One tested entity is allowed to prove every peer sharing its state shape.
 ('entity_proves_its_peers', [(E, '''    let proven_ids: BTreeSet<String> = results
        .iter()
        .filter(|r| r.status == PathStatus::Proven)
        .map(|r| r.entity_id.clone())
        .collect();''', '''    let proven_shapes: BTreeSet<String> = results
        .iter()
        .filter(|r| r.status == PathStatus::Proven)
        .map(|r| r.state_shape_sha256.clone())
        .collect();
    let proven_ids: BTreeSet<String> = plan
        .state_shapes
        .iter()
        .filter(|s| proven_shapes.contains(&s.state_shape_sha256))
        .flat_map(|s| {
            s.highest_balance_entities
                .iter()
                .map(|a| super::entity_id(&plan.population_capture_sha256, a))
        })
        .collect();''')],
  'distinct_amounts_prove_that_cases_run_independently_and_are_never_summed',
  'one_tested_entity_must_not_prove_its_peers'),

 # 2. One tested shape is allowed to stand in for all of its members.
 ('shape_proves_its_members', [
   (E, '''            entities_executed: executed_ids.len(),
            executed_entity_ids: executed_ids.iter().cloned().collect(),
            entities_untested: shape.entities_in_shape - executed_ids.len(),''',
       '''            entities_executed: if executed_ids.is_empty() {
                0
            } else {
                shape.entities_in_shape
            },
            executed_entity_ids: executed_ids.iter().cloned().collect(),
            entities_untested: 0,'''),
   (M, '''        ensure!(
            shape.executed_entity_ids.len() == shape.entities_executed
                && shape.entities_executed <= shape.entities_selected
                && shape.entities_selected <= shape.entities_in_shape
                && shape.entities_untested == shape.entities_in_shape - shape.entities_executed,
            "shape coverage counts must stay consistent with exact executed entities"
        );''', '''        ensure!(
            shape.entities_selected <= shape.entities_in_shape,
            "shape coverage counts must stay consistent with exact executed entities"
        );''')],
  'shape_coverage_never_becomes_entity_coverage',
  'one_tested_shape_must_not_prove_its_members'),

 # 3. The frozen selection may be rewritten once results are known.
 ('selection_rewritten_after_execution', [(E, '''    plan.validate(&observation, &plan.candidate_plan, program_sha256)?;''',
   '''    let _ = plan.validate(&observation, &plan.candidate_plan, program_sha256);''')],
  'the_plan_is_frozen_before_execution_and_cannot_be_rewritten_afterwards',
  'selection_must_not_be_rewritten_after_execution'),

 # 4. Program-controlled authorities are quietly dropped from the report.
 ('unsupported_authority_hidden', [(E, '''    let mut unsupported_summary: Vec<UnsupportedState> = plan
        .state_shapes
        .iter()
        .filter(|s| s.eligibility != Eligibility::ExecutableCandidate)''',
   '''    let mut unsupported_summary: Vec<UnsupportedState> = plan
        .state_shapes
        .iter()
        .filter(|s| s.eligibility == Eligibility::Invalid)''')],
  'a_program_controlled_authority_is_surfaced_and_never_given_a_wallet_signer',
  'unsupported_authorities_must_stay_visible'),

 # 5. Zero-balance accounts are counted as conversion exposure.
 ('zero_balance_counted_as_exposure', [(P, '''        if balance > 0 {
            summary.positive_balance_accounts_observed += 1;
            *summary
                .positive_balance_authority_model_counts
                .entry(key)
                .or_insert(0) += 1;
            balances.insert(e.entity_id.clone(), balance);
        } else {
            summary.zero_balance_accounts_observed += 1;
        }''', '''        summary.positive_balance_accounts_observed += 1;
        *summary
            .positive_balance_authority_model_counts
            .entry(key)
            .or_insert(0) += 1;
        balances.insert(e.entity_id.clone(), balance);
        if balance == 0 {
            summary.zero_balance_accounts_observed += 1;
        }''')],
  'distinct_amounts_prove_that_cases_run_independently_and_are_never_summed',
  'zero_balance_must_not_count_as_exposure'),

 # 6. The whole observed population is reported as if it had been tested.
 ('outputs_summed_as_capacity', [(E, '''        tested_public_balance_raw: sum_once(&pick(&selected_ids)),''',
   '''        tested_public_balance_raw: sum_once(&balances),''')],
  'distinct_amounts_prove_that_cases_run_independently_and_are_never_summed',
  'independent_outputs_must_not_be_summed_as_capacity'),

 # 7. Stress proof is allowed to carry across a refreshed population.
 ('proof_inherited_across_refresh', [
   (E, '''            && plan.population_capture_sha256 == population_sha256
''', ''''''),
   (E, '''    ensure!(
        observation.capture_sha256 == population_sha256
            && observation.run_id == run_id
            && observation.stress_id == stress_id,
        "population capture identity mismatch"
    );''', '''    ensure!(
        observation.run_id == run_id && observation.stress_id == stress_id,
        "population capture identity mismatch"
    );'''),
   (E, '''    plan.validate(&observation, &plan.candidate_plan, program_sha256)?;''',
       '''    let _ = plan.validate(&observation, &plan.candidate_plan, program_sha256);''')],
  'a_refreshed_population_starts_untested_and_inherits_no_proof',
  'refreshed_world_must_not_inherit_stress_proof'),

 # 8. A successful bounded sample is allowed to satisfy population readiness.
 ('population_ready_from_sample', [(R, '''                    if s.positive_balance_accounts > 0
                        && s.proven_cases == s.positive_balance_accounts
                    {
                        FindingEffect::Satisfied''', '''                    if s.proven_cases > 0 {
                        FindingEffect::Satisfied''')],
  'stress_readiness_and_population_readiness_stay_separate_and_neither_is_ready',
  'population_readiness_must_not_follow_from_a_sample'),
]


def main():
    if OUT.exists():
        raise SystemExit('Refusing to overwrite mutation evidence')
    originals = {p: (ROOT / p).read_bytes() for _, edits, *_ in FAULTS for p, _, _ in edits}
    for name, edits, *_ in FAULTS:
        for p, before, _ in edits:
            if originals[p].decode().count(before) != 1:
                raise RuntimeError(f'Nonunique mutation anchor in {name}: ' + before[:70])
    OUT.mkdir(parents=True)
    rows = []
    try:
        for name, edits, test, assertion in FAULTS:
            for p, before, after in edits:
                (ROOT / p).write_text((ROOT / p).read_text().replace(before, after))
            cmd = ['cargo', 'test', '--locked', '-p', 'eplyx-lifecycle-impact',
                   '--test', SUITE, test, '--', '--exact']
            try:
                run = subprocess.run(cmd, cwd=ROOT, text=True,
                                     stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
                log = run.stdout
                (OUT / f'{name}.txt').write_text(log)
                compiled = 'error[E' not in log and 'could not compile' not in log
                killed = (run.returncode == 101 and 'test result: FAILED' in log
                          and test + ' ... FAILED' in log and assertion in log and compiled)
                rows.append(dict(mutation=name, command=cmd, test=test,
                                 named_assertion=assertion,
                                 killed_by_named_assertion=killed,
                                 compiled=compiled, exit_code=run.returncode,
                                 edits=[dict(file=p, before=b, after=a,
                                             mutated_sha256=hashlib.sha256((ROOT / p).read_bytes()).hexdigest())
                                        for p, b, a in edits],
                                 log_sha256=hashlib.sha256(log.encode()).hexdigest()))
                print(name + (': killed' if killed else ': NOT killed'), flush=True)
            finally:
                for p, _, _ in edits:
                    (ROOT / p).write_bytes(originals[p])
            if not killed:
                print(log[-5000:])
                break
    finally:
        for p, data in originals.items():
            (ROOT / p).write_bytes(data)
    report = dict(injected=len(rows),
                  killed=sum(r['killed_by_named_assertion'] for r in rows),
                  compiler_error_kills=sum(0 if r['compiled'] else 1 for r in rows),
                  sources_restored=all((ROOT / p).read_bytes() == data for p, data in originals.items()),
                  source_sha256={p: hashlib.sha256(data).hexdigest() for p, data in originals.items()},
                  results=rows)
    (OUT / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    if report['killed'] != len(FAULTS) or report['compiler_error_kills']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
