# Phase 14: candidate rollout assertions and a local assurance guard

This layer consumes an explicitly submitted demonstration/non-issuer candidate.
It keeps its machine-readable assertions, per-assertion evidence assessment,
existing assurance-policy result and guarded-workflow outcome separate. Display
statements are never parsed. It performs no RPC or VM execution.

## Trust and reuse

The operator supplies `probes/phase14-evidence-binding.json`, independently of the
candidate. It pins the unchanged original Phase 11 policy and Phase 13 artifact.
`FrozenCounterfactualWorld` uses the existing readiness manifest verifier and
independently re-derives captured pre-execution position identity. It regenerates
the complete Phase 13 result and compares it to the pinned published artifact.
The only Phase 13 source extension retains its verified evidence under the same
private immutable ownership and exposes a crate-private read-only borrow.

Candidates cannot construct `VerifiedReadinessEvidence`. Their referenced IDs,
paths and digests must match that verified catalogue; an attached `Proven` field
is invalid. Policy evaluation calls `readiness::evaluate`, including its exact
scope matching, signer assumptions, optional/required behavior and existing
`PreflightFailureMode` records. Assertion checks also use the existing scoped
requirement evaluator for path/principal and complete-exit conditions. Additional
checks compare explicitly claimed token ledgers and observed retained fees/shares/
account state; they do not create another readiness engine.

The operator trust boundary remains the existing trusted policy/manifest model,
not signed inclusion, a new VM attestation or a claim of live state.

## Frozen plans and policies

Four candidate plans live in `probes/phase14-plans/`. Each binds an exact Phase 13
view, world digest, existing `EvidenceScope`, path, signer/runtime assumptions,
structured assertions, evidence references and every required assurance ID.
Normalization sorts identified lists and map keys. Duplicate identities, invented
or omitted required IDs, wrong targets/worlds and altered evidence bindings fail.
A valid assertion with missing/substituted/mismatched proof receives an assessment
rather than being recast as an executed transaction failure.

The original population policy is unchanged. The separately identified mandatory
route variant changes only the existing optional exact-route requirement to
required. The separately identified principal-removal policy retains only the
original exact principal-unwind requirement and uses `DemoEntityReadiness`.
Derivations verify the parent digest and exact permitted policy change; arbitrary
unreported policy changes are invalid. These are explicit demonstration choices,
not issuer requirements or technical remediation.

Requested actions must also be covered by the policy's required exact action
conditions before the stub can proceed. A narrow policy's Ready result cannot
cover extra requested entities/actions even when their historical assertions are
truthful. Complete-position exit, official conversion and population readiness
remain distinct from principal-removal assurance.

## Commands and exit statuses

Evaluate one submitted candidate offline:

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- evaluate-rollout \
  --plan probes/phase14-plans/lp-complete-exit.json --format json \
  --out /tmp/phase14-candidate.json
```

Use fresh output names. Existing outputs are protected. Inputs can be absolute
paths, including `--binding`, when running from outside this repository.
The typed readiness status retains its existing contract: **Ready 0, Blocked 3,
Incomplete 4**. Candidate evaluation/guard commands return those codes, with
**2** for malformed, invalid, tampered or unverifiable input, and **5** for a
valid but unaccepted candidate or uncovered requested scope when the policy's
limited result is Ready. That code does not change the underlying readiness result.

Run one harmless guarded action:

```sh
marker_directory=$(mktemp -d)
cargo run --locked -q -p eplyx-lifecycle-impact -- guard-rollout \
  --plan probes/phase14-plans/principal-removal-positive-control.json \
  --marker "$marker_directory/principal.permitted" --format json
```

The guard always re-evaluates the actual candidate. Only its non-deserializable
verified assessment can authorize a marker. It requires Accepted, Ready, covered
requested scope and a matching typed assurance-command completion status.
Analysis completion, including `compare-scenarios` exit 0, is refused even with
an otherwise Ready candidate. Markers use create-new writes under a temporary
local directory; there is no deployment, wallet, signing or issuer integration.

Run all four local demonstrations and publish precomputed UI data:

```sh
marker_directory=$(mktemp -d)
cargo run --locked -q -p eplyx-lifecycle-impact -- demo-rollout \
  --marker-directory "$marker_directory" --format json \
  --out /tmp/phase14-rollout-demonstrations.json
```

The batch validates every input before attempting markers, sorts cases by plan
identity and reports actual per-case marker observations. Its exit 0 means the
local demonstration completed, not that any rollout is approved. Only the narrow
positive control should create a marker. Neither this marker nor refusal claims
that Eplyx stopped a real issuer rollout, prevented losses or protected mainnet.

## Time, provenance and accounting

The target evaluation time is an explicit Phase 13 policy view. Observer time and
PreEvent applicability do not bypass assurance for a submitted future target.
The retained readiness metadata, actual VM Clock and historical path scopes remain
unchanged. Historical Proven stays historical, with key possession and future
execution availability unknown. No clock warping or new execution occurs.

The counterfactual world retains captured pre-execution bytes. Share removal,
principal deltas, retained position account and accrued fees are observations of
the original independent local withdrawal execution, not current mainnet state.
Protocol-accrued SPACEX/USDC fees remain separate from destination Token-2022
withheld transfer fees. Exact retained route failure is scoped to its input/pool/
bank and rollback; it says nothing about all venues, overall liquidity or a
lifecycle cause, and does not establish a fix by supplying an arbitrary account.

The existing UI displays published assessments and actual local observations
from a pinned artifact using its existing report layout. It never parses claims,
re-evaluates assurance or launches the guard. Download pins are build-time integrity
checks, not fresh engine validation. No UI redesign or replacement is introduced.
