# Phase 4 production lifecycle result

The generic lifecycle consequence model runs entirely offline over the real
SPACEX production world from Phases 2 and 3. The same frozen account bytes and
exposure graph receive different external lifecycle semantics. No mainnet state
was mutated, no conversion executed, and no MintPause implemented.

## Scenario model and Stocklana specification

`LifecycleChange + AssetLifecyclePolicy + LifecycleConsequenceEvaluator` is
dispatched through the existing `ChangeScenario`. The change remains independent
of state inputs. A typed, versioned JSON specification records the asset, timing,
statuses, optional deadline/successor and field-level source provenance. Core
code contains no SPACEX mint, PreStocks rule or issuer-specific conditional.

[scenarios/spacex-transition.json](../scenarios/spacex-transition.json) supplies:

| Field | Value / source |
| --- | --- |
| Scenario | `prestocks-spacex-transition-v1`, version `1.0.0` |
| Asset | `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh` |
| Effective boundary | `2026-09-17T20:00:00Z` — explicit demo assumption |
| Before | `Active` — explicit demo assumption |
| At/after boundary | `TransitionRequired` — issuer notice assertion |
| Deadline | `2027-03-12T23:59:00Z` — issuer notice plus explicit cutoff interpretation |
| At/after deadline | `NoIssuerEntitlement` — policy expiration semantics |
| Successor | `null`; none independently integrated or required by this model |
| Scenario/source capture | `2026-09-17T20:00:13Z` |

