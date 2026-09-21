# Milestone 6 delivery: operator-supplied conversion-plan pre-flight

Milestone 6's research checkpoint stopped at
[stop condition C](on-demand-milestone-6-research.md): no independently executable
SPACEX → SPCXx issuer mechanism was established, so `OfficialTransition` correctly
remains `NotTested`. This milestone does not reverse-engineer further. It implements
the real pre-flight use case instead:

> An operator supplies the conversion plan they intend to deploy. Eplyx executes that
> candidate plan against fresh current production state before rollout.

## 1. Architecture changes

Two new units, no replacement of existing systems:

- `engine/src/conversion/` — `mod.rs` (plan, provenance, exact terms arithmetic, account
  origin, the opaque verified result), `demo.rs` (the registered mechanism adapter:
  derivation, proposed overlay, instruction plan, reconciliation), `current.rs` (bounded
  read-only capture and offline execution), `tests.rs`.
- `programs/eplyx-demo-conversion/` — the registered candidate SBF program.

Everything else is reused: `executor::execute_probe_message` already loads SBF by
`(program_id, loader, bytes)` into a fresh LiteSVM; `probe::meteora_dlmm` supplies the
captured-account, ProgramData, Clock, transfer-fee and byte-delta helpers;
`lifecycle::decode` supplies mint and token-account decoding; `readiness::evaluate`
supplies the exact-scope requirement engine; `preflight` supplies orchestration.
`resolution::current::resolve` is **unchanged** and still accepts only Transfer and
SecondaryMarketExit, which is why `OfficialTransition` stays `NotTested` structurally
rather than by convention.

## 2. Plan provenance model

`PlanProvenance` is `UserProposed | OperatorSupplied | IssuerVerified |
PublicIssuerMechanism`. Only `OperatorSupplied` has an executable adapter; the other
three are rejected by `ConversionPlan::validate` with an explicit reason. A plan's
provenance is fixed server-side and cannot be supplied from the browser.

An OperatorSupplied plan may satisfy `ReplacementConversion` and
`CandidatePlanReadiness`. It never satisfies `OfficialTransition`.

## 3. Supported reference mechanism

One mechanism, registered in this repository and pinned by digest:

- Name: **Eplyx Demo Candidate Conversion**
- Program id: `He4VZWmVgtbXVmHJ3tRmbLKuNDo9WG3tw5Gr36KupJUf`, derived from the published
  preimage `sha256("eplyx-demo-candidate-conversion-v1")`
- Artifact: `artifacts/eplyx_demo_conversion.so`, built by `./scripts/build-programs.sh`
- Loader: `BPFLoader2111111111111111111111111111111111`
- Design: source **burn** through the captured deployed token program, then replacement
  release from a **proposed** reserve, signed by a candidate program-derived authority

It is not deployed on any cluster, holds no issuer authority and is never presented as a
PreStocks, SPACEX or issuer mechanism. The program reads its own configuration account and
computes the fee, ratio and rounding itself in checked `u128` arithmetic before issuing two
real CPIs (`BurnChecked`, then `TransferChecked`). A host-side calculation therefore cannot
stand in for execution: the engine checks the program's own reported arithmetic against the
independently computed expectation.

## 4. Observed state versus proposed state

The execution bank is **current observed state ∪ proposed rollout overlay**.

| Origin | Accounts |
| --- | --- |
| Observed | holder source token account, source mint, holder authority, replacement mint, token programs, ATA program, their ProgramData, Clock |
| Proposed | candidate configuration, candidate reserve authority, candidate reserve vault, the candidate program itself |

Every account in the published fixture carries `origin`. `validate_origins` enforces that an
Observed account has its exact captured RPC record, pointer and slot and no derivation, and
that a Proposed account has no captured evidence and an explicit derivation. Proposed
addresses are derived deterministically from the candidate program, the plan digest and the
observed replacement identity, and the adapter refuses any overlay address that collides with
a captured address. Proposed bytes are never serialized into the capture: they are rebuilt
from the plan at replay. The saved wallet capture is never mutated.

## 5. Candidate program and configuration model

`ConversionPlan` is bounded and serializable: id, version, provenance, mechanism, adapter id,
mechanism reference, source and replacement mint, source account, amount mode and optional
decimal amount, terms, authority model, source-consumption and replacement-delivery design,
proposed reserve funding and optional effective/deadline times. There is no DSL, no scripts,
no arbitrary account metas, no transaction bytes and **no upload path**. The one mechanism
ships with the repository; `conversion-mechanism` reports its pinned digest and states
`accepts_uploaded_programs: false`.

