# Milestone 7 delivery: production-state conversion stress test

Milestone 6 answered *"does this candidate conversion work for one selected
holder?"*. Milestone 7 answers the next question:

> A candidate conversion works for one selected holder. Will it also work across
> the different production states that currently exist around this token?

It answers it **without** pretending that a bounded test proves every account.

```
FRESH CURRENT ASSET STATE
+  OPERATOR-SUPPLIED CANDIDATE CONVERSION PLAN
        ↓
discover current token accounts → classify state shapes → decide eligibility
        → freeze a bounded deterministic test plan → execute each exact case
        → aggregate without extrapolation → stress readiness
```

## 1. What the milestone does and does not claim

| Fact | Status |
| --- | --- |
| One exact tested account, amount, case plan and captured bank converted | Can be `Proven` |
| Every other account sharing that account's state shape | **Untested** |
| The whole observed positive-balance population | **Incomplete** |
| An issuer-defined official transition | **`NotTested`**, always |
| Any account the executor cannot exercise | **`Unsupported`** — a boundary, never proof it cannot convert |

A state shape prioritizes and describes tests. It is never a proof equivalence
class. Stress readiness is a finding under one explicit demonstration policy; it
is never an asset safety judgment and never population readiness.

## 2. Two independent completeness axes

This is the central reporting decision of the milestone. Token-account
enumeration and authority resolution are separate facts and are reported
separately, because *"we do not know how many token accounts exist"* and *"we
have every token account but not every authority model"* are different findings.

```rust
pub enum EnumerationCompleteness { CompleteForQuery, Partial, Unavailable, Unsupported }
pub enum AuthorityResolutionCompleteness { Complete, Partial, NotPerformed }
```

`EnumerationCompleteness` is determined by the scan alone: the response byte cap,
the decode cap, provider failure or refusal, and rows that fail mint or runtime
program verification. **Reaching the authority-lookup budget never downgrades
it.** Authority resolution carries its own axis plus exact `inspected` and
`unresolved` counts.

The stress policy requires both axes independently, so an incomplete acquisition
produces an `Incomplete` finding naming which axis fell short.

## 3. Fresh acquisition only

Population discovery is a new bounded read-only acquisition:

```
getGenesisHash
getAccountInfo   <mint>
getProgramAccounts <mint's own token program>
                 { encoding: base64, commitment: finalized, minContextSlot,
                   withContext: true, filters: [ memcmp @0 == mint ] }
getMultipleAccounts <distinct positive-balance authorities, chunked>   × N
```

There is **no `dataSize` filter**, so Token-2022 accounts with extensions are not
silently excluded. Every returned row has its mint and its runtime program owner
verified before decoding; rows that fail are retained separately as
`UndecodedAccount` and are never counted as zero balances.

A historical Phase 2/6/7 population is a different type and cannot deserialize
into this path. `population_capture.historical_population_used` is a published
`false` that the Node service re-checks. If the provider cannot serve the scan,
the result says `Unavailable` or `Unsupported` and keeps the trustworthy subset —
saved account inventory is never substituted.

Separate finalized contexts are reported as such: `atomic_single_slot` is always
`false`.

## 4. State-shape classification

`eplyx-conversion-stress-shape/v1` digests execution-relevant characteristics
only:

- authority model, resolution, on-curve, runtime owner, executable
- initialized, frozen, delegate present, active delegation, close authority
- sorted account extension types
- mint token program, paused, transfer hook **active** (a configured program id,
  not mere extension presence), transfer fee configured, default account state,
  permanent delegate, confidential transfer, confidential mint/burn,
  non-transferable
- `balance_positive` — zero versus positive only

Raw balance, balance bucket, account address and owner address are deliberately
**absent** from the key. Balance diversity is a separate selection dimension and
identity never enters a grouping key.

## 5. Eligibility

| Observed state | Eligibility | Assumed local signer |
| --- | --- | --- |
| Resolved, on-curve, System-owned, empty, non-executable authority | `ExecutableCandidate` | yes, possession unknown |
| Program-owned authority | `Unsupported` | no |
| SPL multisig authority | `Unsupported` | no |
| Unknown / off-curve / executable / data-bearing authority | `Unsupported` | no |
| Authority outside the lookup budget | `CaptureRequired` | no |
| Uninitialized, frozen, confidential account | `Unsupported` | no |
| Paused mint, confidential mint/burn | `Unsupported` | no |
| Zero balance | `Invalid` — not exposure, never selected | no |

`classify::assumed_local_signer` is the single place that grants the assumed
signer, by authority model alone, so a new case type cannot quietly acquire it.

## 6. Deterministic selection, frozen before execution

`eplyx-conversion-stress-select/v1`, three phases, blind to every result:

1. **`NewStateShape`** — one executable case per distinct discovered shape, shape
   key ascending.
2. **`NewBalanceBucket`** — one executable case per observed balance bucket not
   yet covered anywhere, bucket ascending.
3. **`HighestRemainingBalance`** — remaining budget, largest observed balance
   first.

Ordering inside any group: raw public balance descending, then token account
address ascending. Buckets are rank quartiles over this capture's positive
balances, with exact boundaries recorded; they order tests and are explicitly
**not** economic classes and **not** a representative sample.

The `StressTestPlan` binds the population capture digest, the candidate plan
digest, the candidate program digest, classifier and selector versions, the
budget, the bucket definition, a `classification_sha256` over **every** candidate
classification, every selected case with its reason, and the frozen timestamp.
`plan.validate()` recomputes the entire selection from the capture and compares,
so an expected classification, a selected case, an amount or a selection reason
cannot be rewritten after results are known.

