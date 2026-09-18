# Phase 9 official transition investigation result

**Outcome B, NotTested:** no independently verifiable official SPACEX-to-successor
mechanism was established within the bounded investigation. The published
successor is a real initialized Token-2022 mint. Actual transactions reveal
market routing through both assets, source burn/closure, successor administrative
minting and source-authority fee administration. None establishes an issuer-bound official
execution plan for the selected holder. No official VM execution was attempted,
and no issuer signature, eligibility state, conversion ratio or credit was
invented. This is not a claim that no transition exists.

## Canonical entity and updated path matrix

| Field | Exact value |
| --- | --- |
| Entity | `solana-token-account:741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs` |
| Source account | `741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs` |
| Retained owner | `2wCvQzHiDHAHTvzwPeof9H3uEzq8Bzvg38DFbvZMGkuj` |
| Original observed public amount | `17621` raw SPACEX |
| Source mint | `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh` |
| Lifecycle | `TransitionRequired / RequiresTransition` |
| Policy view | Original hypothetical `2026-09-17T20:00:00Z` |
| Holder signing | Possession unknown; original owner locally assumed to sign only in retained Phase 7 evidence |
| Official execution tested amount | None |

The holder amount is retained frozen evidence, not a newly captured current
balance. Phase 9 captures mint, administrative, market and program observations;
it does not recapture or execute another amount from this holder.

| Path | Status |
| --- | --- |
| OfficialTransition | **NotTested** |
| Redemption | **Unsupported** |
| SecondaryMarketExit | **Proven**, exact Phase 7 contexts only |
| Transfer | **Proven**, exact Phase 7 contexts only |
| Withdrawal | **NotApplicable**, direct holder context |

The four non-official rows are exactly equal to the Phase 8 rows, including
their attempts, contexts, evidence, reasons and limitations. Their proofs do not
inherit to OfficialTransition. The original population/policy and Phase 4
baseline execution field remain unchanged. Historical Phase 1–8 artifacts are
preserved; Phase 9 writes separate investigation, discovery and matrix files.

## Official issuer semantics

