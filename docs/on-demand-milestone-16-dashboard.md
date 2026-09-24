# Milestone 16: local developer dashboard

`eplyx dashboard` serves a read-only workspace for one project's `.eplyx/` run store. The command runs with or without `eplyx.toml`, removes `SOLANA_RPC_URL` from its own environment and makes no provider request. It listens only on `127.0.0.1`, on the first free port from 4173 unless `--port` is set. It prints the project name, run count, counterexample count and URL. It opens a browser only if stdout is a terminal, `CI` is unset and `--no-open` is absent. Without `.eplyx/` it exits `2` and points to `eplyx init`.

## Architecture

The server is `engine/src/dashboard/`, built into the `eplyx` binary with the standard library only. Browser assets live in `frontend/dashboard/` and are embedded at compile time. The dashboard also reuses the main frontend's `mode.js`, `format.js`, `brand.js` and `logo.svg`, so it needs no Node runtime and fetches no web fonts. The single Overview / Technical switch uses the existing `eplyx-detail` preference and changes presentation only: both renderings of each value come from the same API response. `engine/src/local_store.rs` now holds the metadata, saved-counterexample, replay-input, ID and counterexample-ID formats shared by the CLI and dashboard.

Analytical truth stays in engine artifacts. The API copies statuses from `report.json`, `search/counterexamples.json` and saved counterexamples. It computes counts, joins, sort order and field equality. Alternative gate policies come from the engine's own `package_gate::evaluate` over the saved report. Where a search is recorded, the gate including the search finding comes from `evaluate_with_counterexamples` over the saved search result. The saved gate is also checked against the engine gate for its own policy. None of this replays evidence. The UI says so and shows the CLI commands that do. JavaScript filters loaded rows and chooses wording; it never derives readiness, invariant, counterexample or gate status.

## Run source and reproduction history

New runs write metadata `schema_version` 2 with `run_source`: `local` for a developer machine, and `ci` when a non-empty `CI` environment variable is set (other than `0` or `false`). `imported` is reserved for runs brought in from another store; nothing writes it yet. Schema 1 runs predate the field. They show **Not recorded**, and the source is never inferred.

Every `eplyx reproduce <cx-id>` attempt, successful or not, writes one immutable record `.eplyx/reproductions/repro_<UTC millis>_<cx digest>.json`. It holds:
- the counterexample ID, parent run and search digest
- the timestamp
- the outcome (`Reproduced` or `Failed`)
- whether the saved failure signature matched the offline replay
- a sanitized error
- the engine version and binary hash
- `no_rpc`: true once the RPC environment has been removed

A recording failure is reported as a warning and never changes the exit code. The dashboard shows each counterexample's reproduction count and last reproduction, the history table, and project totals. These records are history of what an earlier offline replay concluded, not evidence that replaces running it. Reproductions before this change were not recorded.

## Routes

Pages: `/`, `/runs`, `/runs/:id`, `/counterexamples`, `/counterexamples/:id`, `/compare?left=&right=`, `/production`, `/invariants`, `/gate`, `/project`. Production, Invariants and CI / Gate take `?run=` and default to the latest complete run. The navigation hides Compare until two runs have completed, and hides pages that have no data.

API (GET/HEAD only):
- `/api/project`
- `/api/runs`
- `/api/runs/:id`
- `/api/runs/:id/artifacts/:name`
- `/api/counterexamples`
- `/api/counterexamples/:id`
- `/api/counterexamples/:id/raw`
- `/api/compare?left=&right=`

Artifact names come from a fixed allowlist mapping each public name to one store member: reports, metadata, bindings, package manifest and config, search, authority, stress and capture files. Program bytes are not in the allowlist. Large captures are streamed only when requested and are never parsed for a page.

## Run store index

Summaries are cached in `.eplyx/cache/dashboard-index.json`. Each entry is keyed by the size and modification time of `metadata.json`, `result/report.json`, `search/counterexamples.json`, `package/eplyx.json`, `package/config.json`, or of the saved counterexample file. Every request re-checks those fingerprints and rebuilds changed entries. A missing, corrupt, symlinked or older-version index is rebuilt from sources. A write failure leaves the in-memory index in use. The index is labeled a cache, is never an evidence source and can be deleted. The dashboard writes nothing else. Runs, counterexamples and `project.json` are only read.

A run without `metadata.json` is **Unfinished**; the CLI writes metadata last. A run whose artifacts cannot be read or whose identities disagree is **Unreadable**, with the reason. Neither state becomes the latest result.

