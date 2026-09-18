# Phase 8 lifecycle path resolution result

**Completed:** one real lifecycle-affected entity now has a deterministic,
five-path resolution. Transfer and SecondaryMarketExit are Proven for the exact
Phase 7 executions below, under locally assumed original-owner signing.
OfficialTransition is NotTested, Redemption is Unsupported, and Withdrawal is
NotApplicable to this direct holder. No transaction was broadcast. Phase 7
coverage, selection and evidence retain their original meaning.

## Canonical entity and lifecycle state

| Field | Exact value |
| --- | --- |
| Entity ID | `solana-token-account:741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs` |
| Token account | `741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs` |
| Retained owner authority | `2wCvQzHiDHAHTvzwPeof9H3uEzq8Bzvg38DFbvZMGkuj` |
| Source mint | `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh`, Token-2022, 9 decimals |
| Observed public balance | `17621` raw / `0.000017621` decimal base units |
| Account classification | `WalletCompatible`, no verified protocol/LP role |
| Lifecycle status | `TransitionRequired` |
| Lifecycle impact | `RequiresTransition` |
| Policy view | `2026-09-17T20:00:00Z`, original external policy/scenario |
| Original Phase 4 execution field | `NotTested`, preserved separately |
| Signer possession known | `false` |
| Signer assumed locally | `true` |

The original population source evidence is RPC record 2, `/value/6273/account`,
slot `447865621`, raw-data SHA-256
`fdd7b8048f3a0a81d7c99b3c69c7aea0db1d2642536cf835e030136cda71df33`.
The matrix retains this and the original mint evidence separately from later
execution captures and current mechanism research. The unchanged source balance
does not make those distinct banks one historical execution context.

## Lifecycle path matrix

| Path | Status | Exact established scope / reason |
| --- | --- | --- |
| OfficialTransition | **NotTested** | Issuer describes an on-chain conversion candidate and publishes a successor; no independent official conversion execution for this entity |
| Redemption | **Unsupported** | No verified supported redemption executor; issuer/KYC/backend/entitlement state is outside the available boundary |
| SecondaryMarketExit | **Proven** | Four exact successful inputs at the Phase 7 second DLMM pool, with all outputs and fees retained |
| Transfer | **Proven** | Four exact successful inputs to the real Phase 7 recipient, under locally assumed original-owner signing |
| Withdrawal | **NotApplicable** | Direct token-account holder with no verified protocol/LP position in this entity context |

Proven is a bounded local execution status, not an amount interval, network
inclusion, real authorization, issuer completion, or a claim covering every
context of that path. Each successful row also retains its 17,622-raw invalid
control as Indeterminate. That control does not become Failed: insufficient
source balance prevents any transaction attempt. No production Failed execution
is asserted; controlled tests exercise Failed and rollback requirements.

## OfficialTransition investigation