## 6. Source account and current capture

The plan is revalidated against the exact current wallet run before anything is captured:
the source mint must be that run's mint, the account must have been discovered in that run,
the recorded owner and mint must match, and the amount must be positive and within the
observed public balance. Five bounded read-only finalized requests follow: `getGenesisHash`,
`getAccountInfo` for the source mint, `getAccountInfo` for the replacement mint,
`getMultipleAccounts` for the program headers and `getMultipleAccounts` for the final account
batch. At replay the final captured source bytes are compared with the discovery bytes; any
change returns Indeterminate with an explicit refresh-and-reconfirm requirement. The final
batch is authoritative for the conversion result.

## 7. Replacement asset

The replacement mint is inspected independently inside the check: it must exist, be owned by
a supported token program, decode as an initialized mint, and pass the executor's extension
boundary. Eplyx proves only that this is the mint used in this candidate plan. Economic
equivalence, issuer ownership and any issuer relationship remain unestablished.

## 8. Ratio, rounding and fees

`replacement = round(((consumed − conversion_fee) × numerator) / denominator)` with
`conversion_fee = floor(consumed × bps / 10000)`, all in checked `u128`, rounding `Floor` or
`Ceiling`, never floating point. Overflow is rejected rather than wrapped. Three fee kinds
stay separate and are never double counted:

- `source_token_2022_transfer_fee_raw` — always `0` here, because the source is burned, not
  transferred, and burning incurs no transfer fee
- `conversion_fee_raw` — the candidate plan's own fee, taken before the ratio
- `replacement_token_2022_transfer_fee_raw` — withheld at the holder's replacement account,
  computed independently from the observed replacement mint at the captured epoch

## 9. Authority assumptions

Holder: `signer_possession_known: false`, `signer_assumed_locally: true`. Candidate operator:
`candidate_authority_assumed_locally: true`, `candidate_authority_possession_known: false`.
The candidate authority is a program-derived address of the candidate program, so no real
issuer key is ever assumed or needed. This proves the candidate mechanism works under the
declared authority model. It does not prove that any issuer controls that key.

## 10. Execution fixture

The fixture pins the plan digest, the candidate program digest, every account with its origin
and data digest, the Clock, the exact compiled message, the captured account plan and the
runtime profile, and publishes one `execution_fixture_sha256` over all of it.

## 11. Actual local execution

`executor::execute_probe_message` runs the compiled message in a fresh LiteSVM 0.16 bank with
the captured deployed token and ATA programs plus the candidate program. Outcome, error,
logs, inner instructions, compute units, transaction fee and every watched account's
post-state are recorded. No mainnet transaction is constructed, signed or submitted.

## 12. Reconciliation

Proven requires all of: the candidate program actually invoked at depth 1; a real
`BurnChecked` CPI on the captured source token program; a real `TransferChecked` CPI on the
replacement token program; the program's own reported arithmetic equal to the independently
computed expectation; source debit equal to the requested amount; captured source mint supply
down by exactly that amount; release equal to the expected gross; reserve down by exactly the
release; holder credit equal to release minus the independently calculated replacement
transfer fee; that withheld fee observed at the destination; no withheld-fee change on the
source or reserve; and the candidate configuration account unchanged. A failed transaction
requires zero deltas and verified rollback of every watched account.

## 13. ReplacementConversion status

`Proven` on success with full reconciliation, `Failed` on a real failed instruction with
verified rollback, `Unsupported` at an executor/extension boundary, `Indeterminate` when
required state, preconditions or exact reconciliation could not be established, `NotTested`
when no plan was supplied. `VerifiedReplacementConversion` has no `Deserialize`: no JSON can
assert it.

## 14. CandidatePlanReadiness

A third gate alongside mobility and full transition, using the existing requirement evaluator
through a new `RequirementCondition::CandidateConversion`. It answers: *does this supplied
conversion plan work for this exact selected current account and amount?* It requires the
account observed and isolated, and a Proven, executed, reconciled OperatorSupplied conversion
at the exact scope, plan digest, candidate program digest and replacement asset, with the
holder signer assumption compatible. With no candidate execution the pinned digests match
nothing and the gate stays Incomplete.

## 15. OfficialTransition, separately

Unchanged and separate. The candidate conversion never enters the lifecycle path matrix, so
`OfficialTransition` remains `NotTested` and full transition readiness remains `Incomplete`.
The demo result is therefore: mobility **Ready**, candidate conversion plan **Ready**,
official issuer transition **not established**, full issuer transition readiness
**Incomplete**. That is correct.

## 16. Frontend