The captured [PreStocks SpaceX page](https://prestocks.com/spacex) directs holders
to swap source tokens into SPCXx or another token by its stated deadline,
`11:59pm UTC on 12 March 2027`. It links successor mint
`Xs3oZwbHvqis4NYcf4YKWmEia2eC84wSiVrcYcTqpH8`. The captured
[public FAQ](https://prestocks.com/faq), including its first-party application
data, describes post-IPO conversion into equivalent tokenized public stock as
on-chain without KYC. These are external issuer assertions, bound separately
from observed asset bytes.

Within these sources, no exact official program, instruction, account plan,
claim payload, fixed conversion ratio or completion receipt is specified.
The notice describes swapping; this investigation does not reinterpret it as
automatic conversion, an identified burn/mint contract or a holder redemption
flow. Whether PreStocks infrastructure is required remains unknown. A fixed
ratio cannot be inferred from mint decimals, UI scaling, market reserves or a
DEX price.

The original Phase 8 captures are reused because they already contain the
relevant public mechanism statements. Browsing reconfirmed the notice; no issuer
authentication or private API was attempted. The previously inaccessible X post
and empty syndication response remain limitations, not presumed post content.
Historical mint/redemption KYC language is separate from conversion-specific
FAQ language and does not establish a conversion KYC dependency.

## Current on-chain successor verification

Latest captured successor bytes are from finalized slot **448067723**,
`rpc-account-reconstruction.json`, record 4,
`/response/result/value/47`. The complete report reference is
`/records/4/response/result/value/47`. The existing official SPL/Token-2022
decoder verifies all base and extension state.

| Property | Exact observed value |
| --- | --- |
| Mint | `Xs3oZwbHvqis4NYcf4YKWmEia2eC84wSiVrcYcTqpH8` |
| Runtime owner / token program | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` |
| Program family | Token-2022 |
| Executable | false |
| Initialized | true |
| Decimals | 8 |
| Supply | `55999222847874` raw / `559992.22847874` decimal base units |
| Mint authority | `7pt9tkctJPK7PPNQJ77GKg8ZffSF6QxoMiCFYHxrtaCj` |
| Freeze authority | `JDq14BWvqCRFNu1krb12bcRpbGtJZ1FLEakMw6FdxJNs` |
| Permanent delegate | `5aMNNLQJwAEeoemTEMkv5NVjqKwvvefRYCQ5Z67HFvEq` |
| Metadata update authority | `5aMNNLQJwAEeoemTEMkv5NVjqKwvvefRYCQ5Z67HFvEq` |
| Metadata | `SpaceX xStock`, `SPCXx`; metadata pointer points to the mint itself |
| Metadata URI | `https://xstocks-metadata.backed.fi/tokens/Solana/SPCXx/metadata.json` |
| Pause | false; authority is the freeze authority above |
| Transfer hook | null/inactive; authority is the permanent delegate above |
| Default account state | Initialized |
| Scaled UI | Multipliers 1; authority `S7vYFFWH6BjJyEsdrPQpqpYTqLTrPRK6KW3VwsJuRaS` |
| Confidential transfer | Authority is the permanent delegate; auto-approve false; auditor null |
| Creation slot | Not established |
| Raw account-data SHA-256 | `08ecb23962c7e75f2c2152d8cea8caa4aad182a7c9035e1441c7df46de681e08` |

All eight extensions are retained: ConfidentialTransferMint, DefaultAccountState,
PermanentDelegate, TransferHook, MetadataPointer, TokenMetadata, ScaledUiAmount
and Pausable. The linked external metadata JSON is captured separately with
HTTP status, retrieval time, URL and hash. Its branding/terms link is an external
document, not an on-chain proof of issuer identity or entitlement.

The source mint is independently decoded at the same latest mint-capture slot,
with 9 decimals and supply `8742515288354` raw. Source and successor have no
shared base mint/freeze authority in captured state. Source mint/freeze/permanent
delegate authority `WV9PJN7XTmTLVwbutCLFxp8TyePee6Xq5mRq6Fti5Wc` differs
from the successor authorities. Metadata, delegate and UI authority details
remain explicit rather than being collapsed into an assumed common issuer.
The issuer webpage reference and current mint observation are separately bound.
No creation slot is inferred from the oldest returned signature.

## Bounded transaction-pattern investigation

The public finalized RPC search queried exactly six addresses, returning 90
signature entries. Duplicate entries across addresses are not unique
transactions. It selected **31 unique transaction samples** and recovered all
31. There were 49 transaction requests: the 31 selections plus 18 paced retries
of first requests that received RPC error 429. All 18 original errors remain
preserved. No additional signatures were selected by retrying.

| Queried address | Returned / limit | Returned slot window |
| --- | ---: | --- |
| `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh` | 30 / 30 | `447893357..448065130` |
| `Xs3oZwbHvqis4NYcf4YKWmEia2eC84wSiVrcYcTqpH8` | 30 / 30 | `448065594..448066550` |
| `WV9PJN7XTmTLVwbutCLFxp8TyePee6Xq5mRq6Fti5Wc` | 10 / 10 | `446570288..447186714` |
| `7pt9tkctJPK7PPNQJ77GKg8ZffSF6QxoMiCFYHxrtaCj` | 10 / 10 | `447213465..448040244` |
| `53Ab3Rqx1a5uiV7qmsX4qbdbrqstVDpnH4LoJGfsZsU8` | 5 / 5 | `447186575..447186714` |
| `JDq14BWvqCRFNu1krb12bcRpbGtJZ1FLEakMw6FdxJNs` | 5 / 5 | `416391557..416391781` |

Selection used six newest and four oldest returned signatures per mint, four
per mint authority, three from the observed Squads account and three from the
successor freeze authority, deduplicated. The returned slot windows above are
address-local windows, not a full history or a population sampling campaign.
All signature arrays, query limits, retrieval times and raw RPC bodies are
retained. No archive service, explorer label, private issuer state or legal
eligibility was used. Permanent-delegate/metadata authority transaction history
and older mint histories are not exhaustively searched.

### Actual patterns found

Five samples contain both source and successor in token balances. Their raw
account lists, outer and inner instructions, signer flags, CPI program IDs,
logs, errors and exact pre/post token deltas are all retained. **None has a net
source debit and positive successor credit under the same observed token owner**
in the sampled transaction's balance list. This narrow observation is not proof
that other source-to-successor transactions do not exist.

An example at slot `448065130`, signature
`3Wwh2tVTyHvQmNHbtsdwsF9mmNpRnQ436txAteQBsL29v5jdEjWcwnPPyUVJRZ5tKenRrmuwYBWrWNUtmqmeBMDW`,
has a `JUP6…` outer route, deployed DLMM Swap2 and subsequent market CPIs.
The source owner debits `1606968` raw source; its observed successor account has
zero net change. Successor reserves and other token owners change. This is a
real routed market transaction, not evidence of this holder retaining a
successor conversion credit or receiving an official completion receipt.

A source-only sample at slot `448062178`, signature
`2ZHotoRrypaPJsgpkp3wNt967ghTXDygUbTDb2PTLL3q41n4W9FCiHx1B7Yec7AkDJ7GyGGVWpsMR3j8q3gu3vr5`,
burns `19474` raw source and closes that token account. It contains no established
successor exchange. Separate successor-authority samples at slots `448040244`,
`447793019`, `447758880` and `447622374` show administrative MintTo under the
successor mint authority. Combining unrelated burn and mint transactions cannot
establish a holder conversion or a ratio.

Source-authority/Squads samples at slots `447186714`, `447186682`, `447186645`
and `447186610` execute withheld-fee administration. Their Squads execution,
ATA creation and Token-2022 fee withdrawal are administrative observations,
not migration/claim operations for the canonical holder. Other successor
samples are market routes or successor-only transfers; neither type establishes
source-to-successor official identity by itself.

### Programs, PDAs and candidate account plans

The investigation records **24 observed program IDs**, executable-account
evidence where available, and **20 complete upgradeable ProgramData/ELF records**.
All captured upgradeable loader links validate, with deployment slot, upgrade
authority and ELF input hash. Native/legacy-loader observations do not receive
fabricated ProgramData links. No observed program is labeled a migration program.

Two protocol relationships are reconstructed from raw state:

- The observed Squads multisig `53Ab3Rqx1a5uiV7qmsX4qbdbrqstVDpnH4LoJGfsZsU8` is a canonical PDA under `SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf`, using create key `DhfQ2h1m4KYMH8jcFMj4ASSsqtBiJ26Vn1dfQU3SKV12`. Threshold is 2, time lock 0 and bump 255. Vault index 0 derives the source administrative authority `WV9…`, bump 255. Eplyx does not possess multisig member authorization.
- Observed market pool `9RExWJFGfAviKHCSCQ59CqA6e7eZgmnfKuABqHn1jKG4` contains both mints. Existing DLMM layouts verify its pool and reserve PDAs, PermissionlessV2 state and Token-2022 flags. Reserves are `HxJzgogheSd1JwoBfqDVbQ1rpxjReFVVXQRfCSXvEmgW` and `5C95imQzbVXQumZqFWAAQUNFtNRsPTmGgsmf9kGVSgMe`. This is a transaction-research observation; it is not added to venue inventory, coverage or a new execution adapter.

Candidate records preserve complete observed instruction/account lists and
signers, and classify captured executables, verified PDAs, observed signers and
unknown accounts. Observed signers do not imply key possession. Unknown
accounts are not assumed issuer-controlled. DLMM Swap2/market layouts and
Squads administrative derivations are technically identifiable; an official
conversion instruction/discriminator/layout and canonical-holder account plan
remain unestablished. Published layout files are not source-to-bytecode proofs.

## Execution and eligibility boundary

Publicly verifiable evidence includes the asset configurations, sampled
transactions, deployed program links/code and the administrative/market PDA
relationships above. It does not establish a complete official plan for
`741Z…`, a successor destination controlled by that holder for such a plan,
conversion terms, required issuer approval or completion criteria.

No conversion-specific KYC, backend signature, allowlist, legal entitlement,
non-public seed, server proof or issuer signer requirement was established.
Those requirements remain **unknown**. Administrative MintTo and Squads fee
transactions do require their own authorities; they are not evidence that an
official holder conversion requires those signatures. Calling the official
path Unsupported on that basis would overclaim the dependency.

**NotTested** therefore remains the precise result. Stop rule C is met: after
the finite public investigation, no independently bound executable official
mechanism was established. No substitute transfer, burn/mint, DEX swap or issuer
signature is constructed. No official execution fixture/result is created,
because no official execution occurs. Current account/program captures are
distinct banks from historical transaction pre-state and must not be passed
off as a replay fixture for those sampled transactions.

## Architecture and offline CLI

`OfficialTransitionMechanism` is generic asset/identity/requirement evidence,
not issuer-specific conditionals. `TransitionScope` and `TransitionAssessment`
keep exact scope, eligibility, local authority and nullable existence separate.
An opaque `VerifiedTransitionExecution` cannot be deserialized or constructed
from discovery. The production adapter supplies no official execution receipt.
Controlled classifier tests exercise future execution gates without fabricating
a production transition.

`transition::research` loads digest-bound archives and issuer evidence entirely
offline, enforces recorded finite bounds, decodes current state and preserves
raw transaction provenance. It validates Phase 8 through fresh original replay,
then sends new OfficialTransition discovery through the existing generic
`LifecyclePathResolver`. Only that row's research evidence/reason changes.
No ticker/mint/issuer conditional is introduced in generic logic.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- investigate-transition \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --entity 741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs \
  --research probes/spacex-official-transition-research.json
```

Add `--format json --out <new-report>` and optionally
`--out-resolution <new-matrix> --out-discovery <new-discovery>`.
All saved files are canonical JSON; text is the default display. Existing or
identical output paths are rejected before replay. Absolute input paths and
manifest-relative references work from another directory. There is no RPC flag
or generation timestamp. The [evidence guide](lifecycle-phase-9-transition.md)
describes the schema and proof boundaries.

## Tests and actual mutations

Completed against the final code and published artifacts:

- `make test`: **244 passed, 0 failed, 0 ignored**; both synthetic SBF versions built.
- `make fmt-check`: passed.
- `make lint`: passed for all engine targets and both fixture versions.
- `git diff --check`: passed.
- Fresh offline CLI replay: report/stdout, matrix and discovery are byte-identical to their published artifacts; portable inputs and output protection passed.
- All **7 actual code mutations** failed through their specific assertions; exact source bytes were restored.
- Historical Phase 7/8 manifests retain 79/40 unchanged entries respectively; their four shared files have additive Phase 9 changes. Original data, execution fixtures, results and prior test assertions are preserved.

Suite breakdown: fixture v1 7, fixture v2 7, engine 102, expansion 6,
probe 12, lifecycle consequence 12, coverage 16, exposure 11, resolution 5,
snapshot 13, official transition integration 3, scenario 2, upgrade integration
42 and interface 6. The [validation record](../reports/spacex-phase9-validation.json)
binds the final artifact hashes to the [command logs](../reports/phase9-validation/).

The 15 new unit tests cover successor-only/pair/burn/mint nonclaims, issuer and
KYC/private boundaries, holder versus issuer signing, DEX isolation, exact
holder/mechanism/amount/bank/assets, source/successor/fee reconciliation,
Failed/Indeterminate behavior, unknown eligibility, serialization and generic
asset independence. Three integration tests check real successor/program/
transaction evidence, unchanged non-official rows, canonical offline CLI replay
from another directory, protected outputs, artifact tampering and entity
non-inheritance. All previous Phase 1–8 assertions remain intact.

| Injected fault | Specific catching assertion/test | Result |
| --- | --- | --- |
| Treat successor discovery as Proven transition | `successor_mint_reference_alone_cannot_prove_transition` | Caught; [log 1](../reports/phase9-mutations/01.txt) |
| Treat any source/successor token pair as official identity | `arbitrary_source_successor_pair_is_not_official_identity` | Caught; [log 2](../reports/phase9-mutations/02.txt) |
| Treat locally assumed issuer signing as independent authority | `issuer_assumed_private_signing_is_unsupported` | Caught; [log 3](../reports/phase9-mutations/03.txt) |
| Treat KYC/backend dependency as independently executable | `kyc_backend_dependency_is_unsupported` | Caught; [log 4](../reports/phase9-mutations/04.txt) |
| Allow DEX execution to prove OfficialTransition | `dex_execution_cannot_be_relabelled_official` | Caught; [log 5](../reports/phase9-mutations/05.txt) |
| Inherit transition execution to another holder | `transition_proof_does_not_inherit_to_another_holder` | Caught; [log 6](../reports/phase9-mutations/06.txt) |
| Treat a bounded unestablished mechanism as non-existence | `no_mechanism_is_not_nonexistence` | Caught; [log 7](../reports/phase9-mutations/07.txt) |

The mutations were injected into executable Rust code, not represented as
proposed tests. Each required its named assertion failure; compiler failures do
not count. Exact core bytes were restored, with source hashes and seven failure
logs in the mutation report.

## Artifacts and files

- [Complete investigation](../reports/spacex-official-transition.json): 870,145 bytes; issuer semantics, mint configurations, all selected available transaction observations, searches, program/account evidence, candidates and updated matrix.
- [Successor verification](../reports/spacex-successor-mint-verification.json): precise current properties and raw RPC pointer.
- [Search/program/account evidence](../reports/spacex-official-transition-search.json) and [candidate records](../reports/spacex-official-transition-candidates.json): extracted deterministic views of the complete report.
- [Updated path matrix](../reports/spacex-lifecycle-path-resolution-phase9.json) and [new discovery input](../probes/spacex-lifecycle-path-discovery-phase9.json): original four execution/applicability rows preserved.
- [Portable research manifest](../probes/spacex-official-transition-research.json): original artifacts, public raw captures, sources, limits and stop rule.
- [Raw captures](../evidence/transition/phase9/): initial RPC, paced retries, account/program reconstruction, source-query summary, successor external metadata, Squads layouts and HTTP provenance.
- [Validation](../reports/spacex-phase9-validation.json), [mutations](../reports/spacex-phase9-mutation-results.json), [file list](../reports/spacex-phase9-files.txt) and [checksums](../reports/spacex-phase9-artifacts.sha256).

| Artifact | SHA-256 |
| --- | --- |
| [Research manifest](../probes/spacex-official-transition-research.json) | `723dc28d7c243041086419f31d3ce416e7d8f2449a2b08c4fa030f8edaa875d4` |
| [Discovery input](../probes/spacex-lifecycle-path-discovery-phase9.json) | `5767830a5e32f8931a888030f2bdab6619e902e1dbf3e4e5e4aa16cfd7fc3e6c` |
| [Investigation](../reports/spacex-official-transition.json) | `fb7a79bd6f1a56daf09b34f3af49a0c5d11baa0b479b2922be200ff6b68e6d6d` |
| [Successor verification](../reports/spacex-successor-mint-verification.json) | `b406b5823ac96fe1740e1599f4a137d93044ed5b6bdf36d38f4985ebade9753a` |
| [Candidates](../reports/spacex-official-transition-candidates.json) | `4e71f5734e1f7403531b3eb08a72b7df5ba6fb24ea7c2b0f12a40fe97329e2ab` |
| [Search evidence](../reports/spacex-official-transition-search.json) | `65ee2baaf77b97f12cffa8ca6f0b68719870af1081e6373862e0857549f7aa27` |
| [Path matrix](../reports/spacex-lifecycle-path-resolution-phase9.json) | `993989bcdad106d7eef79895e258f4ce14bad18ec810503932688e0ebeb8e06d` |
| [Mutation results](../reports/spacex-phase9-mutation-results.json) | `cc6260c7bdf91cfea0a87d66551b4978754faaf04a89d132dd00f841978a7ed1` |
| [Validation](../reports/spacex-phase9-validation.json) | `93ce3ad238b28b1101a87712809cf0899542b44c4ff9699cde6544fd4c82e52e` |

The file list names every Phase 9 modified/new path, including command and
mutation logs. Only `AGENTS.md`, `README.md`, engine `lib.rs` and `main.rs` are
shared-file modifications. Transition source/tests/script and the listed data/
documentation are new. No dependency, original test, historical policy,
population, fixture, selection or result is modified. Historical checksum
manifests are retained rather than rewritten to hide additive shared changes.
Work remains on `main`, in the shared uncommitted working tree; no commit/push
occurred. Public acquisition was read-only, with no broadcast or issuer action.

## Product conclusion and Phase 10 recommendation

Eplyx independently verifies the published successor's current token state and
reconstructs actual public administrative and market patterns. It cannot yet
independently reproduce the issuer-defined transition for this real holder.
OfficialTransition remains NotTested, with a reproducible finite boundary and
explicit unknowns rather than invented execution or non-existence claims.
Prior exact transfer and secondary-market proofs retain their original limits;
neither proves lifecycle completion.

The smallest Phase 10 step is to reconstruct **one real SPACEX LP position**,
verify position ownership and authority, and test its actual Withdrawal path
against captured deployed state. Venue vault ownership alone must not stand in
for LP ownership or withdrawal rights. **Phase 10 has not been started.**
