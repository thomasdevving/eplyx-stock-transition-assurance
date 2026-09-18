# Phase 10 production native-withdrawal result

**Withdrawal = Proven** for one real Meteora DLMM PositionV2, specifically its
full captured liquidity principal, under its explicitly assumed original owner
signer. Deployed `RemoveLiquidityByRange2` credited **21,190,323 raw SPACEX** and
**8,132,564 raw USDC**, with **106,485 raw SPACEX** withheld at the destination.
All position liquidity shares became zero. The account and accrued fees remain;
fee collection and complete position closure were not tested. No mainnet
transaction was broadcast.

## 1. Selected real position

| Field | Exact value |
| --- | --- |
| Position | `BpTBNQ7vNaBujkwhgyBoiYyvEc6KrUUNTGiQDsrwTNxN` |
| Owning protocol/program | Meteora DLMM / `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` |
| Existing verified pool | `v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc` |
| Asset X | `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh` — SPACEX, Token-2022, 9 decimals |
| Asset Y | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` — USDC, legacy token, 6 decimals |
| Position range | `-102..-53`, inclusive; 50 bins |
| Nonzero share bins | 21 |
| Summed position shares | `400615463993425143130430305` raw u128 share units |
| Position layout | 8,120 bytes, PositionV2 discriminator `[117,176,212,199,245,180,133,182]`, version 1 |
| Raw position data SHA-256 | `b63dc7ccc951acd7332d824dad9e401dd2d1ee64bdb7d8225dc5bab58bd1f447` |

One finalized `getProgramAccounts` query was bounded to this already verified
pool with PositionV2 discriminator and pool-offset filters. It returned one
position at slot `448082269`. Selection occurred
before execution: positive SPACEX principal, encoded direct authority, a two-array
range, complete public dependencies and existing owner destinations. No failed
selection was replaced and no other holder, pool or LP position was tested.

## 2. Authority proof

The position itself encodes pool at byte offset 8 and original owner at 40:
`55uxDcXEaUjwit2Mv3EoNrtaGqTTUaUCvXpABpkoUUoE`. This owner is on-curve and its final captured account is
non-executable, empty-data, System-owned. Authority model: **DirectSigner**.
The sender meta is this encoded owner; both real destination accounts have that
same SPL owner. The vaults' SPL owner is the pool, which is not the LP owner.

`signer_possession_known = false`; `signer_assumed_locally = true`.
Local signature verification is disabled while message signer privilege and
program authority checks execute. No private key, issuer/admin signature,
multisig approval or PDA caller is acquired or invented. Operator and fee-owner
fields are the zero/System key; lock release is zero and operation bits are zero.
Ownership is directly encoded; no LP token/NFT or position PDA is fabricated.

## 3. Position exposure before execution

| Quantity | SPACEX raw | USDC raw |
| --- | ---: | ---: |
| Captured protocol-derived liquidity principal | 21,296,808 | 8,132,564 |
| Stored pending fees before checkpoint update | 0 | 0 |
| Calculated accrued/retained fees | 126,543 | 113,735 |
| Actual owner public credit from this removal | 21,190,323 | 8,132,564 |
| Actual destination withheld fee increase | 106,485 | 0 |

Principal is independently calculated per captured bin as
`floor(amount × position shares / bin liquidity supply)` and summed using checked
integer arithmetic, matching the pinned official SDK model. Pending/accrued fees
are separate Q64 checkpoint accounting. These calculations are not quotes,
private entitlements or execution proof. Actual VM token and share deltas are
independently reconciled below. Principal decimal base units are `0.021296808`
SPACEX and `8.132564` USDC; no scaled-UI transform or USD valuation is applied.

## 4. Native withdrawal mechanism and exact account plan

The [pinned official IDL](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/idls/dlmm.json)
and [SDK withdrawal builder](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/ts-client/src/dlmm/index.ts)
define Token-2022-aware `remove_liquidity_by_range2`. The discriminator is
`[204,2,195,145,53,145,145,205]`; arguments are little-endian lower `-102`, upper
`-53`, **10,000 basis points / 100%** and a zero-length remaining-account slices
vector. Writable canonical bin arrays follow the fixed accounts. Captured mint
pause is false and transfer hook program is null; no hook accounts are invented.

| Account index | Role | Address | Writable | Signer |
| ---: | --- | --- | --- | --- |
| 0 | PositionV2 | `BpTBNQ7vNaBujkwhgyBoiYyvEc6KrUUNTGiQDsrwTNxN` | True | False |
| 1 | Pool | `v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc` | True | False |
| 2 | Bitmap extension | `4VSLTuneC2hvm82x4DH38w4HMn4mviorTrQKAJRYU6if` | True | False |
| 3 | Owner SPACEX destination | `7JfDLUDDrfephemmoSCBFidpQNPY5gP5fa9wzYYM2fZY` | True | False |
| 4 | Owner USDC destination | `7GwrUQmGMfQPXfpocuyTx41AA8eCtvWGh698cqR36QNh` | True | False |
| 5 | SPACEX reserve | `HgQRhiATjX9jTWh7QgLWnhCeR7PaBqoWVf4vSaL61YVv` | True | False |
| 6 | USDC reserve | `82kPQkc7B8nZ23r1YNqNB3uA3cgYzwxP5partMjwdT6h` | True | False |
| 7 | SPACEX mint | `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh` | False | False |
| 8 | USDC mint | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` | False | False |
| 9 | Original position owner | `55uxDcXEaUjwit2Mv3EoNrtaGqTTUaUCvXpABpkoUUoE` | False | True |
| 10 | Token-2022 | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | False | False |
| 11 | Legacy token | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | False | False |
| 12 | Memo | `MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr` | False | False |
| 13 | Event authority | `D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6` | False | False |
| 14 | DLMM program | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | False | False |
| 15 | Bin array -2 | `2bfW1DbvLdmFfo76LeQsYM91onzLjpxS5VgLgwFZtssi` | True | False |
| 16 | Bin array -1 | `9PPwitu3fYPVzYby2BKFSdCF93psUCZjfdWznUX8bact` | True | False |


