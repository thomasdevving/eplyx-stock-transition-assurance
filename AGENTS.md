# Eplyx Lifecycle Impact

This is a standalone Stocklana hackathon repository, separate from the original
Eplyx program-upgrade product.

## Scope

The core model is state + change + consequence model → diff / impact.
Phase 1 supports ProgramUpgrade and a description-only LifecycleChange.
Lifecycle execution must explicitly report that no consequence model exists.

Keep development focused on this project. Do not restore the old hosted API,
CI bundles, mainnet discovery/acquisition, protocol integrations or website
unless the user explicitly requests them.

## Architecture

- engine/src/scenario.rs separates the change from state inputs.
- engine/src/types.rs and executor.rs carry state and execution primitives.
- diff.rs compares observable results.
- money.rs provides integer fixed-point values.
- corpus.rs, interpret.rs, impact.rs, cluster.rs, shrink.rs and report.rs retain
  synthetic lending regression/demo behavior, not lifecycle domain semantics.
- interface/ and programs/fixture-lending/ are the synthetic harness only.

No path dependency may point outside this repository. The sole working branch
is main. Refer to docs/lifecycle-phase-1-architecture.md for the exact scope.

## Validation

make test generates fixture states, builds synthetic SBF programs and runs the
relevant unit and integration tests. Also run make fmt-check and make lint.
Missing program artefacts remain failures; never silently skip execution tests.
Report checks as pending until they have actually been executed.
