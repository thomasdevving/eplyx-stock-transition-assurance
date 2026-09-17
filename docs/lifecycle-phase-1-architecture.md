# Eplyx Lifecycle Impact — Phase 1

## Scope and provenance

This repository starts from the backend of [thomasdevving/Eplyx](https://github.com/thomasdevving/Eplyx) at commit `dcfc36bad0c77b3971a92de59c1c1dd053f540d6`.
The baseline import is commit `ee9a35d17c09bf481000b8ec80772f4e3ebca282`.
The 109 imported files match their source Git blob hashes and file modes.

The baseline includes the Rust engine, interface, hosted API, existing tests,
program sources, dependency test binary, build scripts and backend reference
documents. It excludes the frontend, its root npm/pnpm configuration, visual
prompt documents and generated `fixtures/states/` files. The original Eplyx
repository is unchanged.

The fixture seeds are deterministic test identities derived from public fixture
IDs and roles in `engine/src/corpus.rs`. The automatic approval review rejected
copying their generated files to GitHub. The generator and tests are retained;
generate fixtures at home before running the suite.

Phase 1 introduces only an in-memory change abstraction and dispatch. No new
PreStocks logic, Token-2022 logic, UI, holder discovery or external integrations
are added. Existing protocol adapters and hosted API code are inherited.

## Existing architecture

The fixture route is:

```text
corpus → executor → diff → interpret → impact → cluster → shrink → report
```

Fixtures specify initial accounts and a transaction. Execution uses real SBF
bytecode in a fresh LiteSVM for each side. The fixture economic interpretation,
aggregation and minimization understand the fixture lending protocol.

The historical route is:

```text
versions + dependencies + screening → historical → replay
                                                   ↓
                                        executor → diff → ProtocolAdapter
```

Historical replay verifies the baseline byte hash and reproduces the recorded
outcome before executing the candidate. Dependencies, transaction, clock and
initial account state remain pinned. Protocol adapters interpret changes without
teaching the raw diff layer new account layouts.

## Where change meant program upgrade

| Location | Existing assumption | Phase 1 treatment |
| --- | --- | --- |
| `engine/src/lib.rs` | Fixture comparison takes V1 and V2 binaries. | Preserve the public function; delegate to a ProgramUpgrade scenario. |
| `engine/src/replay.rs` | Replay always compares two binaries. | Preserve the public functions; dispatch to the unchanged upgrade model. |
| `engine/src/executor.rs` | One execution loads one program build. | Keep as the upgrade execution primitive. |
| `engine/src/ci.rs` and `server/src/api.rs` | A candidate is a program artefact. | Keep the existing upgrade contract; it reaches scenario dispatch via replay. |
| `engine/src/main.rs` | CLI comparison accepts V1 and V2 paths. | Preserve flags and behavior. |
| `engine/src/diff.rs` and `engine/src/report.rs` | Results use V1/V2 and program artefact fields. | Preserve schemas; do not claim these are lifecycle result formats. |
| `engine/src/versions.rs` | Discovery resolves binaries and upgrade boundaries. | Remain upgrade-specific; not a lifecycle discovery layer. |

## Minimal new boundary

```text
state inputs + ChangeScenario
                    ├─ ProgramUpgrade → existing execution/replay → existing diff/report
                    └─ LifecycleChange → explicit unsupported error
```

`ChangeScenario::ProgramUpgrade` contains borrowed baseline and candidate
`ProgramVersion` references. It does not copy binaries or own state. A fixture
and program ID, or historical records and dependencies, are supplied separately.

`LifecycleChange` contains a description only. Both comparison entry points
reject it before any execution or report construction. It defines no conversion
ratio, deadline, entitlement or economic consequence. Future work must supply
appropriate state inputs, lifecycle semantics, evidence and result types.

The legacy `compare_fixture`, `compare_all`, `replay::compare` and
`replay::compare_with_dependencies` APIs keep their signatures. Fixture and
replay comparisons now pass through scenario dispatch. The historical upgrade
body remains in `replay::compare_program_upgrade_with_dependencies` with
crate-only visibility.

This is not a generic VM, protocol-adapter rewrite or universal report schema.
The existing fidelity gate and upgrade economic behavior remain in their
existing model.

## Exact Phase 1 files

These changes are relative to the imported backend baseline:

| File | Change |
| --- | --- |
| `engine/src/scenario.rs` | Add change types and fixture/replay dispatch. |
| `engine/src/lib.rs` | Export the scenario API and delegate legacy fixture comparison. |
| `engine/src/replay.rs` | Delegate legacy replay comparison; retain the existing upgrade body as a crate-only function. |
| `engine/tests/scenario.rs` | Compare scenario execution against direct VM execution and the legacy API; reject lifecycle execution. |
| `engine/tests/replay.rs` | Extend existing tests with scenario report parity and baseline-mismatch rejection through the new API. |
| `README.md` | Explain this backend-only Phase 1 repository and link this document. |
| `docs/lifecycle-phase-1-architecture.md` | Record architecture, scope, provenance, changed files and validation. |

No dependency versions, fixture format, report schema, CLI flags, protocol
adapters, hosted API routes or on-chain program sources change in Phase 1.

## Validation and home checklist

Compilation, formatting tools, lint and tests have **not been run**. The user
will run them at home. Remote checks verify imported blob hashes/modes and that
the existing historical upgrade body is preserved verbatim.

With Rust stable and the existing Solana/Anza SBF toolchain on PATH, use the
Phase 1 branch:

```sh
git switch phase-1-change-scenario
make fixtures
make test
make fmt-check
make lint
```

`make fixtures` generates the omitted fixture states using the unchanged
generator. `make test` builds the fixture V1/V2 and stake-pool candidate/reference
programs, runs program unit tests, and runs the Rust workspace suite. Missing
artefacts remain errors; tests are not skipped.

The existing upgrade suite asserts behavioral/economic classifications,
determinism, serialization, account metadata and minimized counterexamples.
The new scenario tests also check the known critical boundary fixture. The
extended replay tests check identical serialized report output and that a
baseline fidelity mismatch withholds candidate execution.

Existing mainnet demonstration commands still need their original archive
access and artefacts. They are outside Phase 1 and are not validation performed
by this change.
