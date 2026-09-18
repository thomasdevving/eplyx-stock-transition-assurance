# Phase 13 production result: same bytes, new lifecycle consequences

## 1. Scenarios evaluated

The original pinned `scenarios/spacex-transition.json` supplies all semantics.

| View | Exact evaluation time (UTC) | Policy status |
| --- | --- | --- |
| Before transition | 2026-09-17T19:59:59.999999999Z | Active |
| Transition effective | 2026-09-17T20:00:00Z | TransitionRequired |
| Declared deadline effective | 2027-03-12T23:59:00Z | NoIssuerEntitlement |

The effective time is the existing explicit demonstration assumption, not an
observed IPO or official conversion time. The deadline interpretation is the
existing exclusive cutoff at the start of the notice's stated minute. No new
issuer/legal claims, deadlines or conversion mechanics are introduced. The original
scenario is used directly; this phase performs no notice ingestion.

## 2. Production-world identity

Every view uses production digest
`c0cf207b91abd06e70f4a0e8b01a6614e941c4ff6709081e39d6d4880b3ebf78`.
This is the canonical hash of the tuple of the following exact frozen bindings:

| Input | SHA-256 |
| --- | --- |
| Phase 2 snapshot | `70d5c41cfc5deef048b5e13fcc4febe1b572d8f8c8abbacc49f767d9a5c0391f` |
| Augmented exposure snapshot | `6802afb871035a0883a04196c7c9542d021d97514f43748b8f63747ea3ce016e` |
| Phase 10 position fixture | `3d79ad5dcee2a9b7ecc74575fe9af7e025695e229471b7aab5288d69813c844f` |
| Position discovery | `9152453761142ba59a56c9a415aed6a3c600f1e7ba5e89b4a6efa708a6bb6ce1` |
| Raw position bytes | `b63dc7ccc951acd7332d824dad9e401dd2d1ee64bdb7d8225dc5bab58bd1f447` |

Policy and evidence are separately bound: original scenario
`9d7d0556a55fd97885cca6f62105d3550ae8d0946b2ec473502bcba050035cea`,
lifecycle policy
`7cceb6802566d8bec105ad5314534f2b322d8690581c69a5eb9f8ca155a31556`,
exact readiness policy
`162655313c70cba8adc4417d535128f73d0e8de6cc36d8b1d04586e8a1993c4d`,
holder matrix
`993989bcdad106d7eef79895e258f4ce14bad18ec810503932688e0ebeb8e06d`,
and position execution report
`5f74a2b70946efbc937fba9e9bcd028ce6b6586184104424c8f28e7ba39ddff5`.
The artifact includes all original nested evidence references and digests.

This is one unchanged frozen collection of captured observations, with distinct
capture banks. It is not one historical validator bank or a claim that all
captures were simultaneous. Comparison rejects different world, balance, policy,
byte/bank and historical proof bindings. Full regeneration rejects tampered claims
that retain the original declared digest strings.

## 3. Direct-holder diff

Canonical account: `741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs`.

| Observation | Before | Transition | Deadline |
| --- | --- | --- | --- |
| Public raw SPACEX | 17,621 | 17,621 | 17,621 |
| Lifecycle | Active | TransitionRequired | NoIssuerEntitlement |
| Impact | Unaffected | RequiresTransition | StaleExposure |

Captured token-account slot **447865621** and byte digest
`fdd7b8048f3a0a81d7c99b3c69c7aea0db1d2642536cf835e030136cda71df33`
are identical in every view. Counterfactual time changes the interpretation of
that exact amount without inventing holder signing access or conversion proof.

## 4. LP-position diff

Real decoded position: `BpTBNQ7vNaBujkwhgyBoiYyvEc6KrUUNTGiQDsrwTNxN`,
pool `v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc`, bins **-102 to -53**.
The original capture is independently decoded offline; owner, pool, range,
liquidity shares, fees and principal are compared with the original report.

| Observation | Before | Transition | Deadline |
| --- | --- | --- | --- |
| Captured raw SPACEX principal | 21,296,808 | 21,296,808 | 21,296,808 |
| Asset lifecycle | Active | TransitionRequired | NoIssuerEntitlement |
| Exposure impact | Unaffected | StaleExposure | StaleExposure |
| Historical native Withdrawal | Proven | Proven | Proven |

Captured slot **448082469** and raw position digest listed above stay identical.
These are the frozen **pre-execution** production bytes. No assertion is made
about the position's present state after independent local execution. Historical
withdrawal proof retains exact position/range/full-fraction/bank/authority scope,
local owner signing assumption and unknown key possession. It supplies principal
removal evidence, not fee collection, closure, market sale or official conversion.
The position is not added as another token-account entity in population totals.

## 5. Population diff

Counts are calculated from the existing evaluator and captured observations,
not expected-count constants in production code.

