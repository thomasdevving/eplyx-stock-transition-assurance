#!/usr/bin/env python3
"""Inject eight position-withdrawal faults, require a named test failure, restore exact bytes."""
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
CORE = ROOT / "engine/src/position/mod.rs"
ADAPTER = ROOT / "engine/src/position/meteora_dlmm.rs"
faults = [
    (ADAPTER, "state.owner == authority,", "state.owner == authority || state.pool == authority,", "position::meteora_dlmm::tests::pool_vault_authority_cannot_grant_lp_ownership", False, "Treat pool vault authority as LP owner"),
    (CORE, "p.path_type == ExitPathType::Withdrawal && p.scope == *scope", "p.scope == *scope", "position::tests::direct_holder_evidence_cannot_prove_withdrawal", False, "Allow direct-holder evidence to prove Withdrawal"),
    (CORE, "p.scope == *scope", "p.scope.pool == scope.pool", "position::tests::withdrawal_evidence_cannot_inherit_to_another_position", False, "Inherit one LP proof to positions sharing the pool"),
    (ADAPTER, "state.owner == authority,", "true,", "position::meteora_dlmm::tests::wrong_position_authority_is_rejected", False, "Ignore wrong position authority"),
    (ADAPTER, "report.precondition_error = Some(reason);", 'if reason.contains("missing required public account") { report.status = PathStatus::Proven; }\n            report.precondition_error = Some(reason);', "missing_bin_and_bitmap_state_are_explicitly_indeterminate", True, "Ignore missing bin/bitmap public state and grant execution assurance"),
    (ADAPTER, "else if rollback == Some(true) {\n        PathStatus::Failed", "else if rollback == Some(true) {\n        PathStatus::Proven", "position::meteora_dlmm::tests::failed_withdrawal_never_becomes_proven", False, "Mark failed withdrawal as Proven"),
    (CORE, "(ExitPathType::OfficialTransition, PathStatus::NotTested,", "(ExitPathType::OfficialTransition, withdrawal,", "position::tests::withdrawal_never_proves_official_transition_or_redemption", False, "Allow Withdrawal to prove OfficialTransition"),
    (ADAPTER, "signer_possession_known: false,", "signer_possession_known: true,", "signer_assumption_never_claims_private_key_possession", True, "Treat locally assumed signer as known key possession"),
]


def main():
    out = ROOT / "reports/spacex-phase10-mutation-results.json"
    if out.exists():
        raise SystemExit("refusing to overwrite mutation report")
    original = {p: p.read_bytes() for p in (CORE, ADAPTER)}
    results = []
    logs = ROOT / "reports/phase10-mutations"
    logs.mkdir(exist_ok=False)
    try:
        for number, (path, before, after, test, integration, description) in enumerate(faults, 1):
            source = original[path].decode()
            if source.count(before) != 1:
                raise RuntimeError(f"mutation {number} replacement is not unique")
            path.write_text(source.replace(before, after))
            try:
                command = ["cargo", "test", "--locked", "-p", "eplyx-lifecycle-impact"]
                command += ["--test", "protocol_position"] if integration else ["--lib"]
                command += [test]
                run = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
                log = run.stdout
                (logs / f"{number:02d}.txt").write_text(log)
                caught = (run.returncode != 0 and "test result: FAILED" in log
                          and f"{test} ... FAILED" in log)
                results.append({"mutation": number, "fault": description,
                                "test": test, "exit_code": run.returncode,
                                "caught_by_test_assertion": caught,
                                "log_file": f"phase10-mutations/{number:02d}.txt",
                                "log_sha256": hashlib.sha256(log.encode()).hexdigest()})
                print(f"{number}/8 {test}: {'caught' if caught else 'NOT caught'}", flush=True)
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
    return 0 if len(results) == 8 and report["caught"] == 8 and report["source_restored"] else 1


if __name__ == "__main__":
    sys.exit(main())
