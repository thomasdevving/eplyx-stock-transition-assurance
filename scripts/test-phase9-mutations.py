#!/usr/bin/env python3
"""Inject seven official-transition faults, require a named test failure, restore exact bytes."""
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
CORE = ROOT / "engine/src/transition/mod.rs"
ADAPTER = CORE
faults = [
    (CORE, '(PathStatus::NotTested, "No independently verifiable official transition mechanism was established within the bounded investigation.")', '(PathStatus::Proven, "No independently verifiable official transition mechanism was established within the bounded investigation.")', "successor_mint_reference_alone_cannot_prove_transition", "Treat successor discovery as Proven transition"),
    (CORE, "&& !m.external_policy_evidence.is_empty()", "&& !m.external_policy_evidence.is_empty() || m.identity.source_destination_pair_observed", "arbitrary_source_successor_pair_is_not_official_identity", "Treat any source/successor token pair as official identity"),
    (CORE, "s.role != AuthorityRole::Holder", "s.role == AuthorityRole::Unknown", "issuer_assumed_private_signing_is_unsupported", "Treat locally assumed issuer signing as independent authority"),
    (CORE, "EligibilityInput::KycBackend\n                    | EligibilityInput::PrivateEntitlement", "EligibilityInput::PrivateEntitlement", "kyc_backend_dependency_is_unsupported", "Treat KYC/backend dependency as independently executable"),
    (CORE, "e.path_type == ExitPathType::OfficialTransition\n            && e.scope.entity_id == scope.entity_id", "e.scope.entity_id == scope.entity_id", "dex_execution_cannot_be_relabelled_official", "Allow DEX execution to prove OfficialTransition"),
    (CORE, "\n            && e.scope.entity_id == scope.entity_id", "", "transition_proof_does_not_inherit_to_another_holder", "Inherit transition execution to another holder"),
    (CORE, "transition_exists_established: if identified { Some(true) } else { None },", "transition_exists_established: if identified { Some(true) } else { Some(false) },", "no_mechanism_is_not_nonexistence", "Treat a bounded unestablished mechanism as non-existence"),
]


def main():
    out = ROOT / "reports/spacex-phase9-mutation-results.json"
    if out.exists():
        raise SystemExit("refusing to overwrite mutation report")
    original = {p: p.read_bytes() for p in (CORE, ADAPTER)}
    results = []
    logs = ROOT / "reports/phase9-mutations"
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
                    "--lib", "transition::tests::" + test,
                ], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
                log = run.stdout
                (logs / f"{number:02d}.txt").write_text(log)
                caught = (run.returncode != 0 and "test result: FAILED" in log
                          and f"{test} ... FAILED" in log)
                results.append({"mutation": number, "fault": description,
                                "test": test, "exit_code": run.returncode,
                                "caught_by_test_assertion": caught,
                                "log_file": f"phase9-mutations/{number:02d}.txt",
                                "log_sha256": hashlib.sha256(log.encode()).hexdigest()})
                print(f"{number}/7 {test}: {'caught' if caught else 'NOT caught'}", flush=True)
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
    return 0 if len(results) == 7 and report["caught"] == 7 and report["source_restored"] else 1


if __name__ == "__main__":
    sys.exit(main())