## Comparison

Comparison lists declared inputs (candidate/package/config hashes, adapter, program ID, mints, terms, reserve, source account, owner, amount, invariant definitions, gate policy) separately from observed results (population counts, completeness, conversion, readiness, stress counts, official transition). It then lists invariant status changes, added and removed gate reasons, and Git/tooling fields. It states that no causality is inferred.

Local `cx_` IDs hash an engine counterexample that embeds its package run, so they never match across runs. Counterexamples are therefore matched by observed token account, plus the search dimension for derived variants. Search conditions are *equivalent* only when search version, budget limits, observed domain description and the entire derived domain are identical, including the seed account, the bounds and the captured replacement supply. Statuses:

- **New** — only in B. A searched under equivalent conditions and, for an observed state, executed that account without failure.
- **Only in B** — any other counterexample that appears only in B.
- **Persistent** — found in both runs with the same failure signature and boundary values. **Persistent · changed** — found in both, but the signature or a boundary value differs.
- **Resolved (equivalent search)** — only in A. Conditions are equivalent, and B re-executed the state without failure.
- **Re-executed without failure** — B executed the same token account without failure, but conditions differ. This is not reported as resolved, and the captured state may differ.
- **Only in A** — B did not re-test the state under equivalent conditions, or has no search.

The UI never says "fixed". Every comparison page opens with a banner stating either the equivalent conditions or **"Search domains differ — counterexample disappearance does not prove resolution."** The run header and the Overview change summary carry the same badge.

## Security boundary

- Requests must carry `Host: 127.0.0.1:<port>` or `localhost:<port>`, which rejects DNS-rebinding pages.
- Targets may contain only ASCII letters, digits and `/_-.?=&`. Percent-encoding, backslashes, `..` and `//` are rejected before routing.
- IDs must be lowercase `run_`/`cx_` IDs present in the index.
- Every store read walks fixed server-chosen components under the canonical `.eplyx/`. It rejects a symlink at any component and checks the canonical result.
- Symlinked run directories and counterexample files are ignored, a symlinked member makes its run Unreadable, and a symlinked `.eplyx/` is refused.
- Recorded provider origins are reduced to scheme and host.
- Home directories are shown as `~`.
- Responses send `Content-Security-Policy: default-src 'self'` (attribute styles only), `X-Frame-Options: DENY`, `nosniff`, `no-referrer` and `no-store`.
- Canonical artifact paths are forward-slash on every platform.

## Deliberately CLI-only

Preflight, search, reproduce, replay, config and gate-policy changes all stay in the CLI. The dashboard shows copyable `eplyx reproduce <cx-id>`, `eplyx show <run-id>` and engine replay commands and never runs them. The dashboard reads reproduction history but never runs a reproduction. There are no accounts, cloud sync, telemetry, remote execution or editable forms.

## Acceptance

The three local Milestone 15 projects were viewed as recorded. For a same-project comparison, the healthy and underfunded run directories and saved counterexamples were cloned unchanged into one scratch project named `transition-acceptance`, using the underfunded `eplyx.toml`. Its latest run (#4) shows BLOCKED, candidate Failed, 0/10 stress proven, and 35 observed plus 2 derived counterexamples. The minimized reserve boundary shows 747,775,404,621 FAIL and 747,775,404,622 PASS, with rollback verified.

Comparing healthy run #3 with underfunded run #4 shows:
- WARN → BLOCK
- candidate binary unchanged
- proposed reserve 1,000,000,000,000 → 0
- conversion Proven → Failed
- stress 10/10 → 0/10
- 0 → 37 counterexamples
- both blocking invariants Satisfied → Violated

The comparison marks the search conditions as not directly equivalent: the healthy search had no failing seed, so it recorded no derived domain. The reverse direction reports 10 accounts re-executed without failure and 27 only in A; none are resolved. The second asset displays through the same generic address and term rendering. Real offline `eplyx reproduce` runs of the reserve boundary and one observed counterexample each took about 50 seconds, both verified, and produced the two reproduction records used by the fixture.

`fixtures/dashboard/` holds a presentation-only copy of those runs, built by `scripts/dashboard-fixture.mjs`. Captures and program bytes are omitted, and duplicated per-case execution contexts are dropped from `report.json`. `engine/tests/local_dashboard.rs` and `npm run test:dashboard` use it. The fixture cannot be replayed. Checks and acceptance values are recorded in `reports/milestone16-validation.json`.
