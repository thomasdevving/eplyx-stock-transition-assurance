# Phase 6 portfolio assurance result

The offline coverage engine joins the complete Phase 4 population with fresh
Phase 5 probe executions. **One token-account entity has conditional full public
amount evidence on one secondary-market venue.** No class peers, other venues,
official transition or vault withdrawals inherit that evidence.

## Population and coverage

| Quantity | Exact result |
| --- | ---: |
| Token-account entities | 17,957 |
| Positive public-balance entities | 10,155 |
| Distinct observed owner authorities | 17,950 |
| Authorities with measured amount evidence | 1 |
| `Proven` entities | 1 |
| `PartiallyProven` entities | 0 |
| `Untested` entities | 12,378 |
| `Unsupported` entities | 5,578 |
| Represented public SPACEX amount | 8,741,534,482,051 raw |
| Conditional measured amount envelope | 60,354 raw |
| Represented amount without execution evidence | 8,741,534,421,697 raw |

`Proven` is narrowly defined: a successful full exact-input local execution at a
matching source balance on at least one requested path, under retained signer
and runtime assumptions. It does not prove signature possession, inclusion,
future liquidity, official conversion or every requested path. `Unsupported`
means current adapter/authority-model limits; it does not establish that an
asset cannot exit. Zero public balances are never vacuously proven.

The entity counts include 7,802 zero-public-balance accounts. They remain explicit
in classification counts, contribute zero represented amount, and do not imply
human holder identities. Account counts and owner-authority counts are separate.

| Account type | Entities | Positive balances | Proven | Untested | Unsupported | Represented raw | Covered raw |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| WalletCompatible | 12,379 | 7,118 | 1 | 12,378 | 0 | 8,074,468,973,174 | 60,354 |
| ProgramOwnedAuthority | 165 | 68 | 0 | 0 | 165 | 299,279,967,199 | 0 |
| Unknown | 5,413 | 2,969 | 0 | 0 | 5,413 | 367,785,541,678 | 0 |

Phase 3 protocol observations are overlapping descriptions of population vault
accounts, not additional holders or capital. No USD valuation is introduced.

## Real amount execution matrix

Target: `solana-token-account:124XHuTYUNnCf2ABCNcCEdQFpHQ7Y6MnPe9NeouDB1az`.
The retained original authority is locally assumed to sign. Every transaction
resets captured state and executes deployed DLMM `swap2` and both token programs.

Venue: `v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc`.
Input: SPACEX Token-2022. Output: legacy-token USDC.
All successful probes retain the test minimum output of one raw USDC.
This is not a recommended production slippage setting.

| Exact input raw SPACEX | Status | Actual output raw USDC | Token-2022 withheld fee raw | DLMM fee raw SPACEX | Compute units |
| ---: | --- | ---: | ---: | ---: | ---: |
| 1,000 | Succeeded | 606 | 5 | 1 | 39,932 |
| 10,000 | Succeeded | 6,065 | 50 | 2 | 45,101 |
| 30,000 | Succeeded | 18,109 | 150 | 5 | 45,075 |
| 60,354 | Succeeded | 36,388 | 302 | 9 | 45,078 |
| 60,355 | Indeterminate | — | — | — | — |

The 60,355-raw case fails the insufficient-source-balance precondition and never
executes a transaction. The original 10,000-raw Phase 5 result is separately
verified against fresh replay, giving five successful transaction cases but
only four distinct successful inputs. Token/vault/bin/event conservation and
fees reconcile on every success. Protocol and host fees are zero in these runs.

The full-balance simulated source ends at zero; its destination increases by
36,388 raw USDC. **No transaction was broadcast.** These are local post-states,
not a statement of the holder's current mainnet balance or actual funds movement.
The portfolio evidence amount is **max(1,000, 10,000, 30,000, 60,354) = 60,354**,
never the sum. Outputs are independent execution observations, not simultaneous
portfolio proceeds or interpolated quotes.

Execution stays at the captured Phase 5 Clock:
`slot=447884621`, `epoch=1036`, `unix_timestamp=1789676472`.
Policy interpretation stays at `2026-09-17T20:00:00Z`. The report does not pretend
to execute a historical Phase 4 bank or recapture a later bank. Matching captured
source balance permits conditional population amount evidence; differing source
balances receive zero historical-population amount coverage.

