# Phase 15: consumer-first frontend and real offline analysis

## 1. Changes to the existing frontend

The existing blue landscape, logo/sculpture geometry, hero composition, orbit
animation, typography and navigation remain. The existing rolling headline,
including stock split, is unchanged at the user's explicit request. A guided
analysis form sits immediately below the hero. The primary hero CTA opens it and
focuses its review selector. Default content explains saved example holdings and
requirements; precise published evidence remains available in Technical mode.

## 2. One presentation toggle

The existing switch under the logo now reads Overview / Technical. Overview is
the default for new visitors. Its preference applies to the entire application,
including the saved evidence route and new results. Switching changes presentation
only: the same typed engine result, selections, active run and policy remain.
There is no second mode switch, weaker evidence rule or authorization capability.

## 3. Supported inputs

- Asset: only the SPACEX PreStocks saved example.
- Review: transition overview, example token holding, example liquidity position,
  or a rollout assumption.
- Stage: before transition, during transition, or after the stated deadline.
- For rollout assumptions: full position exit, exact mandatory sale route,
  Transfer/official conversion, or narrowly scoped principal removal.

These are captured examples, not the visitor's wallet. Other assets, arbitrary
accounts/amounts, paths, URLs, policies and submitted execution statuses are rejected.
Each selection enters the validated request or selects its actual result scope.
Scenario stages reuse the same saved observations. They do not predict future
balances, liquidity or execution availability.

## 4. Request → engine → result

The existing Node development/production server hosts a small same-origin local
API. `POST /api/runs` validates a bounded selection and UUID request key. The
service invokes the already-built Rust executable using fixed argument arrays,
without a shell. Overview/holding/position run `compare-scenarios`; the selected
view/scope is rendered from that engine output. Assumptions run `evaluate-rollout`
with the allowlisted original plan and an existing pinned target view.

The additive `--target-view` flag selects one of the three verified existing
counterfactual views in memory. It never rewrites the original candidate, policy,
scenario or execution proof. Existing defaults and canonical Phase 13/14 output
remain unchanged. No frontend requirement evaluator was added.

A unique run envelope records selection, resolved identifiers, processing state,
operation, executable/artifact hashes, canonical result references and timestamps.
Timestamps stay outside deterministic engine JSON. A separate download returns the
exact stdout bytes bound by the run's canonical SHA-256.

## 5. Fresh evaluation versus saved evidence

Every submitted run invokes the engine anew; `cached` is false. Counterfactual
analysis verifies the immutable captured world and reuses the original frozen
readiness findings. Rollout analysis verifies/regenerates its pinned bindings,
assesses the selected claims and evaluates the selected existing assurance policy.
These operations do not rerun swaps/withdrawals, capture current accounts or submit
transactions. Historical local execution is retained with its original clock,
amount, range, signer and venue scope. The saved published report is explicitly
labeled as saved, and restored completed runs are labeled as earlier evaluations.

## 6. Start commands

```sh
npm ci
npm run build:engine
npm run dev
```

Open **http://127.0.0.1:4173**. `npm run dev` starts both the frontend and analysis
API in the same local service. There is no separate backend command or wallet.
Do not compile on each click. For a built frontend, run `npm run build` followed
by `npm start`; that same Node server supplies the API. A static host alone can
show saved reports but cannot execute analysis. If an older server already uses
4173, restart that project server or use `PORT=4184 npm run dev`. No public
deployment was added.

## 7. Four actual end-to-end outcomes

The final browser submitted four actual validated requests to the local service;
all four invoked the real built engine. Outputs and downloaded canonical bytes
matched their recorded SHA-256 bindings. No marker or authorization was invoked.

| Actual frontend request | Processing | Engine exit | Assurance | Scope |
| --- | --- | ---: | --- | --- |
| overview / general / after_transition | Completed | 0 | Incomplete | PopulationRolloutReadiness |
| assumption / complete-exit / after_transition | Completed | 4 | Incomplete | PopulationRolloutReadiness |
| assumption / required-sale / after_transition | Completed | 3 | Blocked | PopulationRolloutReadiness |
| assumption / principal-removal / after_deadline | Completed | 0 | Ready | DemoEntityReadiness |

The positive control used the after-deadline view and remained narrowly scoped
to historical principal-removal assurance. The broader saved population result
remains Incomplete. [Actual browser observations](../reports/phase15-validation/real-browser-observations.json),
[complete run envelopes](../reports/phase15-validation/actual-run-envelopes.json).

## 8. Loading, error and reload behavior

Jobs are Queued / Running / Completed / Error. Polling reads actual job state;
there are no invented percentages or detailed unobserved stages. One engine child
runs at a time, with at most four queued jobs. Concurrent requests sharing one key
produce one job; repeated clicks are disabled while a run is pending.

