# Phase 11 lifecycle readiness / pre-flight result

**PopulationRolloutReadiness = Incomplete** under the explicit demonstration
policy `stocklana-spacex-preflight-v1`. Exact direct-holder mobility and native LP
principal withdrawal satisfy their conditions. Official transition, complete LP
exit and population-wide actionability do not have sufficient evidence. The gate
performs no new execution, capture or mainnet action. This is not an asset safety
judgment or the issuer's internal rollout policy.

## 1. Readiness policy

[The serializable policy](../policies/stocklana-spacex-preflight-v1.json) records
`policy_type=DemoAssurancePolicy` and `not_issuer_policy=true`. It is separate from
[the original economic lifecycle scenario](../scenarios/spacex-transition.json).
The scenario describes TransitionRequired; this policy describes required proof.

| Requirement ID | Why required | Current effect |
| --- | --- | --- |
| `direct-mobility` | One exact full-balance non-official movement or sale for the selected holder | Satisfied |
| `direct-official-transition` | Distinct official successor-conversion proof for that holder/amount | IncompleteEvidence |
| `lp-principal-unwind` | Exact encoded-owner native full-range principal removal | Satisfied |
| `lp-official-transition` | Native removal cannot substitute for issuer conversion | IncompleteEvidence |
| `lp-complete-exit` | Principal removal cannot establish fee collection or closure | IncompleteEvidence |
| `population-actionability` | Each positive observed account needs its own full-amount requested-path proof | IncompleteEvidence |
| `evidence-isolation` | Preserve path/entity/venue/bank/principal/fee claim boundaries | Satisfied |
| `optional-failed-route` | Retain the earlier actual failed route without automatically requiring it | Informational |

The exact path alternatives accept Proven only. Failed/Unsupported are declared
blocking for these path conditions only at matching scope. Missing, NotTested or
incompatible evidence is Incomplete. The complete-exit condition leaves retained
fees non-forbidden in this demo: missing fee/closure proof remains Incomplete.
Explicitly forbidding residual fees yields Blocked in controlled tests. Signer
assumptions must be explicitly allowed; no key possession is asserted.

## 2. Overall gate result

**Incomplete**, derived from required findings. No required condition has an
exact explicit blocker under the current policy. A successful unrelated path
cannot erase a required blocker; a missing path cannot become Failed. Ready is
reserved for all required conditions being satisfied under the declared scope.

Both selected entity readiness rows are Incomplete under their full lifecycle
requirements. Their mobility/principal subconditions are satisfied. A controlled
entity-only policy limited to those two subconditions is Ready; it has no rollout
readiness result and makes no lifecycle-completion assertion.

## 3. Direct-holder readiness

Canonical entity:
`solana-token-account:741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs`.
Retained authority: `2wCvQzHiDHAHTvzwPeof9H3uEzq8Bzvg38DFbvZMGkuj`.
Requested and observed original public balance: **17,621 raw SPACEX**.

The policy selects actual full-input Transfer and SecondaryMarketExit attempts
from [the unchanged Phase 9 matrix](../reports/spacex-lifecycle-path-resolution-phase9.json).
Transfer is to the real earlier recipient
`124XHuTYUNnCf2ABCNcCEdQFpHQ7Y6MnPe9NeouDB1az` at slot `448018365`.
Market exit is on `22PthLk8TYnurtbWKRyECFd99cHHbfsbPNHfeMetzfZg` at slot
`448018537`. Both are separate successful local transactions with exact retained
input, capture, Clock and authority assumptions. They satisfy direct mobility;
neither proves official conversion. OfficialTransition stays NotTested,
Redemption Unsupported, Withdrawal NotApplicable for this direct-holder shape.

## 4. LP-position readiness

Selected entity:
`solana-program-position:BpTBNQ7vNaBujkwhgyBoiYyvEc6KrUUNTGiQDsrwTNxN`.
Encoded owner: `55uxDcXEaUjwit2Mv3EoNrtaGqTTUaUCvXpABpkoUUoE`.
Pool: `v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc`.
Exact range **-102..-53**, **10,000 bps / 100%**, finalized bank **448082469**.

