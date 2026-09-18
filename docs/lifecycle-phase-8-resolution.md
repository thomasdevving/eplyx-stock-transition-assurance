# Phase 8 lifecycle path resolution

The generic `LifecyclePathResolver` joins one observed entity and external
lifecycle impact with typed mechanism discovery and independently replayed
execution evidence. It emits all five distinct paths: OfficialTransition,
Redemption, SecondaryMarketExit, Transfer and Withdrawal. It does not change
Phase 4 policy or Phase 6–7 portfolio classification rules.

## Status contract

| Status | Meaning |
| --- | --- |
| Proven | At least one listed exact entity/path/context/amount execution succeeded and reconciled |
| Failed | Listed bounded executions failed at captured state, with no established successful point |
| Indeterminate | Preconditions/state/runtime/reconciliation could not establish success/failure |
| Unsupported | Mechanism is outside supported execution/evidence capability; not non-existence |
| NotTested | Candidate or unresolved path has no established direct execution |
| NotApplicable | Path does not apply logically to this exact entity/context |

The generic model reuses the existing path enum without issuer/mint/ticker
conditionals. Discovery cannot specify Proven or Failed: its mechanism boundary
maps to NotTested, Unsupported or Indeterminate. Proven and Failed come through
the execution adapter. Direct wallet-compatible accounts with no verified
protocol/LP role receive Withdrawal NotApplicable; a venue vault cannot confer
withdrawal rights on that holder. A controlled protocol-role test stays
NotTested and does not resolve another real production entity.

## Evidence boundaries

The matrix keeps four distinct layers:

1. Observed entity/source/mint state, account RPC pointers, slots, hashes and authority classification.
2. Local deployed-program execution, exact debit/output/fees, capture state/Clock, execution assumptions and result digest.
3. External lifecycle policy/scenario, policy time and evidence-source IDs.
4. First-party mechanism communications, bounded research observations/inferences and unsupported issuer/backend/entitlement boundaries.

The schema-1 discovery manifest binds asset, original snapshot and original
scenario hashes. Each source has an explicit evidence kind, reference, local
artifact and digest. Loading checks every artifact. Unknown fields, boundaries,
duplicate paths/contexts and unbound claims fail closed. Research happens
outside the offline engine; resolution never scrapes sites or infers legal rights.

The schema-1 evidence bundle references the unchanged complete Phase 6 baseline,
Phase 4 impact, Phase 7 plan/inventory/capture manifest/execution index and delta.
Every referenced file is hash-checked. The adapter regenerates the original
selection, validates population/policy and report/index bindings, loads the
requested entity's original fixtures and fresh-replays its exact existing
matrix. It verifies complete evidence equality before projecting a result.
No new amount, holder selection or venue capture is performed. Other entities'
execution reports are not replayed or resolved in this Phase 8 flow.

`VerifiedExecution` has no public or deserialization constructor. A caller
cannot submit an arbitrary JSON status as verified proof. The replay adapter
requires real successful execution and economic reconciliation for Proven;
Failed requires an unsuccessful transaction and watched rollback. Controls
never grant positive assurance. Input hash consistency alone is insufficient:
a public test rehashes fabricated results, index and coverage copies, and still
fails fresh replay. Production artifacts remain unmodified.

## Exact scopes and mixed results

The resolver filters independently by entity and path, then by context. Each
point retains its exact input, source-before amount, captured state/fixture
identity, actual output mint/account/raw amount, transfer/DLMM/protocol fees,
Clock, signer flags, blocker or execution error, rollback and evidence digest.
Full VM logs/CPI/post-state remain in the original content-addressed files.

A row/context Proven status means at least one bounded successful point; it
does not erase failures or controls. A requested alternative context with no
point gets its own NotTested or capability-boundary status even if the path
has proof elsewhere. All successful points are listed; no amount interval,
max-to-every-smaller extrapolation or simultaneous capacity is asserted.

Transfer never proves OfficialTransition. A USDC swap never proves Redemption
or issuer-defined completion. Proof never moves to another source or pool.
Local signing retains `signer_possession_known=false` and
`signer_assumed_locally=true` from the original evidence. Authorization and
network inclusion are outside this local model.

Policy time remains the Phase 4 view and does not set VM Clock. The matrix
preserves Phase 4 baseline execution NotTested separately. A Phase 8 discovery
status NotTested or applicability status NotApplicable does not retroactively
rewrite the Phase 7 capability classification or coverage counts.

## Determinism and CLI

Rows have fixed path order. Sources, claims, contexts and measured points are
sorted deterministically. Reordered execution/discovery inputs produce identical
resolved rows. Provenance hashes continue to identify the specific bound input
files. JSON is canonical pretty printing with one newline and no generation
time. `LifecycleResolution::validate` recomputes through verified offline replay.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- resolve-paths \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --entity 741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs \
  --coverage reports/spacex-lifecycle-coverage-phase7.json \
  --discovery probes/spacex-lifecycle-path-discovery.json \
  --evidence-bundle probes/spacex-phase8-evidence-bundle.json
```

`--entity` accepts the raw account or complete entity ID. Text is the default;
`--format json` prints the full matrix. `--out <new-path>` saves canonical JSON
regardless of display format; JSON stdout and the saved file are byte-identical.
Existing output paths are rejected before loading or replay. Absolute input
paths work from another working directory; fixture/evidence references remain
relative to their manifests. No RPC option exists in `resolve-paths`.

The [production report](lifecycle-phase-8-production-report.md) records actual
statuses and research limits. Run `python3 scripts/test-phase8-mutations.py` on
an isolated writable checkout with new report/log paths to inject seven named
faults. It requires assertion failures, restores exact source bytes and refuses
to overwrite saved mutation evidence. Full prior phase assertions remain intact.