| Consequence | Before | Transition | Deadline |
| --- | ---: | ---: | ---: |
| Positive public-balance entities | 10,155 | 10,155 | 10,155 |
| RequiresTransition entities | 0 | 10,154 | 0 |
| StaleExposure entities | 0 | 1 | 10,155 |
| Stale verified protocol exposures | 0 | 1 | 1 |
| Unaffected zero-public-balance entities | 7,802 | 7,802 | 7,802 |
| Unknown-role affected positive entities | 0 | 2,969 | 2,969 |
| Verified protocol integrations affected | 0 | 1 | 1 |
| On-chain balances changed | No | No | No |

There are **17,957** captured token-account entities. Before → transition changes
10,155 classifications: 10,154 Unaffected → RequiresTransition, and one verified
vault Unaffected → StaleExposure. Transition → deadline changes 10,154
RequiresTransition → StaleExposure; the vault was already stale.

The separate verified vault observation remains **682,216,229 raw SPACEX**,
slot **447877573**, digest
`5efbe6e383c9f901487a338502a0196030f8a209e0e7977aab6e228d320c3a69`.
This independent Phase 3 amount is not substituted for the earlier Phase 2 vault
amount or summed with LP principal. Its classification changes without byte change.
The zero-public-balance/confidential distinction in the existing model is preserved.

## 6. Deadline diff

The explicit policy changes TransitionRequired → NoIssuerEntitlement. Every
positive public-balance entity is now StaleExposure under that policy. Token
accounts, pool/position state and captured programs still exist unchanged. The
verified vault and LP position remain stale, with a changed asset lifecycle status.
This is policy-defined issuer-entitlement expiration terminology, not evidence of
worthlessness, illegality, loss, stranding or a failed technical exit.

## 7. Readiness diff

Before transition: **PreEvent**, with no applicable lifecycle rollout result.
This is applicability, not a Ready assurance finding. The full original assurance
report remains retained and unmodified.

Transition and deadline: **Applicable / Incomplete**, from the same existing
assurance policy and verified pinned evidence. Required official holder/position
conversion proof is NotTested; complete LP exit still lacks independent fee and
closure proof; population actionability is not established by the bounded sample.
The original optional failed route remains optional and scoped. No new blocking
condition is invented from deadline passage, and Incomplete does not become Blocked.

Causal chain for `direct-official-transition`:
**effective boundary crossed → lifecycle requirement becomes relevant → unchanged
OfficialTransition NotTested evidence → required policy finding IncompleteEvidence
→ applicable rollout Incomplete**. Active failure-mode IDs reference the unchanged
`PreflightFailureMode` records, including assumptions, scope, observed evidence,
policy effect and remediation. No real incident or intervention is claimed.

## 8. Path relevance

| Historical exact action | Before proof | Transition proof | Deadline proof |
| --- | --- | --- | --- |
| Direct Transfer | Proven | Proven | Proven |
| Direct SecondaryMarketExit | Proven | Proven | Proven |
| Position Withdrawal | Proven | Proven | Proven |
| Direct/position OfficialTransition | NotTested | NotTested | NotTested |

Before the event, those paths have no immediate lifecycle-action requirement.
Afterward, mobility, native unwind and the unresolved distinct official path become
relevant considerations. Relevant paths need not be executable: Unsupported and
NotTested remain their original statuses. Full original matrices retain Redemption,
applicability, signer assumptions, scenario and exact execution contexts. No path
provides another path's proof; no execution is rerun for this analysis.

## 9. Change attribution

The comparisons explicitly record `/policy/effective_at` and
`/policy/deadline/at` crossings. LifecycleMeaningChanged is true and
OnChainStateChanged is false for both comparisons. ImpactClassificationChanged is
also true; PathRelevanceChanged and ReadinessChanged occur at the effective boundary.
Readiness and path proof/relevance remain unchanged across the deadline.

Unchanged facts are token-account bytes/balances, vault/pool bytes, original
position/range/shares/exposure, program bytecode, and historical execution
results/banks/signer assumptions. The diff includes changed selected-holder/position
and protocol-vault rows as well as full-population classification groups. The only
independent variable is policy evaluation time.

## 10. Artifacts

- [Canonical counterfactual UI JSON](../reports/spacex-counterfactual-lifecycle.json).
- [Human-readable CLI output](../reports/spacex-counterfactual-lifecycle.txt).
- [Artifact checksums](../reports/spacex-phase13-artifacts.sha256).
- [Validation record](../reports/spacex-phase13-validation.json).
- [Executable mutation results](../reports/spacex-phase13-mutation-results.json).
- [Model, exact bindings and offline replay](lifecycle-phase-13-counterfactual.md).

