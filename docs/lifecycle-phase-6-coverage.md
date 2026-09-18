# Phase 6 lifecycle coverage engine

`coverage/` joins a validated Phase 4 `LifecycleImpactReport` with captured
Phase 5 execution probes. It retains immutable population balances, validates the
embedded policy and every impact, binds snapshot/scenario/impact/plan hashes,
and executes supplied probes in fresh local VMs before counting evidence.
It makes no RPC call and submits no transaction.

The result is portfolio **evidence coverage under captured local assumptions**,
not a promise that a portfolio can exit. `OfficialTransition` remains `NotTested`.

## Inputs and deterministic CLI

Replay the checked-in population and matrix:

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- coverage \
  --snapshot snapshots/spacex-exposure.json \
  --impact reports/spacex-transition-impact.json \
  --plan probes/spacex-lifecycle-coverage-plan.json
```

Save a new complete JSON assurance report:

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- coverage \
  --snapshot snapshots/spacex-exposure.json \
  --impact reports/spacex-transition-impact.json \
  --plan probes/spacex-lifecycle-coverage-plan.json \
  --format json --out /tmp/spacex-coverage-replay.json
```

JSON stdout is identical to the saved file, including its trailing newline.
Existing outputs are refused before execution. Serialization adds no wall-clock
stamp; `evaluated_at` is the input policy view time. Entities, cases, classes and
aggregate keys have stable ordering. The full report supports deterministic
`CoverageReport::validate`, which re-evaluates the population and repeats execution.

Create a new representative/amount/path matrix:

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- coverage-plan \
  --snapshot snapshots/spacex-exposure.json \
  --impact reports/spacex-transition-impact.json \
  --probe probes/spacex-usdc-dlmm-exit.json \
  --amounts-raw 1000,10000,30000,60354,60355 \
  --out probes/new-coverage-plan.json
```

Repeat `--probe` to include additional captured holders or venues. Seeds retain
captured fixtures and fingerprints; generated exact-input cases change only
probe ID, input amount and the explicit amount rationale. The original seed is
included independently. Duplicate requested amounts are deduplicated.
Fixtures are resolved relative to the plan; sibling fixture references remain
portable. A fixture outside that directory receives a resolved absolute reference,
which must be adjusted when moving the plan. The checked-in plan uses relative
fixture and prior-report references only.

The checked-in plan additionally sets `execution_report` on the baseline case
to the original Phase 5 result. This optional prior result is accepted only if
it equals the complete fresh offline replay. It is never trusted because its
status says success. Replaying requires real fixtures/code even for prior results;
missing files, bad hashes, malformed amounts and tampering are errors.

## Plan, cases and representative classes

`CoveragePlan` names explicit requested paths and an exact population/policy
view. Each `CoverageCase` names an entity ID, path type, optional venue, positive
canonical raw input and optional inline `ExecutionProbeSpec`/prior result.
Inline specs must agree with case target, path, venue and amount. Their snapshot,
scenario and fixture fingerprints are independently checked by the probe engine.
Case IDs and probe IDs must be unique. Plan/report structures reject unknown fields.

Representative classes distinguish observed authority type, account state,
active delegation, confidential extension presence and verified protocol role.
Within each class, selection is deterministic: first zero balance, smallest
positive and largest positive public balance, ordered by raw balance and entity
ID. Duplicate selections are removed. Zero representatives are retained in the
selection report but receive no fabricated positive-input request.

Representative requests are scoped to their individual account. A sample does
not cover its class, prove an authority's identity, quantify encrypted balances,
or fabricate another account to make execution possible. Cases without captured
routes are `Untested` where the current model supports the path/authority type,
otherwise `Unsupported`. Selecting a representative is not executing it.

Every entity gets coverage for every explicitly requested path, even if no case
was selected. The supported execution slice currently remains the Phase 5
wallet-compatible, assumed-owner-signed Meteora DLMM `SecondaryMarketExit`.
`OfficialTransition`, `Redemption`, `Withdrawal`, `Transfer` and unsupported
source authority models receive explicit capability gaps. A secondary-market
venue without a supplied fixture is untested. No quote, guessed transaction,
unverified venue or invented adapter supplies execution assurance.

## Classification and amount accounting

| Classification | Exact meaning |
| --- | --- |
| `Proven` | Positive represented public balance has a successful full exact-input local witness on at least one requested path, with matching captured source balance, under retained signer/runtime assumptions. |
| `PartiallyProven` | A positive bounded exact-input witness covers less than the represented public balance. |
| `Untested` | No matching-balance positive successful witness, and at least one requested path is supported by the current authority/adapter model. Failed or indeterminate attempts remain explicit. |
| `Unsupported` | All requested paths lack support in the current authority/adapter model. This does not prove that the asset cannot exit. |

Path classification applies the same rules independently. An entity's
classification uses its largest measured amount across requested paths; it does
not mean every path is proven. In particular, secondary-market success cannot
prove official lifecycle transition. Zero public balances are never vacuously
`Proven`; confidential exposure remains outside quantified public amounts.

For each entity/path:

```text
largest_successful_input = max(exact successfully executed inputs)
represented_amount_covered = max(successful inputs with matching source balance)
represented_amount_without_evidence = Phase 4 public balance - covered amount
```

A successful case's actual debit must equal its requested input, its VM execution
must succeed, and protocol/token/event reconciliation must pass. The full report
retains every case's complete execution result, message, Clock, post-state,
preconditions, assumptions and blocker. Failed transactions and insufficient
balance cases contribute zero amount evidence.

Execution uses the captured Phase 5 bank. A matching public source balance is
required to describe evidence for the Phase 4 represented amount, but does not
prove execution of the historical Phase 4 bank. With differing source balances,
the measured case and its largest successful input remain visible, while earlier
population amount coverage is zero. Policy before-view times may differ without
altering source evidence, after-view or captured execution Clock.

Use **maxima**, not sums, across independently reset amounts, paths, fixtures and
venues. A 1,000-raw and a 10,000-raw success cover a 10,000-raw evidence envelope,
not 11,000 raw. This does not interpolate success at untested inputs below or
between tested amounts. Portfolio sums count disjoint token-account evidence
amounts; they never represent simultaneous pool capacity or promised proceeds.

## Portfolio aggregation and assurance boundaries

The portfolio and account-type/path-type/venue cohorts report:

- Token-account entity count and positive-public-balance entity count.
- Counts of each coverage classification.
- Distinct observed owner authorities and authorities with measured amounts.
- Exact public represented, covered and without-evidence raw amounts.

Holder units are explicitly token-account entities; distinct authorities are
separate observations, never human identities or possession of signing keys.
Public amounts sum in checked `u128` arithmetic; individual token quantities are
canonical `u64` strings. No float, scaled display transform, USD value or new
price source is introduced.

Account-type aggregates partition the population. Each path aggregate covers
the population for that path; paths overlap and must not be added. Venue cohorts
include only explicitly requested entities for that named venue. Other holders
are not silently assigned to a pool, and venue cohorts can overlap. An untested
alternative venue never inherits another venue's execution result.

The population entity list is counted once. Phase 3 protocol observations describe
overlapping vault accounts and are not added as new holders/capital. Vault token
ownership does not prove LP entitlement, withdrawal or vault exitability.
Scenario policy remains external semantics. No actual signature possession,
mainnet inclusion, future liquidity, complete validator-bank fidelity or signed
RPC inclusion proof is established by local replay.
