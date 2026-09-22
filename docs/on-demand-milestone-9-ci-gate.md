# Milestone 9: CI deployment gate

The operator flow is:

```text
build candidate SBF → assemble transition package → validate package
→ eplyx-lifecycle preflight ./package --gate block-only → PASS / WARN / BLOCK
```

Use `node scripts/assemble-transition-package.mjs <template> <candidate.so> <new-output-directory>` to copy an operator's manifest and config and bind the exact candidate SBF digest. The script writes the candidate bytes into the new package; the engine validates every package field before RPC. `preflight` uses fresh bounded read-only observations and local VM execution. It writes `report.json`, `report.md`, captures, frozen plan, result bindings and replay inputs under `--out`. The package itself and those outputs must be retained together. No funds move.

```sh
eplyx-lifecycle preflight ./package --gate block-only --out ./ci-result
eplyx-lifecycle replay-package-preflight ./package --result ./ci-result
eplyx-lifecycle replay-package-preflight ./package --result ./ci-result --gate strict
```

`block-only` is the documented default. It maps Ready to **PASS** (exit 0), Incomplete to **PASS WITH WARNINGS** (exit 0), and Blocked to **BLOCK** (exit 3). `strict` maps Ready to PASS and both Incomplete and Blocked to BLOCK (exit 3). Invalid packages, invalid analytical values, mismatched evidence and engine failures exit 2. The gate considers CandidatePlanReadiness, ConversionStressReadiness and PopulationRolloutReadiness. OfficialTransition remains a separate NotTested fact and is never promoted by either policy. `Incomplete` does not mean execution failed; `Blocked` requires an explicit failed condition. A stress sample can have ten Proven exact cases while population coverage remains Incomplete.

The report retains all analytical fields and adds `deployment_gate`, `gate_policy`, `gate_outcome`, `gate_reasons`, `selected_stress_counts` and `failures`. The CLI prints a compact JSON pointer to stdout and the human report to stderr. `report.md` includes the gate decision, reasons, package and candidate SHA-256 values and an offline replay command. The package and bound report are replayed without RPC; the saved gate result is recomputed and compared with the saved JSON and Markdown. `--gate` on replay first verifies the saved report, then recomputes a different policy from the same analytical findings without altering the saved artifacts. Historical Milestone 8 reports still replay byte for byte and retain their original exit 4 for Incomplete unless `--gate` is explicitly supplied.

The validated package's exact `program.so` bytes flow through conversion and stress fixture construction into the LiteSVM `LoadedProgram` list. Immediately before execution each path verifies that the sole candidate program at the registered ID has the same bytes, loader and SHA-256. The registry fixes the ABI and program ID only. A changed binary, config or term changes the canonical package identity; saved evidence cannot be reused. The regression test `packaged_candidate_bytes_are_the_only_vm_program` substitutes a stale repository binary after a one-byte package mutation and checks that the VM boundary rejects it.

[`examples/ci/eplyx-preflight.yml`](../examples/ci/eplyx-preflight.yml) is a reusable GitHub Actions example. It assumes a Linux runner with Rust, `cargo-build-sbf`, Node and access to a bounded read-only `SOLANA_RPC_URL`. Set the repository variable `EPLYX_PACKAGE_TEMPLATE` to the manifest/config template path. It builds SBF, assembles and validates the package, runs `block-only`, replays without RPC, writes the Markdown Job Summary and uploads the package plus result directory even when the gate blocks. Fork PRs are skipped because this example runs checked-out code on a runner with an RPC secret; run it for trusted PRs or release candidates. The example is not installed as an active repository workflow.
