#!/usr/bin/env python3
"""Inject ten semantic faults, require a named test failure, restore exact bytes."""
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
SELECTOR = ROOT / "engine/src/expansion/selector.rs"
PIPELINE = ROOT / "engine/src/expansion/pipeline.rs"
faults = [
    (SELECTOR, ".and_modify(|n| *n = (*n).max(b))", ".and_modify(|n| *n += b)",
     "predicted_independent_contexts_use_maxima", "Predict independent contexts by summing amounts"),
    (PIPELINE, "            changed.insert(pos);", """            changed.insert(pos);
            let class = entities[pos].representative_class.clone();
            let amount = entities[pos].represented_amount_covered_raw.parse::<u64>()?;
            for (peer_idx, peer) in entities.iter_mut().enumerate() {
                if peer.representative_class == class {
                    let balance = peer.represented_balance_raw.parse::<u64>()?;
                    let covered = amount.min(balance);
                    peer.represented_amount_covered_raw = covered.to_string();
                    peer.represented_amount_without_evidence_raw = (balance - covered).to_string();
                    peer.classification = classify(balance, covered, false);
                    changed.insert(peer_idx);
                }
            }""", "a_class_sample_never_proves_its_peers", "Propagate representative evidence to class peers"),
    (PIPELINE, "    let after = metrics(&entities, &measured, &owners, &venues, &paths, &classes);",
     "    venues.extend(i.contexts.iter().filter(|c| c.kind == ContextKind::MeteoraDlmm).map(|c| c.id.clone()));\n    let after = metrics(&entities, &measured, &owners, &venues, &paths, &classes);",
     "second_venue_has_no_brand_or_first_venue_inheritance", "Inherit all same-brand venue evidence"),
    (SELECTOR, """        matches!(
            c.eligibility,
            Eligibility::CaptureRequired | Eligibility::ExecutableCandidate
        ),
""",
     '        true,\n',
     "unsupported_and_invalid_cannot_be_scored", "Score unsupported candidates"),
    (PIPELINE, "expected: group.expected_gain.clone(),",
     "expected: { let mut g = group.expected_gain.clone(); g.entities = measured.len(); g },",
     "execution_never_rewrites_expected_selection_gains", "Rewrite expected gains from measured results"),
    (PIPELINE, "measured_entities: measured.len(),", "measured_entities: owners.len(),",
     "distinct_accounts_with_one_authority_stay_distinct", "Conflate entity and authority counts"),
    (PIPELINE, "if e.status != CaseStatus::Succeeded || e.invalid_control {",
     "if e.status == CaseStatus::Unsupported || e.invalid_control {",
     "failed_indeterminate_unsupported_and_controls_gain_nothing", "Allow failed probes to gain assurance"),
    (PIPELINE, "let max = old.max(n);", "let max = old + n;",
     "independent_swaps_are_maxima_never_summed_capacity", "Sum independent executions as capacity"),
    (PIPELINE, "    CurrentFinalizedProduction,",
     '    #[serde(rename = "HistoricalLifecycleBank")]\n    CurrentFinalizedProduction,',
     "current_capture_cannot_be_labeled_a_historical_bank", "Label current capture as a historical bank"),
    (PIPELINE, ".find(|p| p.path_type == e.path_type)",
     ".find(|p| p.path_type == ExitPathType::OfficialTransition)",
     "official_transition_and_redemption_cannot_inherit_swap_or_transfer", "Inherit swap and transfer into official transition"),
]


def main():
    out = ROOT / "reports/spacex-phase7-mutation-results.json"
    if out.exists():
        raise SystemExit("refusing to overwrite mutation report")
    original = {p: p.read_bytes() for p in (SELECTOR, PIPELINE)}
    results = []
    logs = ROOT / "reports/phase7-mutations"
    logs.mkdir(exist_ok=False)
    try:
        for number, (path, before, after, test, description) in enumerate(faults, 1):
            source = original[path].decode()
            if source.count(before) != 1:
                raise RuntimeError(f"mutation {number} replacement is not unique")
            path.write_text(source.replace(before, after))
            try:
                run = subprocess.run([
                    "cargo", "test", "--locked", "-p", "eplyx-lifecycle-impact",
                    "--lib", test,
                ], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
                log = run.stdout
                (logs / f"{number:02d}.txt").write_text(log)
                caught = (run.returncode != 0 and "test result: FAILED" in log
                          and f"{test} ... FAILED" in log)
                results.append({"mutation": number, "fault": description,
                                "test": test, "exit_code": run.returncode,
                                "caught_by_test_assertion": caught,
                                "log_file": f"phase7-mutations/{number:02d}.txt",
                                "log_sha256": hashlib.sha256(log.encode()).hexdigest()})
                print(f"{number}/10 {test}: {'caught' if caught else 'NOT caught'}", flush=True)
            finally:
                path.write_bytes(original[path])
            if not caught:
                print(log[-6000:], file=sys.stderr)
                break
    finally:
        for path, data in original.items():
            path.write_bytes(data)
    report = {"schema_version": 1, "injected": len(results),
              "caught": sum(r["caught_by_test_assertion"] for r in results),
              "source_restored": all(p.read_bytes() == data for p, data in original.items()),
              "source_sha256": {str(p.relative_to(ROOT)): hashlib.sha256(data).hexdigest()
                                for p, data in original.items()}, "results": results}
    out.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if len(results) == 10 and report["caught"] == 10 and report["source_restored"] else 1


if __name__ == "__main__":
    sys.exit(main())