Completed Incomplete, Blocked and valid rejected-candidate outcomes are completed
analyses, not infrastructure errors. Child timeout, invalid input, missing executable,
verification failure and malformed result are explicit errors without a readiness
grant. The previous completed result remains visible during a new run. Browser
selection/run references and local service records restore after reload. Interrupted
runs are marked Error after a service restart rather than silently rerun.

## 9. Security and evidence boundaries

The service binds to loopback, restricts origin/Host, limits request bodies to
2 KiB and accepts only registry choices. Browser input cannot select commands,
executables, files, RPC endpoints, evidence or self-declared Proven results.
The original engine verifier is authoritative. Error messages omit stderr and
local absolute paths. There is no wallet, signing, issuer administration, mainnet
transaction or Phase 14 marker action in this API. Run envelopes explicitly record
`authorization: false` and `guarded_action_invoked: false`.

Readiness 0/3/4 and valid candidate rejection 5 are retained as analytical outcomes;
verification/input error 2, crashes and timeout are errors. Analysis exit 0 never
supplies rollout permission. Narrow principal-removal Ready never becomes population
Ready. Demonstration requirements, assumed signing/unknown key access, separately
saved observation times and hypothetical effective time remain visible in plain
language. Raw digests, enums, slots, provenance and JSON belong to Technical mode.

## 10. Desktop/mobile evidence

Before/after screenshots are in `reports/phase15-screenshots/`:

- [Before desktop](../reports/phase15-screenshots/before-desktop.png).
- [After desktop](../reports/phase15-screenshots/after-desktop.png).
- [Before mobile](../reports/phase15-screenshots/before-mobile.png).
- [After mobile](../reports/phase15-screenshots/after-mobile.png).
- [Desktop result](../reports/phase15-screenshots/result-desktop.png).
- [Mobile result](../reports/phase15-screenshots/result-mobile.png).

The final desktop/mobile hero and result screenshots were visually inspected.
Logo, landscape, colors and hero composition remain; readable results lead with
the answer and fit the mobile viewport. Keyboard CTA focus, existing technical
path details, form labels, live status announcements and navigation were checked.

## 11. Files changed

The [exact Phase 15 file list](../reports/spacex-phase15-files.txt) separates these changes from the existing dirty tree. [Artifact checksums](../reports/spacex-phase15-artifacts.sha256) bind delivery files. Changes cover
small engine CLI view selection, the existing Node server/registry/job adapter,
consumer/technical presentation helpers, guided form and result rendering, preserved
existing browser assertions adapted to the new toggle, focused browser/service tests,
and documentation. No captured data or historical published artifact is overwritten.

## 12. Final validation

All final checks ran after the last code change (CTA keyboard focus):

- `make test`: **354 passed, 0 failed, 0 ignored**; both synthetic SBF versions built.
- `make fmt-check` and `make lint`: passed for the engine and both fixture versions.
- `npm run check:frontend` and `npm run build`: passed.
- `npm run test:service`: **9 passed, 0 failed**.
- `npm run test:frontend -- --workers=1`: **10 passed**, including all six retained original browser tests and four new tests, with four real engine evaluations through the actual frontend.
- `git diff --check`: passed.
- **552 historical input/artifact files unchanged**, including original captured bytes, program evidence, policies and Phase 13/14 results.
- **673 executable source/config/input bindings unchanged across final checks**, digest `38420faf9195299d27706d7a6c0ae7b71ecbe06c65212b38861cef1bb037dfe8`.

[Before-test identity](../reports/phase15-validation/final-source-before-tests.json),
[after-test identity](../reports/phase15-validation/final-source-after-tests.json),
[validation record](../reports/spacex-phase15-validation.json).
Only completion documentation and delivery records were finalized afterward.
The original six-browser baseline is recorded separately. Earlier raw-detail
visibility and CTA focus assertions exposed genuine UI problems and were fixed;
interrupted pre-fix suite runs are retained under `aborted-before-*` and are not
counted as final validation. A two-worker runner reached ten passing assertions
but did not exit; its log is retained separately and the unchanged suite was
rerun serially for a complete exit-0 result. Original engine assertions were not weakened.


## 13. Unsupported functionality

No arbitrary assets/addresses/amounts, fresh blockchain scan, automatic event
detection, live monitoring, new execution replay, fee collection, closure, official
conversion adapter, auto-fix, valuation, issuer access, funds movement or public
hosted service is implemented. Quantities are exact unscaled token units because
the selected output does not provide verified display decimals/scaling. BigInt
formatting preserves large integers and positive small normalized values when
provided. Principal, recipient credit, withheld fees and protocol-accrued fees
remain separate; company shares/dollar guarantees/proceeds are never invented.
The stock-split copy and rolling headline were left intact. No subsequent phase
has begun; no reset, commit or push was performed.