A compute-budget instruction requests 1,400,000 CU. No ATA creation is needed:
both real initialized/unfrozen destinations already exist. No fee/reward claim,
close-position, swap, redemption or conversion instruction is included. Oracle
is not an account of this native instruction. Published layout/interface
provenance does not claim source-to-deployed-bytecode equivalence.

## 5. Coherent captured execution state

Public standard RPC: `https://api.mainnet-beta.solana.com`.
Capture completed `2026-09-18T12:23:43.501382+00:00`. Initial planning/header batch slot
`448082468`; final execution
batch **`448082469`**, epoch **`1037`**, Clock timestamp
**`1789734213`**. All execution state/code comes from the final
batch of 22 accounts, rather than mixed older pool or position balances.
The discovery, genesis check and complete RPC results are retained verbatim as
parsed JSON with their exact request parameters. No RPC error occurred.

| Index | Address | Runtime owner | Present |
| ---: | --- | --- | --- |
| 0 | `2bfW1DbvLdmFfo76LeQsYM91onzLjpxS5VgLgwFZtssi` | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | Yes |
| 1 | `3gvYRKWyXRR9xKWe1ZjPhLY5ZJRN7KDB4rFZFGoJfFk2` | `BPFLoaderUpgradeab1e11111111111111111111111` | Yes |
| 2 | `4VSLTuneC2hvm82x4DH38w4HMn4mviorTrQKAJRYU6if` | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | Yes |
| 3 | `55uxDcXEaUjwit2Mv3EoNrtaGqTTUaUCvXpABpkoUUoE` | `11111111111111111111111111111111` | Yes |
| 4 | `7GwrUQmGMfQPXfpocuyTx41AA8eCtvWGh698cqR36QNh` | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | Yes |
| 5 | `7JfDLUDDrfephemmoSCBFidpQNPY5gP5fa9wzYYM2fZY` | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | Yes |
| 6 | `82kPQkc7B8nZ23r1YNqNB3uA3cgYzwxP5partMjwdT6h` | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | Yes |
| 7 | `9PPwitu3fYPVzYby2BKFSdCF93psUCZjfdWznUX8bact` | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | Yes |
| 8 | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` | `BPFLoader2111111111111111111111111111111111` | Yes |
| 9 | `BpTBNQ7vNaBujkwhgyBoiYyvEc6KrUUNTGiQDsrwTNxN` | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | Yes |
| 10 | `D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6` | `11111111111111111111111111111111` | Yes |
| 11 | `DoU57AYuPFu2QU514RktNPG22QhApEjnKxnBcu4BHDTY` | `BPFLoaderUpgradeab1e11111111111111111111111` | Yes |
| 12 | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | Yes |
| 13 | `HZcJwcJ2njPDxZtpPoKnF8v2w9QAx2rS7TdJPSRkbEhu` | `BPFLoaderUpgradeab1e11111111111111111111111` | Yes |
| 14 | `HgQRhiATjX9jTWh7QgLWnhCeR7PaBqoWVf4vSaL61YVv` | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | Yes |
| 15 | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | `BPFLoaderUpgradeab1e11111111111111111111111` | Yes |
| 16 | `MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr` | `BPFLoader2111111111111111111111111111111111` | Yes |
| 17 | `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh` | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | Yes |
| 18 | `SysvarC1ock11111111111111111111111111111111` | `Sysvar1111111111111111111111111111111111111` | Yes |
| 19 | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | `BPFLoaderUpgradeab1e11111111111111111111111` | Yes |
| 20 | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | `BPFLoaderUpgradeab1e11111111111111111111111` | Yes |
| 21 | `v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc` | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | Yes |

These indices resolve to `/records/2/response/result/value/<index>` in the
[execution capture](../evidence/withdrawal/phase10/execution-capture.json).
The report carries runtime owners, raw data hashes and exact references.
All accounts in this selected final plan were present; no absent token or bin
account was substituted. The event authority is canonically derived. The
optional ATA executable was captured but no ATA instruction executed.

| Program | Deployment slot | Allocated ELF input SHA-256 |
| --- | ---: | --- |
| `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | 423977638 | `d296c6771cec945601027613ca637c1be6721044c5859009d41c453477844c1f` |
| `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | 427147035 | `0999dbf708971e723b08d1caafc988826a59c6001ed6dc02260da07defbe1469` |
| `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | 419472000 | `8190d3f7ceb6cb7a7a8d8924bff89f9f611e15ce1f806f2b6237f3311a98f697` |
| `MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr` | Unavailable for legacy loader | `f520eaf096361abbb9639ea4dc3e5388a87b9330e121f476607b87c46ef67954` |

