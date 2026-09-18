# Phase 14 production report: validate the rollout assumption

## 1. Implementation scope and reuse

A thin offline `rollout/` layer accepts bounded structured demonstration candidates,
assesses their exact cited historical evidence, calls the existing Phase 11
requirement/readiness evaluator and produces a separate local workflow disposition.
It reuses Phase 13 immutable world verification and fresh regeneration, original
path facts, CompleteExitFact accounting and PreflightFailureMode records. Its only
change to Phase 13 retains verified evidence privately for read-only reuse; published
Phase 13 output remains unchanged. No original execution assertion is weakened.

The starting actual dirty worktree was validated before implementation: **332
engine/fixture tests passed**, both SBF programs built, and **five frontend browser
tests passed** with syntax/pin/build checks. All **489** recorded starting files
retained their hashes before the first implementation edit. Starting HEAD is
`3787a697bbdd9ed08d87e5b63800724d4de2d55c`, branch `main`, worktree digest
`9ef06f321ed1b1d8e48f60def79b376f5f8c7250e90dfb9a250c0b6738bce256`.
[Baseline record](../reports/phase14-validation/baseline-result.json).
Existing uncommitted work was preserved; no commit or push was performed.

## 2. Candidate plans and policy variants

All four plans explicitly declare DemonstrationNonIssuer provenance, version 1,
the exact Phase 13 `after_transition` view at **2026-09-17T20:00:00Z**, frozen world
digest, requested action scopes, signer/runtime assumptions, structured assertions,
pinned catalogue references and every required assurance-condition ID. Display
statements are not parsed. Each new policy declares its parent digest, separate
identity, exact derivation and rationale.

| Candidate | Assurance policy | Purpose |
| --- | --- | --- |
| lp-complete-exit-claim | Original stocklana-spacex-preflight-v1 | Assert principal removal establishes complete exit |
| required-failed-route-claim | stocklana-phase14-required-exact-route-v1 | Make the retained exact optional sale mandatory |
| transfer-official-conversion-claim | Original stocklana-spacex-preflight-v1 | Substitute Transfer proof for official conversion |
| principal-removal-positive-control | stocklana-phase14-principal-removal-only-v1 | Claim only measured local principal removal |

The required-route variant changes only `optional-failed-route.required` from
false to true; all other population requirements remain unchanged. The narrow
variant retains only `lp-principal-unwind` and uses DemoEntityReadiness. It does not
weaken or replace the original population policy. Derivation verification rejects
unreported requirement changes, not just altered hash strings.

## 3. LP complete-exit assessment

Position `BpTBNQ7vNaBujkwhgyBoiYyvEc6KrUUNTGiQDsrwTNxN`, pool
`v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc`, full range **-102 to -53**,
**10,000 bps**, captured/local VM slot **448082469**, recorded owner
`55uxDcXEaUjwit2Mv3EoNrtaGqTTUaUCvXpABpkoUUoE` assumed to sign locally;
key possession remains unknown.

Supported assertions preserve the original successful exact principal withdrawal,
its reconciled token deltas and zero local post-execution liquidity shares.

| Raw ledger | SPACEX | USDC |
| --- | ---: | ---: |
| Principal removed | 21,296,808 | 8,132,564 |
| Public owner credit | 21,190,323 | 8,132,564 |
| Destination withheld transfer fees | 106,485 | 0 |
| Retained protocol-accrued position fees | 126,543 | 113,735 |

The last two rows are separate accounting categories. They are read from verified
original artifacts, not production-code constants. The position account remained.
No remaining protocol-fee exposure and complete exit are **Contradicted** by the
retained local observation. Fee collection is **NotEstablished / NotTested**.
Closure completion is contradicted by the retained account while independent
closure execution remains **NotTested**. Principal removal supplies no official
conversion or complete lifecycle proof.

