# Eplyx local CLI quick start

## Install Eplyx

```sh
# macOS (Apple Silicon) / Linux (x86_64)
curl -fsSL https://github.com/thomasdevving/eplyx-stock-transition-assurance/releases/latest/download/install.sh | sh
```
```powershell
# Windows (x86_64)
irm https://github.com/thomasdevving/eplyx-stock-transition-assurance/releases/latest/download/install.ps1 | iex
```

The installer verifies the release archive's SHA-256 before installing the single
`eplyx` binary into a user-owned directory, and prints a PATH command if needed.
Check the install with `eplyx --version` (or `eplyx version --json` in CI). See the
[README](../README.md#install) for manual downloads and checksum verification.

## Use it in your Solana project

```sh
cd my-solana-project
eplyx init
# Edit eplyx.toml: public mints, owner, source token account, proposed reserve and terms.
cargo build-sbf
export SOLANA_RPC_URL='https://your-mainnet-provider.example'
eplyx doctor
eplyx preflight
eplyx search
eplyx dashboard
eplyx runs
eplyx reproduce cx_<id>
```

Eplyx itself needs no Rust, Node or Eplyx checkout. `cargo build-sbf` builds your
own candidate program with your Solana toolchain. `eplyx doctor` lists what Eplyx
needs (the installed binary, `eplyx.toml` and a read-only mainnet RPC) separately
from your project (the candidate `.so`). `eplyx doctor --offline` skips the RPC check.

`eplyx init` creates `eplyx.toml` and `.eplyx/` and ignores the local store in Git. It detects a single `target/deploy/*.so` program when one exists. It leaves addresses blank. `--force` is required to replace a config; `--minimal` omits example invariants.

The config uses `[project] name`, `[transition] source_mint`, `replacement_mint`, `adapter = "fixed_ratio_conversion_v1"`, and `effective_at` (RFC 3339); `[program] path`; `[terms] numerator`, `denominator`, `rounding`, and `fee_bps`; `[execution] public_owner`, `source_account`, optional `amount_decimal`, and `reserve_funded_replacement_raw`; zero to 16 `[[invariants]]` with the package's existing `type`, `severity`, and optional `path`; and `[gate] policy = "block-only"` or `"strict"`. All addresses are public Solana addresses. The candidate path must remain under the project root, including after symlink resolution. No RPC URL, key, instruction, executable command, or claimed proof belongs in the config.

The CLI compiles this config into the existing canonical package format. The fixed adapter also fixes its program ID. Package identity comes from the engine's validated manifest, config, and exact SBF bytes. `.eplyx/runs/run_<UTC timestamp>_<program hash>/package/` holds those immutable inputs; `result/` holds the report, read-only captures, frozen plans, and offline replay inputs; `search/` holds bounded search output. Metadata is local and contains hashes, timestamp, Git commit/branch/dirty state where available, the selected gate outcome and `run_source` (`local`, or `ci` when `CI` is set). RPC credentials are never saved. `.eplyx/counterexamples/cx_<digest>.json` references a parent run and search digest. `.eplyx/project.json` gives the local project a stable ID; `cache/` is for temporary validation.

`eplyx search` creates a fresh preflight and then searches its exact state, including bounded additional read-only observed waves. `eplyx search --run <run-id>` replays and checks the saved run against the current config and SBF bytes before searching it. `--offline` skips additional observed waves; without `--run`, the fresh preflight still requires RPC. Search never substitutes a different run or failed case. `eplyx reproduce <cx-id>` removes the RPC environment and re-executes the entire saved search in the local VM, checking the selected counterexample and its failure signature. It requires no RPC or current config, and records each attempt, successful or not, under `.eplyx/reproductions/`. `eplyx runs`, `eplyx show` and `eplyx dashboard` also read the local store without requiring the current config. All capture remains read-only and no mainnet funds move.

## Local dashboard

After `eplyx preflight` and `eplyx search`, run:

```sh
eplyx dashboard            # opens http://127.0.0.1:4173 (or the next free port)
eplyx dashboard --no-open  # print the URL only; --port <n> picks a port
```

The dashboard is a read-only view of `.eplyx/`, which remains the local source of run history. It shows project status, PASS / WARN / BLOCK history, run details, counterexamples, run-to-run comparison, production-state, invariant and gate summaries, and local usage counts. Runs stay on this machine. It needs no account, sends nothing anywhere, reads no RPC URL and works without the current config once runs exist. It cannot change terms, invariants, policy or artifacts. Replay and reproduction still happen through the CLI; the dashboard only shows copyable commands. Its only write is the rebuildable summary cache `.eplyx/cache/dashboard-index.json`. See [Milestone 16](on-demand-milestone-16-dashboard.md).

Exit codes: preflight `0` for gate pass or warnings, `3` for gate block, `2` for invalid input or engine failure. Search returns `0` when completed even if it finds a counterexample, otherwise `2`. Reproduce returns `0` when verified, otherwise `2`.

Only the registered fixed-ratio candidate adapter is supported. A proven candidate conversion is an exact local execution result under the declared authority model. OfficialTransition remains NotTested; issuer authorization, key possession, and population-wide readiness are separate questions. A bounded search with no finding states only its recorded domain and budget.

## Contributor / development builds

Build the CLI from this repository instead of installing a release:

```sh
cargo build --release --locked -p eplyx-lifecycle-impact --bin eplyx
export PATH="$(pwd)/target/release:$PATH"
```

A source-built binary and a release binary read and write the same `.eplyx/`
formats. Release packaging, checksums and installer tests are described in
[Milestone 17](on-demand-milestone-17-release.md).