Canonical JSON SHA-256:
`268182e18bcce4b6dfba2b4dfcf785619033a13f01923cb01c98a35488926a47`.
CLI text SHA-256:
`81344c17de6278c76aba82fe48aad795314dba07e6e137e1719947e9ad1e8ca6`.
[The direct checksum file](../reports/spacex-counterfactual-lifecycle.sha256)
binds both artifacts.

The JSON carries all summaries and selected examples, shared full-population
identities/classifications, both explicit comparisons and historical readiness/path
contexts. The UI does not need to recompute lifecycle logic. Output is deterministic,
roundtrips canonically and is protected against overwrite.

## 11. Tests and mutations

Completed checks:

- `make test`: **332 passed, 0 failed, 0 ignored**; both synthetic SBF versions built.
- Final focused counterfactual tests: **19 passed, 0 failed, 0 ignored**.
- `make fmt-check`: passed against final source.
- `make lint`: passed for all engine targets and both fixture versions against final source.
- `git diff --check`: passed.
- **293 historical frozen input/artifact files unchanged**, verified against the recorded baseline.
- Actual final CLI integration: portable absolute inputs, JSON stdout/saved/published byte identity, valid analysis exit 0 and protected output exit 2 passed.

The full suite passed before the final new-module attribution clarification;
that change only makes NotApplicable context explicit and avoids claiming relevance
under unknown semantics. All 19 focused tests and format/lint were then rerun
against the final source and regenerated JSON. Original test files and shared
consequence semantics were not weakened. No execution assertion is skipped.

The 19 focused tests cover the requested digest, boundary, balance, holder, vault,
zero-balance, deadline, stored proof, Transfer, Withdrawal, OfficialTransition,
policy readiness, mixed-world rejection, ordering, canonical roundtrip and offline
CLI invariants, plus population/scope reconciliation, tampered report regeneration
and an actual balance-tampered snapshot input.

Final executable campaign: **8 injected, 8 caught by named assertions**, no compiler failures.
The original source was restored byte for byte; its SHA-256 is
`e132fa559829935181b3a7da28bb49d0e3c96aec8343b17864bbb66df60c8b46`. The detector requires a failed exact named test
and an assertion message. The zero-balance and different-world messages are
explicitly checked. Earlier development/harness-review logs are retained separately
and are not counted as the final campaign.

| Injected executable fault | Named assertion test | Result |
| --- | --- | --- |
| Change captured holder balance only after the event | [`balances_never_change`](../reports/phase13-mutations/01.txt) | Caught, exit 101 |
| Report lifecycle meaning change as an on-chain mutation | [`vault_semantics_change_without_byte_changes`](../reports/phase13-mutations/02.txt) | Caught, exit 101 |
| Keep selected holder Unaffected after transition | [`holder_requires_transition_after_event`](../reports/phase13-mutations/03.txt) | Caught, exit 101 |
| Classify public zero-balance accounts as transition exposure | [`zero_balance_never_becomes_public_exposure`](../reports/phase13-mutations/04.txt) | Caught, exit 101 |
| Promote proven token movement to official transition proof after deadline | [`official_transition_never_inherits_proof`](../reports/phase13-mutations/05.txt) | Caught, exit 101 |
| Change readiness without assurance-policy justification | [`readiness_changes_only_with_policy_and_applicability`](../reports/phase13-mutations/06.txt) | Caught, exit 101 |
| Accept distinct production-state digests as identical worlds | [`different_worlds_are_non_comparable`](../reports/phase13-mutations/07.txt) | Caught, exit 101 |
| Let scenario evaluation rewrite stored historical execution matrix | [`historical_execution_evidence_is_immutable`](../reports/phase13-mutations/08.txt) | Caught, exit 101 |

Independent standalone `compare-scenarios` replay after restoring the source
returned **0**. Its saved JSON and human-readable stdout are byte-identical to
the published artifacts. The final integration suite separately checks canonical
JSON stdout, saved JSON and portable absolute inputs, plus protected-output exit 2.

## 12. Product conclusion

Eplyx demonstrates that an explicit tokenized-stock lifecycle event can change
required actions and economic interpretation over an unchanged real captured
production world. It separates chain existence, policy meaning, exact historical
actionability and rollout assurance. The result is deterministic consequence
attribution rather than new execution coverage or a prediction of future state:
**same captured bytes + new lifecycle time = new consequences**.

## 13. Smallest next recommendation

Add one offline prevented-incident/failure-injection demonstration: declare the
rollout assumption that proven holder Transfer implies official conversion, inject
that claim into a separate candidate rollout plan, and show the existing exact-path
isolation/readiness gate rejecting it after the event. Retain the unchanged world,
policy, official NotTested evidence and baseline counterfactual report; publish the
injected assumption and named violated requirement alongside the gate result.
Frame it as a prevented unsupported rollout assumption, not a real incident or
issuer safety judgment. No new capture, execution or UI is required. **That next
phase has not begun.**
