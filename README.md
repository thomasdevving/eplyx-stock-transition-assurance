# Eplyx

Eplyx tests a Solana token-transition program against current production state
before you ship it. It runs on your machine: read-only capture, local execution,
local results.

## Install

**macOS (Apple Silicon) and Linux (x86_64):**

```sh
curl -fsSL https://github.com/thomasdevving/eplyx-stock-transition-assurance/releases/latest/download/install.sh | sh
```

**Windows (x86_64, PowerShell):**

```powershell
irm https://github.com/thomasdevving/eplyx-stock-transition-assurance/releases/latest/download/install.ps1 | iex
```

The installer picks your platform's archive and checks its SHA-256 against the
release `SHA256SUMS` before installing anything. It installs one file:
`~/.local/bin/eplyx` on macOS/Linux, or
`%LOCALAPPDATA%\Programs\eplyx\bin\eplyx.exe` on Windows. It needs no
sudo or Administrator rights and never edits your shell profile, PATH or
registry; if the directory is not on your PATH it prints the command to add it.
Set `EPLYX_INSTALL_DIR` to choose another directory, or `EPLYX_VERSION=v0.1.0`
to pin a release.

**Manual download:** take the archive for your platform from
[GitHub Releases](https://github.com/thomasdevving/eplyx-stock-transition-assurance/releases),
together with `SHA256SUMS`, and verify it before extracting:

```sh
shasum -a 256 -c SHA256SUMS --ignore-missing       # macOS
sha256sum -c SHA256SUMS --ignore-missing           # Linux
```
```powershell
(Get-FileHash .\eplyx-v0.1.0-windows-x86_64.zip -Algorithm SHA256).Hash   # compare with SHA256SUMS
```

Each archive contains only the `eplyx` binary. Supported builds:
`darwin-arm64`, `linux-x86_64` (glibc 2.35 or newer) and `windows-x86_64`.
Other platforms can build from source (see below).

## Use it in your Solana project

```sh
eplyx --version
cd my-solana-project
eplyx init                 # writes eplyx.toml and .eplyx/
cargo build-sbf            # your own program build, as usual
export SOLANA_RPC_URL='https://your-mainnet-provider.example'
eplyx doctor
eplyx preflight
eplyx search
eplyx dashboard
```

Eplyx itself needs no Rust, Node or Eplyx checkout; only building your own
candidate program uses your Solana toolchain. See the
[developer quick start](docs/developer-cli.md).

**What stays local.** Installing or running Eplyx creates no account, contacts
no Eplyx service, sends no telemetry and uploads no run artifacts, source code or
candidate binary, unless you opt in to the team sync below. The only network use is the read-only Solana RPC you set in
`SOLANA_RPC_URL` for `doctor`, `preflight` and `search`; it never receives your
candidate program, which executes only in the local VM. Runs, counterexamples and
reproduction history stay in each project's `.eplyx/`. To uninstall, delete the
installed binary.

## Optional team sync

Cloud sync is optional. Eplyx execution stays local.

```sh
eplyx login        # approve a code in your browser; the CLI never asks for a password
eplyx link         # bind this project to one cloud project
eplyx sync         # upload complete runs, counterexamples and reproduction records
```

A hosted Eplyx workspace then shows local and CI run history, counterexamples,
reproduction history and run comparisons to your team. Each run's metadata,
engine report, bindings, package manifest and config, and search result are sent
as exact bytes with their SHA-256. Source code, the candidate `.so`, captures,
`eplyx.toml`, RPC URLs, environment variables and local paths are never sent.
`eplyx sync --dry-run --json` prints exactly what would go. Synced runs are
immutable, re-syncing is idempotent, and viewing them never reruns anything.
Preflight, search, reproduce, the dashboard and the CI gate never need the cloud.
See [Milestone 18](docs/on-demand-milestone-18-cloud.md).

**Build from source (contributors):**
`cargo build --release --locked -p eplyx-lifecycle-impact --bin eplyx`.
Releases are produced by [.github/workflows/release.yml](.github/workflows/release.yml);
see [Milestone 17](docs/on-demand-milestone-17-release.md).

---

> **Prospective on-demand pre-flight:** Open `/analysis#analysis`, select a catalogue
> asset or custom mint, and fetch a public wallet. Focus an account, optionally run
> fresh local Transfer / supported market-exit checks, then prepare a hypothetical
> replacement-token transition with normal form controls. Compare unchanged current
> bytes before and after the proposed effective time. Mobility assurance covers exact
> tested paths; full transition readiness separately requires conversion proof.
> No funds move or wallet signing occurs. The saved historical example stays separate.
> See [milestone 5 progress, evidence and start instructions](docs/on-demand-progress.md).

# Eplyx Lifecycle Impact

## Operator transition packages

The `eplyx-lifecycle preflight ./transition-package` CLI validates a strict
registered fixed-ratio candidate package, captures fresh read-only current
state, executes the candidate in a bounded local VM and runs the existing
production stress test. It writes `report.json` and `report.md`; use
`replay-package-preflight` to reproduce the result offline. The package is
OperatorSupplied, and even a Proven candidate conversion leaves
OfficialTransition NotTested. See the [Milestone 8 package contract](docs/on-demand-milestone-8-package.md)
and [example packages](examples/transitions/demo-fixed-ratio).

## Stock Transition frontend

The frontend adapts the original Eplyx visual style with a blue theme, the original
extruded logo, and three orbiting stones per input/capability view. Its landing
page and read-only evidence viewer describe this project's frozen SPACEX
demonstration. Published readiness remains **Incomplete** and OfficialTransition
remains **NotTested**.

```sh
npm ci
npm run dev
# Open http://localhost:4173
```

`npm run build` produces `dist/`; `npm start` serves that production build.
`npm run check:frontend` checks JavaScript and the pinned report digests.
`npm run test:frontend` runs desktop/mobile browser checks using installed Google
Chrome. See [frontend scope and maintenance](docs/stock-transition-frontend.md).
The viewer reads published reports; it does not execute the engine or connect to
an RPC service. Existing engine workflows and historical evidence stay separate.

Phase 10 proves **native liquidity-principal Withdrawal** for one real SPACEX
Meteora DLMM PositionV2 under its explicitly assumed original owner signer.
The deployed program removes every captured position liquidity share, credits
21,190,323 raw SPACEX and 8,132,564 raw USDC, and retains accrued fees and the
position account. No mainnet transaction is submitted. OfficialTransition remains
NotTested; this unwind does not prove issuer conversion or redemption.
See the [Phase 10 production report](docs/lifecycle-phase-10-production-report.md)
and [position/withdrawal guide](docs/lifecycle-phase-10-withdrawal.md).

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- discover-position \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --discovery evidence/withdrawal/phase10/position-discovery.json \
  --fixture evidence/withdrawal/phase10/execution-capture.json

cargo run --locked -q -p eplyx-lifecycle-impact -- probe-withdrawal \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --discovery evidence/withdrawal/phase10/position-discovery.json \
  --position probes/spacex-dlmm-position.json \
  --fixture evidence/withdrawal/phase10/execution-capture.json
```

Both commands are offline. Add `--format json --out <new-path>` for canonical
JSON; existing outputs are protected. [State-shape comparison](reports/spacex-lifecycle-state-shape-comparison.json)
keeps the canonical direct-holder evidence and this LP withdrawal independent.


Phase 9 reconstructs the public official-transition boundary for the same
Phase 8 holder. It verifies the current published successor's full Token-2022
configuration, inspects 31 real transaction samples across six addresses, and
records observed program/account/PDA relationships. Routed swaps, source burn,
successor administrative minting and fee withdrawal do not establish an
issuer-bound official plan for that holder. **OfficialTransition remains
NotTested**; no official VM transaction or issuer signature was invented.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- investigate-transition \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --entity 741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs \
  --research probes/spacex-official-transition-research.json
```

The command is offline and accepts `--format json`, `--out <new-path>`,
`--out-resolution <new-path>` and `--out-discovery <new-path>`. See the
[Phase 9 result](docs/lifecycle-phase-9-production-report.md) and
[mechanism evidence guide](docs/lifecycle-phase-9-transition.md).

A standalone Stocklana hackathon project for evaluating how a proposed asset
lifecycle change affects existing on-chain state.

The core model is:

```text
STATE₀ + CHANGE + CONSEQUENCE MODEL → STATE₁ → DIFF / IMPACT
```

Phase 1 establishes the boundary between a change and its state inputs:

- `ProgramUpgrade` executes the existing differential model.
- `LifecycleChange` separates the change description from its state and policy inputs.
- Account snapshots, state diffs and integer economic values form the reusable core.

Phase 2 captures real Solana token-asset production state: mint configuration,
Token-2022 extensions, every enumerated token account, conservative authority
classification and full raw RPC evidence. Frozen JSON snapshots can be verified
and loaded offline.

[Production discovery architecture and CLI](docs/lifecycle-phase-2-discovery.md)

[Verified SPACEX production capture and validation report](docs/lifecycle-phase-2-production-report.md)

Phase 3 attaches a production exposure graph using one evidence-driven Meteora
DLMM adapter. Pool and reserve PDAs, paired mints, vault authorities and public
balances are verified on-chain and linked to unchanged Phase 2 entities.
Schema-1 snapshots remain loadable; enriched snapshots use explicit schema 2.

[Exposure architecture and CLI](docs/lifecycle-phase-3-exposure.md) ·
[Verified DLMM exposure and validation report](docs/lifecycle-phase-3-production-report.md)

Phase 4 applies a generic external lifecycle policy offline to the same frozen
world before and after a chosen semantic boundary. On-chain evidence and issuer
policy/scenario assumptions remain separate. Positive accounts require transition;
verified liquidity exposure becomes stale. Transaction execution remains `NotTested`.

```sh
cargo run --locked -p eplyx-lifecycle-impact -- impact \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --before 2026-09-17T19:59:59Z --at 2026-09-17T20:00:00Z
```

The demo boundary is an explicit assumption, not a historical IPO timestamp.
The issuer notice and a separately stated cutoff interpretation provide deadline
semantics. `--at 2027-03-13T00:00:00Z` evaluates issuer-entitlement expiration.
No RPC is used by `impact`. `--out` saves a new deterministic JSON report.

[Lifecycle policy, consequence model and CLI](docs/lifecycle-phase-4-consequence.md) ·
[Production lifecycle proof, counts and validation](docs/lifecycle-phase-4-production-report.md)

Phase 5 executes one actual Token-2022-aware DLMM secondary-market exit in a fresh
local LiteSVM against captured deployed bytecode and route accounts. The selected
holder spends 10,000 raw SPACEX and receives 6,065 raw USDC; withheld transfer fees
and DLMM fees reconcile against account/bin deltas. Signer access is explicitly
assumed locally. Official successor transition remains `NotTested`.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- probe \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --probe probes/spacex-usdc-dlmm-exit.json
```

The replay uses no RPC and sends no mainnet transaction. Captured execution
accounts use one Phase 5 bank context, separately linked to earlier snapshots.

[Execution-probe architecture and CLI](docs/lifecycle-phase-5-exitability.md) ·
[Production execution evidence and complete report](docs/lifecycle-phase-5-production-report.md)

Phase 6 joins the full Phase 4 population with independently replayed execution
matrices, selects deterministic account-class representatives, and reports
`Proven`, `PartiallyProven`, `Untested` or `Unsupported` evidence coverage.
A proof is conditional on the captured local bank and assumed original signer.
Amounts and alternative venues/paths use a maximum per entity, never added pool
capacity. Official transition remains `NotTested`; samples never cover peers.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- coverage \
  --snapshot snapshots/spacex-exposure.json \
  --impact reports/spacex-transition-impact.json \
  --plan probes/spacex-lifecycle-coverage-plan.json
```

`coverage-plan` creates a matrix from repeated `--probe` routes and
`--amounts-raw`; `coverage --format json --out <new-path>` saves the complete
portfolio assurance report. All processing is offline. The checked-in matrix
executes multiple real amounts on the one captured DLMM venue; other paths and
uncaptured venues receive no positive execution assurance.

[Coverage engine, classifications and CLI](docs/lifecycle-phase-6-coverage.md) ·
[Portfolio assurance result and validation](docs/lifecycle-phase-6-production-report.md)

Phase 7 selects immutable probes from explicit assurance gaps, captures bounded
production routes read-only, and executes independent multi-entity DLMM and
Token-2022 transfer matrices locally. It records exact marginal coverage gains
with content-addressed evidence. Transfer amount coverage proves token movement;
secondary-market and official-transition coverage remain separate.

[Expansion planner, scoring and staged CLI](docs/lifecycle-phase-7-expansion.md) ·
[Before/after production assurance and validation](docs/lifecycle-phase-7-production-report.md)

Phase 8 resolves five lifecycle action paths for the one real small source.
Its exact Transfer and secondary-market executions remain conditional proofs;
the independently researched official conversion is NotTested, Redemption is
Unsupported, and direct-holder Withdrawal is NotApplicable. Discovery cannot
promote execution status or infer issuer-defined completion.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- resolve-paths \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --entity 741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs \
  --coverage reports/spacex-lifecycle-coverage-phase7.json \
  --discovery probes/spacex-lifecycle-path-discovery.json \
  --evidence-bundle probes/spacex-phase8-evidence-bundle.json
```

Replay is entirely offline; add `--format json --out <new-path>` for canonical
JSON. No new wallet/venue campaign or mainnet transaction occurs.

[Path resolution architecture and CLI](docs/lifecycle-phase-8-resolution.md) ·
[One-entity matrix, issuer investigation and validation](docs/lifecycle-phase-8-production-report.md)

The synthetic lending program is a regression harness for the engine. Its
positions, liquidation math and USD aggregation are test/demo semantics; they
are not an asset lifecycle implementation.

This repository has one branch, `main`, and no dependency on the original Eplyx
repository. Hosted CI, web UI, historical acquisition, global integration indexing
and stake-pool integrations are outside this project.

[Architecture, retained files and scope](docs/lifecycle-phase-1-architecture.md)

## Test at home

With Rust stable and the Solana/Anza SBF toolchain on PATH:

```sh
make test
make fmt-check
make lint
```

`make test` generates the deterministic fixtures, builds the two synthetic
program versions and runs the relevant program and engine tests. Generated
fixtures and compiled binaries stay ignored by Git.

The CLI is `eplyx-lifecycle`; the Rust package is `eplyx-lifecycle-impact`:

```sh
cargo run --locked -p eplyx-lifecycle-impact -- list
make compare
make report
```

For production capture and offline replay, see the Phase 2 guide linked above.

## Phase 11: lifecycle readiness gate

The explicit `stocklana-spacex-preflight-v1` demonstration policy evaluates to
**Incomplete**. Exact direct-holder mobility and LP liquidity-principal withdrawal
are satisfied; official transition, retained fee/closure handling and population
coverage remain unresolved. This is not the issuer's policy or an asset safety
judgment. The gate consumes frozen measurements without RPC or new execution.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- readiness \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --policy policies/stocklana-spacex-preflight-v1.json \
  --direct-resolution reports/spacex-lifecycle-path-resolution-phase9.json \
  --position-resolution reports/spacex-dlmm-withdrawal.json \
  --coverage reports/spacex-lifecycle-coverage-phase7.json
```

Exit codes: Ready **0**, Blocked **3**, Incomplete **4**, invalid inputs/output
**2**. Add `--format json --out <new-path>` for canonical JSON; existing outputs
are protected. See [the gate contract](docs/lifecycle-phase-11-readiness.md),
[the production report](docs/lifecycle-phase-11-production-report.md) and
[the complete readiness artifact](reports/spacex-lifecycle-readiness.json).

## Phase 12: begin with the real issuer notice

One pinned historical PreStocks notice now produces a provenance-bound event
and lifecycle scenario offline. Published source/successor addresses are separately
checked against Phase 9 raw mint evidence. The notice allows SPCXx **or any other
token** and gives a deadline; it supplies no conversion program or ratio.
OfficialTransition remains NotTested. Evaluation time stays demo configuration.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- ingest-notice \
  --workflow probes/spacex-notice-workflow.json
cargo run --locked -q -p eplyx-lifecycle-impact -- scenario-from-event \
  --workflow probes/spacex-notice-workflow.json --format json
cargo run --locked -q -p eplyx-lifecycle-impact -- preflight-from-notice \
  --workflow probes/spacex-notice-workflow.json
```

The final command returns **Incomplete, exit 4** under the unchanged Phase 11 demo
assurance policy. Generated impact uses the new scenario. Existing exact path
matrices and gate proofs keep their original contexts and hashes after explicit
economic-policy compatibility checks. No network or VM runs in these commands.
Use `--format json --out <new-path>` to save canonical JSON; existing outputs are
protected. See [workflow and boundaries](docs/lifecycle-phase-12-notice.md) and
[production result](docs/lifecycle-phase-12-production-report.md).

## Phase 13: same bytes, new lifecycle consequences

Compare the same pinned production world at the explicit transition and deadline
boundaries without capture or execution:

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- compare-scenarios \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --readiness-policy policies/stocklana-spacex-preflight-v1.json
```

The 10,155 positive token-account balances stay unchanged. The policy changes
`Active` to `TransitionRequired` and then `NoIssuerEntitlement`; exact historical
Transfer, market sale and LP Withdrawal proof remains separate from official
conversion. `PreEvent` describes gate applicability; the applicable policy result
remains `Incomplete` even after the deadline.

[Canonical UI artifact](reports/spacex-counterfactual-lifecycle.json),
[model and replay](docs/lifecycle-phase-13-counterfactual.md), and
[production result](docs/lifecycle-phase-13-production-report.md).

## Rollout assumption validation (Phase 14)

Evaluate an explicit submitted demonstration candidate against existing pinned
evidence entirely offline:

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- evaluate-rollout \
  --plan probes/phase14-plans/lp-complete-exit.json
```

This case retains proven principal removal while rejecting complete-position exit;
the original population assurance remains Incomplete (exit 4). A separately
identified required exact failed-route variant yields Blocked (exit 3). A narrow
principal-removal-only control is Accepted/Ready for that entity/action (exit 0),
without replacing population readiness. Only the latter permits a temporary local
stub marker. Analysis-command exit 0 never authorizes the guard.

See [scope, trust, commands and exit contract](docs/lifecycle-phase-14-rollout.md),
the [production report](docs/lifecycle-phase-14-production-report.md), and the
[precomputed UI artifact](reports/spacex-rollout-assumptions.json).

## Consumer analysis workflow (Phase 15)

```sh
npm ci
npm run build:engine
npm run dev
```

Open http://127.0.0.1:4173/analysis#analysis to choose a catalogue asset or enter a Solana mint. The saved SPACEX demonstration remains a separate review choice.
The existing switch below the logo controls Overview / Technical detail across the
application. The same local server hosts the bounded job API and invokes the built
engine; static hosting alone supports only saved reports. No wallet or blockchain
transaction is involved. See [supported inputs, architecture and validation](docs/lifecycle-phase-15-production-report.md).