The current [issuer SpaceX notice](https://prestocks.com/spacex) links SPCXx mint
`Xs3oZwbHvqis4NYcf4YKWmEia2eC84wSiVrcYcTqpH8` and tells holders to swap
into it or another token before its stated deadline. The issuer's
[FAQ](https://prestocks.com/faq), including its captured public application
chunk, describes post-IPO conversion as on-chain without KYC. These are external
mechanism and policy assertions. They justify **NotTested** for a candidate
official path; they do not justify calling that path wholly off-chain.

Read-only public RPC research captured the source mint, the published successor
mint and the source's administrative authority at finalized slot `448024657`.
Both mints are initialized, non-executable Token-2022 accounts; the successor
has 8 decimals. The source's mint/freeze/permanent-delegate authority is
`WV9PJN7XTmTLVwbutCLFxp8TyePee6Xq5mRq6Fti5Wc`. Its captured address is
empty, System-owned and non-executable. A mint address or an administrative
authority is not itself a migration executor.

The bounded investigation inspected at most five recent signatures and one
parsed transaction for each of that authority and successor mint:

- The authority sample at slot `447186714` uses Squads, System and compute-budget instructions, with inner ATA creation and Token-2022 withheld-fee withdrawal. It establishes sampled fee administration, not conversion or redemption for this holder.
- The successor sample at slot `448024658` invokes `LanMV…` and token transfers. Those transfers do not establish the source-to-successor exchange, issuer completion criteria or this entity's eligibility.

No verified conversion instruction/account plan, claim or migration execution
fixture, or independently reconciled successor exchange was established in
this limited research. Generic Burn/MintTo, permanent-delegate powers and fee
withdrawals cannot substitute for that mechanism. The existing USDC swap stays
SecondaryMarketExit evidence even though the notice mentions swapping to other
tokens. This report does not reinterpret that wording as direct official proof.
The original lifecycle scenario, including its null successor field and
hypothetical policy boundary, remains unchanged.

Raw first-party pages, public FAQ code, HTTP capture records and all five RPC
results are preserved under [research evidence](../evidence/resolution/phase8/).
Initial Python certificate-store failures are recorded; successful captures
used TLS-validated system curl. The issuer X page returned 403 through browsing,
and its syndication endpoint returned an empty object. No unavailable post text
is presumed. This investigation is not a complete transaction history or proof
that no other conversion mechanism exists.

## Redemption investigation

The issuer's [2025-08-07 launch newsletter](https://newsletter.prestocks.com/p/prestocks-are-live-trade-spacex-openai)
distinguishes minting/redemption requiring KYC from ordinary token trading.
The current FAQ describes arbitrage involving off-chain exposure. Neither
source supplies a concrete verified redemption instruction and payout flow
that this executor can independently run for the selected holder. The bounded
transaction sample does not establish one either.

**Unsupported** denotes the current execution/evidence capability boundary.
Issuer onboarding, KYC, backend state and off-chain entitlement are unavailable;
historical general language establishes neither this holder's current
eligibility nor functioning redemption. This does not prove redemption
non-existence. DEX output, transfer, token ownership and a burn instruction
cannot independently prove redemption. No legal or investment conclusion is
made.

## Reused exact Phase 7 execution evidence

Only the selected entity's original transfer group 0 and swap group 3 are
replayed. Each transaction starts from its own unchanged captured state; the
outputs below are independent observations, never cumulative proceeds or
simultaneous capacity. No new holder, amount or venue campaign was introduced.

### Transfer

Destination: `124XHuTYUNnCf2ABCNcCEdQFpHQ7Y6MnPe9NeouDB1az`, a real captured
SPACEX token account. Its retained owner is
`D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6`; it is not a fabricated
local token destination.

| Exact source debit raw | Status | Recipient public credit raw | Recipient withheld increase raw | Compute units |
| ---: | --- | ---: | ---: | ---: |
| 176 | Proven | 175 | 1 | 4,622 |
| 4,405 | Proven | 4,382 | 23 | 4,622 |
| 8,810 | Proven | 8,765 | 45 | 4,622 |
| 17,621 | Proven | 17,532 | 89 | 4,622 |
| 17,622 | Indeterminate | — | — | — |

The deployed Token-2022 TransferChecked executes at captured Clock
`slot=448018365`, `epoch=1037`, `unix_timestamp=1789717146`.
The full input leaves a simulated source public balance of zero:
`17,621 debit = 17,532 recipient public increase + 89 withheld increase`.
Signer flags remain explicitly false/true for known possession/local assumption.
Neither the sender's real authorization nor a current mainnet balance follows.

### SecondaryMarketExit

Pool: `22PthLk8TYnurtbWKRyECFd99cHHbfsbPNHfeMetzfZg`.
Output mint: legacy-token USDC
`EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`.
Real initialized output account:
`EqY4swg9YTmipDPWBYNVEo3giMMkT2A42c4fr1o58J5i`.
The original test minimum is one raw USDC; it is not a production slippage
recommendation.

| Exact input raw | Status | Actual USDC output raw | Token-2022 fee raw | DLMM total fee raw | Included protocol fee raw | Compute units |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 176 | Proven | 105 | 1 | 4 | 0 | 39,437 |
| 4,405 | Proven | 2,641 | 23 | 88 | 8 | 39,436 |
| 8,810 | Proven | 5,282 | 45 | 176 | 17 | 39,439 |
| 17,621 | Proven | 10,567 | 89 | 351 | 35 | 39,436 |
| 17,622 | Indeterminate | — | — | — | — | — |

The captured deployed DLMM swap2 and both token programs execute at Clock
`slot=448018537`, `epoch=1037`, `unix_timestamp=1789717192`.
All successes reconcile account, vault, bin and independently decoded event
changes. Full-input conservation is:

```text
17,621 source debit = 17,532 X-vault public increase + 89 withheld increase
17,532 X-vault public increase = 17,181 MM-bin X increase + 351 DLMM fee
   351 DLMM fee = 316 LP fee + 35 protocol fee
10,567 user USDC increase = 10,567 Y-vault decrease = 10,567 MM-bin Y decrease
```

Host fee is zero. Protocol/LP fees are components of the vault public amount,
not extra token conservation terms. The output account changes from 674 to
11,241 raw USDC in the full-input simulation. Full local messages, program
inputs, logs, CPI payloads and watched post-account bytes remain in the original
digest-bound Phase 7 result files; this matrix is their scoped projection.

### Exact evidence references

Every digest below names the original JSON file under
`reports/phase7-evidence/results/<SHA-256>.json`. No original result is rewritten.
Fixture/state digests, source-before amounts, Clock, output mint/destination,
execution assumptions and errors are also preserved for each matrix point.

| Case | Matrix status | Original result SHA-256 |
| --- | --- | --- |
| `group-3-raw-176` | Proven | `429614ecbf4d015213b77e2047efe97e41bb5c7ded4e5518b71791d611eab8f0` |
| `group-3-raw-17621` | Proven | `697e63336a6468cb340cbbfdc614c5e27c2e1f64b80ccde7c1b7e65a17c822a6` |
| `group-3-raw-17622` | Indeterminate | `7f80ff9d726b076e3abaff78a4a360c88d750fd75200d548cf7a004dc6b54bd7` |
| `group-3-raw-4405` | Proven | `19cd07e2310a0707f48a7c0aa70648796066c29dffcfb750cc02f1dc574f0f0f` |
| `group-3-raw-8810` | Proven | `b3105e98c3c84aa5e4ef8879cfcab09a5b42dd4ba469a9b251bf42732605cb6d` |
| `group-0-raw-176` | Proven | `ab06340581cda069bc7875e974a7d51dde84ceba4d7ef5f2051960ccb4290cb9` |
| `group-0-raw-17621` | Proven | `85e1ee3067ce9a33bba222b2d88559d6a9bde91091b0ba10d661c37ac182895a` |
| `group-0-raw-17622` | Indeterminate | `493edfdecddb83193831108b12fbc3dc6595105a7ac37a7f7e5a0978605f4463` |
| `group-0-raw-4405` | Proven | `8911247a21051627a2e47e0a20f26b46829f4300f0380db3f85690f39d3a4e54` |
| `group-0-raw-8810` | Proven | `c1bec2bdaa04a8ce75c7daeb4a8970826f7115c25d6cf33cb7838df2c2a870d2` |

## Boundaries and evidence isolation

Observed population, local executions, external lifecycle policy and mechanism
research remain distinct typed evidence layers. Policy time does not set VM
Clock. Runtime defaults/features, captured program and account state, synthetic
local fee payer, original-owner signing assumption, transaction freshness and
RPC provenance limits remain explicit in the original evidence and projected
execution assumptions. RPC data and hashes are not signed inclusion proofs.

There is no established private-key possession, real-world authorization,
network inclusion, future liquidity, official completion, redemption payout,
backend eligibility or legal entitlement. No second production entity is
resolved. A controlled protocol-role test demonstrates Withdrawal NotTested;
the selected direct holder remains NotApplicable. A venue vault cannot confer
LP ownership or a withdrawal path on it.

The resolver filters exact entity and path, then resolves contexts independently.
A requested alternate pool without direct evidence retains its own NotTested
status. Every amount point remains explicit; full-balance proof does not create
an interval or prove other holders, amounts, recipients or venues. Transfer
cannot prove OfficialTransition, and SecondaryMarketExit cannot prove Redemption.
Each row retains unsuccessful/indeterminate controls alongside successful
points rather than silently erasing them.

## Architecture and deterministic CLI

`LifecyclePathResolver` combines one `LifecycleImpact`, generic path discovery
and verified execution facts into five `PathResolution` rows and one
`LifecycleResolution`. The strict statuses are Proven, Failed, Indeterminate,
Unsupported, NotTested and NotApplicable. The generic core and Phase 7 replay
adapter have no issuer, ticker or mint-specific conditionals; stock facts are
in manifests and captured evidence.

Discovery boundaries cannot grant Proven or Failed. `VerifiedExecution` has no
public/deserialization constructor. The adapter checks artifact digests and
original population/policy/selection/index bindings, then fresh-replays only
this entity's original ten cases and requires complete evidence equality.
Successful proof requires real VM success and economic reconciliation; Failed
requires an unsuccessful VM execution with verified rollback. A public test
rehashes fabricated execution/index/coverage copies and still fails that replay
gate. Consistent hashes alone do not establish execution proof.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- resolve-paths \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --entity 741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs \
  --coverage reports/spacex-lifecycle-coverage-phase7.json \
  --discovery probes/spacex-lifecycle-path-discovery.json \
  --evidence-bundle probes/spacex-phase8-evidence-bundle.json
```

Text is the default. Add `--format json --out <new-path>` for full canonical
JSON. Output paths are protected; saved JSON and JSON stdout are byte-identical.
Absolute inputs work from another directory, with portable manifest-relative
evidence references. Resolution has no RPC option or generation timestamp.
Reordering execution/discovery inputs leaves resolved rows identical. The
[architecture guide](lifecycle-phase-8-resolution.md) documents the contracts.

## Completed tests and mutations

- `make test`: **226 passed, 0 failed, 0 ignored**, including both actual synthetic SBF builds and all unchanged Phase 1–7 assertions.
- `make fmt-check`: passed for engine/interface and both fixture versions.
- `make lint`: passed for all engine targets and both fixture versions.
- `git diff --check`: passed.
- Public offline CLI replay from another working directory with RPC environment unset: canonical stdout, saved JSON and published JSON identical; existing output protected.
- Full evidence fresh replay and report roundtrip: passed.
- Rehashed fabricated execution/index/coverage copies: rejected by fresh replay.
- All seven real code mutations: caught by their specified test assertions, with exact source bytes restored.
- Original artifact checksums: verified. The Phase 7 83-file manifest has 79 unchanged entries and four additive shared-file updates (`AGENTS.md`, `README.md`, engine `lib.rs` and `main.rs`); historical checksums were not rewritten.

The 21 new tests comprise 16 unit tests and five integration tests. They cover
path/entity/venue non-inheritance, exact points/fees/banks/controls, signer flags,
applicability, Failed/Indeterminate isolation, strict parsing, discovery source
tampering, real successor bytes, ordering, serialization, public replay,
portable CLI and output protection. No existing test was changed or weakened.
Suite breakdown: fixture v1 7, fixture v2 7, engine 87, expansion 6,
execution probes 12, lifecycle consequence 12, coverage 16, exposure 11,
resolution integration 5, snapshot 13, original scenario 2,
upgrade integration 42, interface 6.

| Injected fault | Specific catching test | Result |
| --- | --- | --- |
| Allow Transfer to imply OfficialTransition | `transfer_cannot_prove_official_transition` | Caught; [log 1](../reports/phase8-mutations/01.txt) |
| Allow SecondaryMarketExit to imply Redemption | `market_exit_cannot_prove_redemption` | Caught; [log 2](../reports/phase8-mutations/02.txt) |
| Treat Unsupported as Failed | `unsupported_is_not_failed_or_nonexistent` | Caught; [log 3](../reports/phase8-mutations/03.txt) |
| Treat NotTested as Proven | `not_tested_never_becomes_proven` | Caught; [log 4](../reports/phase8-mutations/04.txt) |
| Inherit one entity proof to another entity | `entity_proof_does_not_inherit` | Caught; [log 5](../reports/phase8-mutations/05.txt) |
| Inherit one venue proof to every requested venue | `venue_proof_does_not_inherit` | Caught; [log 6](../reports/phase8-mutations/06.txt) |
| Treat assumed signer as real signature possession | `local_assumed_signer_never_becomes_possession` | Caught; [log 7](../reports/phase8-mutations/07.txt) |

[Validation JSON](../reports/spacex-phase8-validation.json), its complete command
logs, [mutation results](../reports/spacex-phase8-mutation-results.json) and
seven individual assertion-failure logs preserve the actual completed checks.
Compiler failures are not accepted as caught mutations.

## Artifacts and files changed

- [Canonical matrix](../reports/spacex-lifecycle-path-resolution.json): 49,076 bytes, all rows, exact points, typed provenance and explicit boundaries.
- [Human CLI output](../reports/spacex-lifecycle-path-resolution.txt): actual default text output from offline replay.
- [Discovery manifest](../probes/spacex-lifecycle-path-discovery.json): first-party facts, research scope, boundaries and artifact hashes.
- [Evidence bundle](../probes/spacex-phase8-evidence-bundle.json): portable references to original Phase 4, 6 and 7 artifacts.
- [Complete Phase 8 file list](../reports/spacex-phase8-files.txt) and [checksums](../reports/spacex-phase8-artifacts.sha256): every modified/new Phase 8 file, including raw captures, validation and mutation logs.

| Artifact | SHA-256 |
| --- | --- |
| [spacex-lifecycle-path-discovery.json](../probes/spacex-lifecycle-path-discovery.json) | `01d9a2a723dcea3969cc9e4db5168d9c99b7a1c6730a7197ffd0b1003fe16a2d` |
| [spacex-phase8-evidence-bundle.json](../probes/spacex-phase8-evidence-bundle.json) | `7165811ebb44c8672c6b215c4cd98c9d7a2ae1cac96c3324c0657c7afc4bcdd2` |
| [spacex-lifecycle-path-resolution.json](../reports/spacex-lifecycle-path-resolution.json) | `0e8d353464d6444a6436b5fc1cd3013e203b554bd53c215ecdb494c9119a3dc6` |
| [spacex-lifecycle-path-resolution.txt](../reports/spacex-lifecycle-path-resolution.txt) | `dc0fce04745c958da5b89d3b5d7cb04970f4db308196e94f892a91f07117e272` |
| [spacex-phase8-mutation-results.json](../reports/spacex-phase8-mutation-results.json) | `4409e52f2dfa3a828656fb0a7253c9a3a2c2501d07b82576f11af3cbdc1a8346` |
| [spacex-phase8-validation.json](../reports/spacex-phase8-validation.json) | `a5f05e52607bb234938e43454e43e0e5fd16ebd304d962838382e9f30ef60c17` |

Adjacent `.sha256` files verify the two inputs, JSON matrix, human summary,
mutation results and validation JSON from repository root. The full Phase 8
checksum manifest includes all listed files except itself.

The following 45 paths constitute Phase 8. The four shared files were modified; all other listed paths are new for this phase.

| File | Change |
| --- | --- |
| [AGENTS.md](../AGENTS.md) | Modified |
| [README.md](../README.md) | Modified |
| [docs/lifecycle-phase-8-production-report.md](../docs/lifecycle-phase-8-production-report.md) | New |
| [docs/lifecycle-phase-8-resolution.md](../docs/lifecycle-phase-8-resolution.md) | New |
| [engine/src/lib.rs](../engine/src/lib.rs) | Modified |
| [engine/src/main.rs](../engine/src/main.rs) | Modified |
| [engine/src/resolution/mod.rs](../engine/src/resolution/mod.rs) | New |
| [engine/src/resolution/phase7.rs](../engine/src/resolution/phase7.rs) | New |
| [engine/src/resolution/tests.rs](../engine/src/resolution/tests.rs) | New |
| [engine/tests/lifecycle_resolution.rs](../engine/tests/lifecycle_resolution.rs) | New |
| [evidence/resolution/phase8/chain-investigation.json](../evidence/resolution/phase8/chain-investigation.json) | New |
| [evidence/resolution/phase8/http-capture-first-failure.json](../evidence/resolution/phase8/http-capture-first-failure.json) | New |
| [evidence/resolution/phase8/http-capture.json](../evidence/resolution/phase8/http-capture.json) | New |
| [evidence/resolution/phase8/issuer-faq-chunk-0.js](../evidence/resolution/phase8/issuer-faq-chunk-0.js) | New |
| [evidence/resolution/phase8/issuer-faq.html](../evidence/resolution/phase8/issuer-faq.html) | New |
| [evidence/resolution/phase8/issuer-launch.html](../evidence/resolution/phase8/issuer-launch.html) | New |
| [evidence/resolution/phase8/issuer-spacex-post.json](../evidence/resolution/phase8/issuer-spacex-post.json) | New |
| [evidence/resolution/phase8/issuer-spacex.html](../evidence/resolution/phase8/issuer-spacex.html) | New |
| [evidence/resolution/phase8/source-metadata.json](../evidence/resolution/phase8/source-metadata.json) | New |
| [probes/spacex-lifecycle-path-discovery.json](../probes/spacex-lifecycle-path-discovery.json) | New |
| [probes/spacex-lifecycle-path-discovery.sha256](../probes/spacex-lifecycle-path-discovery.sha256) | New |
| [probes/spacex-phase8-evidence-bundle.json](../probes/spacex-phase8-evidence-bundle.json) | New |
| [probes/spacex-phase8-evidence-bundle.sha256](../probes/spacex-phase8-evidence-bundle.sha256) | New |
| [reports/phase8-mutations/01.txt](../reports/phase8-mutations/01.txt) | New |
| [reports/phase8-mutations/02.txt](../reports/phase8-mutations/02.txt) | New |
| [reports/phase8-mutations/03.txt](../reports/phase8-mutations/03.txt) | New |
| [reports/phase8-mutations/04.txt](../reports/phase8-mutations/04.txt) | New |
| [reports/phase8-mutations/05.txt](../reports/phase8-mutations/05.txt) | New |
| [reports/phase8-mutations/06.txt](../reports/phase8-mutations/06.txt) | New |
| [reports/phase8-mutations/07.txt](../reports/phase8-mutations/07.txt) | New |
| [reports/phase8-validation/git-diff-check.txt](../reports/phase8-validation/git-diff-check.txt) | New |
| [reports/phase8-validation/make-fmt-check.txt](../reports/phase8-validation/make-fmt-check.txt) | New |
| [reports/phase8-validation/make-lint.txt](../reports/phase8-validation/make-lint.txt) | New |
| [reports/phase8-validation/make-test.txt](../reports/phase8-validation/make-test.txt) | New |
| [reports/spacex-lifecycle-path-resolution.json](../reports/spacex-lifecycle-path-resolution.json) | New |
| [reports/spacex-lifecycle-path-resolution.sha256](../reports/spacex-lifecycle-path-resolution.sha256) | New |
| [reports/spacex-lifecycle-path-resolution.txt](../reports/spacex-lifecycle-path-resolution.txt) | New |
| [reports/spacex-lifecycle-path-resolution.txt.sha256](../reports/spacex-lifecycle-path-resolution.txt.sha256) | New |
| [reports/spacex-phase8-artifacts.sha256](../reports/spacex-phase8-artifacts.sha256) | New |
| [reports/spacex-phase8-files.txt](../reports/spacex-phase8-files.txt) | New |
| [reports/spacex-phase8-mutation-results.json](../reports/spacex-phase8-mutation-results.json) | New |
| [reports/spacex-phase8-mutation-results.sha256](../reports/spacex-phase8-mutation-results.sha256) | New |
| [reports/spacex-phase8-validation.json](../reports/spacex-phase8-validation.json) | New |
| [reports/spacex-phase8-validation.sha256](../reports/spacex-phase8-validation.sha256) | New |
| [scripts/test-phase8-mutations.py](../scripts/test-phase8-mutations.py) | New |

Earlier phases remain in the shared uncommitted working tree. Changes in this
phase are additive to the four common files listed above; earlier phase modules,
tests, policy, population, plans, fixtures and result artifacts are preserved.
No dependency was added. Work remains on `main`; no commit or push occurred.
There was no broadcast, authenticated issuer action or issuer transition
simulation.

## Product conclusion and smallest Phase 9 recommendation

This one holder has two independently replayed, bounded local execution paths:
Transfer and SecondaryMarketExit. Their exact full-input points move 17,621 raw
SPACEX to 17,532 raw recipient public SPACEX plus 89 withheld, or to 10,567 raw
USDC respectively. The original owner is locally assumed to sign. These are
alternative simulations, not actual or simultaneous portfolio actions.

Eplyx can establish that captured token mobility. It cannot establish completion
of the issuer-defined lifecycle transition or redemption. The lifecycle impact
therefore remains RequiresTransition; the five-path matrix makes the remaining
mechanism and applicability boundaries explicit.

The smallest Phase 9 step is one read-only Stocklana demonstration view of this
frozen entity matrix: five statuses, exact-point drill-down, digest-linked
evidence and visible policy/signer/runtime boundaries. It can consume the
existing deterministic artifacts without another capture, holder, venue,
execution campaign or coverage score. **Phase 9 has not been started.**