Amount policy: the **full observed public balance** of each selected account,
pinned as an exact `Custom` amount whose decimal round-trip is asserted.
`amount_capped` is a published `false`; there is no silent capping path.

## 7. Execution

Each case takes its own five bounded read-only requests and then runs through the
**existing Milestone 6 adapter** — `demo::build`, `execute_probe_message`,
`demo::reconcile`. There is no second execution engine and no second
reconciliation. Cases start from their own captured banks and are never applied
sequentially to one another.

Per case the result records `Proven`, `Failed`, `Indeterminate` or `Unsupported`
with the Milestone 6 meanings: `Failed` requires an actual executed instruction
that failed with verified rollback; `Indeterminate` is missing or unverifiable
public state; `Unsupported` is an executor boundary. None is collapsed into
another.

## 8. Aggregation without extrapolation

`assert_no_proof_inheritance` runs in aggregation **and** again in offline replay:

- every selected case keeps exactly one result, bound to its exact case id,
  entity, account, amount, case plan and shape
- only a selected, executed and reconciled entity can be `Proven`
- `Proven` requires actual local execution
- no case may claim issuer binding, signer possession, fund movement or an
  official transition
- shape coverage counts stay consistent with the exact executed entities, and a
  shape can never report evidence for an entity that was never selected

Shape rows carry **no status**. Raw balance is attributed to an exact entity id
exactly once, so nothing is double counted, and no field sums independent case
outputs into capacity or liquidity.

## 9. Four readiness scopes, none overwriting another

| Scope | Question |
| --- | --- |
| `SelectedEntityReadiness` | Can this exact tested amount move or be sold? |
| `CandidatePlanReadiness` | Does the supplied plan work for this one account? |
| **`ConversionStressReadiness`** | Does it work across the production states we tested? |
| `PopulationRolloutReadiness` | Is the whole observed population ready? |

The stress policy `eplyx-conversion-stress-readiness-v1` is data, declared
non-issuer, and evaluated by the **existing** requirement engine through four new
conditions — `EvidenceIsolation`, `PopulationAcquisition`,
`SupportedShapeCoverage` and `SelectedCaseOutcomes` — plus one
`CandidateConversion` requirement per exact case. A real failed case is the only
condition that **blocks**; unsupported and indeterminate outcomes and an
incomplete acquisition leave the finding **incomplete**.

`PopulationRolloutReadiness` is evaluated separately through
`PopulationConversionCoverage`, which requires every positive-balance account to
have its own proven conversion at its own full amount. A bounded sample cannot
satisfy it.

## 10. Refresh

Entity ids embed the population capture digest
(`current-stress:<capture sha256>:<token account>`), so a refreshed world produces
new identities, a new plan digest and no inherited proof. Old evidence cannot be
replayed against a new capture.

## 11. Security boundaries

The browser may request exactly one thing — *stress-test the registered candidate
plan of this completed conversion* — and supplies only a request key. The API
rejects any other body field. It cannot supply a budget, mint, account, amount,
program, program id, instruction, account meta, transaction byte, filesystem
path, RPC endpoint or claimed status.

The server controls population acquisition, the candidate plan, the classifier,
the selector, the execution budget, VM construction and artifacts. The two fresh
capture children receive the RPC URL; the offline planning and execution children
receive `PATH` only, so no provider credential can reach the VM. No keys, no
mainnet signing, no transaction broadcast.

## 12. Bounds

| Bound | Default | Override |
| --- | --- | --- |
| Response bytes (population scan) | 128 MiB | `EPLYX_STRESS_MAX_RESPONSE_MB` |
| Decoded accounts | 100 000 | `EPLYX_STRESS_MAX_ACCOUNTS` |
| Authority lookups | 20 000 (batches of 100) | `EPLYX_STRESS_MAX_AUTHORITIES` |
| Selected cases | 10 | `EPLYX_STRESS_MAX_CASES` |
| Executions per case | 1, full observed balance | fixed |
| RPC requests per case | 5 | fixed |
| Concurrency | serial (1 RPC, 1 VM) | fixed |
| Population timeout | 420 s | `EPLYX_STRESS_POPULATION_TIMEOUT_SECONDS` |

Every override is server-side and clamped to the validated range; the browser
reaches none of them. The budget is persisted into the frozen plan, so no runtime
or resource limit is implicit.

## 13. Commands

```bash
eplyx-lifecycle conversion-stress-budget
eplyx-lifecycle capture-conversion-stress-population --mint <mint> --run-id <run> --stress-id <id> --out <population.json>
eplyx-lifecycle plan-conversion-stress            --population <population.json> --plan <candidate-plan.json> --out <stressplan.json>
eplyx-lifecycle capture-conversion-stress-cases   --population <population.json> --stress-plan <stressplan.json> --out <cases.json>
eplyx-lifecycle replay-conversion-stress          --population <population.json> --stress-plan <stressplan.json> --cases <cases.json> \
                                                  --run-id <run> --stress-id <id> \
                                                  --population-sha256 <..> --stress-plan-sha256 <..> \
                                                  --cases-sha256 <..> --program-sha256 <..>
```

`replay-conversion-stress` performs **no RPC**. It re-derives the population from
its raw responses, recomputes the entire frozen selection, re-executes every
selected conversion in the VM, and rebuilds the coverage and both readiness
findings. It never trusts a serialized `Proven` row:
`VerifiedConversionStressTest` has no deserialization constructor.

## 14. Validation

Pending. Results are recorded here once the commands have actually been run.