The [official PreStocks notice](https://prestocks.com/spacex) requires holders
to exchange existing tokens before its March 2027 cutoff. This is external
issuer policy, not independent verification of an IPO or a working conversion.
The captured HTML is retained locally with its digest in the scenario.
The page does not establish an exact historical transition start. The chosen
demo boundary and preceding Active status are explicitly hypothetical. The
minute-level deadline is conservatively interpreted as an exclusive cutoff at
the start of 23:59 UTC, rather than silently extending it to midnight. The
assumption source also supports this interpretation. Ending issuer entitlement
under this policy does not establish zero secondary-market value or legal rights.

## Before/after proof

Both semantic views use the identical production snapshot SHA-256:

```text
6802afb871035a0883a04196c7c9542d021d97514f43748b8f63747ea3ce016e
```

The complete transition report evaluates `2026-09-17T19:59:59Z` against
`2026-09-17T20:00:00Z`. The complete terminal report uses the same baseline
against `2027-03-13T00:00:00Z`. Neither requests RPC data at those dates.

| Frozen observation | Public raw / decimal amount | Before | After boundary | Technical state |
| --- | --- | --- | --- | --- |
| Wallet-compatible token account `124XHuTYUNnCf2ABCNcCEdQFpHQ7Y6MnPe9NeouDB1az` | `60354` / `0.000060354` | `Active`, `Unaffected` | `TransitionRequired`, `RequiresTransition` | Unchanged |
| Verified DLMM SPACEX vault `HgQRhiATjX9jTWh7QgLWnhCeR7PaBqoWVf4vSaL61YVv` | `682216229` / `0.682216229` | `Active`, `Unaffected` | `TransitionRequired`, `StaleExposure` | Unchanged |

The wallet token amount is backed by Phase 2 RPC record 2,
`/value/20/account`, slot `447865621`; its existing on-curve, empty, non-executable
System-owned authority is backed by record 103, `/value/24`, slot `447865721`.
The token bytes hash is
`c7b5bc50cd91d9f1c058f949f291186118a0e8fa6811128dff29aa4ab421238b`.
Wallet compatibility proves no human identity or signing access.

The vault proof is the Phase 3 finalized six-account verification at slot
`447877573`, backed by the original Meteora DLMM adapter, canonical pool/vault
PDAs and executable program account. Its pool is
`v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc`. The entity remains originally
`ProgramOwnedAuthority`, with the separate proven `LiquidityVault` refinement.
All raw proof pointers/hashes are retained in the full reports and compact demo.

The earlier Phase 2 observation of this same vault remains `678320992` raw,
`0.678320992` decimal units. Its entity impact uses that earlier amount. The
protocol impact uses the later `682216229` amount. They are two frozen contexts
for the same holding, not additional capital and not a simulated balance change.
Before and after evaluation of each context use exactly its own unchanged amount.

## Deterministic impact counts

| Count | Active before boundary | Transition required | After deadline |
| --- | ---: | ---: | ---: |
| Entities evaluated | 17,957 | 17,957 | 17,957 |
| Positive public balances | 10,155 | 10,155 | 10,155 |
| Zero public balances | 7,802 | 7,802 | 7,802 |
| `Unaffected` | 17,957 | 7,802 | 7,802 |
| `RequiresTransition` | 0 | 10,154 | 0 |
| `StaleExposure` entity impacts | 0 | 1 | 10,155 |
| `Unresolved` lifecycle impacts | 0 | 0 | 0 |
| Positive balances with original `Unknown` role | 2,969 | 2,969 | 2,969 |
| Positive balances lacking wallet compatibility / proven integration | 3,036 | 3,036 | 3,036 |
| Confirmed public economic meaning changes vs Active baseline | 0 | 10,155 | 10,155 |
| Verified liquidity venues | 1 | 1 | 1 |
| Stale protocol exposures, counted separately | 0 | 1 | 1 |

The 2,969 Unknown-role positive accounts are included in the ordinary transition
count, retaining `entity_type: Unknown` and `role_uncertain: true`. The additional
67 uncertain-role positive accounts have other original authority categories.
`Unresolved = 0` means this policy is evaluable for every represented public
amount; it does not mean every owner role or execution path is known.
No SPACEX account in the capture has a confidential account extension. Generic
confidential-zero cases are conservatively unresolved in the test suite.

Total Phase 2 represented public amount: `8741534482051` raw,
`8741.534482051` decimal base units. The transition/terminal scenarios apply to
that public amount; this is not USD value, recoverable value, total mint supply,
LP entitlement or proof of liquidity. The verified Phase 3 protocol observation
separately contains `682216229` raw, `0.682216229` decimal base units. It is not
added to the Phase 2 total. Withheld fees and encrypted/display quantities are
excluded. Every entity and protocol impact has execution status `NotTested`.

## Evidence separation and Eplyx architecture

`onchain_evidence` uses the existing production evidence model: raw-account
digests, RPC transcript IDs, JSON pointers, real slots and pinned decoders.
Original authority observations/references and classification are preserved.
`lifecycle_evidence` contains separately typed source references and policy field
bindings. The full specification and canonical digest are embedded in each
report, distinguishing `ExternalPolicy` from `ScenarioAssumption`. Scenario
loading verifies the locally captured source hash without accessing the web.

The evaluator takes validated immutable `STATE₀`, an explicit `CHANGE`, and its
policy consequence rules. `STATE₁` is the derived lifecycle view, not modified
token accounts. The resulting `DIFF` records unchanged technical state and
changed lifecycle status/meaning for represented public exposure. The before
and after snapshot fingerprints are identical. There is no alternate issuer
engine or replacement of ProgramUpgrade behavior. Full report validation
recomputes all records, aggregates and proof references against the snapshot.

## Artifacts and reproduction

- [Complete transition impacts](../reports/spacex-transition-impact.json): 49,993,274 bytes, all 17,957 entities and one protocol observation.
- [Complete terminal impacts](../reports/spacex-expired-impact.json): 49,889,393 bytes, same world after policy expiration.
- [Compact demo proof](../reports/spacex-lifecycle-demo.json): exact counts and the two complete selected proof records, plus the earlier vault entity observation.
- [Policy and CLI guide](lifecycle-phase-4-consequence.md): before/after commands, rule table and evidence limits.

Reproduce into a new output path:

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- impact \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --before 2026-09-17T19:59:59Z --at 2026-09-17T20:00:00Z \
  --out /tmp/new-spacex-transition-impact.json
```

Canonical transition report SHA-256:
`0d9770ac8e360f37ae5e7cda7700feb20886af223904e027085d633f1c9e3e15`.
Canonical terminal report SHA-256:
`d819fcbd405f4edb173e31249cac2cdc10958a319e80bc58918256ab3469d4be`.
The offline production transition evaluation was repeated with final code and
matched canonical saved JSON byte-for-byte; CLI stdout adds one terminal newline.
Both original production snapshot checksums were verified unchanged.

## Validation

Completed against the final consequence code:

- `make test`: **147 passed, 0 failed, 0 ignored**. Both real synthetic SBF versions were built; no missing-artifact skips.
- `make fmt-check`: passed.
- `make lint`: passed for engine targets and both fixture versions.
- `git diff --check`: passed.
- Production transition and terminal CLI evaluations: passed entirely offline.
- Repeated production transition JSON: matched saved canonical bytes.
- Snapshot and report checksums, world identity, complete entity counts and all `NotTested` statuses: verified.

Suite breakdown: fixture v1 7, fixture v2 7, engine unit tests 47, Phase 2
snapshot tests 13, Phase 3 exposure tests 11, Phase 4 consequence tests 12,
original scenario tests 2, upgrade integration tests 42, interface tests 6.
The original tests retain their behavior and assertions.

The 12 new tests cover active time, inclusive transition/expiry boundaries,
same frozen wallet bytes, zero/confidential-zero amounts, verified vault roles
and separate amounts, Unknown roles, distinct resolvable evidence classes,
repeatable lifecycle diffs, deterministic serialization and tamper detection,
optional deadlines/legacy snapshots/Unknown policy, malformed provenance and
wrong inputs, and local artifact hash checking. No unit test uses RPC or the web.

## Phase 4 files changed

These 18 files were added or updated in this phase:

1. `AGENTS.md` — current scope and evidence requirements.
2. `README.md` — lifecycle route and pointers.
3. `engine/src/lib.rs` — current lifecycle architecture documentation.
4. `engine/src/main.rs` — offline `impact` command and time/report options.
5. `engine/src/scenario.rs` — lifecycle model dispatch; compatible change description and unchanged upgrade path.
6. `engine/src/lifecycle/mod.rs` — policy/consequence module exports.
7. `engine/src/lifecycle/policy.rs` — generic typed specification and provenance validation.
8. `engine/src/lifecycle/consequence.rs` — pure impacts, deterministic diff/report, aggregates and replay validation.
9. `engine/tests/lifecycle_consequence.rs` — 12 focused offline tests.
10. `scenarios/spacex-transition.json` — Stocklana scenario data and explicit assumptions.
11. `evidence/lifecycle/prestocks-spacex-2026-09-17.html` — frozen official issuer source.
12. `reports/spacex-transition-impact.json` — complete transition impact artifact.
13. `reports/spacex-transition-impact.json.sha256` — artifact digest.
14. `reports/spacex-expired-impact.json` — complete terminal impact artifact.
15. `reports/spacex-expired-impact.json.sha256` — artifact digest.
16. `reports/spacex-lifecycle-demo.json` — compact proof/counts extract.
17. `docs/lifecycle-phase-4-consequence.md` — model/CLI guide.
18. `docs/lifecycle-phase-4-production-report.md` — this final report.

No dependency changes or existing test edits were needed in Phase 4. Earlier
Phase 2/3 changes remain in the shared uncommitted working tree; their full file
lists are in their production reports. Work remains on `main`, with no commit
or push performed in this phase.

## Limitations and smallest Phase 5 probe

Phase 4 proves policy applicability to captured balances and integrations, not
whether positions successfully exit or transition. Snapshot bytes do not prove
transferability. A stale pool is not a stranded LP position. Successor identity,
LP ownership, issuer conversion, current market prices, legal entitlement and
historical/future chain state are outside this result. The chosen hypothetical
Active period must not be presented as actual issuer history.

The smallest useful next execution probe is **one bounded exact-input
SPACEX → USDC swap simulation through the already verified DLMM pool**, using
one selected positive wallet-compatible account and the exact route accounts,
mint/vault state, bin arrays/oracle and deployed program bytes captured read-only.
Run it in a local isolated VM with a documented signer assumption, retaining the
original token owner and all applicable Token-2022 fee rules. Check instruction
status and exact source/destination/vault/fee deltas. Nothing is broadcast.
This can establish whether that selected amount and route can produce an exit
in the captured execution model, or why it fails. It cannot establish actual
signing access, exits for other accounts/LPs, or issuer conversion/entitlement.
It requires additional route-state capture and has **not been started**.