| Existing Phase 10 measurement | SPACEX raw | USDC raw |
| --- | ---: | ---: |
| Principal removed from pool | 21,296,808 | 8,132,564 |
| Public owner credit | 21,190,323 | 8,132,564 |
| Destination withheld increase | 106,485 | 0 |
| Remaining accrued position fees | 126,543 | 113,735 |

[The original native withdrawal](../reports/spacex-dlmm-withdrawal.json) proves
principal removal under the assumed original owner signer. All liquidity shares
end at zero, while the position and fee ledger remain. Fee collection and closure
are **NotTested**; complete-position-exit assurance is **IncompleteEvidence**.
Destination withheld fees and retained protocol fees are separate quantities.
No fee claim, closure, post-withdrawal sale, successor conversion or redemption
executes here. The LP is not added to the public token-account population.

## 5. Official-transition readiness

[Phase 9 research](../reports/spacex-official-transition.json) did not establish
an independently executable issuer-bound SPACEX-to-successor mechanism.
**OfficialTransition remains NotTested for both state shapes.** The policy requires
its own distinct Proven evidence and therefore records IncompleteEvidence.
This does not establish transition non-existence, impossibility or an issuer/KYC
blocker. Published successor identity, Transfer, market sale, withdrawal and
administrative mint/burn observations cannot grant official proof.

## 6. Population readiness

The reader joins all [Phase 6 population rows](../reports/spacex-lifecycle-coverage.json)
with [the original Phase 7 updates](../reports/spacex-lifecycle-coverage-phase7.json),
checking each original authority/type/balance against the validated snapshot.

| Quantity | Exact value |
| --- | ---: |
| Token-account entities | 17,957 |
| Positive public-balance entities | 10,155 |
| Zero-public-balance accounts | 7,802 |
| Distinct observed owner authorities | 17,950 |
| Entities with full represented-amount mobility evidence | 4 |
| Positive entities without matching requested-path evidence | 10,151 |
| Represented public amount raw | 8,741,534,482,051 |
| Conditional measured amount envelope raw | 1,457,718,144,803 |
| Represented amount without evidence raw | 7,283,816,337,248 |

Unresolved positive account types are **7,114 WalletCompatible**, **68
ProgramOwnedAuthority**, **2,969 Unknown**. These remain evidence/capability gaps,
not proof of issuer restrictions. Each entity needs its own full represented
amount on an allowed population path (Transfer, SecondaryMarketExit or distinct
OfficialTransition). Four proven samples do not prove peers. Zero balances never
become execution proof. Protocol/vault observations are overlapping exposure,
not additional holders/capital. Independent paths/venues use the existing envelope,
not summed transactions, simultaneous capacity or proceeds. No valuation is added.

## 7. Prevented rollout conditions

The report records neutral `PreflightFailureMode` entries for these assumptions:

- The canonical holder can complete an official successor transition.
- Native LP removal establishes official conversion.
- Zero principal shares establish complete LP exit, including fees and closure.
- A bounded successful sample establishes population-wide actionability.
- The earlier large-holder sale can execute on its exact failed route/context.

That optional historical route is entity
`solana-token-account:ENc8TdLutJ2ziFnz9x4uV8pEmdnk6iWYaBHaTAPCejpV`, input
**14,577,177,576 raw**, venue `22PthLk8TYnurtbWKRyECFd99cHHbfsbPNHfeMetzfZg`,
bank **448018480**. Its actual existing VM failure is
`BitmapExtensionAccountIsNotProvided`, custom **6036**, with verified rollback.
The finding is Informational because the policy does not require this sale.
Making that exact condition required produces Blocked in a controlled policy test.
The failure says nothing about unrelated paths or the asset generally.

Eplyx **would prevent a rollout from relying on these unverified or contradicted
assumptions**. Every record has `real_incident_claimed=false`. No real rollout was
blocked and no actual loss prevention is claimed.

## 8. Evidence bindings