Under the proposed replacement transition, a **How will holders convert?** section offers
*No conversion plan* or *Test candidate conversion plan*. The candidate form exposes the
mechanism (fixed), the replacement token (taken from the proposed replacement above), the
ratio, rounding, an optional conversion fee in basis points, the amount and the proposed
reserve funding, with one CTA, **Test conversion plan**. The result reads: passed or failed
in local simulation, input, received, the exact scope, that no funds moved, and that the
official issuer conversion is not independently verified. The pre-flight panel then shows
mobility, candidate plan readiness and full transition readiness as three separate lines plus
a separate official-conversion line. Program bytes, account metas and digests stay in the
existing Technical disclosure. The existing Overview/Technical toggle and design are
unchanged.

## 17. Second asset

The same generic adapter converts a second freshly observed source asset (OPENAI) to a
different hypothetical replacement at a different ratio, in both the engine suite and the
browser suite. A test asserts the conversion engine hardcodes no mint address and names no
issuer outside its single explicit disclaimer. No issuer truth is claimed for that asset
either.

## 18. Replay

`replay-conversion-check` is the offline replay: it verifies the wallet capture, the capture
digest, the plan digest and the candidate program digest, rebuilds the proposed overlay from
the plan, reruns the actual program in the VM, reconciles again and reproduces the canonical
output byte for byte. It performs no RPC. Serialized success alone is never sufficient.

## 19. Security boundaries

No uploads are supported, and this is deliberate: one repository-registered mechanism,
hash-pinned, is substantially safer than accepting candidate binaries. The browser may choose
terms only. `validateConversionFields` rejects every unknown key, and the server fixes the
mechanism, adapter, provenance, design and source mint. Nothing executable runs on the host:
only SBF in the VM, with no network, an explicit compute budget, a bounded request budget and
a child process that receives `PATH` only, so no RPC URL or provider secret can enter the VM.
Engine and artifact digests are re-verified around every step.

## 20. Tests and mutations

Engine: 23 candidate-conversion integration tests (real SBF execution), 7 conversion unit
tests, 5 candidate-plan pre-flight tests, 4 candidate-program arithmetic tests. Service: 4
Node tests driving the real engine against a local stub that serves captured bytes. Browser:
one acceptance test covering cases A–E. Mutations: 8 injected faults, 8 killed by named
assertions, all sources restored.

Final validation, all after the last executable change: `make test` **440 passing, 0 failed**;
`make fmt-check` and `make lint` clean; **33/33** Node service tests; frontend build and check;
the full browser suite **16 passed, 0 failed, 7 skipped** (the skipped tests are the pre-existing
opt-in live-capture ones). **1061** historical data artifacts and **152** source/build identities
are unchanged, and the mutated sources match the final sources exactly. Evidence is under
`reports/milestone6-validation/`.

The milestone 2–5 browser specs rewrite their own fixture outputs on every run, so those files
were restored to their committed bytes after the suite; no historical evidence changed.

## 21. Files changed

Added `programs/eplyx-demo-conversion/`, `engine/src/conversion/`,
`engine/tests/candidate_conversion.rs`, `engine/tests/common/candidate.rs`,
`frontend/conversion-service.mjs`, `frontend/tests/conversion.test.mjs`,
`frontend/tests/conversion.spec.js`, `scripts/test-milestone6-mutations.py`,
`reports/milestone6-validation/`, this document. Modified `Makefile`,
`scripts/build-programs.sh`, `scripts/test-programs.sh`, `engine/src/lib.rs`,
`engine/src/main.rs`, `engine/src/preflight.rs`, `engine/src/readiness/{mod,current,
evidence,tests}.rs`, `engine/tests/current_preflight.rs`, `frontend/analysis-service.mjs`,
`frontend/preflight-service.mjs`, `frontend/src/preflight.js`, `frontend/src/styles.css`,
`AGENTS.md` and `docs/on-demand-progress.md`. No existing evidence, report, fixture or
historical artifact bytes were edited.

## 22. Remaining limitations

A proven candidate plan is evidence about that supplied plan, at that account, amount,
replacement asset, plan version, candidate program build and captured bank, under locally
assumed holder and candidate-operator signing. It is not an issuer-defined transition, an
issuer relationship, an entitlement, an authorization or a prediction of mainnet inclusion.
The reserve, configuration and authority are proposed rollout state, not observed balances.
One mechanism and one delivery design are supported; another operator's mechanism would need
its own adapter. Population readiness is still not assessed, and the gap identified in the
research checkpoint — independent evidence of an exact issuer-bound executable source →
replacement mechanism and its terms — remains open.