The candidate is not accepted. Under the unchanged original complete-exit policy,
readiness remains **Incomplete**, not a manufactured Blocked. Withdrawal remains
Proven. Evidence pointers identify `/token_reconciliation`, post-position shares/
fees, retained-account state and original paths. These are post-state observations
of the original independent local execution, not present mainnet state; Phase 13
still uses captured pre-execution bytes.

Truthful wording: “The captured principal withdrawal succeeded locally; accrued
protocol fees and the position account remained, and collection/closure have not
been independently executed.” Rewriting the statement does not repair the position
or make broader rollout readiness Ready.

## 4. Exact required failed-route assessment

The original retained measurement `group-2-raw-14577177576` binds entity
`solana-token-account:ENc8TdLutJ2ziFnz9x4uV8pEmdnk6iWYaBHaTAPCejpV`,
input **14,577,177,576 raw SPACEX**, pool
`22PthLk8TYnurtbWKRyECFd99cHHbfsbPNHfeMetzfZg`, captured VM bank **448018480**,
custom **6036 / BitmapExtensionAccountIsNotProvided**, rollback verified.
The captured source-before amount remains **1,457,717,757,690 raw** and is not
substituted for the requested input or an earlier population observation.

The claimed successful sale is Contradicted. The original optional requirement
and original Incomplete policy result remain unchanged. Making this exact retained
sale mandatory causes the existing Path requirement evaluator to return Blocking
and the separate demonstration policy to return **Blocked (exit 3)**.

**This exact required route failed under this captured execution context.**
It does not establish that the holder cannot sell elsewhere, that the token lacks
liquidity, that the lifecycle event caused the error, that an arbitrary supplied
account fixes it, or that a mainnet transaction failed due to this demo.
The candidate relies on an already contradicted exact route assumption.

## 5. Transfer and official conversion

Canonical holder `741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs` retains exact
**17,621 raw** successful Transfer proof and original locally assumed holder signing.
Its measurement binds original destination/context, fixture digest and VM Clock at
slot **448018365**. Transfer success is Supported; claiming it completes official
conversion is **EvidenceSubstitution**, with PathEvidenceSubstitution and
OfficialTransitionNotTested reason codes.

OfficialTransition remains **NotTested**, execution_attempted false. No official
conversion was executed or failed. Original policy readiness remains Incomplete.
Transfer is still Proven for its original exact scope.

## 6. Truthful positive control

The principal-removal-only candidate claims exactly the original local operation,
reconciled principal token deltas and zero remaining local liquidity shares.
It is **Accepted / Ready / DemoEntityReadiness** under its separate one-requirement
policy. This is principal-removal assurance, not complete-position exit, official
conversion, population rollout readiness or current/future execution availability.
Original population readiness remains Incomplete. Additional requested actions
not covered by the narrow policy cannot pass the guard even if their historical
assertions are truthful.

## 7. Actual guarded-demo behavior

Final independent replay after mutation restoration returned **0** and reproduced
both published JSON and text byte for byte. Actual directory inspection found
**one marker**, belonging only to the principal-removal positive control; its
contents matched the recorded SHA-256. Refused cases created no marker.

| Candidate | Claim acceptance | Readiness / exit | Actual local marker |
| --- | --- | --- | --- |
| lp-complete-exit-claim | NotAccepted | Incomplete / 4 | Not created |
| principal-removal-positive-control | Accepted | Ready / 0 | Created |
| required-failed-route-claim | NotAccepted | Blocked / 3 | Not created |
| transfer-official-conversion-claim | NotAccepted | Incomplete / 4 | Not created |

[Actual observations](../reports/phase14-validation/final-offline-observations.json). The guard requires typed Accepted, Ready, covered
requested action scope and a matching assurance-command exit status. A privately
constructed verified assessment is required; deserializing a claimed report cannot
create authorization. Markers are inert create-new files in temporary local storage.

Readiness codes remain **Ready 0, Blocked 3, Incomplete 4**. Evaluation/guard errors
return **2**; a valid unaccepted candidate or uncovered action under a narrowly Ready
policy returns **5** while the underlying readiness remains Ready. Batch demo and
Phase 13 analysis exit 0 mean completed analysis, not assurance or authorization.
The typed AnalysisCompleted context is refused even for an otherwise Ready control.

