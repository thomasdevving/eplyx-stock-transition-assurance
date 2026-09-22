# Milestone 8: operator transition packages

An operator can run a registered candidate conversion package without editing Eplyx source:

```sh
eplyx-lifecycle validate-transition-package examples/transitions/demo-fixed-ratio
eplyx-lifecycle preflight examples/transitions/demo-fixed-ratio --out reports/my-preflight
eplyx-lifecycle replay-package-preflight examples/transitions/demo-fixed-ratio --result reports/my-preflight
```

`preflight` chooses a timestamped output directory if `--out` is omitted. It validates the entire package before the first RPC request, captures a fresh current wallet and population, freezes a deterministic stress plan, captures its selected cases, and executes the registered conversion program in LiteSVM. The replay command performs no RPC; it rechecks package, config, program, wallet, conversion, population, plan and case digests and reruns every local VM conversion before comparing `report.json` and `report.md`.

## Package contract

`eplyx.json` has schema version 1 and rejects unknown or duplicate fields. It declares `sourceMint`, `replacementMint`, `adapter`, `candidateProgram` (`programId`, relative `artifact`, lowercase `sha256`), exact string integers in `terms` (`numerator`, `denominator`, `rounding`, `feeBps`), `effectiveAt`, a relative `config` path and `configSha256`. `config.json` contains `publicOwner`, `sourceAccount`, optional `amountDecimal`, and string integer `reserveFundedReplacementRaw`. The config and manifest cannot supply status, evidence, commands, account metas, transactions, programs outside the package or RPC endpoints. The current registered adapter is `fixed_ratio_conversion_v1` with the program ID and account/instruction ABI of Eplyx Demo Candidate Conversion. New candidate SBF builds under this adapter are identified by their own hash and must pass actual VM execution and exact reconciliation. Different ABIs need a separately registered adapter.

The identity `transition_package_sha256` hashes canonical parsed manifest data, exact program SHA-256, exact config SHA-256 and adapter/version. The config SHA-256 is also declared and checked. A changed term, binary or config changes or invalidates this identity. Package bytes remain OperatorSupplied and Proposed. Source mint, replacement mint, public owner and direct token account are independently re-inspected. The proposed reserve, configuration, candidate authority and program cannot be confused with captured accounts; the existing adapter rejects address collisions. No issuer authorization, signing-key possession or official transition follows from a valid package or successful local VM run.

## Bounds and decisions

The manifest and config are each limited to 16 KiB; the SBF artifact to 2 MiB. The loader requires 64-bit little-endian shared ELF with the supported BPF/SBF machine type; native host binaries are rejected and never run. Package members must resolve inside the canonical package root, including through symlinks. The existing bounded RPC and stress budgets apply; the candidate instruction has an explicit 1,400,000 compute-unit limit. The CLI itself does not impose a separate wall-clock timeout on LiteSVM, so that remains an operational boundary for an external job runner.

The declared pre-flight status is Ready only if the exact selected candidate conversion is Proven and ConversionStressReadiness is Ready. An actual failed conversion or a Blocked stress result gives Blocked; remaining cases give Incomplete. The full population gate and OfficialTransition remain separate. The original Milestone 8 CLI returned `0` Ready, `3` Blocked, `4` Incomplete and `2` for invalid packages or processing errors. Saved Milestone 8 reports retain those exit and offline replay semantics. New runs use the separate [Milestone 9 deployment gate](on-demand-milestone-9-ci-gate.md), with `block-only` as the default. `report.json` contains package/program/config identities, exact selected cases, conversion outcomes, unresolved/unsupported states, all four readiness/status axes, and `funds_moved=false`. `report.md` is a short companion summary. No funds move.

The examples are Eplyx demo candidates, never official PreStocks transitions. `demo-second-asset` exercises distinct asset identities and terms through the same path. `demo-underfunded` has a proposed reserve of zero and is meant to produce a local execution failure and Blocked finding.