[The pinned manifest](../probes/spacex-readiness-evidence.json) supplies exact
snapshot/scenario, direct/LP matrices, population baseline/delta, selected plan,
execution index, capture manifest, LP capture/discovery and official-research
references. The gate verifies all **30** selected Phase 7 measured case files and
**6** captured group fixtures. Their existing case counts are **20 Succeeded,
4 Failed, 6 Indeterminate**. This is historical evidence consumption, not another
execution campaign. The readiness JSON contains every used artifact digest and
its manifest-relative path.

Path matching checks asset/entity/state shape/authority, exact raw amount,
range/fraction, path, venue/context, full Clock/capture context, fixture and
captured-state digest, original source balance and scenario digest. None is not a
wildcard. Different requested bank/venue/entity/amount gets no assurance. Local
assumed signing stays separate from possession. Hypothetical economic policy time
remains `2026-09-17T20:00:00Z`; direct/LP execution banks remain separate.

Digest integrity is relative to the explicitly trusted demo policy and original
published measurements; it is not signed inclusion or a new execution attestation.
No current/future freshness or funds movement is inferred. Original discovery
canonical parsed-JSON hashes and file-byte hashes are both preserved.

| Artifact | SHA-256 |
| --- | --- |
| [Demo policy](../policies/stocklana-spacex-preflight-v1.json) | `162655313c70cba8adc4417d535128f73d0e8de6cc36d8b1d04586e8a1993c4d` |
| [Evidence manifest](../probes/spacex-readiness-evidence.json) | `9154dda0427f70478a16dd144d37987d8f5c94bb2083778353bb8fa2ad3cc892` |
| [Canonical readiness](../reports/spacex-lifecycle-readiness.json) | `fb2b093170d5b4c85461c2d83b4d99569b2d80d9f6b62c3783bbfd2a843a4953` |
| [Final mutations](../reports/spacex-phase11-mutation-results-verified.json) | `0891c5a382d9083996bf3bfc79c755ca44cbca79b5f299cacb342deb5f993727` |
| [snapshot](../snapshots/spacex-exposure.json) | `6802afb871035a0883a04196c7c9542d021d97514f43748b8f63747ea3ce016e` |
| [scenario](../scenarios/spacex-transition.json) | `9d7d0556a55fd97885cca6f62105d3550ae8d0946b2ec473502bcba050035cea` |
| [direct resolution](../reports/spacex-lifecycle-path-resolution-phase9.json) | `993989bcdad106d7eef79895e258f4ce14bad18ec810503932688e0ebeb8e06d` |
| [position resolution](../reports/spacex-dlmm-withdrawal.json) | `5f74a2b70946efbc937fba9e9bcd028ce6b6586184104424c8f28e7ba39ddff5` |
| [coverage](../reports/spacex-lifecycle-coverage-phase7.json) | `79b78e877b63c9fa3fd5ef80755b0759c47ae8ac9771a56cc3046a387217e07f` |
| [baseline coverage](../reports/spacex-lifecycle-coverage.json) | `f3ff991165d23f9ff851b8e7d9c141a92aac4668bf26e2bc83926a12a2e4d920` |
| [expansion plan](../probes/spacex-lifecycle-expansion-plan.json) | `6758fbd4e3d99c44e726c638fe2bcbe514ce702957af6dfd121bc8d23cd4aade` |
| [execution index](../reports/phase7-evidence/execution-index.json) | `e14c449199c972b91b5390b325b8ed8285f07efa70c5a41df960e2a142ee3c52` |
| [capture manifest](../probes/phase7-captures/capture-manifest.json) | `5357ef0ff0436649e27c626f81f16be99022a48ce232a640d9114228d0befdc6` |
| [withdrawal fixture](../evidence/withdrawal/phase10/execution-capture.json) | `3d79ad5dcee2a9b7ecc74575fe9af7e025695e229471b7aab5288d69813c844f` |
| [withdrawal discovery](../evidence/withdrawal/phase10/position-discovery.json) | `9152453761142ba59a56c9a415aed6a3c600f1e7ba5e89b4a6efa708a6bb6ce1` |
| [official research](../reports/spacex-official-transition.json) | `fb7a79bd6f1a56daf09b34f3af49a0c5d11baa0b479b2922be200ff6b68e6d6d` |