The local gate can truthfully be said to have prevented its stub from proceeding.
It makes no claim about real issuer operations, user losses or mainnet intervention.

## 8. Evidence and policy bindings

World digest remains
`c0cf207b91abd06e70f4a0e8b01a6614e941c4ff6709081e39d6d4880b3ebf78`.
The independently supplied Phase 14 trust binding pins the original parent policy
and exact Phase 13 artifact; existing readiness verification remains authoritative.
Fresh Phase 13 regeneration is required. Candidate references are checked against
verified artifact IDs/path/digests, never trusted as independent attestation.

| Assurance artifact | Raw SHA-256 |
| --- | --- |
| `policies/stocklana-spacex-preflight-v1.json` | `162655313c70cba8adc4417d535128f73d0e8de6cc36d8b1d04586e8a1993c4d` |
| `policies/phase14-required-exact-route.json` | `d836cdd3f28a7a2a6bde7de5a716f0ebdcdb075d9de3807f23a14d2d39a20b32` |
| `policies/phase14-principal-removal-only.json` | `060cabd7cba03c4e26f759717e83be7b3e43ed71f3c5b183d07f2958e52ba855` |

All existing scopes retain historical scenario
`9d7d0556a55fd97885cca6f62105d3550ae8d0946b2ec473502bcba050035cea`.
Target policy time, legacy readiness evaluation metadata, actual captured VM Clock
and historical execution bank remain separate. Observer time / PreEvent do not
bypass future-target requirements. Proven historical evidence is never rewritten
as new future-state proof.

## 9. Unknown and untested

Official conversion mechanics, execution, eligibility and key possession remain
unknown/NotTested. Independent fee collection and position closure are NotTested.
Current/future position state, liquidity and execution availability are not proven.
Other routes, peers and population-wide actionability do not inherit sampled proof.
No new capture, execution campaign, issuer authentication, closure/fee adapter,
valuation, risk score, natural-language interpretation or next phase was added.

## 10. Files changed and artifacts

New structured candidate/policy data, `engine/src/rollout/mod.rs`, focused tests,
CLI commands, small Phase 13 read-only retention, freeze/mutation scripts and
Phase 14 documentation implement this phase. The existing frontend gains one
published case panel using existing styles and its pinned artifact pipeline; no
layout redesign, new UI assurance engine or guard invocation is introduced.
The [exact Phase 14 file list](../reports/spacex-phase14-files.txt) separates this work
from the pre-existing dirty tree. The canonical UI JSON SHA-256 is
`f2bf9ffb52d742e2f36f0bae99a6916e90fd74d75c9d6a046e7d465d65039978`.
[Direct JSON/text checksums](../reports/spacex-rollout-assumptions.sha256)
and the aggregate manifest bind the delivered files.

- [Canonical UI artifact](../reports/spacex-rollout-assumptions.json).
- [Candidate cases](../probes/phase14-demo-cases.json).
- [Design, trust and commands](lifecycle-phase-14-rollout.md).
- [Validation record](../reports/spacex-phase14-validation.json).
- [Targeted mutation record](../reports/spacex-phase14-mutation-results.json).
- [Files changed](../reports/spacex-phase14-files.txt).
- [Artifact checksums](../reports/spacex-phase14-artifacts.sha256).

## 11. Final validation and mutations

All final checks ran **after the last executable code change and mutation restoration**:

