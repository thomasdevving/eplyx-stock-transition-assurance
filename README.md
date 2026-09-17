# Eplyx Lifecycle Impact

A standalone Stocklana hackathon project for evaluating how a proposed asset
lifecycle change affects existing on-chain state.

The core model is:

```text
STATE₀ + CHANGE + CONSEQUENCE MODEL → STATE₁ → DIFF / IMPACT
```

Phase 1 establishes the boundary between a change and its state inputs:

- `ProgramUpgrade` executes the existing differential model.
- `LifecycleChange` is a description-only placeholder and explicitly refuses execution.
- Account snapshots, state diffs and integer economic values form the reusable core.

The synthetic lending program is a regression harness for the engine. Its
positions, liquidation math and USD aggregation are test/demo semantics; they
are not an asset lifecycle implementation.

This repository has one branch, `main`, and no dependency on the original Eplyx
repository. Hosted CI, web UI, RPC discovery, historical acquisition, Token-2022
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

Compilation, formatting and tests have not yet been run. Changes were prepared
and reviewed remotely; local validation is deferred until the user is home.
