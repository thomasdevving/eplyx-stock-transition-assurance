# On-demand analysis — milestone 6 operator-supplied conversion pre-flight

Milestone 6 is delivered: an operator supplies the conversion plan they intend to
deploy, and Eplyx executes that candidate plan against freshly captured current
production state before rollout. The full delivery report is
[here](on-demand-milestone-6-conversion.md).

Fresh current wallet state + an operator-supplied candidate plan + a registered
candidate mechanism → local Solana execution → exact old-token → replacement-token
reconciliation → `ReplacementConversion` evidence → `CandidatePlanReadiness`.

The mechanism is one repository-registered, hash-pinned program, **Eplyx Demo
Candidate Conversion**. It is not deployed on any cluster, holds no issuer
authority and is never presented as a PreStocks, SPACEX or issuer mechanism. It
burns the observed source through the captured deployed token program and releases
the replacement from a **proposed** reserve under a candidate program-derived
authority, enforcing the exact ratio, rounding and conversion fee itself in checked
integer arithmetic. No program code, instruction, account meta, transaction byte,
filesystem path, RPC endpoint or claimed status can be supplied from the browser,
and there is no upload path.

A successful candidate plan makes `ReplacementConversion = Proven` and
`CandidatePlanReadiness = Ready` for that exact account, amount, replacement asset,
plan version, candidate program build and captured bank. It never makes
`OfficialTransition` anything but **NotTested**, and full transition readiness stays
**Incomplete**. The demo result — mobility Ready, candidate conversion plan Ready,
official issuer transition not established, full issuer transition readiness
Incomplete — is correct. Transfer, market exit, a DEX swap into the replacement
token, successor-mint existence and an unrelated burn plus MintTo still never
satisfy conversion. Refresh starts untested; no proof crosses runs.

## Milestone 6 final validation

`make test` **440 passing, 0 failed**; `make fmt-check` and `make lint` clean; **33/33** Node
service tests; frontend build and check; the full browser suite **16 passed, 0 failed, 7
skipped** (opt-in live capture); **8/8** mutation kills by named assertion with every source
restored. **1061** historical data artifacts and **152** source/build identities are unchanged.
The registered candidate mechanism is pinned at
`a3468decc45e3b740a2479bca4c01f1a80c09b393e4d6e25386c1cd98de1fc5b`. Browser cases A–E
(no plan, candidate plan, failed candidate plan, refresh, second asset) all pass. No mainnet
transaction is submitted and no later milestone is started.

## Milestone 6 research checkpoint (completed earlier)

Milestone 6's proof contract and a new bounded public SPACEX → SPCXx
mechanism search are recorded in
[the research checkpoint](on-demand-milestone-6-research.md). The current
PreStocks page directs holders to swap SPACEX into SPCXx **or another token**;
the public FAQ describes an on-chain post-IPO conversion, and the linked public
PreStocks announcement says conversion happens through normal trading. These
sources supply no exact pair-specific program, instruction, account plan or
terms. Sixteen fixed current public
transaction samples showed failed source routes, SPCXx market trades, source
fee administration and other xStock minting, with no coupled SPACEX debit and
SPCXx credit. A rate-limited first pass was retried once for the same selected
signatures; all 16 responses are preserved. Search stop condition C applies:
no independently verifiable executable mechanism was established within the
bounded search. This does not establish mechanism non-existence or a private
issuer blocker.

No production conversion adapter, fresh conversion capture, VM execution,
readiness promotion or enabled conversion action was added. The research
assessment of ReplacementConversion and the current OfficialTransition path
remain **NotTested**; Full
transition readiness remains **Incomplete**. Milestone 5 mobility evidence
retains its original exact scope. This is a research checkpoint, not a claim
that Milestone 6 conversion execution is complete.

That section describes the research checkpoint's own boundary. The conversion
execution delivered above is an **operator-supplied candidate** plan, not an
issuer mechanism: it does not change the research finding, and the current
`OfficialTransition` path still remains **NotTested**.

## Milestone 5 completed delivery

Milestone 5 is complete: prospective, UserProposed replacement-token pre-flight
works for one freshly observed wallet account. Fresh SPACEX and OPENAI acceptance,
refresh isolation, successor inspection and canonical offline replay passed.
Final validation passed: **401 Rust/program tests, 29 service tests, 22 browser
tests (zero skipped), 7/7 mutation kills**, format, lint and frontend build/check.
All **893** pre-existing evidence/report/fixture files and **138** pinned source/build
identities remain unchanged. No mainnet transaction is submitted; no later milestone
is started. The final evidence and production preview are recorded below.

## Milestone 4 foundation (completed earlier)

Milestone 4 added current-run Transfer and bounded Meteora market-exit checks.
Its completed delivery and validation remain recorded below. All subsequent
historical milestone sections describe their original delivery boundaries.

## Milestone 3 foundation (completed earlier)

Milestone 3 implements **current public-wallet / account analysis**, end to end
through the browser, existing Node service and Rust engine. Real mainnet acceptance
and offline replay succeeded. All final validation is complete; results and the browser startup retry are
recorded below. Its immutable wallet observations retain null lifecycle/readiness
and false execution flags. Milestone 4 now links separate, newly executed checks
to those observations without rewriting their original findings.

## Public-wallet workflow

Open `/analysis#analysis`, choose **Analyze a public wallet** under Analysis scope,
select a catalogue asset or enter a custom mint, and supply a **Public wallet
address**. **Fetch latest data and analyze** starts a new read-only acquisition.
No wallet connection, private key, seed phrase, signing or transaction is requested.
Token overview and the explicitly saved demonstration remain available separately.
Protocol-position scope and proposed lifecycle scenarios are not exposed.

The result leads with observed exposure, a successful absence of direct accounts,
or unavailable holdings. Every successfully decoded matching token account is shown
with its exact public quantity, state, delegate/close-authority conditions and
supported extensions. Rejected/undecodable rows are identified separately and retain
original evidence. Quantities use raw integers and mint decimals; display scaling,
interest transforms, encrypted balances and withheld fees are not included.

A focused account can be selected under **Selected for deeper checks**. That choice
is stored by run ID in browser UI state, survives reload and the sole hero
Overview / Technical toggle, and grants no assurance. The wallet summary remains
separate from the chosen account. Changing owner, mint, scope, source version or
sampling selection hides results for the previous scope. Refresh creates another
run and leaves the original capture intact. Overview explains the result and
unchecked paths without red NotTested failures; Technical exposes the exact
addresses, authorities, extensions, contexts, method outcomes, hashes and downloads.

## Acquisition, decoder and replay boundaries

The inspection first parses the mint and supplied owner using the existing Rust
Solana `Address` parser, before any RPC. Owner addresses need not be funded, on an
allowlist or Ed25519 curve points. It then performs at most four method calls:

1. `getGenesisHash`, requiring mainnet identity.
2. `getAccountInfo(selected mint)`, finalized, full base64 bytes. The actual runtime
   owner chooses legacy SPL Token versus Token-2022 decoding; catalogue metadata
   cannot choose the token program.
3. `getAccountInfo(public owner)`, finalized, with the mint context as a lower bound.
   A clearly decoded token account, mint or executable program is not silently
   reinterpreted as an owner. A token account can supply its decoded recorded owner
   as a suggested input for a separate run. Absent accounts and ordinary/program-data
   possible authorities remain eligible; control and signer possession stay unknown.
