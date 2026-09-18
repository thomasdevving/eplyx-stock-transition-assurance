#!/usr/bin/env python3
"""Inject seven path-resolution faults, require a named test failure, restore exact bytes."""
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
CORE = ROOT / "engine/src/resolution/mod.rs"
ADAPTER = ROOT / "engine/src/resolution/phase7.rs"
FILTER = ".filter(|a| a.entity_id == impact.entity_id && a.path_type == path)"
faults = [
    (CORE, FILTER, ".filter(|a| a.entity_id == impact.entity_id && (a.path_type == path || (path == ExitPathType::OfficialTransition && a.path_type == ExitPathType::Transfer)))", "transfer_cannot_prove_official_transition", "Allow Transfer to imply OfficialTransition"),
    (CORE, FILTER, ".filter(|a| a.entity_id == impact.entity_id && (a.path_type == path || (path == ExitPathType::Redemption && a.path_type == ExitPathType::SecondaryMarketExit)))", "market_exit_cannot_prove_redemption", "Allow SecondaryMarketExit to imply Redemption"),
    (CORE, "| MechanismBoundary::NoSupportedAdapter => PathStatus::Unsupported,", "| MechanismBoundary::NoSupportedAdapter => PathStatus::Failed,", "unsupported_is_not_failed_or_nonexistent", "Treat Unsupported as Failed"),
    (CORE, """MechanismBoundary::OnchainCandidateUnverified | MechanismBoundary::Unresolved => {
            PathStatus::NotTested
        }""", """MechanismBoundary::OnchainCandidateUnverified | MechanismBoundary::Unresolved => {
            PathStatus::Proven
        }""", "not_tested_never_becomes_proven", "Treat NotTested as Proven"),
    (CORE, FILTER, ".filter(|a| a.path_type == path)", "entity_proof_does_not_inherit", "Inherit one entity proof to another entity"),
    (CORE, ".filter(|a| a.context_id == id)", ".filter(|_| true)", "venue_proof_does_not_inherit", "Inherit one venue proof to every requested venue"),
    (ADAPTER, "signer_possession_known: e.authority.signer_possession_known,", "signer_possession_known: e.authority.signer_assumed_locally,", "local_assumed_signer_never_becomes_possession", "Treat assumed signer as real signature possession"),
]


def main():
    out = ROOT / "reports/spacex-phase8-mutation-results.json"
    if out.exists():
        raise SystemExit("refusing to overwrite mutation report")
    original = {p: p.read_bytes() for p in (CORE, ADAPTER)}
    results = []
    logs = ROOT / "reports/phase8-mutations"
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
                    "--lib", "resolution::tests::" + test,
                ], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
                log = run.stdout
                (logs / f"{number:02d}.txt").write_text(log)
                caught = (run.returncode != 0 and "test result: FAILED" in log
                          and f"{test} ... FAILED" in log)
                results.append({"mutation": number, "fault": description,
                                "test": test, "exit_code": run.returncode,
                                "caught_by_test_assertion": caught,
                                "log_file": f"phase8-mutations/{number:02d}.txt",
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
