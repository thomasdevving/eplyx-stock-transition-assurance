# Phase 4: offline lifecycle consequences

`STATE₀ + CHANGE + CONSEQUENCE MODEL → STATE₁ → DIFF` now has a production
lifecycle implementation. `STATE₀` is a validated Phase 2 snapshot, optionally
including the Phase 3 graph. `LifecycleChange` remains a change description;
`AssetLifecyclePolicy` supplies explicit entitlement semantics. The
`LifecycleConsequenceEvaluator` applies that policy to two time views of the
same immutable observations. No transaction executes and no network is needed.

`ChangeScenario::compare_lifecycle` dispatches this model. `ProgramUpgrade` still
uses `compare_fixture`, fresh VMs and the original differential execution path.
A description alone still cannot execute against synthetic lending fixtures.
The original scenario and upgrade tests retain their assertions.

## Policy format

See [the complete scenario](../scenarios/spacex-transition.json). JSON is the
repository's existing specification format. The policy is generic:

```json
{
  "asset_mint": "<Solana mint address>",
  "effective_at": "2026-09-17T20:00:00Z",
  "before": "Active",
  "after": "TransitionRequired",
  "deadline": {
    "at": "2027-03-12T23:59:00Z",
    "after": "NoIssuerEntitlement"
  },
  "successor": null
}
```

Times use RFC3339 and normalize to UTC. `effective_at` and `deadline.at` are
inclusive boundaries. Before the effective boundary the first status applies;
at/after the deadline its terminal status applies. `Unknown` is supported at
every phase. The optional terminal statuses are `Expired` and
`NoIssuerEntitlement`; neither asserts market value or technical exitability.
Without a deadline, the post-effective status continues. A successor is optional
declarative identity, never a conversion route; the SPACEX demo supplies none.

Every scenario records an ID, version, capture timestamp, description and sources.
`ExternalPolicy` and `ScenarioAssumption` remain distinct kinds. Sources bind to
policy JSON pointers, and every applicable policy field needs provenance.
Duplicate IDs, missing bindings, invalid addresses, unknown policy fields and
reversed deadlines fail. Local source artifacts have SHA-256 digests; the CLI
checks those bytes when loading the scenario. Artifact paths are relative to
the scenario file. Capture timestamps do not establish historical validity.
Hashes bind reproducible bytes; they do not authenticate an issuer or an RPC.

## Consequence rules

| Observed public account amount | Applicable policy | Impact |
| --- | --- | --- |
| Zero, no confidential account extension | Any | `Unaffected` |
| Zero with confidential account extension | Any | `Unresolved` |
| Positive | `Active` | `Unaffected` |
| Positive | `Unknown` | `Unresolved` |
| Positive, ordinary account | `TransitionRequired` | `RequiresTransition` |
| Positive, proven liquidity vault | `TransitionRequired` | `StaleExposure` |
| Positive | `Expired` / `NoIssuerEntitlement` | `StaleExposure` |

Unknown holder roles remain Unknown. A positive amount can be subject to a
transition requirement without identifying its owner's economic role. Other
program-owned authorities are never promoted to liquidity vaults without the
Phase 3 proof. Wallet compatibility establishes an authority category, not human
ownership or signing access. `role_uncertain` identifies an account without
wallet compatibility or a verified integration; it is not human identity proof.

Positive public amounts with confidential account state remain quantifiable
public exposure, with additional encrypted exposure explicitly unquantified.
Zero public amounts with such state do not establish zero total exposure and
do not contribute to the count of confirmed economic meaning changes.
Withheld fees and encrypted quantities are excluded from all public amount sums.
`economic_meaning_changed` requires a positive observed public amount and a
change in the applicable lifecycle status. Asset-wide policy status can change
even for zero accounts, without counting those accounts as affected capital.

The model has no `Stranded` or successful execution variant. Every impact has
`execution_status: NotTested`. Expiration makes positive amounts stale, rather
than implying a deadline can still be met or any recovery route works.

## Evidence and observation boundaries

Entity impacts retain the Phase 2 entity ID, original classification, amount,
authority observation/reference and token/mint proof. Proven roles attach the
Phase 3 refinement evidence without replacing original classifications.
`onchain_evidence` contains RPC pointers, slots, decoders and raw byte hashes;
`lifecycle_evidence` contains policy source IDs and field bindings. The full
scenario and its canonical digest are embedded once in the report. References
resolve through that scenario or the separately retained production snapshot.

Entity balances use `Phase2TokenAccount`. Protocol observations use
`Phase3VerifiedVault` and reference the same entity. The SPACEX vault is
678,320,992 raw in the earlier account enumeration and 682,216,229 raw in the
later six-account protocol verification. Both measurements remain frozen.
Protocol amounts overlap entity holdings; never sum them as additional capital.
The two contexts do not constitute a single atomic historical world.

`before` and `after` carry exactly the same canonical snapshot SHA-256, different
evaluation times and their applicable policy statuses. `diff.technical_state`
is `Unchanged` by construction: no bytes are written or simulated. This does not
claim real chain state remains unchanged between dates. Every impact retains
pre/post lifecycle statuses and classifications, with a reason. Deterministic
ordering and exact string amounts avoid float/JSON precision drift. A report's
`validate(snapshot)` recomputes every record, proof, hash, diff and aggregate.

## Offline CLI

```sh
# Before: Active. Default baseline is clamped to the requested earlier time.
cargo run --locked -q -p eplyx-lifecycle-impact -- impact \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json --at 2026-09-17T19:59:59Z

# Explicit before/after comparison, without RPC or snapshot mutation.
cargo run --locked -q -p eplyx-lifecycle-impact -- impact \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --before 2026-09-17T19:59:59Z --at 2026-09-17T20:00:00Z

# After the specified expiration cutoff.
cargo run --locked -q -p eplyx-lifecycle-impact -- impact \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json --at 2027-03-13T00:00:00Z
```

`--format json` emits the complete report. `--out NEW_PATH` saves complete JSON
while retaining the selected stdout format. Existing report files cannot be
overwritten. Without `--before`, the baseline is the earlier of `--at` and one
nanosecond before the effective boundary; explicit reversed times fail.
Schema-1 snapshots remain supported, with no invented protocol classification.
Snapshot loading first replays the frozen raw evidence and protocol proofs.

The SPACEX boundary is a demo assumption. The
[official issuer notice](https://prestocks.com/spacex) supplies transition and
expiration assertions; the specification separately records how its minute-level
cutoff is interpreted. The captured source is part of the offline artifact set.
No IPO history, successor validity, executable program/source equivalence, USD
valuation, transferability, withdrawal, swap, migration or exitability is proved.
