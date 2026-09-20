# Milestone 6 research checkpoint: replacement conversion

## Proof contract, frozen before the new search

`ReplacementConversion = Proven` requires one replayed local execution of the
actual identified mechanism for the exact current wallet run, selected source
account, amount, source mint, replacement mint, destination account, captured
bank, program code and mechanism ID. The mechanism must be independently tied
to this source/replacement pair, with its instruction, account metas, authorities,
eligibility and terms established. Terms include the input/output relationship,
checked integer arithmetic, rounding, fees, bounds and time conditions where
applicable. Required current accounts and deployed code must be captured;
locally assumed holder signing must be disclosed, and no issuer/private signer or
credential may be fabricated. The actual instruction must run in the offline VM.
Its source debit or burn, replacement credit, fees, rounding and other required
state changes must reconcile, and offline replay must reproduce the outcome.

`OfficialTransition = Proven` requires all of the above **and** independent
evidence binding that same executed mechanism to the issuer-defined transition.
A UserProposed scenario, successor mint, name match, DEX acquisition, source burn,
administrative MintTo or shared account appearance cannot supply that binding.
Transfer and SecondaryMarketExit remain distinct paths. Missing material facts
leave the path NotTested or Indeterminate; an established private dependency is
Unsupported. Failed requires a real failed instruction and watched-state rollback.
No proof transfers across runs, accounts, amounts, destinations, mechanisms or
banks. Entity readiness cannot imply population readiness.

The existing `transition::OfficialTransitionMechanism` and opaque
`VerifiedTransitionExecution` supply a generic discovery/assessment starting
point. They are not a current conversion adapter: no production constructor
exists, and the Milestone 5 resolver only accepts current Transfer or
SecondaryMarketExit replay. Full transition readiness explicitly requires a
Proven OfficialTransition path. The current UserProposed scenario does not
declare a verified issuer relationship, ratio or mechanism.
The generic resolver/readiness/transition types contain no SPACEX-specific
branch. SPACEX and SPCXx identities, issuer wording and sampled transactions
live in captured evidence and research manifests; the old Phase 9 adapter
analyzes those frozen artifacts. Its sampled burns, market routes, successor
minting and authority actions are useful candidate context only and cannot be
imported as fresh execution proof.

## Bounded public search plan

Search date: 2026-09-20 UTC. Read-only; no authenticated requests, signing,
broadcast or issuer operation. Preserve page and RPC response bytes, response
status, retrieval time and SHA-256 under `evidence/transition/milestone6/`.

1. Fetch the public PreStocks `/spacex` and `/faq` pages and inspect their
   embedded public data. Fetch at most 12 same-origin JavaScript chunks directly
   linked by those pages, prioritizing chunks that mention swap, conversion,
   SPACEX, SPCXx, API or transaction construction. Do not infer a protected
   backend dependency from an absent public route.
2. Query finalized `getSignaturesForAddress` with limit 20 for each of four
   pinned addresses: source mint, successor mint, source mint authority and
   successor mint authority. Record returned slots and errors. Select at most
   16 unique signatures for `getTransaction` (parsed JSON, max supported
   transaction version 0): newest source and successor entries first, then
   distinctive authority entries. No replacement for failed selected probes.
3. Inspect signers, outer/inner instructions, pre/post token balances, mint and
   owner fields, logs and errors. Separate market routing, administrative
   mint/burn, and a genuine coupled source debit/replacement credit. Cross-check
   any apparent conversion against first-party semantics and explicit terms.

Supplemental linked-source boundary, fixed after the SpaceX page exposed its
"Learn more" link: request that **one** public PreStocks X post URL once with
ordinary read-only HTTP, without login or alternate mirrors. Preserve the
status, body and digest. An inaccessible post stays inaccessible; do not infer
its content.

Stop at (A) an independently established executable public mechanism, (B) an
identified mechanism with an evidenced private dependency, or (C) exhaustion
of the above boundary without exact mechanism identity. The sample is not a
complete transaction history and cannot establish non-existence.

## Findings