## Representative classes and path/venue gaps

Seven observed classes select 15 deterministic representatives using account
authority type, initialized state, active delegation, confidential extension
presence and verified role. Each class selects first zero, smallest positive
and largest positive balances with deterministic ordering. Samples never cover
other accounts. Zero representatives receive no fabricated positive-input case.

The plan contains 51 cases: 5 `Succeeded`, 1 `Indeterminate`, 2 `Untested` and
43 `Unsupported`. Capability requests are not VM execution claims. The two
wallet representative secondary-market requests lack captured routes and remain
untested. Other authority models and unsupported requested paths receive no
positive amount evidence.

All 17,957 entities have independent coverage for five requested paths:
`OfficialTransition`, `SecondaryMarketExit`, `Redemption`, `Withdrawal`, `Transfer`.
Only `SecondaryMarketExit` has measured amounts. **OfficialTransition remains
NotTested**, including for the conditionally proven entity. Redemption,
withdrawal and transfer are unsupported by the current execution adapter; this
is not proof that they are technically impossible. Vault/LP exitability is not
proved by swapping from the selected holder.

The one execution venue cohort contains the explicitly requested holder only,
not the entire population or its vault reserves. The engine accepts multiple
captured holder/venue specs and keeps venue cohorts independent. **A second real
venue and additional executed path types are not supplied or tested in this
production artifact.** An uncaptured alternative venue is covered by a
controlled test and receives zero execution assurance. Additional actual routes
require verified captured state and an executable supported adapter; no venue,
quote or successor transaction was invented to fill the gap.

## Artifacts and replay

- [Portable coverage plan](../probes/spacex-lifecycle-coverage-plan.json): full population/policy/impact fingerprints, exact amount matrix, representative requests and original Phase 5 result reference.
- [Complete portfolio assurance JSON](../reports/spacex-lifecycle-coverage.json): 61,218,022 bytes; all entities, per-path classifications, account-type/path/venue aggregates, representative selections and complete actual execution reports.
- [Coverage architecture and CLI](lifecycle-phase-6-coverage.md): classification contract, context bounds, input validation and offline replay.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- coverage \
  --snapshot snapshots/spacex-exposure.json \
  --impact reports/spacex-transition-impact.json \
  --plan probes/spacex-lifecycle-coverage-plan.json
```

Add `--format json --out <new-path>` for complete deterministic JSON.
No wall-clock generation timestamp enters the artifact. A saved JSON file and
JSON stdout are byte-identical. Existing files are never overwritten.

Plan SHA-256:
`8d0501e75a85a9c22932b76e7e013dfcf1406feb5e4ffe7d04a92fc58c1018bf`.
Report SHA-256:
`f3ff991165d23f9ff851b8e7d9c141a92aac4668bf26e2bc83926a12a2e4d920`.
Adjacent `.sha256` files verify from repository root.

## Completed validation

- `make test`: **175 passed, 0 failed, 0 ignored**. Both synthetic SBF versions built; no missing artifacts or skipped execution tests.
- `make fmt-check`: passed for engine/interface and fixture programs.
- `make lint`: passed for all engine targets and both fixture versions.
- `git diff --check`: passed.
- Canonical production report and CLI JSON stdout: byte-identical.
- Full population replay against final code: byte-identical to the checked-in assurance artifact.
- Plan/report digests and original Phase 2–5 artifact checksums: verified.

The 16 new focused tests cover real amount/max aggregation, full-balance success,
insufficient balance, alternative-venue evidence isolation, representative
selection/non-extrapolation, overlapping vault non-addition, zero/no-evidence
classification, unsupported official-only capability, actual slippage failure
and rollback, population/spec fingerprints, malformed raw amounts, missing
fixtures, duplicate cases, prior-result fresh replay/tamper rejection, report
roundtrip/revalidation, CLI portability/output protection, strict plan parsing,
invalid seed/amount matrices, differing-bank-balance noncoverage and independent
policy before-view handling. Controlled tests reconstruct earlier observation
provenance explicitly; actual execution uses the full unchanged Phase 5 fixture.

Phase 6 adds no dependency, capture, mainnet transaction, price source, issuer
integration or UI. Previous phase work remains in the shared uncommitted working
tree. Work remains on `main`; no commit or push was performed.