All final Phase 11 source, output and validation hashes are listed in
[the artifact manifest](../reports/spacex-phase11-artifacts.sha256).

## 9. Machine-readable gate

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- readiness \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --policy policies/stocklana-spacex-preflight-v1.json \
  --direct-resolution reports/spacex-lifecycle-path-resolution-phase9.json \
  --position-resolution reports/spacex-dlmm-withdrawal.json \
  --coverage reports/spacex-lifecycle-coverage-phase7.json
```

Stable exit codes: **Ready 0**, **Blocked 3**, **Incomplete 4**, **invalid input or
protected output 2**. Add `--format json --out <new-path>` for deterministic JSON.
Incomplete/Blocked decisions still emit the complete report. Saved JSON and
stdout are identical; existing outputs are protected. Absolute input paths work
from another directory with no RPC environment. No wall-clock generation timestamp
enters the artifact. This demo's exit 4 is an evaluated gate decision.

## 10. Architecture

`readiness/` contains generic `LifecycleReadinessPolicy`, exact `EvidenceScope`,
requirements/findings, entity/rollout views, `PreflightFailureMode` and canonical
`LifecycleReadinessReport`. The frozen-artifact adapter constructs opaque
`VerifiedReadinessEvidence` only after integrity and measurement relationships
are checked. The evaluator has no issuer/asset conditional and never calls an
executor or replay validator. Readiness policy remains separate from external
economic lifecycle policy. No dependency is added.

## 11. Tests and mutations

Completed against final source and published artifacts:

- `make test`: **290 passed, 0 failed, 0 ignored**; both synthetic SBF versions built.
- `make fmt-check`: passed.
- `make lint`: passed for all engine targets and both fixture versions.
- `git diff --check`: passed.
- Independent offline readiness replay: saved JSON/stdout/published report byte-identical; valid Incomplete exit **4**.
- CLI integration checks: actual Ready **0**, Blocked **3**, Incomplete **4**, invalid/tampered/protected output **2**; portable inputs and output protection passed.
- **All 10 final actual code mutations caught** by named assertions; source restored byte-for-byte and digests verified.
- All **43 original checksum files** checked: **238 matching entries**, only **16** allowed shared-file entries across historical manifests differ.
- Historical Phase 7/8/9/10 manifests retain **79/40/45/35** unchanged entries; only the four shared scope/export/CLI files receive additive edits. No earlier assertions are weakened or execution tests skipped.

Suite breakdown: fixture v1 7, fixture v2 7, engine 135, expansion 6, probe 12,
lifecycle consequence 12, coverage 16, exposure 11, readiness 3, resolution 5,
snapshot 13, official transition 3, protocol position 10, scenario 2,
upgrade integration 42 and interface 6.

[The validation record](../reports/spacex-phase11-validation.json) binds final
policy/report/source/mutation hashes to [complete command logs](../reports/phase11-validation/).
[Canonical JSON](../reports/spacex-lifecycle-readiness.json) and
[the actual CLI text](../reports/spacex-lifecycle-readiness.txt) are retained.

The 20 focused unit tests cover missing/NotTested/Unsupported status distinctions,
policy-controlled failure/fee blocking, Transfer/market/withdrawal non-inheritance,
complete-exit fees/closure, entity/venue/bank/fixture isolation, entity/population
separation, peer non-extrapolation, unrelated-success blocker preservation,
optional failure, signer assumption, ordering/canonical roundtrip, invalid policy,
generic asset independence, stable exit codes and actual-execution/reconciliation
requirements. Three integration tests consume unchanged production artifacts,
reproduce the published JSON, verify current gaps/fee ledger/error, reject stale
requested scopes, test CLI codes 0/3/4/2, portable absolute inputs, saved/stdout
identity, output protection and input tampering. They perform no new VM execution.

Ten actual executable code faults are caught through named assertions:
NotTested promotion; Transfer/Withdrawal replacing OfficialTransition; principal
replacing complete exit; sampled-holder population inheritance; ignored entity or
venue/context/bank; missing proof marked Ready; Incomplete changed to Blocked;
unrelated success erasing a required failure. Final source restoration and actual
assertion-failure logs are retained. No compiler failure counts as a caught fault.

| Injected fault | Named assertion | Actual result |
| --- | --- | --- |
| Treat NotTested as Proven | `readiness::tests::not_tested_never_becomes_proven_or_failed` | Caught; [log 1](../reports/phase11-mutations-verified/01.txt) |
| Transfer substitutes for OfficialTransition | `readiness::tests::transfer_never_satisfies_official_transition` | Caught; [log 2](../reports/phase11-mutations-verified/02.txt) |
| Withdrawal substitutes for OfficialTransition | `readiness::tests::withdrawal_never_satisfies_official_transition` | Caught; [log 3](../reports/phase11-mutations-verified/03.txt) |
| Principal unwind substitutes for complete position exit | `readiness::tests::principal_is_not_complete_position_exit` | Caught; [log 4](../reports/phase11-mutations-verified/04.txt) |
| Inherit one sampled holder to population | `readiness::tests::sampled_entity_never_proves_population` | Caught; [log 5](../reports/phase11-mutations-verified/05.txt) |
| Ignore evidence entity mismatch | `readiness::tests::evidence_cannot_inherit_between_entities` | Caught; [log 6](../reports/phase11-mutations-verified/06.txt) |
| Ignore venue/context/bank mismatch | `readiness::tests::venue_context_bank_and_fixture_are_exact` | Caught; [log 7](../reports/phase11-mutations-verified/07.txt) |
| Missing evidence becomes Ready | `readiness::tests::missing_evidence_is_incomplete_not_ready` | Caught; [log 8](../reports/phase11-mutations-verified/08.txt) |
| Incomplete becomes Blocked without policy justification | `readiness::tests::not_tested_never_becomes_proven_or_failed` | Caught; [log 9](../reports/phase11-mutations-verified/09.txt) |
| Unrelated success erases a required failure | `readiness::tests::unrelated_success_cannot_erase_required_failure` | Caught; [log 10](../reports/phase11-mutations-verified/10.txt) |

[The final mutation record](../reports/spacex-phase11-mutation-results-verified.json)
binds all ten assertion-failure logs and the exact restored final source digest.
Earlier development runs are retained separately; final assurance uses this
verified campaign.

## 12. Product conclusion

Eplyx now refuses to call the declared broader rollout ready when required
production-state actions lack exact proof. It permits only the assurance that the
policy and existing evidence support: measured selected-holder mobility and
principal unwind remain useful without implying conversion, complete LP exit,
peer actionability or key possession. The result is a deterministic pre-flight
condition check, not an asset verdict, subjective score or claim of intervention.

Work remains on `main` in the shared uncommitted tree. No commit/push, new capture,
new holder/pool execution, official retry, redemption, fee collection, position
closure, issuer integration, UI, notice ingestion, price source or valuation is
introduced. Original Phase 1–10 artifacts and assertions are retained; only four
shared scope/export/CLI files receive additive changes. See
[the complete Phase 11 file list](../reports/spacex-phase11-files.txt).

## 13. Smallest Phase 12 recommendation

Import **one manually selected real issuer lifecycle notice** into the existing
deterministic `LifecycleScenario` schema. Retain raw content, source URL, capture
metadata and content digest; explicitly map asset identity, effective time,
asserted change and any successor/deadline fields with exact provenance pointers.
Keep published notice assertions separate from observed chain state and execution
proof. Validate the mapping offline and feed that explicit change into the same
impact/readiness pipeline; notice ingestion must never grant OfficialTransition
proof. No live feed, automatic discovery, issuer credentials or new execution is
needed. **Phase 12 has not been started.**