4. [`getTokenAccountsByOwner(owner, {mint})`](https://solana.com/docs/rpc/http/gettokenaccountsbyowner),
   finalized, base64, with the inspected owner context as `minContextSlot`.
   This is independent of `getTokenLargestAccounts` and never queries it.

The same owner + mint method is used for both token programs; no endpoint rotation
or hidden program-filter fallback is performed. A provider lacking that method for
the observed program produces an unavailable result. Each returned account is
independently checked for actual runtime program, exact mint and recorded owner.
The existing official SPL decoders preserve initialized/frozen state, delegate,
delegated amount, close authority, native reserve and supported Token-2022 extensions.
Unknown/malformed layouts create explicit gaps rather than zero amounts.

A successful empty response is **Completed**, count 0, public total `"0"`, with
“Direct token holdings found: none” and “Protocol positions were not checked”.
Partial results label the returned-row count separately from the count of matching
decoded accounts. This makes no claim about prior holdings, external economic exposure, other assets
or positions. RPC failure is **Unavailable** with count/total null. A row decoder or
scope gap produces **Partial** with the known public subtotal and a null full total.
Duplicate keys, invalid contexts or oversized responses cannot establish zero holdings.

All rows within a successful response are processed; there is no ATA-only assumption
or silent truncation. The existing 2 MiB response limit and a 1,000-account decode
budget bound the lookup. Each raw balance remains a u64 string; the public subtotal
uses checked u128 addition and is serialized as a string. Decimal rendering uses
BigInt through the full u8 decimal range. Confidential balances remain explicitly
unknown, even when a public quantity is zero. Complete response decoding is only
coverage of this owner + mint query, not a global portfolio or a proof of legal ownership.

Wallet captures use schema **3**, `current-wallet/v1`, and an optional pinned
`public_owner` selection field; older schema 1/2 captures retain their original
replay behavior. `.analysis-runs/<server UUID>.capture.json` is written create-new,
with the original response bytes, request parameters, per-method times/outcomes,
provider origin, finalized contexts and decoder version. The manifest binds the
selection, capture, engine and result hashes. `replay-current` freshly decodes these
retained responses offline and cannot acquire live data. No serialized execution
claim is accepted as evidence. Mint inspection failure is fatal; owner/discovery
failures preserve a truthful current-mint result where possible.

The existing two-attempt client, one-second retry backoff, 12-second request timeout,
90-second fresh-job timeout, serialized queue and session isolation remain. Browser
inputs cannot select RPC URLs, programs, commands, filesystem/artifact paths or proof
statuses. Calls use structured argument arrays. Captures expose only provider origin;
credential-bearing paths/query strings and provider error text are excluded. Source
text is escaped, and no metadata URLs are fetched. No dependencies were added/upgraded.

## Real mainnet browser acceptance

The actual frontend, service and built engine acquired five new mainnet wallet runs.
These are **live RPC results**, separate from deterministic fixture tests. No operator
RPC was configured; `https://api.mainnet-beta.solana.com` served every run. No owner
lookup was rate-limited. Milestone 2's largest-account 429 remains a historical
limitation and was not a prerequisite for these successful owner-scoped lookups.

Known public query target (no human identity or signing claim):
`2wCvQzHiDHAHTvzwPeof9H3uEzq8Bzvg38DFbvZMGkuj`.
Its address was taken from the saved example only as an input. Every balance in this
milestone came from the new owner-scoped response. SPACEX returned account
`741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs`, public raw balance `17621`, or
`0.000017621` base token units. The refresh independently returned the same amount.
OpenAI and custom Wrapped SOL returned zero direct accounts for this owner.

The separate zero-holdings target was
`CktRuQ2mttgRGkXJtyksdKHjUdc2C4TgDzyB98oEzy8`, queried for SPACEX.
No multiple-account live result was observed; that behavior is verified by official-layout
fixtures through both the engine and real browser/service path.

| Run | Exact mint | Fresh acquisition interval (UTC) | Accounts | Public raw total |
| --- | --- | --- | --- | --- |
| known | PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh | 2026-09-19T14:17:03.972Z → 2026-09-19T14:17:06.078Z | 1 | 17621 |
| refresh | PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh | 2026-09-19T14:17:07.907Z → 2026-09-19T14:17:09.460Z | 1 | 17621 |
| zero-target | PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh | 2026-09-19T14:17:10.912Z → 2026-09-19T14:17:12.440Z | 0 | 0 |
| second-asset | PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF | 2026-09-19T14:17:13.839Z → 2026-09-19T14:17:15.379Z | 0 | 0 |
| custom | So11111111111111111111111111111111111111112 | 2026-09-19T14:17:17.004Z → 2026-09-19T14:17:18.524Z | 0 | 0 |

Full run IDs, methods, contexts, source bindings and hashes are retained in
[`live-browser-runs.json`](../reports/milestone3-validation/live-browser-runs.json).
Every `live-*.capture.json` reproduced its corresponding result **byte-for-byte**
through offline `replay-current`. The live browser also exercised invalid-address
rejection, refresh, reload, mode switching on the same run and the separate saved
SPACEX report. Deterministic browser tests separately verify cross-session denial,
multiple accounts, exact large totals, focused-account persistence, invalid programs,
provider 429 retry/unavailability, source isolation and no-RPC invalid input.
Desktop/mobile wallet screenshots were visually inspected; neither layout overflowed.

## Validation

Baseline, before substantive edits: **11 current-observation engine tests** and
**22 service tests** passed. New wallet tests cover legacy/Token-2022, one/multiple/
zero accounts, wrong mint/owner rejection, delegate/frozen state, exact arithmetic,
confidential unknowns, invalid owner/mint/program input, response/context errors,
refresh, immutable captures, replay and no lifecycle/execution inheritance.

Final `make test`: **375 passed, 0 failed, 0 ignored**, including SBF program
builds and all 10 wallet integration tests. Format, lint, frontend syntax/published-digest
checks, production build and **24 service tests** passed. The complete browser run
passed **13 cases**, including the deterministic and live wallet cases; one older
opt-in live mint/sampling campaign was skipped. Its sole initial failure was Chrome
context startup timing out before application assertions. The unchanged-source
`--last-failed --timeout=60000` retry completed with **1 passed**: all **14 executed
browser cases** are now validated. An earlier isolated navigation diagnostic passed
its assertions but stalled during runner shutdown and was stopped; it is not counted.
An earlier browser run was also interrupted for the final partial-result label fix.
See `final-browser.txt` and `final-browser-retry.txt` for the definitive pair of runs.
Seven prior milestone 1/2 captures still reproduce their original canonical hashes.
All **262 recorded historical data artifacts** and **97 final source/configuration
bindings** remain unchanged. `git diff --check` passed. The
[final validation summary](../reports/milestone3-validation/final-validation-summary.json)
records counts, preservation checks and each final live replay. Final logs are under
`reports/milestone3-validation/`. The initial sandbox browser launch was denied
loopback listening; the same validation was rerun outside that restriction.

## Files and next milestone

Implementation changes: `engine/src/lifecycle/current.rs`,
`frontend/analysis-service.mjs`, `frontend/src/analysis.js`,
`frontend/src/presentation.js`, `frontend/src/styles.css`.
Tests: `engine/tests/current_observations.rs`, new `engine/tests/current_wallet.rs`,
`frontend/tests/service.test.mjs`, new `frontend/tests/wallet.spec.js`, and
report-directory overrides in `frontend/tests/analysis.spec.js` and
`frontend/tests/multiasset.spec.js` to preserve earlier validation records.
Documentation: this progress document, README and AGENTS scope note.
The existing RPC client, official decoders, CLI commands and catalogue infrastructure
are reused. The dirty working tree was preserved; no reset, commit, push, branch
change, new dependency, paid provider or historical-proof replacement occurred.

Start with `npm run build:engine`, `npm run build`, `npm start`, then open
`http://127.0.0.1:4173/analysis#analysis`. The production preview was restarted there;
its health and served-source hash were verified after the build. Restart a running
service after future source changes.
For replay: `target/debug/eplyx-lifecycle replay-current --input
reports/milestone3-validation/live-known.capture.json`.
Normal regressions use `npm run test:service` and `npm run test:frontend -- --workers=1`.
The bounded live test is opt-in with `EPLYX_WALLET_LIVE=1` and
`EPLYX_ACCEPTANCE_OWNER=<public query target>`; its name is `live wallet browser`.

Milestone 4 remains unimplemented: freshly capture route state for a focused
account, connect bounded local execution, reconcile its outcomes and persist new
execution evidence. No fresh Transfer, swap, Withdrawal, lifecycle-transition or
redemption execution was added here. This delivery stops at current wallet state.

---

## Milestone 2 — historical delivery record

Milestone 2 implements catalogue selection and custom-mint inspection through the
existing browser, Node job service and Rust acquisition/replay path. Implementation
and all final validation are complete. Optional live account sampling remains
unavailable under the recorded provider rate limit. Milestone 3 is recorded above.

## User-visible functionality

Open `/analysis#analysis`. Search the imported PreStocks catalogue by name, symbol
or mint, or choose **Enter a token address**. Both paths work in Overview mode.
The selected mint is shown explicitly. The controls retain the site's blue surfaces,
fonts and focus styles, with mobile layouts. The hero, sculpture, rolling headline,
navigation and sole Overview / Technical toggle under the logo remain in place.
The toggle preserves the asset, typed draft, active run and results without posting
another job. Reload restores the same run with a saved-run label and original dates.

The account sample is optional. **Token information only** records `NotRequested`;
provider failure records `Unavailable` with a null sample count. A successful sample
records the number decoded, its method, contexts and scoped gaps. It never supplies
a holder count, random sample, independently owned wallet count, concentration score,
or supply reconciliation. Current results lead with what was retrieved and a concise
statement that transactions were not simulated. Technical mode retains raw IDs,
source assertions, exact JSON, acquisition errors and downloads.

## Catalogue source and immutable selection

The actual [official PreStocks products page](https://prestocks.com/products)
contains a structured `products` array in JSON-encoded Next flight records. The
adapter parses those JSON records with `JSON.parse`, without executing scripts,
fetching images or following metadata URLs. No guessed API schema or frontend
array supplies asset identities. The import retains nine canonical Solana-mainnet
mint identities: Anduril, Anthropic, Figure AI, Kalshi, Neuralink, OpenAI, Polymarket,
SpaceX and xAI. All nine addresses passed the existing Rust Solana address parser.

- Source retrieval: **2026-09-19T10:33:42.635Z**, HTTP **200**.
- Source-content SHA-256: `0500315983475df561c9c0680cc2aac86a2ec5383c3b07d52a3efa7fe15f3c73`.
- Immutable catalogue version: `273ba7bc15259e60f4fc94070c2adc4dc41d35673aea0a28ac5c957fda944493`.
- Original HTML and retrieval envelope: `evidence/catalogue/273ba7bc15259e60f4fc94070c2adc4dc41d35673aea0a28ac5c957fda944493.json`.
- Source-derived fields are `splMint`, `name`, `symbol` and `decimals`; the adapter
  explicitly labels the Solana reference/mainnet-genesis basis. No price, issuer
  notice, successor, lifecycle date, protocol address or execution flag is imported.

The UI labels the imported source **saved**, with its original retrieval date.
Chain acquisition has its own later interval. The server pins the chosen version,
exact mint and all distinct source assertions into a server-owned selection file,
job, capture and result. Duplicate names can identify distinct mints; conflicting
assertions for one mint are retained. Archived versions remain resolvable even
when the current pointer changes. A version/mint mismatch is rejected.

Imports use only the fixed first-party URL, refuse redirects, and bound response
size to 2 MiB and request duration to 15 seconds. Acquisition attempts, including
failures, are retained separately. A failed import preserves the dated catalogue;
missing catalogue data does not prevent custom-address inspection. Tests exercise
this failure path; the actual import succeeded.

## Real browser acceptance and fresh observations

These runs used the actual frontend, server and built Rust executable, with finalized
mainnet RPC responses and no mocked acquisition. Full job IDs, timestamps, hashes,
source bindings and findings are in
[`live-browser-runs.json`](../reports/milestone2-validation/live-browser-runs.json).
Each row has its own `<label>.capture.json` and `<label>.result.json` in that directory.
Every retained capture reproduced its original result **byte-for-byte offline**.

| Run | Exact mint / supplied address | Fresh acquisition interval, UTC | Mint/account response slot | Inspection / sample |
| --- | --- | --- | --- | --- |
| spacex | `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh` | 2026-09-19T10:44:02.022Z → 2026-09-19T10:44:06.611Z | 448384077 | Completed / Unavailable |
| openai | `PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF` | 2026-09-19T10:44:08.737Z → 2026-09-19T10:44:13.210Z | 448384103 | Completed / Unavailable |
| openai-refresh | `PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF` | 2026-09-19T10:44:15.514Z → 2026-09-19T10:44:20.217Z | 448384128 | Completed / Unavailable |
| custom | `So11111111111111111111111111111111111111112` | 2026-09-19T10:44:23.710Z → 2026-09-19T10:44:25.486Z | 448384159 | Completed / NotRequested |
| nonmint | `11111111111111111111111111111111` | 2026-09-19T10:44:30.683Z → 2026-09-19T10:44:32.799Z | 448384187 | Unsupported / NotRequested |

SPACEX and OpenAI are independently associated with their exact mints by the
retained first-party product entries. Current on-chain metadata says “SpaceX
PreStocks” and “OpenAI PreStocks”, while source names say “SpaceX” and “OpenAI”.
Both are retained and the name difference is disclosed. The source association,
independent mint decoding and unperformed execution remain distinct statements.

The custom mint is Wrapped SOL, outside the PreStocks catalogue; it required no
registration or code change. Its stock/issuer association is **unconfirmed** and
its actual legacy mint fields were decoded. The mint's recorded supply is zero;
this is not an inference about native SOL holdings or token-account balances.
The valid system-program address was clearly identified as an executable program,
with `mint: null` and no invented token properties. Malformed input produced a
clear address error and no fallback. Deterministic official-layout fixtures also
cover token-holding accounts, unsupported owners and uninitialized mint base fields.
Unsupported configuration retains raw observations and only independently decoded
base fields, with a separate decoder boundary and unknown extension findings.

Refresh created a new OpenAI acquisition and preserved the prior run. Mode switches
and reload made no extra submissions. Another browser session could not access the
run. The browser acceptance also opened the separately labeled saved report.

## Optional sample and operator diagnostic

**No live account sample succeeded. That acceptance sub-result remains unresolved.**
All three requested samples in this milestone ended in HTTP-level `429 Too Many
Requests` from `getTokenLargestAccounts`; valid mint results remained available.
The two milestone-1 captures show the same HTTP-level error. These retained captures
do not establish a JSON-RPC-level 429, permanent method incompatibility or zero holders.

`getGenesisHash` and `getAccountInfo` succeeded in every completed live run.
`getMultipleAccounts` was not reached in these live runs. Successful sample decoding
and cross-mint rejection are covered by deterministic fixtures, not claimed as live
success. The existing bounded client permits two attempts per method, with 250 ms
pacing, one-second backoff before its one retry for HTTP 429/5xx or supported retryable
JSON-RPC codes, and a 12-second request timeout. Captures retain the final method
outcome; they do not claim an individual-attempt transcript.

`SOLANA_RPC_URL` was not set in this validation environment, so the existing public
`https://api.mainnet-beta.solana.com` provider was used. The operator can supply a
compatible provider through that existing server environment variable. No provider
rotation, paid plan, public endpoint selector, full scan or historical account-list
substitution was introduced. Provider paths, query strings and credentials are
excluded from retained origins and public error summaries.

## Scope isolation and security

The request mint passes the Solana `Address` parser without a curve/signing test,
then remains exact through structured CLI arguments, source binding, RPC requests,
account decoding, capture, result, refresh and replay. The server rejects browser
paths, endpoints, commands, arbitrary source URLs and supplied proofs. Existing
same-origin checks, session access, job bounds, capture paths and artifact hashes remain.

Every current result has `execution_performed: false`, `local_execution_performed:
false`, `authorization: false`, `lifecycle_event: null`, `readiness: null`, and five
separate `NotTested` execution paths. OpenAI's result contains no SPACEX mint and no
saved policy/bundle. Cross-asset and source-version result mismatches, altered source
assertions and injected execution/lifecycle claims are rejected. Switching to a new
asset or between saved and current workflows hides incompatible previous results.
Original v1 current captures retain their schema and byte-exact replay behavior.

## Validation and preservation

The relevant baseline checks finished **before any executable source changes**:
20 focused Rust tests, format/lint, 13 service tests and frontend checks passed.
The full baseline browser run had 10 passed, one optional live test skipped, and
one failed: the existing saved required-sale evaluation exceeded its five-minute
service limit. The saved-workflow timeout is now bounded at ten minutes; current
acquisition retains its separate shorter budget. Saved policy/evidence semantics
are unchanged.

An initial final-validation attempt found that the browser sent a local draft field
which the strict API rejected before acquisition. That validation attempt was stopped
before the fix; its logs are explicitly retained as interrupted. The form now sends
only the selection and request key, with a browser regression assertion. Definitive
validation restarted after the correction with 120 executable/configuration files
hashed to detect later changes.

Final validation completed successfully:

- Service/catalogue tests: **22 passed**.
- Frontend syntax, published-report digests and production build: **passed**.
- Bounded real browser acceptance: **passed**, including five acquired/replayed
  account observations and the rejected malformed input.
- Both historical milestone-1 live captures: **byte-exact offline replay passed**.
- Remaining complete browser suite: **12 passed**; together with live acceptance, **13 browser tests passed**. All four saved engine outcomes completed correctly.
- Complete `make test`: **365 passed, 0 failed, 0 ignored**, including both SBF builds. [Full log](../reports/milestone2-validation/final-make-test.txt).
- `make fmt-check` and `make lint`: **passed**, including both fixture-program versions.
- Final binding check: **all 120 executable/configuration files unchanged** during definitive validation; **all 424 historical artifacts unchanged**. `git diff --check` also passed.
- Additional offline CLI check accepted a valid 32-byte address that is not a canonical Ed25519 point, confirming address validation does not require signer curve membership; no RPC was performed.

The machine-readable [final validation summary](../reports/milestone2-validation/final-validation-summary.json) records the checks and artifact counts. Final validation logs and screenshots are in `reports/milestone2-validation/`.
The final mobile form, custom-result layout, desktop catalogue form and desktop hero
screenshots were visually reviewed. The original hero and sole presentation toggle retain their placement.

## Start, import, refresh and replay

```sh
npm run build:engine
# Optional bounded refresh of the official source; saved catalogue works without it:
npm run import:catalogue
npm run dev
# Open http://127.0.0.1:4173/analysis#analysis
# Optional: set SOLANA_RPC_URL in the server environment before starting it.

npm run build
npm start
# Restart a running Node service after source changes, then reload the browser.

target/debug/eplyx-lifecycle replay-current \
  --input reports/milestone2-validation/openai.capture.json

# Normal regressions; live acquisition is opt-in:
npm run test:service
npm run test:frontend -- --workers=1
EPLYX_LIVE_TEST=1 npm run test:frontend -- --workers=1 --grep 'live generic'
```

The production build is running at `http://127.0.0.1:4173/analysis#analysis`; its
health endpoint confirms that the built engine is available.

Static hosting alone cannot perform acquisition. Import and analysis use the prebuilt
allowlisted executable; no browser request triggers compilation. Saved catalogue
versions are retained alongside their raw source. Each refresh creates a new run;
it does not update old captures or manufacture supply/balance differences.

## Files changed and remaining milestones

Changed implementation: `engine/src/lifecycle/current.rs`, the engine CLI,
`frontend/analysis-service.mjs`, `frontend/src/analysis.js`, its styles and the generic
hero scope caption. Added `frontend/catalogue.mjs` and `scripts/import-catalogue.mjs`.
Updated current/service/browser tests, added catalogue and multi-asset browser tests,
and added import/test scripts to `package.json`. Documentation changes are confined
to README, the repository scope note and this existing progress document. Source
captures are under `evidence/catalogue/`; validation records use the new milestone-2
directory. Existing uncommitted work was preserved; no reset, commit, push, branch
change or dependency upgrade was performed.

Milestone 3 (public wallet/account selection), full population acquisition, new
protocol integrations, lifecycle input and fresh execution assurance remain outside
this delivery. No wallet integration, signing, transaction submission or execution
campaign was added. This work stops after milestone 2.

---

## Milestones 0 and 1 — historical delivery record

Milestone 1 adds a real browser-triggered SPACEX mint inspection and a bounded,
optional token-account sample. `/analysis#analysis` is the separate analysis page.
The landing hero, sculpture, rolling headline and its original presentation toggle
remain. Select controls now use the site's blue surfaces, typography and focus
styles. No milestone 2 asset catalogue or custom-mint UI has been started.

## Audit and smallest change

The actual repository matched the Phase 15 service description: a same-origin
Node server, serialized job queue, prebuilt Rust executable, strict fixed choices,
and saved `compare-scenarios` / `evaluate-rollout` operations. The working tree
already contained substantial tracked and untracked work. No reset, commit, push,
dependency upgrade or branch change was made.

| Area | Existing reusable parts | Demonstration-specific wiring / disposition |
| --- | --- | --- |
| Asset selection | `assets/prestocks-spacex.json` separates issuer references from chain identity | Node validator, UI choices and saved presentation assume SPACEX; intentionally retained for milestone 1 only |
| Job orchestration | `frontend/analysis-service.mjs`: bounded queue, fixed executable, argument arrays, persisted envelopes, timeout | Saved entity IDs, policy IDs and Phase 14 plans stay on the historical branch; current requests have no policy/bundle/entity substitution |
| Mint and account discovery | `lifecycle/rpc.rs`, `decode.rs`, `mod.rs`: finalized RPC and official SPL layouts | Full population source requires enumeration and owner scans; too broad for a bounded initial interactive inspection |
| Exposure / positions | `lifecycle/exposure/meteora_dlmm.rs`, `expansion/discovery.rs`, `position/meteora_dlmm.rs` | Verified pool/vault and PositionV2 decoding is reusable; discovery, fixture and positive-exposure expectations need separate later work |
| Capture / VM | `probe/capture.rs`, `probe/token_transfer.rs`, `executor.rs`, `expansion/pipeline.rs` | Real account/code capture and LiteSVM remain; fresh UI execution is not connected in this milestone |
| Transfer / market exit / withdrawal | `probe/token_transfer.rs`, `probe/meteora_dlmm.rs`, `position/meteora_dlmm.rs` | Exact instruction builders, reconciliation and rollback are reusable; dependencies, signing, recipient and fixture scopes must be prepared per new run |
| Scenario parsing / consequence | `scenario.rs`, `lifecycle/policy.rs`, `consequence.rs` | Existing explicit semantic evaluation remains unchanged; no event or hypothetical policy is copied into current inspection |
| Resolution / readiness | `resolution/mod.rs`, `readiness/mod.rs` | `resolution/phase7.rs`, `readiness/evidence.rs`, `counterfactual/`, `rollout/` verify pinned production worlds and original measurements; these bindings remain intact |
| Historical verification | Hash checking in engine adapters and `frontend/evidence.mjs` | No bypasses, changed pinned paths or new hashes substituted for old proof |
| Presentation / tests | `analysis.js`, `presentation.js`, mode and hero modules; Rust integration, service and Playwright suites | Saved result adapters remain separate from the new typed current-inspection result |

The smallest usable change was a Rust `lifecycle/current.rs` observation adapter,
two CLI commands, one additional allowlisted service operation, and a dedicated
route using the existing frontend. There is no JavaScript financial evaluator or
new execution backend.

## What runs now

1. Choose **Current state · fresh data** and **Fetch latest data and analyze**.
2. The prebuilt engine checks mainnet genesis and fetches the configured mint with
   `getAccountInfo`, finalized and full base64 bytes.
3. It attempts `getTokenLargestAccounts` (at most 20); when available, it fetches
   those exact accounts together with `getMultipleAccounts`. This is a sample,
   never a complete holder population. `minContextSlot` is a lower bound, not an
   atomic-bank or historical-slot request.
4. Official existing SPL/Token-2022 decoders reconstruct mint facts and supported
   token-account state. Integer amounts stay strings. Original bytes remain in
   the capture, including data that cannot be decoded.
5. The new result reports retrieval intervals, separate context slots, decoder
   version, provider origin, explicit gaps and five **NotTested** paths. Readiness
   and lifecycle event are null; authorization and execution are false.
6. **Refresh analysis** creates another capture and run identity. Reload restores
   the saved run with its original acquisition time and a saved label.

The issuer association uses the existing saved asset reference dated September 17;
it is not claimed as newly retrieved issuer evidence. Current mint properties are
independently decoded. No deadline, successor, policy, wallet or historical success
is attached to the current result. Scaled display amounts are not computed; supply
is clearly shown in unscaled token units when appropriate.

## Evidence, bounds and failure behavior

Server-controlled `.analysis-runs/<uuid>.capture.json`, `.artifact`, `.manifest.json`
and `.json` retain original observations, deterministic findings, capture/result/
engine hashes and the request/job envelope. Capture files use create-new writes.
Offline replay re-derives findings from raw responses; no serialized `Proven`
claim is accepted. Downloaded artifacts are hash checked. Existing historical
artifact bytes are preserved.

Fresh requests use a server-configured `SOLANA_RPC_URL`, defaulting to the public
`https://api.mainnet-beta.solana.com`. Visitors cannot select URLs, programs,
commands, paths or proofs. The bounded client follows no redirects, allows at most
two attempts per method, limits each response to 2 MiB and each HTTP request to
12 seconds; the fresh child has a 90-second total timeout. RPC concurrency is one,
with at most four methods per capture. One job runs at a time with four queued jobs;
the local store accepts at most 200 jobs and then requires operator maintenance.
There is no automatic evidence deletion. Child environments omit unrelated secrets;
only fresh acquisition receives the server's RPC configuration.

HTTP-only SameSite session cookies scope jobs and idempotency keys. A run ID alone
does not grant another session access. Existing pre-session local jobs remain on
disk but are not exposed to a new session. The server stays loopback-only.

A sample acquisition failure preserves valid mint findings and records partial
coverage. Failure to acquire/validate the required mint produces an error, never a
saved-example substitute. Network/provider failures are not failed execution.
Saved example review remains available as a deliberately selected separate action.
Cancellation, arbitrary wallets, position discovery, current-run execution,
lifecycle forms and multi-run history browsing remain later milestones.

## Live demonstration

The actual Playwright browser, Node service and prebuilt Rust executable completed
two independent public-mainnet acquisitions and offline replays. No acquisition
mock was used. Both live calls to `getTokenLargestAccounts` returned provider code
429 after the bounded retry; the mint was successfully retrieved on both runs.
The exact method and failure are retained in the captures. A provider supporting
that method without this rate limit is needed for a live account sample; no paid
service was enabled. This is successful fresh **mint inspection with partial
discovery**, not a successful holdings scan or local execution.

Live run IDs, timestamps, slots, hashes and results are in
[`live-browser-runs.json`](../reports/on-demand-validation/live-browser-runs.json).
`live-1.capture.json` and `live-2.capture.json` reproduce their original findings
byte-for-byte with `replay-current`. Equal supplies across the two independent
retrievals do not imply cached acquisition. Screenshots in the same directory cover
the desktop/mobile form and results; their layouts were visually inspected.

## Start and replay

```sh
npm run build:engine
npm run dev
# Open http://127.0.0.1:4173/analysis#analysis
# Optional server-side provider: set SOLANA_RPC_URL before starting the service.

# Deterministic offline decoding of a retained capture:
target/debug/eplyx-lifecycle replay-current \
  --input reports/on-demand-validation/live-1.capture.json

# Full browser suite including bounded live integration:
EPLYX_LIVE_TEST=1 npm run test:frontend -- --workers=1
# EPLYX_TEST_PORT can select a separate local test port.
```

`npm run build` and `npm start` support the same `/analysis` route. A static host
alone cannot perform acquisition. The server never compiles on a request. Restart an already running Node service
to load the new API; a browser reload alone only reloads frontend files. During
this delivery a separate working service is available at
`http://127.0.0.1:4185/analysis#analysis`, leaving the pre-existing service untouched.

## Validation status

Baseline `make test`, format/lint, frontend syntax, service tests and browser tests
were started before implementation. Initial browser startup hit sandbox loopback
restrictions and was rerun outside that sandbox. Baseline format/lint, syntax and
nine service tests passed. The initial full Rust run completed with 354 passed,
zero failed and zero ignored. Long-running baseline checks overlapped implementation,
so the final frozen-source runs are the conclusive validation. The baseline browser suite overlapped
implementation: eight assertions passed, but two were invalidated by the rebuilt
executable hash and the deliberately relocated form. That run is **not** claimed
as a clean baseline or as evidence of pre-existing application regressions. Final
validation runs against the finished code are authoritative.

Final validation completed successfully:

- `make test`: **361 passed, 0 failed, 0 ignored**, including both SBF builds and
  all seven new current-observation tests. [Full log](../reports/on-demand-validation/final-make-test.txt).
- `make fmt-check` and `make lint`: passed for engine and both fixture versions.
- Frontend syntax/published-report verification and production build: passed.
- Service suite: **13 passed**. [Log](../reports/on-demand-validation/final-service.txt).
- Full browser suite with real engine and opt-in live mainnet capture: **12 passed**,
  including both fresh runs, refresh, offline replay and all retained historical
  browser cases. [Log](../reports/on-demand-validation/final-browser.txt).
- `git diff --check`: passed. All 83 recorded historical input bindings and all 92
  final source/config bindings remain unchanged during final validation. Published
  report digest verification and historical-artifact preservation tests also passed.
- An intermediate browser run was deliberately interrupted before the final suite
  after the last UI recovery fix; it is retained separately and is not counted as
  final validation. No code changed during the definitive 12-test browser run.

Focused checks cover fresh RPC
invocation, rate-limit partial results, exact u64 amounts, request/cluster/program/
context binding, no-event non-readiness, injected-proof rejection, offline replay,
refresh preservation, session isolation, artifact tampering and no demo fallback.
Deterministic mock-RPC tests are explicitly separate from the live browser record.

## Changed files and next milestone

Changes are limited to `engine/src/lifecycle/current.rs`, its module export,
`lifecycle/rpc.rs`, the engine CLI, `engine/tests/current_observations.rs`, the
existing frontend service/server, landing/app/analysis/shell/style files, relevant
service/browser tests, Playwright port configuration, README, AGENTS and this
single progress document. Validation outputs are under `reports/on-demand-validation/`.

Next: milestone 2, verified catalogue and custom-mint inspection through the same
pipeline. It has not begun; this delivery stops after milestone 1 as requested.

Official references reviewed for acquisition semantics:
[getAccountInfo](https://solana.com/docs/rpc/http/getaccountinfo),
[getProgramAccounts](https://solana.com/docs/rpc/http/getprogramaccounts),
[getTokenAccountsByOwner](https://solana.com/docs/rpc/http/gettokenaccountsbyowner),
[getMultipleAccounts](https://solana.com/docs/rpc/http/getmultipleaccounts),
[getTokenLargestAccounts](https://solana.com/docs/rpc/http/gettokenlargestaccounts),
[Token-2022 extensions](https://www.solana-program.com/docs/token-2022/extensions).

## Milestone 4 — current local execution (2026-09-20)

Implementation and definitive validation are complete. Milestone 5 is not
implemented.

### Consumer workflow and evidence model

Fetch a public wallet for a catalogue asset or custom mint, focus one account
returned by that acquisition, then select a local check. Availability is computed
by the engine from the immutable wallet capture. Transfer requires an explicitly
entered, existing compatible recipient **token account**. No recipient is selected
automatically and no initialized token account is fabricated. The amount can be
the full observed public balance or an exact custom **base token quantity**.
Mint decimal precision is applied with checked integer parsing; scaled display
and interest conversions are not requested or applied.

The wallet observation remains visible while a separate check prepares current
state, runs the local VM, reconciles the outcome and prepares evidence. Overview
and Technical render the same engine result. Reload retrieves the saved check;
refresh creates a new wallet run with no applicable execution checks. Failure of
a path does not erase the wallet observation or another completed check.

Each check has a server-generated ID, exact parent ID and wallet-capture digest,
a new immutable execution capture, an engine binary digest and a canonical result
digest. `GET /api/runs/:parent` exposes linked `execution_checks` outside the
unchanged canonical wallet observation; `/checks` supplies the check history.
The original wallet artifact still truthfully records that discovery itself did
not execute anything. Each execution artifact records its own actual execution
flags and exact path outcome. Neither artifact grants lifecycle readiness.

`probe::current::VerifiedExecution` cannot be deserialized. Only rebuilding the
fixture, running the existing executor and reconciling its outcome can construct
it. Request JSON accepts no claimed status, hashes, programs, instructions,
account metas, signatures, endpoint or executable path. The service captures in
one RPC-enabled child and replays in a separate child whose environment contains
only PATH. It checks the binary identity across both phases. Capture is read-only;
no network transaction submission or wallet connection exists in this flow.

### Reuse and isolation

| Existing component | Current-run integration |
| --- | --- |
| Version 3 wallet observation and focused account | Re-evaluated from original responses; source must occur in that exact owner/mint lookup. Browser focus remains a selection, not proof. |
| Historical Token-2022 Transfer builder | Historical wrappers adapt into generic `TransferContext`; current callers use the same builder/executor/reconciliation, also supporting deployed legacy SPL Token. |
| Historical DLMM builder | Historical exposure checks remain in their wrapper. Generic `SwapParameters`, route layout/PDA validation, message construction and reconciliation serve the independently captured current path. No fabricated historical snapshot is used. |
| Historical capture and expansion pipeline | Saved manifests, plans, policies and Phase 7/10 results are not current-proof inputs. Historical wrappers and published artifacts remain preserved. |
| Fresh acquisition | Four Transfer calls, or up to seven market discovery/preparation calls; finalized contexts increase monotonically, and executable dependencies share one final batch. Each response is bounded to 16 MiB; accepted offline execution containers are capped at 64 MiB. |
| Replay | Verifies parent/check/digests, reconstructs exact requests and deterministic route selection, validates deployed code/loader links and executes the instruction again in LiteSVM. No RPC occurs. |

Source data, runtime owner, executable state and lamports are compared with wallet
discovery before execution. Changed source state produces an explicit request to
refresh and reconfirm, never a silently clamped amount. Mint/owner differences are
reported separately; the final fixture is authoritative. Program code hashes,
ProgramData links/deployment slots, account hashes, exact message, Clock, runtime,
pre/post watched accounts, logs, compute units, fees and reconciliation are retained.
Discovery and execution are separate observations, not one atomic validator bank.
Execution-account `rpc_record` references index the reconstructed four-record
execution fixture: Transfer maps to outer observations 0–3; market execution maps
to outer observations 3–6, after its three discovery/paired-mint observations.
The final account batch is therefore outer observation 3 or 6 respectively.
Offline replay regenerates this fixture and verifies its reported digest.

The signer must still be an on-curve, captured system-owned, non-executable,
empty-data authority. Local owner signing is assumed only when executing;
`signer_possession_known` remains false. The synthetic fee payer pays transaction
fees and, for a missing paired-asset ATA, funds actual captured ATA-program
creation. No private key, issuer privilege, program authority or multisig is
inferred. Transfer proves movement only. Each amount, recipient, route, mint,
account, bank and signing assumption remains isolated.

### Current market boundary

The browser offers a USDC-paired Meteora test and requires an explicit positive
minimum output with no default. The engine supports the existing bounded
PermissionlessV2 version 1 adapter: selected Token-2022 mint as X, legacy token as Y,
and at most four relevant internal-bitmap bin arrays. It scans at most 64 returned
904-byte DLMM candidates, orders compatible layouts lexicographically, freezes one
selection and validates that route's current pool, vaults, mints, owner, bins,
optional bitmap, oracle, event authority and deployed programs. An unsuccessful
selected route is never replaced. This is neither best execution nor a global
market search. A scan without a compatible route says only that no supported route
was verified in that scan.

A successful VM transaction also needs exact source, public/withheld transfer fee,
vault, bin, protocol fee and paired-token credit reconciliation. A failed VM
transaction needs watched-state rollback to classify as Failed. Missing capture,
unverifiable prerequisites or failed reconciliation remain Indeterminate.
Unsupported configurations remain distinct from actual executed failures.

### Acceptance completed before definitive validation

Transfer was completed first, including live browser acquisition, VM execution,
reconciliation, offline replay and refresh isolation, before market work began.
The first current SPACEX fixture used finalized slot **448567075**, input **17621**,
recipient credit **17532** and withheld fee **89**, with a second independently
captured check after refresh. A later bounded current DLMM scan observed 48
candidates, five compatible layouts and selected pool
`22PthLk8TYnurtbWKRyECFd99cHHbfsbPNHfeMetzfZg`. Its local execution at slot
**448569388** returned **10400** raw USDC against an explicit **100** raw minimum;
input **17621**, transfer fee **89**, DLMM fee **351**, LP fee **316** and protocol
fee **35** reconciled. These were preliminary acceptance captures; the final
acceptance identities below supersede them for final validation.

For second-asset discovery, exactly three previously observed public owner
addresses were queried for OPENAI. Two returned compatible token accounts; one
returned none. The addresses were query candidates only. Fresh browser acquisition
then selected OPENAI from the dynamic catalogue and executed a new Transfer from
`F3L4drnkirxAnqeFVpJiRgTUeNMdBFZnV4bwZKuyrSpy` to the explicitly configured current
recipient `Ft3bKc8RLPz8NaXEwUdttnG1PgKiN8q3ULMX5pmQ1WeZ`. The preliminary execution
debited **149232852141**, credited **148486687880**, and withheld **746164261** raw
units. The bounded search responses and subsequent fresh checks are retained under
`reports/milestone4-validation/`.

Deterministic integration cases use reconstructed banks with genuine captured
program bytes. They are labeled separately from live-mainnet-state-derived local
simulations. They cover legacy SPL Token with another mint, conservative
confidential/frozen/paused/hook boundaries, unsupported authority, exact parsing,
wrong account/mint/run/fixture, source change, missing dependencies, successful VM
execution, actual overflow/minimum-output failures with rollback, fee conservation,
route isolation and repeated offline VM execution. No reconstructed failure is
presented as a production failure.

### Validation and reproducibility

All seven executable mutations were killed by named assertions, and the original
source bytes were restored. `scripts/test-current-execution-mutations.py` and
`reports/milestone4-validation/mutations/results.json` record the exact edits,
commands, assertions, logs and hashes. Compiler failure does not count.

The final validation includes `make test`, `make fmt-check`, `make lint`, service
tests, frontend build/check, all browser tests with bounded live acceptance enabled,
offline VM replay, source/binary binding checks and a before/after historical
artifact hash inventory. Every definitive command passed after mutation
restoration, with no later executable source changes.

A saved execution can be replayed without network or provider credentials:

```sh
target/debug/eplyx-lifecycle replay-current-check \
  --input CHECK.capture.json \
  --run-id PARENT_RUN_ID --check-id CHECK_ID \
  --wallet-sha256 PARENT_CAPTURE_SHA256 \
  --capture-sha256 CHECK_CAPTURE_SHA256
```

The exact IDs/digests are in each check's downloaded result and retained `.job.json`.
Compare stdout (including its trailing newline) to the canonical result digest.
The command reruns the VM, rather than accepting serialized success fields.

Changed implementation files: `engine/src/probe/current.rs`,
`engine/src/probe/current_market.rs`, `engine/src/probe/token_transfer.rs`,
`engine/src/probe/meteora_dlmm.rs`, `engine/src/probe/mod.rs`,
`engine/src/lifecycle/rpc.rs`, `engine/src/main.rs`,
`frontend/analysis-service.mjs`, `frontend/src/analysis.js`,
`frontend/src/styles.css`; focused tests in `engine/tests/current_execution.rs`,
`frontend/tests/execution.spec.js`, `frontend/tests/service.test.mjs`; the wallet
browser test now honors the report-directory override so final validation does not
overwrite Milestone 3 artifacts. The mutation script, this report and `AGENTS.md`
are also updated. No dependency upgrade, reset, commit, push or branch change was
performed.

Remaining limitations: existing recipient required for Transfer; unsupported
confidential accounts and active hooks; direct owner signing only; one bounded
DLMM direction/layout and selected route; no global liquidity, best execution,
future-state inclusion or ownership guarantee. Milestone 5 remains the future
combination of proposed lifecycle changes, fresh path resolution and readiness.
No lifecycle scenario form or new readiness evaluator was added.

Primary protocol references used while implementing the shared boundary:
[Solana Transfer Fees](https://solana.com/docs/tokens/extensions/transfer-fees) and
[getMultipleAccounts](https://solana.com/docs/rpc/http/getmultipleaccounts).
The implementation continues using the repository's pinned SDK/IDL and existing
executor, without a dependency upgrade.

### Final live acceptance identities

All four checks below were captured through the real browser/service/engine on
2026-09-20 and completed as **Proven / Passed in local simulation**. Each
canonical result was reproduced byte-for-byte by a separate offline VM execution
with no RPC environment. Complete IDs, program/fixture/account hashes, logs,
amounts, fees and source versions are retained in
[`final-live-acceptance.json`](../reports/milestone4-validation/final-live-acceptance.json)
and the linked check capture/result/job files in that directory.

| Check | Wallet fetched (UTC) | Execution captured (UTC) | Final slot | Raw debit → public credit | Withheld fee |
| --- | --- | --- | --- | --- | --- |
| live-transfer | 00:45:06.666Z – 00:45:07.767Z | 00:45:08.643488Z – 00:45:11.303826Z | 448573147 | 17621 → 17532 | 89 |
| live-refresh-transfer | 00:45:14.549Z – 00:45:15.794Z | 00:45:17.495081Z – 00:45:20.214625Z | 448573181 | 17621 → 17532 | 89 |
| live-market | 00:45:26.102Z – 00:45:27.201Z | 00:45:28.073245Z – 00:45:34.846829Z | 448573222 | 17621 → 10400 | 89 |
| live-second-transfer | 00:45:43.586Z – 00:45:44.896Z | 00:45:46.955280Z – 00:45:49.952598Z | 448573291 | 149232852141 → 148486687880 | 746164261 |

For the market row, the public credit is raw USDC; Transfer credits are in the
same mint as the input. The exact market minimum was **0.0001 USDC / 100 raw**.
The selected current pool remained `22PthLk8TYnurtbWKRyECFd99cHHbfsbPNHfeMetzfZg`;
its current scan observed 48 candidates, with five compatible layouts. Its
**351** raw DLMM fee splits into **316** LP and **35** protocol units, on the input
asset. Transfer fees are not counted twice as holder or pool capacity.

- **live-transfer**: parent `9e972d33-f054-474a-acd7-54d133e755c4`, check `a7f8adef-6ae9-4d64-bd0b-8f37f427fe2d`; fixture SHA-256 `182f50404c8e043bad181e76de80a101f4b01b931dd5d0aac0b30f0f120951f7`.
- **live-refresh-transfer**: parent `94bd94e1-6000-452f-95b7-70fce1c37411`, check `f113269d-0eee-4540-993c-71723f233542`; fixture SHA-256 `aed5f9be1b86a73fe1a6d14811ab23091faaec87b6c8959349ed3af376c4d2fe`.
- **live-market**: parent `2dbfaed8-50b3-4a69-b597-432c72ceb988`, check `15e7ec6a-eba2-499e-8d15-19822e765d11`; fixture SHA-256 `3823db856ab4fc0393142c43d9d33d031d572857ac3894fe43427d08253c6994`.
- **live-second-transfer**: parent `e1266c6c-83f3-4f6d-b636-67675cdf44f9`, check `bca6537f-f32a-415e-8e2b-d3b68868f566`; fixture SHA-256 `12846652b3d746de8e87f7170a65f7dcadc716e37a4e7fdcaafb9819cfe5d59e`.

The SPACEX source was `741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs`, owned by
`2wCvQzHiDHAHTvzwPeof9H3uEzq8Bzvg38DFbvZMGkuj`. Both Transfer tests used the
explicitly configured public recipient
`123aUGPWa93jiga876U3rLdBP86JNFSoz9tSQWCAskMc`, independently recaptured for each
check. This is not the old Phase 7 default recipient. The OPENAI source owner was
`6GJbPKBtovsrMEEMcic5KMi5tswh9qSyT5ZYLMqEwNgt`; its source and recipient are
listed above. All checks assumed owner signing locally and retained unknown key
possession. No mainnet transaction was submitted.

Final engine SHA-256: `9d8818bacd808f701f9546306a2c14ffb7d5646ac33b5be5746c28317c30953f`.

The first two preliminary Transfer captures are also preserved as
`preliminary-transfer-{1,2}.{capture.json,artifact,manifest.json,job.json}`. They
record the earlier development engine identity and establish that the complete
Transfer slice preceded market implementation. Final validation uses the four
checks above and the final engine identity.


### Final validation outcome

| Validation | Result |
| --- | --- |
| `make test` | **389 passed**, zero failures; includes both synthetic SBF program variants and all workspace tests. |
| `make fmt-check` / `make lint` | Passed, including both fixture-program feature variants. |
| `npm run test:service` | **26 passed**, zero failures. |
| `npm run build` / `npm run check:frontend` | Passed; published report digests verified. |
| Full Playwright suite, all live acceptance flags enabled | **19 passed**, **zero skipped**; 7.5 minutes. |
| Executable mutations | **7 injected / 7 killed by named assertions**; every source restored. |
| New live execution replay | Four canonical results reproduced exactly by actual offline VM execution. |
| Earlier observation compatibility | Schema 1 result matches its recorded original digest; schema 2/3 results match their original bytes. |
| Historical preservation | **746 / 746** pre-existing evidence/report/fixture files unchanged. |
| Final source/binary identity | **129 / 129** pinned source/build identities unchanged during validation; mutation-tested sources match final source. |
| Production preview | Restarted at `http://127.0.0.1:4173/analysis#analysis`; Chrome verified nine catalogue options, wallet input and the exact final served JavaScript bytes. |
| `git diff --check` | Passed; dirty worktree preserved, no commit or branch change. |

Machine-readable summaries and logs:
[`final-validation-summary.json`](../reports/milestone4-validation/final-validation-summary.json),
[`final-integrity.json`](../reports/milestone4-validation/final-integrity.json),
[`final-source-before.json`](../reports/milestone4-validation/final-source-before.json),
[`final-make-test.txt`](../reports/milestone4-validation/final-make-test.txt),
[`final-browser.txt`](../reports/milestone4-validation/final-browser.txt),
[`mutations/results.json`](../reports/milestone4-validation/mutations/results.json),
and [`production-smoke.json`](../reports/milestone4-validation/production-smoke.json).

The only changes after definitive executable validation were this delivery
report and validation evidence summaries. The production preview serves the
validated build. No Milestone 5 work was started.


## Milestone 5 — prospective lifecycle pre-flight

### Implementation and scope

After fetching a current public wallet and focusing a returned token account,
choose **Move to a replacement token**, enter a future effective time and optionally
a replacement mint and later deadline. Dates use the browser's local timezone and
are submitted as UTC. The only deadline rule is **Transition still required**;
no expiration, legal entitlement or value judgment is inferred. **No change**
retains inspection-only behavior. Both assurance presets use explicit demonstration
policies for this selected entity, never an issuer policy.

The normal form sends bounded structured fields, selected completed check IDs and
an idempotency key. The engine constructs a UserProposed lifecycle scenario bound
to the parent run, exact wallet digest, mint, owner, focused account, public balance,
account bytes and check captures/results. Independently inspected replacement mint
state remains a separate observation. Mint existence does not establish an issuer
relationship, ratio, official authorization or executable conversion.

Before the proposed event the shared consequence classifier returns Active /
Unaffected for a positive direct account, with PreEvent gate applicability and
no Ready finding. At the inclusive effective boundary it returns
TransitionRequired / RequiresTransition. An optional inclusive deadline returns
PostDeadlineTransitionRequired / RequiresTransition. Every view has the same raw
balance and account digest; no future balance or liquidity is predicted.

The existing path resolver consumes only current-run VerifiedExecution values
regenerated through actual offline VM execution. OfficialTransition stays NotTested;
Transfer/market exit retain exact selected check statuses; Redemption is Unsupported;
Withdrawal is NotApplicable for this direct account. Path relevance changes across
the proposed boundary; execution status and original artifacts do not.

The existing readiness evaluator evaluates two entity-only presets. Mobility
requires at least one selected exact Transfer/market-exit proof. This covers that
tested amount, recipient/route, minimum output, bank and locally assumed owner
signature; it does not establish whole-balance or future actionability. Full
transition additionally requires independently Proven OfficialTransition. Missing,
unsupported, indeterminate or unsuccessful alternatives remain Incomplete under
these presets, which define no blocking status. Population readiness is null.

A successful replacement observation is reused for re-evaluation within the same
wallet run/source/replacement mint. The new scenario and result retain its original
capture digest/date. A changed mint or refreshed wallet requires a separate bounded
inspection. No check is silently executed. Users may evaluate without checks,
explicitly run available checks, and re-evaluate on the unchanged wallet capture.
There is no age-based future-validity promise: exact capture times remain visible,
and refresh creates a new unverified run. Earlier results appear separately as
**Saved previous pre-flight**.

### Architecture and security

`preflight.rs` orchestrates existing `AssetLifecyclePolicy::status_at`, shared
`classify_public_exposure`, `LifecyclePathResolver` and the generic readiness gate.
Historical snapshot, Phase 7 resolver, Phase 13 ProductionWorld and frozen readiness
adapters are not called by this pipeline. No synthetic historical population is
created. The current resolver requires exact run/wallet/source/mint/owner bindings.
Selected check digests and original canonical outputs must match fresh replay.

The serialized bundle contains immutable wallet bytes, the engine-constructed
scenario and its digest, independent replacement observation, selected execution
captures/digests and engine identity. Replay reconstructs all evidence rather than
accepting serialized results. It checks external expected run/pre-flight/wallet/
scenario/bundle digests. The server's separate acquisition child alone receives RPC
configuration; preparation and replay children receive PATH only. Existing queue,
session/origin access, immutable artifact and executable-hash checks apply.
Browser fields cannot supply endpoints, paths, program IDs, policy JSON or status.
At most four selected checks enter a pre-flight. Successor inspection uses the
existing two-method mint acquisition without account sampling.

Operator-only `EPLYX_RUN_DIRECTORY` can isolate test/server workspaces without
deleting existing runs; browser input cannot set it. Final test runs use a separate
workspace to avoid exhausting the existing 200-job store or altering old artifacts.

### Replay

```sh
target/debug/eplyx-lifecycle replay-current-preflight \
  --input PREFLIGHT.capture.json \
  --run-id PARENT_RUN_ID --preflight-id PREFLIGHT_ID \
  --wallet-sha256 WALLET_CAPTURE_SHA256 \
  --scenario-sha256 SCENARIO_SHA256 \
  --capture-sha256 PREFLIGHT_BUNDLE_SHA256
```

Exact arguments are retained in the downloaded job/result. Stdout, including its
newline, must match the canonical result digest. This command performs no RPC;
selected checks rerun the existing VM because their verified type cannot be
constructed from serialized success. Original execution artifacts remain unchanged.

### Acceptance and validation

Definitive validation passed; the complete outcome is recorded at the end of this
section. Preliminary reconstructed and live SPACEX browser acceptance succeeded
before second-asset acceptance began. Logs are retained under
`reports/milestone5-validation/`; earlier failed development checks are not final
acceptance. One initial sandbox browser start could not listen on loopback and was
rerun outside that restriction. Browser development found and corrected evidence
checkbox initialization and a screenshot timing race.

### Files and remaining product boundaries

New implementation: `engine/src/preflight.rs`, `engine/src/resolution/current.rs`,
`engine/src/readiness/current.rs`, `frontend/preflight-service.mjs` and
`frontend/src/preflight.js`. Shared adapters are connected through `lib.rs`, the CLI,
`resolution/mod.rs`, `readiness/mod.rs`, `lifecycle/policy.rs` and the shared
consequence classifier. Service/UI integration changes `analysis-service.mjs`,
`serve.mjs`, `analysis.js` and `styles.css`. Tests are in `current_preflight.rs`,
`preflight.test.mjs`, `preflight.spec.js` and the targeted mutation script.
README and AGENTS describe the new scope. No dependencies were added/upgraded;
existing dirty work was preserved, with no reset, commit, push or branch change.

Remaining boundaries: only a proposed replacement-token transition, direct selected
accounts, bounded existing Transfer/DLMM adapters, explicit recipients/minimum output,
and public quantities are supported. No official conversion adapter, conversion
ratio, issuer relationship verification, automated issuer notice discovery, new
protocol integration, population capture/readiness, valuation, signing or transaction
submission is implemented. Local success is neither future liquidity nor inclusion
assurance. No later milestone is started.

### Final live prospective acceptance — 2026-09-20

The final **22-case browser suite passed with zero skips**, with every bounded live
flag enabled. Its four saved historical engine cases also passed, separately from
current pre-flights. The whole browser suite took 17.8 minutes; saved historical
cases accounted for 12.9 minutes. No executable source changed during this run.

| Case | Newly fetched wallet interval, UTC | Focused public amount | Selected current evidence | Proposed-active mobility / full transition |
| --- | --- | --- | --- | --- |
| SPACEX | 13:16:27.330 – 13:16:28.486 | `17621` raw / `0.000017621` base units | New Transfer + market exit | **Ready / Incomplete** |
| SPACEX refresh | 13:16:48.145 – 13:16:49.322 | `17621` raw / `0.000017621` base units | None | **Incomplete / Incomplete** |
| OPENAI | 13:16:58.487 – 13:16:59.606 | `149232852141` raw / `149.232852141` base units | New Transfer | **Ready / Incomplete** |

SPACEX focused account: `741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs`;
public owner query: `2wCvQzHiDHAHTvzwPeof9H3uEzq8Bzvg38DFbvZMGkuj`;
mint: `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh`.
OPENAI focused account: `F3L4drnkirxAnqeFVpJiRgTUeNMdBFZnV4bwZKuyrSpy`;
public owner query: `6GJbPKBtovsrMEEMcic5KMi5tswh9qSyT5ZYLMqEwNgt`;
mint: `PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF`.
These are public query inputs, with no human identity or key-possession claim.

The SPACEX proposal used hypothetical USDC mint
`EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`, effective
**2026-10-20T13:16:00Z**, with a **2026-11-19T13:16:00Z** neutral deadline.
Its independent mint inspection ran **13:16:38.784 – 13:16:39.356 UTC**, confirming
an initialized legacy SPL mint with six decimals. The OPENAI proposal used
hypothetical Wrapped SOL mint `So11111111111111111111111111111111111111112`,
effective **2026-11-04T13:17:00Z**, with no deadline. Independent inspection ran
**13:17:05.975 – 13:17:06.542 UTC**, confirming an initialized legacy SPL mint
with nine decimals. Both are explicitly **UserProposed** associations. Neither
is issuer-verified, and neither supplies a conversion ratio or mechanism.

For both positive accounts, before-event interpretation is **Active / Unaffected**;
proposed-active interpretation is **TransitionRequired / RequiresTransition**.
SPACEX's deadline view is **PostDeadlineTransitionRequired / RequiresTransition**.
The raw balance and account digest are identical in every view. Before-event
readiness is null with **PreEvent** applicability, not Ready. Checks retain their
original current-run statuses as their lifecycle relevance changes.

| Path | SPACEX | Refreshed SPACEX | OPENAI |
| --- | --- | --- | --- |
| OfficialTransition | NotTested | NotTested | NotTested |
| Transfer | Proven | NotTested | Proven |
| SecondaryMarketExit | Proven | NotTested | NotTested |
| Redemption | Unsupported | Unsupported | Unsupported |
| Withdrawal | NotApplicable | NotApplicable | NotApplicable |

Full transition remains **Incomplete** because the required replacement conversion
has not been independently executed. This is missing evidence, not a demonstrated
conversion failure or issuer blocker. Even though the hypothetical SPACEX replacement
and tested market output are both USDC, a market sale still cannot satisfy
OfficialTransition. No selected-entity finding grants population readiness.

Fresh selected checks:

| Check | ID | Capture interval, UTC | Final slot | Raw input → public credit |
| --- | --- | --- | --- | --- |
| SPACEX Transfer | `5f8789a5-dad0-46ed-910a-29d36695e631` | 13:16:29.322235 – 13:16:30.674657 | 448742472 | 17621 → 17532 |
| SPACEX market exit | `03b88b76-534b-4717-853b-bba5454ca661` | 13:16:32.255777 – 13:16:34.780564 | 448742486 | 17621 → 10400 USDC raw |
| OPENAI Transfer | `fae8519f-aa14-4d65-a66b-320a4aa738e9` | 13:17:00.660989 – 13:17:01.997345 | 448742587 | 149232852141 → 148486687880 |

Transfer recipients were explicitly entered:
SPACEX `123aUGPWa93jiga876U3rLdBP86JNFSoz9tSQWCAskMc`,
OPENAI `Ft3bKc8RLPz8NaXEwUdttnG1PgKiN8q3ULMX5pmQ1WeZ`.
The selected DLMM pool was `22PthLk8TYnurtbWKRyECFd99cHHbfsbPNHfeMetzfZg`;
minimum output was explicitly **100 raw / 0.0001 USDC**. Market reconciliation
recorded **89** raw transfer fee and **351** raw DLMM fee (**316** LP + **35**
protocol). All fees and account deltas remain in the original execution evidence.
No private keys, mainnet signing or transaction submission were used.

Exact parent / pre-flight identities:

- SPACEX: parent `77b916d4-4123-40c8-8d83-f64b4e69cdfc`, pre-flight `d9c2a6d4-e6de-40af-848d-9ba9b6bab297`, scenario digest `2f39f660c17687e2d955c8f0b3a5ad82d935d696814c87e99c8d0d5f47012954`.
- Refresh: parent `f4abc65a-87f6-4231-b64b-f6ea49467755`, pre-flight `7d924ed2-3676-4530-b90b-ad5b14e7c7c9`, scenario digest `f3b8f53affc679f346af771835a5532e7bdb31c9bd66b5cf494b0be4ab60719f`.
- OPENAI: parent `bd076870-fe5c-4fe8-add4-8e922ae8bc3e`, pre-flight `77c7c8d9-72c8-4148-9a90-d0d4b58c95f7`, scenario digest `d541a85190556819aeed2cb335ed577f82297f766c72f56e5e4fc8d511cd7ca2`.
- Invalid replacement: pre-flight `69afa6a3-cad7-4e7e-992c-143c5090b753` cleanly rejected the system-program address as **InvalidScenario / NotSupportedMint**, with no readiness views. The job itself completed its inspection successfully.

All four final live pre-flight artifacts reproduced their canonical output exactly
in a separate PATH-only offline replay. The SPACEX result contains only the two
new check IDs above; the refresh contains none. OPENAI contains only its own fresh
Transfer, with no SPACEX mint, successor, deadline, policy or execution evidence.
No Phase 7 path row supplied current assurance. The reconstructed browser case also
verified a missing-evidence pre-flight, an explicitly selected later Transfer,
and offline re-evaluation using the same wallet and successor-capture digests.

Machine-readable identities, acquisition records, program/fixture/account hashes,
path matrix, policy findings, fees and replay bindings are retained in
[`final-live-acceptance.json`](../reports/milestone5-validation/final-live-acceptance.json),
with canonical bundles/results/jobs under
[`final-browser/`](../reports/milestone5-validation/final-browser/).
The seven executable mutation results are in
[`mutations/results.json`](../reports/milestone5-validation/mutations/results.json).

### Production preview verification

The repository's previous production preview (PID 88900, verified repository cwd
and `node frontend/serve.mjs --production`) was replaced by the built Milestone 5
preview on **http://127.0.0.1:4173/analysis#analysis**. Existing saved runs were
preserved. Chrome confirmed nine catalogue options and exact equality between the
served pre-flight JavaScript and the validated source.

A separate production smoke check fetched one new SPACEX wallet observation and
created pre-flight `cbd5add1-0d05-4bb5-ac3c-5a00c4768f37` with no successor or
execution checks selected. It correctly returned Unaffected before the proposed
boundary, RequiresTransition after it, and Incomplete assurance. Both presentation
modes used the same result. Desktop and mobile screenshots were visually inspected;
no horizontal overflow was observed. This smoke case supplies no new execution
proof and is separate from the three fresh checks in final live acceptance.
See [`production-smoke.json`](../reports/milestone5-validation/production-smoke.json),
[`production-preflight-desktop.png`](../reports/milestone5-validation/production-preflight-desktop.png)
and [`production-preflight-mobile.png`](../reports/milestone5-validation/production-preflight-mobile.png).


### Final validation outcome

| Validation | Result |
| --- | --- |
| `make test` | **401 passed**, zero failed/ignored; both SBF variants built and all workspace tests completed. |
| `make fmt-check` / `make lint` | Passed, including both fixture-program variants. |
| `npm run test:service` | **29 passed**, zero failures. |
| `npm run build` / `npm run check:frontend` | Passed; published report digests verified. |
| Full browser suite with every live flag enabled | **22 passed**, **zero skipped**. |
| Executable mutation campaign | **7 injected / 7 killed by named assertions**; no compiler-error kills; every source restored. |
| Canonical live pre-flight replay | Four results reproduced byte-for-byte offline; the three selected fresh checks were reconstructed by actual VM execution. |
| Historical preservation | **893 / 893** recorded evidence/report/fixture files unchanged. |
| Source/binary preservation | **138 / 138** pinned identities unchanged throughout final validation; mutation-tested sources match final source. |
| Production preview | Current build running at port 4173; real Chrome smoke, served-source equality, both modes and desktop/mobile layout passed. |
| Whitespace | Tracked `git diff --check` and explicit new/changed-file checks passed. |

Final engine SHA-256:
`fea3f0c1ac3be12a783c9000107fd110aa63dc90022c879b68e54d956f6396ce`.

The final suite ran only after mutation restoration. An earlier browser start with
an incorrect test environment value was interrupted and restarted with the exact
public owner address; that interrupted attempt is not counted. A development
browser run was also stopped before qualification to keep it separate from mutation
builds. All definitive commands passed, and no executable code changed afterwards.
Only delivery documentation and evidence summaries were completed after validation.
The sole branch remains `main`; existing uncommitted work is preserved.

Evidence indexes:
[`final-validation-summary.json`](../reports/milestone5-validation/final-validation-summary.json),
[`final-integrity.json`](../reports/milestone5-validation/final-integrity.json),
[`final-source-before.json`](../reports/milestone5-validation/final-source-before.json),
[`final-commands.json`](../reports/milestone5-validation/final-commands.json),
[`final-make-test.txt`](../reports/milestone5-validation/final-make-test.txt),
[`final-browser.txt`](../reports/milestone5-validation/final-browser.txt),
[`final-live-acceptance.json`](../reports/milestone5-validation/final-live-acceptance.json),
[`mutations/results.json`](../reports/milestone5-validation/mutations/results.json).

Start with `npm run build:engine`, `npm run build`, `npm start`; open
**http://127.0.0.1:4173/analysis#analysis**. Restart the service after future source
changes. The running production preview already serves this validated build.

## Milestone 8 — transition packages

The operator CLI now accepts a strict versioned transition package with a
registered fixed-ratio adapter. The manifest and config carry exact terms,
candidate SBF and hash bindings; no package field can supply an RPC endpoint,
instruction, account meta, result or issuer status. The CLI validates before
read-only acquisition, executes the candidate in an isolated bounded offline VM
worker, reuses the Milestone 7 frozen selection/stress pipeline and writes
`report.json` plus `report.md`. Offline replay rechecks the package and every
capture/plan/case digest and reruns the VM. See
[Milestone 8 package contract](on-demand-milestone-8-package.md).

Three fresh production runs are retained: the standard demo package has Proven
selected-account conversion and ten Proven stress cases, while the explicit
stress policy remains Incomplete because the selected sample cannot resolve
all population conditions; the underfunded reserve produces actual local
failures and Blocked candidate/stress readiness; a second source asset uses the
same package pipeline and retains its own exact identity. OfficialTransition
remains NotTested, population rollout readiness remains Incomplete, and no
funds moved. The final acceptance artifacts live under
`reports/milestone8-healthy-worker/`, `reports/milestone8-underfunded/` and
`reports/milestone8-second-asset/`.