- `make test`: **354 passed, 0 failed, 0 ignored**, including all 332 original engine/fixture tests and 22 new rollout tests; both SBF versions built.
- `make fmt-check`: passed.
- `make lint`: passed for all engine targets and both fixture versions.
- `npm run check:frontend` and `npm run build`: passed.
- `npm run test:frontend`: **6 passed**, preserving all five original browser tests.
- Independent four-case offline demonstration: exit 0, saved JSON/text byte-identical to published artifacts; exactly one actual inert marker, positive control only.
- `git diff --check`: passed.
- **490 historical frozen input/artifact bytes unchanged**, including original policies/execution outputs and Phase 13 report/validation artifacts.
- **5 executable faults injected; 5 caught through exact named assertion failures**. Compiler failures do not count. Final rollout source restored byte for byte: `84573450e70dbf3cb30c0637bbcab23e1bf7bf43db94c540f56b155e3c899568`.

Final executable source and tested input digest is
`6b303f2856a49247a20c2bcb0c160c43d453613665aa661c6b2ef105f75e658d`. Its **613** source/input bindings are identical before and after final validation.
[Before-test identity](../reports/phase14-validation/final-source-before-tests.json),
[after-test identity](../reports/phase14-validation/final-source-after-tests.json),
and [final complete worktree identity](../reports/phase14-validation/final-worktree.json).
Only completion documentation/validation records were finalized afterward;
no executable source, candidate, policy or evidence changed after these checks.
Starting and final full-suite runs are recorded separately, never substituted.

| Executable injected fault | Named assertion | Result |
| --- | --- | --- |
| Promote principal withdrawal to complete position exit | [`principal_withdrawal_is_not_complete_exit`](../reports/phase14-mutations/01.txt) | Caught, exit 101 |
| Promote movement proof to official conversion | [`transfer_is_not_official_conversion`](../reports/phase14-mutations/02.txt) | Caught, exit 101 |
| Ignore explicit optional-to-required policy variant | [`required_exact_failure_blocks`](../reports/phase14-mutations/03.txt) | Caught, exit 101 |
| Permit Incomplete assurance through the guard | [`incomplete_never_authorizes_stub`](../reports/phase14-mutations/04.txt) | Caught, exit 101 |
| Treat generic analysis exit zero as assurance completion | [`analysis_success_is_not_assurance_approval`](../reports/phase14-mutations/05.txt) | Caught, exit 101 |

The new tests cover retained principal/fee/share/account facts, official proof
substitution, exact failure optional/required distinction, entity/amount/venue/bank/
range/fraction/signer isolation, narrow acceptance and extra-action scope, future
views/PreEvent, analysis exit 0, typed guard refusal/permission, malformed/tampered
inputs, missing cited proof, canonical order/roundtrip, protected portable CLI and
preserved historical bytes. The actual Phase 13 CLI still completes with exit 0
without granting assurance authorization. Invalid and verification-error inputs
never created a stub marker. Development compiler/lint/selector failures were fixed
and retained separately; they are not presented as successful validation or mutation
assertion catches.


## 12. Short Stocklana demo sequence

1. Open the existing evidence viewer. The top population readiness stays Incomplete.
2. Inspect the principal/owner-credit/withheld/protocol-fee table and retained position.
3. Open Rollout assumptions → complete LP exit: supported principal removal is retained,
   while accrued fees/retained account contradict completion and readiness is Incomplete.
4. Open the required failed-route case: exact mandatory failure yields Blocked under
   its explicit variant; the original optional population policy is not rewritten.
5. Open Transfer/official conversion: movement is Supported, official proof is NotTested.
6. Open the narrow positive control: Accepted/Ready only for principal removal; its
   observed inert marker was created. Show Sources for the precomputed JSON and hashes.

Viewing the published panel does not re-evaluate claims or run any action.

## 13. What this adds

Existing safeguards already enforced exact proof isolation, incomplete complete-exit
assurance, optional/required failure semantics and separation of entity/population
readiness. Phase 14 adds submitted machine-readable rollout assertions, per-claim
assessment and truthful wording, checked explicit policy derivations, a causal
case artifact and an actual local consuming workflow that obeys typed assurance.
It demonstrates the assumption a plan relies on and why evidence does or does not
justify it. It does not establish asset safety or prevention of a financial incident.
No subsequent phase has begun.
