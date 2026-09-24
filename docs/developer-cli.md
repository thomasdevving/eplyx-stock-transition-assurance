# Eplyx local CLI quick start

Build the CLI from this repository and make `eplyx` available on your `PATH`:

```sh
cargo build --release -p eplyx-lifecycle-impact --bin eplyx
export PATH="$(pwd)/target/release:$PATH"
```

In a Solana project, run:

```sh
eplyx init
# Edit eplyx.toml: public mints, owner, source token account, proposed reserve and terms.
cargo build-sbf
export SOLANA_RPC_URL='https://your-mainnet-provider.example'
eplyx doctor
eplyx preflight
eplyx search
eplyx runs
eplyx reproduce cx_<id>
```

`eplyx init` creates `eplyx.toml` and `.eplyx/` and ignores the local store in Git. It detects a single `target/deploy/*.so` program when one exists. It leaves addresses blank. `--force` is required to replace a config; `--minimal` omits example invariants.

The config uses `[project] name`, `[transition] source_mint`, `replacement_mint`, `adapter = "fixed_ratio_conversion_v1"`, and `effective_at` (RFC 3339); `[program] path`; `[terms] numerator`, `denominator`, `rounding`, and `fee_bps`; `[execution] public_owner`, `source_account`, optional `amount_decimal`, and `reserve_funded_replacement_raw`; zero to 16 `[[invariants]]` with the package's existing `type`, `severity`, and optional `path`; and `[gate] policy = "block-only"` or `"strict"`. All addresses are public Solana addresses. The candidate path must remain under the project root, including after symlink resolution. No RPC URL, key, instruction, executable command, or claimed proof belongs in the config.

The CLI compiles this config into the existing canonical package format. The fixed adapter also fixes its program ID. Package identity comes from the engine's validated manifest, config, and exact SBF bytes. `.eplyx/runs/run_<UTC timestamp>_<program hash>/package/` holds those immutable inputs; `result/` holds the report, read-only captures, frozen plans, and offline replay inputs; `search/` holds bounded search output. Metadata is local and contains hashes, timestamp, Git commit/branch/dirty state where available, and the selected gate outcome. RPC credentials are never saved. `.eplyx/counterexamples/cx_<digest>.json` references a parent run and search digest. `.eplyx/project.json` gives the local project a stable ID; `cache/` is for temporary validation.

`eplyx search` creates a fresh preflight and then searches its exact state, including bounded additional read-only observed waves. `eplyx search --run <run-id>` replays and checks the saved run against the current config and SBF bytes before searching it. `--offline` skips additional observed waves; without `--run`, the fresh preflight still requires RPC. Search never substitutes a different run or failed case. `eplyx reproduce <cx-id>` removes the RPC environment and re-executes the entire saved search in the local VM, checking the selected counterexample and its failure signature. It requires no RPC or current config. `eplyx runs` and `eplyx show` also read the local store without requiring the current config. All capture remains read-only and no mainnet funds move.

Exit codes: preflight `0` for gate pass or warnings, `3` for gate block, `2` for invalid input or engine failure. Search returns `0` when completed even if it finds a counterexample, otherwise `2`. Reproduce returns `0` when verified, otherwise `2`.

Only the registered fixed-ratio candidate adapter is supported. A proven candidate conversion is an exact local execution result under the declared authority model. OfficialTransition remains NotTested; issuer authorization, key possession, and population-wide readiness are separate questions. A bounded search with no finding states only its recorded domain and budget.