Upgradeable program headers/ProgramData links and canonical loader PDAs are
verified, including upgrade authority where present. Complete allocated ELF
input bytes are hashed and loaded under their original loaders. The runtime
supplies native System/compute-budget/loader support, instruction sysvar and
its mainnet feature/default rent/epoch/syscall profile. It is not a full validator
bank. The only local seeded account is the separately recorded synthetic fee
payer `Dc4mCR5W9cVG7pGn2j1V5CprwJnQechgS8LRFci1czxb`, initially 1,000,000,000 lamports.

## 6. Actual execution result

**Proven**, conditional on retained owner signer/runtime assumptions. Actual
transaction result: success, no error; **156,559 compute units**;
**10,000 lamports** fee from the synthetic payer.
Logs show `Instruction: RemoveLiquidityByRange2`, Token-2022 `TransferChecked`,
legacy-token transfer CPI and DLMM event CPI. Complete instruction payloads,
message, logs, inner instructions and watched post-account bytes are retained in
[the execution report](../reports/spacex-dlmm-withdrawal.json).

The original owner is unchanged; all 70 share entries end at zero. The position
is retained, with pending SPACEX/USDC fees **126,543 / 113,735 raw**. Claimed-fee
and claimed-reward totals stay zero. This establishes full liquidity-principal
removal, not fee collection, position closure or complete lifecycle completion.

## 7. Exact token, pool, bin and position reconciliation

