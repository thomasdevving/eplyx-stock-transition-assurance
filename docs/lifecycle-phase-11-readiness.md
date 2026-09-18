# Lifecycle readiness / pre-flight gate

An economic lifecycle policy describes what changes. A readiness policy describes
which exact actions must be proven before its declared scope can be called ready.
The gate evaluates existing frozen artifacts only: no RPC, VM execution or quote.
It does not classify an asset as safe or unsafe.

## Decision contract

| Outcome | Meaning | Process code |
| --- | --- | ---: |
| Ready | Every required condition has sufficient matching evidence | 0 |
| Blocked | An exact explicit policy violation exists for a required condition | 3 |
| Incomplete | Required evidence is absent, untested or incompatible | 4 |
| Input/output error | Invalid schema, integrity/linkage failure or protected output | 2 |

Blocking findings dominate incomplete findings; a satisfied unrelated condition
cannot erase a blocker. A Path requirement is satisfied by any one of its declared
alternatives. A failure in an optional route is informational. Missing evidence
cannot be reclassified as Failed. NotTested and Indeterminate default to incomplete;
an exact recorded status blocks only if the policy explicitly forbids it.
Unsupported becomes a blocker only if the exact requirement declares it blocking.
The current gate accepts Proven only with retained actual execution/reconciliation
and compatible signer assumptions. Ready never establishes private-key possession.

`DemoEntityReadiness` has no rollout result. `PopulationRolloutReadiness` requires
an explicit population coverage condition. Each positive population entity needs
its own full represented-amount evidence on an accepted path. Multiple population
requirements use the intersection of entity proofs. Zero balances are not proofs;
protocol positions are not added to the token-account population. Independent
contexts, amounts and venues are not simultaneous capacity or proceeds.

## Policy data

[The demonstration policy](../policies/stocklana-spacex-preflight-v1.json) is
`stocklana-spacex-preflight-v1`, `DemoAssurancePolicy`, `not_issuer_policy=true`.
It has seven required conditions and one optional historical route observation:

1. OfficialTransition for the selected direct holder.
2. Exact full-amount Transfer or SecondaryMarketExit for that holder.
3. OfficialTransition for the selected protocol position.
4. Exact native full-range, 100% principal Withdrawal for that position.
5. Complete position exit: principal, fee collection, closure and no residual fees.
6. Entity-specific full-amount mobility proof for every positive observed account.
7. Evidence isolation across paths, entities, venues, principal/fees/closure.
8. Optional previously failed large-holder SecondaryMarketExit route.

The complete-exit condition does not forbid remaining fees as an explicit blocker
in this demo: untested fee collection/closure produces Incomplete. Setting
`forbid_remaining_fees=true` makes a nonzero recorded fee ledger Blocking.
The official requirements have no invented executable route or capture. Existing
NotTested observations cannot satisfy their requested amounts. The mobility-only
entity policy used in controlled tests can be Ready without claiming lifecycle
completion or rollout readiness.

Requirements contain explicit scope, expected statuses, permitted signer
assumptions, blocking statuses, rollout assumption and remediation. Their IDs and
alternatives are normalized before evaluation; semantic policy digest and canonical
report are invariant to requirement ordering. The policy is separate from the
unchanged external lifecycle scenario and is not the issuer's internal policy.

## Evidence and integrity

The policy pins [a readiness evidence manifest](../probes/spacex-readiness-evidence.json).
Manifest references resolve relative to the manifest; its own reference resolves
relative to the policy. Named CLI inputs must match those declared hashes.
Execution-index and capture references are resolved using their original parent
and normalized back to manifest-relative report references. Artifact IDs are
stable; all findings link their actual artifacts through `evidence_refs`.

The reader verifies published digests, snapshot/scenario/coverage/plan/index links,
all selected amount cases, capture fixtures, actual measured status and token
reconciliation, exact direct-resolution attempts, official research linkage,
withdrawal scope/token/share conservation and the original population/update
identity and amounts. The original discovery stores a canonical parsed-JSON digest;
its file-byte digest is separately pinned. Both conventions are retained.

Exact path matching includes asset, entity/state shape, authority, raw amount,
range/fraction, path, venue/context, full captured Clock, capture context,
fixture/captured-state digest, source balance and economic scenario digest. `None`
is a value, not a wildcard. Proof of another amount, entity, venue, bank or action
receives no matching assurance. The policy's acceptance of locally assumed owner
signing is explicit. It does not change `signer_possession_known=false`.

Digest integrity is relative to the explicitly trusted demonstration policy and
its pinned published measurements. These are not signed inclusion proofs or a new
VM attestation. The gate does not recapture state, judge wall-clock freshness or
claim current/future execution: changing a requested bank/context makes the old
proof incompatible. Corrupted declared inputs are input errors; missing matching
proof within valid inputs is an Incomplete decision. Original matrices and
historical artifacts remain immutable.

## CLI and replay

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- readiness \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --policy policies/stocklana-spacex-preflight-v1.json \
  --direct-resolution reports/spacex-lifecycle-path-resolution-phase9.json \
  --position-resolution reports/spacex-dlmm-withdrawal.json \
  --coverage reports/spacex-lifecycle-coverage-phase7.json
```

Add `--format json --out <new-path>` to save the complete canonical report. JSON
stdout and saved bytes are identical, including when the decision is Incomplete
or Blocked. Outputs are created without overwriting an existing file. Absolute
input paths work from another directory without RPC configuration. No generation
timestamp enters the report. Exit 4 for this demo is a valid evaluated decision,
not a parsing or command failure.

```sh
sha256sum -c reports/spacex-phase11-artifacts.sha256
python3 scripts/test-phase11-mutations.py --suffix new-run
```

The mutation runner intentionally protects its retained production logs/report;
repeating it requires a fresh evidence destination, rather than overwriting the
original results. The production report records completed validation separately.

## Architecture

`readiness::LifecycleReadinessPolicy` and `RequirementCondition` contain generic
assurance logic. `evidence::ReadinessEvidenceManifest` adapts the existing typed
Phase 6–10 artifacts into opaque `VerifiedReadinessEvidence`. The generic evaluator
cannot deserialize or publicly construct that verified evidence. It never calls
an executor or fresh-replay validator. `LifecycleReadinessReport` carries findings,
entity/rollout status, residual fee ledger, scoped path facts, population summary,
artifact references and limitations.

`PreflightFailureMode` explains an assumption the platform would prevent a rollout
from relying on. `real_incident_claimed=false` is retained for every record. The
report makes no claim that a real rollout was blocked or any loss was prevented.
No issuer/asset conditional, new dependency, risk score or UI is introduced.