**Stop condition C.** No exact independently executable SPACEX → SPCXx
conversion mechanism was established within the frozen public search. This is
not evidence that no mechanism exists. There is no new conversion fixture, VM
execution, replay receipt or current-run OfficialTransition proof.

### Public issuer evidence

The [current PreStocks SpaceX page](https://prestocks.com/spacex) was fetched
successfully at `2026-09-20T14:15:31.821028Z` (HTTP 200, SHA-256
`3a1db0f647944fb7f763e8bb67ec07d11844695d4abd8efdbd99847c4d613829`).
The exact pointer is `attempt-2/issuer-spacex.body`, the SpaceX warning: holders
are directed to swap SPACEX into the linked SPCXx mint **or any other token**
before `2027-03-12 23:59 UTC`. That is an issuer instruction to dispose of the
expiring source exposure; it does not identify a pair-specific program,
instruction, ratio, account plan or completion receipt.

The [current FAQ](https://prestocks.com/faq) was fetched at
`2026-09-20T14:15:32.069906Z` (HTTP 200, SHA-256
`3634091626b766527b9b01b2bee92af7dfc6f713ea27e1b4c686cad37a9a94c9`).
Its current public page chunk, `attempt-2/issuer-script-01.body`, under
`Mechanics / What happens if the company goes public?`, calls the post-IPO
conversion fully on-chain and without KYC. Other FAQ entries identify Jupiter
as a trading venue and describe variable trading/conversion fees. These broad
claims do not supply exact conversion terms or establish an issuer-controlled
backend requirement. We therefore leave eligibility and any private dependency
**unknown**, not Unsupported. The page and FAQ assertions remain distinct from
on-chain observations.

The two initial pages and 12 directly referenced same-origin JavaScript chunks
all returned HTTP 200. `attempt-2/manifest.json` records their URLs, retrieval
times, body hashes and HTTP statuses; matching `.headers` files preserve the
headers. In the inspected page-specific chunks, no SPACEX → SPCXx conversion
transaction builder, instruction discriminator, account metas, exact ratio,
backend authorization request or registered conversion program was identified.
The 12-chunk bound does not cover every possible lazy-loaded route or service.
The SpaceX page links [one PreStocks X announcement](https://x.com/PreStocks/status/2063623768535363940).
Its one public read-only retrieval returned HTTP 200 at `2026-09-20T14:39:37Z`
(SHA-256 `d1bb272189916e9f602140e3bfce7182ead8c8e1bf9039e0283a504debcd6519`).
The HTML title explicitly says conversion happens through normal trading and
is fully on-chain, without KYC. This establishes issuer-defined **general
trading semantics**. It still does not identify an exact SPACEX → SPCXx route,
program, instruction, account plan or conversion terms. The public metadata
truncates the post after a split reference; no ratio was extracted or inferred.
`attempt-4-linked-x/manifest.json` records the retrieval and exact pointers.

### Current public Solana transaction sample

The public finalized RPC endpoint was queried with `getSignaturesForAddress`
for exactly four addresses, 20 entries each. The fixed selection took the four
newest unique signatures per address, 16 total, then requested parsed
`getTransaction` results. No sample was replaced. The first ten returned HTTP
200; the remaining six returned RPC HTTP 429. One spaced retry of those same
six returned HTTP 200. All request payloads, response bodies, statuses,
retrieval times and hashes are in `attempt-2/manifest.json` and
`attempt-3-retry/manifest.json`. The failed Python TLS attempt is preserved in
`attempt-1/` and did not yield issuer or RPC data.

| Queried address role | Address | Returned signatures | Observed slot range |
| --- | --- | ---: | --- |
| SPACEX mint | `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh` | 20 | 448755727–448755134 |
| SPCXx mint | `Xs3oZwbHvqis4NYcf4YKWmEia2eC84wSiVrcYcTqpH8` | 20 | 448755782–448755465 |
| SPACEX mint authority | `WV9PJN7XTmTLVwbutCLFxp8TyePee6Xq5mRq6Fti5Wc` | 20 | 448343233–447186682 |
| SPCXx mint authority | `7pt9tkctJPK7PPNQJ77GKg8ZffSF6QxoMiCFYHxrtaCj` | 20 | 448527130–448164907 |

The four sampled SPACEX-mint transactions (`transaction-00` through `03`)
all failed at an outer instruction with custom error 6004; their finalized
pre/post token balances show no source debit or replacement credit. The four
SPCXx-mint transactions (`transaction-04` through `07`) show DEX activity,
including Jupiter and pool programs, and SPCXx movement but **no SPACEX
debit**. For example, `transaction-04.body` at slot 448755782 shows a
2,301,148-raw SPCXx movement between sampled market accounts, with no SPACEX
balance entry. The four source-authority samples (`transaction-08` through
`retry-transaction-11`) are Squads `VaultTransactionExecute` calls that set
Token-2022 transfer fees for *other* PreStocks mints; they have no source or
replacement token balance changes. The four successor-authority samples
(`retry-transaction-12` through `15`) are `MintTo` batches for *other* xStock
mints, with no SPACEX debit or SPCXx credit. A shared authority address is
not a linked holder conversion.

The sampled transactions' full account lists, signer flags, outer and inner
instructions, logs, errors and pre/post token balances remain in the RPC
response bodies. This sample does not enumerate the full history or prove that
no coupled transaction has ever occurred. The [typed candidate records](../reports/milestone6-mechanism-candidates.json)
keep issuer-directed ordinary trading, failed source routes, SPCXx market
trades, source fee administration and successor mint administration separate.

### Mechanism and assurance boundary

No pair-specific conversion program, instruction, complete account plan,
conversion ratio, rounding, fee rule or exact destination semantics is known.
The source and replacement mint identities are known from the pinned historical
verification and current issuer link, but existence and naming do not establish
the mechanism. Holder possession remains unknown; the current evidence does not
establish a required issuer signer, backend authorization, KYC state or
entitlement. Independent local conversion execution is therefore **not
available**. Classifying a private dependency as Unsupported would overstate
this evidence.

The Milestone 6 research assessment is `ReplacementConversion = NotTested`;
the current product's `OfficialTransition` path remains `NotTested` for a
fresh wallet run. No exact conversion proof exists. The saved Milestone 5
SPACEX preflight remains Mobility **Ready** for its exact tested transfer/market
scope and FullTransition **Incomplete** at its active views. It is historical
context only and was not refreshed, executed or promoted by this research.
The existing consumer preflight already says the replacement conversion has
not been independently executed and shows full transition as needing more
evidence. No "Test conversion" action or adapter is added for an unestablished
mechanism. Offline conversion replay and current conversion reconciliation are
not applicable because no conversion execution was performed.

The remaining gap is independent evidence of an exact issuer-bound executable
source → replacement mechanism and its terms. Any later investigation must
start a separately pinned search/capture plan; this checkpoint stops here.

## Validation and files

The two capture scripts parse as Python; every captured body matches its
recorded SHA-256. The retry manifest binds the first successful capture
manifest, all 16 selected signatures are distinct and match their returned
transaction bytes, all five structured candidates have the required fields
and live evidence references, and the linked issuer post body matches its
manifest hash. `make test`, `make fmt-check`, `make lint`, the 29 Node service
tests, frontend build/check and the six isolated static browser tests passed.
The Milestone 5 historical hash lists still verify **893/893** data artifacts
and **138/138** source/build identities unchanged. The first concurrent full
browser attempt had fixture failures and was stopped. An isolated six-test
static browser rerun passed. A serial full run was interrupted after 5.9
minutes in an unrelated saved-demo engine flow while waiting for its output;
the full browser suite is **not validated** at this checkpoint. No conversion
fixture, live wallet recapture, VM execution, replay, mutation run or conversion
browser action is applicable at stop condition C.

This checkpoint added `scripts/capture-milestone6-research.py`,
`scripts/retry-milestone6-transactions.py`,
`evidence/transition/milestone6/`,
`reports/milestone6-mechanism-candidates.json`, this document, and a progress
entry in `docs/on-demand-progress.md`. Existing engine, frontend, policy,
fixture and historical artifact bytes were not edited for this milestone.