| Quantity | Before raw | After raw | Delta raw |
| --- | ---: | ---: | ---: |
| Owner SPACEX public amount | 9 | 21,190,332 | +21,190,323 |
| Owner SPACEX destination withheld | 0 | 106,485 | +106,485 |
| Pool SPACEX reserve public amount | 679,080,878 | 657,784,070 | -21,296,808 |
| Owner USDC public amount | 247,139 | 8,379,703 | +8,132,564 |
| Owner USDC destination withheld | 0 | 0 | +0 |
| Pool USDC reserve public amount | 9,037,900 | 905,336 | -8,132,564 |


```text
21,296,808 SPACEX reserve debit = 21,190,323 owner public credit + 106,485 destination withheld
21,296,808 SPACEX reserve debit = 21,296,808 summed bin principal decrease
 8,132,564 USDC reserve debit   =  8,132,564 owner public credit = 8,132,564 summed bin decrease
400615463993425143130430305 removed position shares = identical summed bin supply decrease
```

The captured epoch-1037 SPL transfer-fee calculation (50 basis points) matches
the actual 106,485-raw withheld increase. Both vault withheld amounts remain
unchanged. The fee is withheld at the recipient and is separate from its public
spendable amount; it is not counted twice. Both mint accounts remain byte-identical.
Pool protocol-fee accumulators remain `657654999` X / `789456` Y raw.

Every position bin's share/supply and principal deltas reconcile independently.
Bin prices and fee/order fields remain unchanged; all peer bins outside this
position range are unchanged. The position's fee checkpoints update exactly
with retained accrual; ownership, range and claim totals remain bound. Pool and
bitmap relationships remain recorded. All eight changed external accounts have
exact old/new byte ranges and hashes, with complete post-state available.

## 8. LP lifecycle path matrix

| Path | Result |
| --- | --- |
| OfficialTransition | NotTested |
| Redemption | Unsupported |
| SecondaryMarketExit | NotApplicable to this position's native unwind |
| Transfer | NotApplicable to direct SPL movement of this protocol position |
| Withdrawal | Proven for exact full-range liquidity principal |

External policy interpretation remains **TransitionRequired** at
`2026-09-17T20:00:00Z`. The VM remains at its captured Clock. The lifecycle
rule does not alter DLMM bytes or fabricate a future/historical execution bank.
Withdrawal creates directly held tokens without converting them to a successor.
No indirect sale or position-transfer operation was executed; those would require
separate plans and proof. OfficialTransition and Redemption gain no evidence.

## 9. Canonical direct-holder comparison

| Path | Canonical direct token account | Selected real LP position |
| --- | --- | --- |
| OfficialTransition | NotTested | NotTested |
| Redemption | Unsupported | Unsupported |
| SecondaryMarketExit | Proven | NotApplicable / requires separate post-unwind sale |
| Transfer | Proven | NotApplicable / direct SPL movement is not native unwind |
| Withdrawal | NotApplicable | Proven, liquidity principal only |

The direct holder remains account
`741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs`, retained owner
`2wCvQzHiDHAHTvzwPeof9H3uEzq8Bzvg38DFbvZMGkuj`, original balance 17,621 raw.
Its complete Phase 9 matrix and all exact earlier proof contexts are unchanged.
[The deterministic comparison](../reports/spacex-lifecycle-state-shape-comparison.json)
binds both report hashes. The same external asset policy has different action
applicability across state shapes. Their execution banks remain separate; the
comparison is not simultaneous capacity, proceeds or portfolio coverage.

## 10. Evidence boundaries

This proves one real position's defined principal removal through one native
path, in one captured finalized bank, under an explicitly assumed original owner
signer. It does not prove possession/authorization, recent-blockhash freshness,
mainnet inclusion, current/future balances, peer LP ownership or all venues/ranges/
fractions. RPC observations are not signed inclusion proofs. No mainnet funds
move and the simulated zero shares are not asserted as current chain state.

