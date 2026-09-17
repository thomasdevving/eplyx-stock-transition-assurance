# Eplyx Lifecycle Impact — standalone Phase 1

## Identity and model

This is the independent Stocklana hackathon repository. Its Rust package is
`eplyx-lifecycle-impact`, library is `eplyx_lifecycle_impact` and CLI is
`eplyx-lifecycle`. All project code lives on `main`. There are no submodules,
external repository path dependencies, hosted endpoints or links needed to run
the project.

Selected engine code originated in `thomasdevving/Eplyx` at
`dcfc36bad0c77b3971a92de59c1c1dd053f540d6`. This is attribution, not a runtime
or repository dependency. The original Eplyx repository is unchanged.

```text
STATE₀ + ChangeScenario + execution/consequence model → STATE₁ → diff / impact
                  ├─ ProgramUpgrade: executable
                  └─ LifecycleChange: description only, execution unsupported
```

## Reusable boundary

`ChangeScenario` is an in-memory enum. `ProgramUpgrade` borrows baseline and
candidate binaries; the fixture and target program ID are separate inputs.
The legacy `compare_fixture` and `compare_all` helpers retain their behavior and
route through this scenario boundary.

Execution constructs a fresh LiteSVM for each side. State, transaction and
environment inputs remain the same; only the program bytes vary. The diff
compares observable results, including account bytes, balances, errors, compute
and invocation shape. Economic values use integer arithmetic.

`LifecycleChange` contains a description, with no conversion ratio, deadline,
entitlement rule or evidence claim. Evaluating it fails before VM execution or
report construction. A later phase must define its state inputs, consequence
model, evidence and result schema.

The existing V1/V2 fixture/report schemas are retained for the upgrade model.
They are not presented as a generic lifecycle report schema.

## Regression harness, not lifecycle domain logic

The retained offline pipeline is:

```text
synthetic corpus → executor → diff → interpretation → economic impact
                                             → clusters → minimization → report
```

The lending corpus, position decoding, liquidation math and collateral valuation
are a synthetic harness proving differential execution and economic reporting.
Clustering and minimization remain because the existing economic regression
suite exercises them. They do not define what a lifecycle transition means.

`interface/` holds this harness's wire format and independent reference math.
`programs/fixture-lending/` builds its V1/V2 SBF binaries under a separate
workspace and lockfile. There is no production protocol integration.

## Removed inherited product features

The old hosted API/server, expectation review and CI bundle system, archive and
RPC ingestion, activity discovery/selection, historical replay orchestration,
Token-2022/stake-pool adapters, extra candidate programs, hosted/mainnet demos,
deployment/onboarding documents, TypeScript report checker and original agent
instructions have been removed. There is no web UI.

The former `compare_replay` scenario method and replay tests are removed with
historical orchestration. Historical baseline fidelity is not a capability
claimed by this trimmed Phase 1 project. No lifecycle or holder discovery was
implemented.

The CLI keeps only offline `compare`, `generate`, `reproduce` and `list`.
Workspace members, direct dependencies and build/test commands cover only the
retained engine and synthetic harness. Cargo.lock is reduced to the retained
dependency graph, preserving existing resolved registry versions/checksums.
Cargo must validate it during the deferred home checks.

## Exact repository contents

The current tree contains 37 relevant files:

```text
.gitignore
AGENTS.md
Cargo.lock
Cargo.toml
Makefile
README.md
docs/lifecycle-phase-1-architecture.md
engine/Cargo.toml
engine/src/cluster.rs
engine/src/corpus.rs
engine/src/diff.rs
engine/src/executor.rs
engine/src/hexfmt.rs
engine/src/impact.rs
engine/src/interpret.rs
engine/src/lib.rs
engine/src/main.rs
engine/src/money.rs
engine/src/numfmt.rs
engine/src/report.rs
engine/src/scenario.rs
engine/src/shrink.rs
engine/src/types.rs
engine/tests/scenario.rs
engine/tests/upgrade_diff.rs
fixtures/program-id.txt
interface/Cargo.toml
interface/src/lib.rs
programs/fixture-lending/Cargo.lock
programs/fixture-lending/Cargo.toml
programs/fixture-lending/src/error.rs
programs/fixture-lending/src/lib.rs
programs/fixture-lending/src/math.rs
programs/fixture-lending/src/processor.rs
rust-toolchain.toml
scripts/build-programs.sh
scripts/test-programs.sh
```

The cleanup replaces README/architecture/project instructions, renames the
engine package and CLI, trims workspace/dependencies/lockfile and commands,
removes replay dispatch, and updates test imports. The upgrade regression test
assertions are retained; its precision-source scan now covers only files that
remain in this repository. Scenario tests still compare direct VM execution
with the new and legacy APIs and reject unsupported lifecycle execution.

## Deferred home validation

No local files were created or used for implementation. Compilation, tests,
rustfmt and clippy have not been executed.

Remote inspection checked module references, repository-relative paths, the
retained lockfile dependency graph and the final GitHub file/branch state.
These checks do not substitute for compilation.

With Rust stable and the Solana/Anza SBF toolchain available, run on `main`:

```sh
make test
make fmt-check
make lint
```

`make test` generates the deterministic fixture states, builds V1/V2, runs
program unit tests for both versions and runs the Rust workspace tests. Missing
artefacts are failures, not silently skipped tests. The generator uses public
fixture IDs and roles for synthetic signing identities. Generated fixture
states are ignored rather than copied to GitHub.

Lifecycle execution remains intentionally unsupported after successful checks.