A vault's pool authority does not prove the encoded LP owner. Direct-holder
Transfer/market-exit evidence does not prove Withdrawal; LP Withdrawal proves
neither issuer conversion nor redemption. Remaining accrued fees keep SPACEX
exposure in protocol accounting. Fee/reward claim and close-position execution
remain untested. No new price, valuation, score, global LP index, UI, issuer
integration or horizontal wallet/venue execution is introduced.

## 11. Architecture and deterministic CLI

`position::ProtocolPosition` records generic position/pool/authority/asset/exposure
facts and context. `WithdrawalProbe`/`WithdrawalScope` specify exact range/fraction
and fixture. Opaque `VerifiedWithdrawal` can only come from fresh adapter
execution; generic resolution checks the full scope and path type. The Meteora
adapter owns layouts, PDAs, native instruction construction and reconciliation.
It re-decodes and compares all original position facts before executing through
the existing `executor::execute_probe_message` backend. No asset/issuer
conditional is introduced in generic core and no dependency is added.

`discover-position` and `probe-withdrawal` are deterministic offline commands
using snapshot, scenario, discovery and capture. The latter also takes the
selected position artifact. Both protect existing outputs and produce canonical
JSON identical to saved files. Absolute input paths work from another directory
without RPC environment. The [architecture/CLI guide](lifecycle-phase-10-withdrawal.md)
contains copyable commands. No wall-clock generation timestamp enters outputs.

## 12. Tests and actual mutations

Completed against final code and published artifacts:

- `make test`: **267 passed, 0 failed, 0 ignored**; both synthetic SBF versions built.
- `make fmt-check`: passed.
- `make lint`: passed for all engine targets and both fixture versions.
- `git diff --check`: passed.
- Fresh offline execution and both CLI outputs: byte-identical to published position/withdrawal artifacts; portable inputs and output protection passed.
- Actual compute-starved native transaction: Failed, with every watched external account unchanged.
- All **8 actual code mutations** caught by their specific assertions; exact source restored.
- Historical Phase 7/8/9 manifests retain **79/40/45** unchanged entries; only their four shared scope/export/CLI files receive additive Phase 10 edits. Original policy, population, capture, plan, fixture, result and test assertions remain intact.

Suite breakdown: fixture v1 7, fixture v2 7, engine 115, expansion 6,
probe 12, lifecycle consequence 12, coverage 16, exposure 11, resolution 5,
snapshot 13, official transition integration 3, protocol position integration 10,
scenario 2, upgrade integration 42 and interface 6. The [validation record](../reports/spacex-phase10-validation.json)
binds the final report/fixture hashes to [complete command logs](../reports/phase10-validation/).

The 13 new unit tests check ownership/pool/authority binding, exact token/share
conservation, failed status, fraction/context isolation, direct-holder nonclaims,
issuer/redemption nonclaims, position non-inheritance, integer rounding/overflow
and generic asset independence. Ten integration tests execute the real captured
programs, reconcile token/bin/share/fee state, reject wrong authority/pool/vaults
and tampered accounting, require missing bin/bitmap Indeterminate preconditions,
check explicit signer assumptions, verify actual compute-starved VM failure with
rollback of every watched production account, repeat identical offline execution,
and reproduce both published CLI artifacts from another directory with protected
outputs. Controlled earlier provenance is explicitly reconstructed; execution
uses the unchanged complete Phase 10 capture. Prior assertions are preserved.

| Injected code fault | Named catching assertion | Actual result |
| --- | --- | --- |
| Treat pool vault authority as LP owner | `position::meteora_dlmm::tests::pool_vault_authority_cannot_grant_lp_ownership` | Caught; [log 1](../reports/phase10-mutations/01.txt) |
| Allow direct-holder evidence to prove Withdrawal | `position::tests::direct_holder_evidence_cannot_prove_withdrawal` | Caught; [log 2](../reports/phase10-mutations/02.txt) |
| Inherit one LP proof to positions sharing the pool | `position::tests::withdrawal_evidence_cannot_inherit_to_another_position` | Caught; [log 3](../reports/phase10-mutations/03.txt) |
| Ignore wrong position authority | `position::meteora_dlmm::tests::wrong_position_authority_is_rejected` | Caught; [log 4](../reports/phase10-mutations/04.txt) |
| Ignore missing bin/bitmap public state and grant execution assurance | `missing_bin_and_bitmap_state_are_explicitly_indeterminate` | Caught; [log 5](../reports/phase10-mutations/05.txt) |
| Mark failed withdrawal as Proven | `position::meteora_dlmm::tests::failed_withdrawal_never_becomes_proven` | Caught; [log 6](../reports/phase10-mutations/06.txt) |
| Allow Withdrawal to prove OfficialTransition | `position::tests::withdrawal_never_proves_official_transition_or_redemption` | Caught; [log 7](../reports/phase10-mutations/07.txt) |
| Treat locally assumed signer as known key possession | `signer_assumption_never_claims_private_key_possession` | Caught; [log 8](../reports/phase10-mutations/08.txt) |

All eight faults were injected into executable code, caught by named assertions
rather than compiler failures, and restored byte-for-byte. Actual failure logs
and source hashes are retained in the mutation artifact. No execution test is
skipped or weakened.

## 13. Product conclusion and artifacts

Eplyx can independently reconstruct and execute this real protocol-embedded
SPACEX position's native principal unwind. That action changes protocol liquidity
shares into direct public token holdings, with exact recipient transfer fees and
retained position-fee accounting. This is a new state shape, with different path
applicability from a direct token holder, while issuer transition remains unresolved.
It is not a complete readiness assertion.

- [Selected position](../probes/spacex-dlmm-position.json).
- [Full actual execution and reconciliation](../reports/spacex-dlmm-withdrawal.json).
- [Direct-holder/LP comparison](../reports/spacex-lifecycle-state-shape-comparison.json).
- [Raw discovery, execution fixture and interface sources](../evidence/withdrawal/phase10/).
- [Validation](../reports/spacex-phase10-validation.json) and [mutations](../reports/spacex-phase10-mutation-results.json).
- [Complete Phase 10 file list](../reports/spacex-phase10-files.txt) and [artifact checksums](../reports/spacex-phase10-artifacts.sha256).

| Artifact | SHA-256 |
| --- | --- |
| [Selected position](../probes/spacex-dlmm-position.json) | `6507259b54a2da2109cb3381d19d522ee43fbe16b98b3519db548b603c656a72` |
| [Execution capture](../evidence/withdrawal/phase10/execution-capture.json) | `3d79ad5dcee2a9b7ecc74575fe9af7e025695e229471b7aab5288d69813c844f` |
| [Native withdrawal](../reports/spacex-dlmm-withdrawal.json) | `5f74a2b70946efbc937fba9e9bcd028ce6b6586184104424c8f28e7ba39ddff5` |
| [State-shape comparison](../reports/spacex-lifecycle-state-shape-comparison.json) | `1e213b4746720b76cdcc42c5ada610f74c5f6716d7ae4690451911d472180584` |
| [Mutations](../reports/spacex-phase10-mutation-results.json) | `0f5204d80995aae754fa8f9b69659e58ec78e7d922b49cc3e64352711cd0f573` |
| [Validation](../reports/spacex-phase10-validation.json) | `2242ac31a156229a603f74339e91965ed4e4b033d0fff4a1986cc9cb55bf5003` |

Work remains on `main` in the shared uncommitted working tree. Only the four
shared scope/export/CLI files receive additive Phase 10 edits; original Phase
1–9 policies, captures, plans, results, tests and checksum files remain intact.
No commit or push occurred.

## 14. Smallest Phase 11 recommendation

Add one deterministic **Lifecycle Readiness / Pre-flight Gate** over the existing
exact path matrices. Require an explicit requested action and exact entity/
amount/range/context; return allow/block/unknown with the actual proof references,
remaining fee exposure and signer/runtime assumptions. A successful native unwind
must not imply official conversion or complete readiness, and stale/mismatched/
unresolved proof must remain explicit. Use the existing frozen artifacts; no
capture, new scoring, issuer integration or UI is needed. **Phase 11 has not been
started.**
