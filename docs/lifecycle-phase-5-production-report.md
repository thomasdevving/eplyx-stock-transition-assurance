# Phase 5 production execution result

**Succeeded:** the actual deployed Meteora DLMM `swap2` executed locally against
captured production state, debiting 10,000 raw SPACEX and crediting 6,065 raw USDC.
Both token programs executed, and account, withheld-fee, bin and swap-event deltas
reconcile. This is a bounded **SecondaryMarketExit** under the explicit local
signer/runtime assumptions. **OfficialTransition remains NotTested.** No mainnet
transaction was submitted or state mutated.

## Probe target

| Field | Exact value |
| --- | --- |
| Probe ID | `dlmm-secondary-market-exit-v1` |
| Source entity | `solana-token-account:124XHuTYUNnCf2ABCNcCEdQFpHQ7Y6MnPe9NeouDB1az` |
| Original classification | `WalletCompatible`; no human identity or signing access proved |
| Actual retained owner | `D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6` |
| Source mint | `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh` — Token-2022 |
| Input | `10000` raw / `0.00001` decimal base units |
| Source balance before | `60354` raw / `0.000060354` decimal base units |
| Venue | Meteora DLMM pool `v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc` |
| Paired/output mint | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` — USDC, legacy token program |
| Real initialized output ATA | `2Q2teh8BvwLXup6u62CFtZ31Rxj3xhfzpoLa69mjmwwi` |
| Minimum output | `1` raw USDC; test threshold, not a production slippage setting |
| Captured bank | Finalized slot `447884621`, epoch `1036` |
| Capture completed | `2026-09-17T20:21:23.700067Z` |

The amount is a small deterministic raw-unit input, below the holder's full
balance and about 0.00147% of the verified SPACEX vault's public amount. It avoids
testing the whole wallet or asserting exitability beyond the specified amount.
It crosses from active bin -62 to -63; a small fraction of total reserves does
not prove negligible price impact or available depth at all other amounts.

## Preconditions and actual execution

All ten recorded preconditions passed from raw final-batch data and the original
entity/venue proof:

- Pool program, target/paired mints and reserve addresses match the previously verified venue; pool/vault/oracle/bin PDAs are canonical.
- Source retains the original on-curve owner, whose captured authority remains an empty non-executable System-owned account.
- Source contains sufficient raw public SPACEX and is initialized and unfrozen.
- The real USDC destination is initialized, unfrozen and owned by the same holder.
- Token-2022 mint is unpaused; its transfer hook program is null/inactive.
- Both vaults have the pool as their SPL owner, correct mint/program and initialized/unfrozen states.
- Required initialized bin arrays, oracle and bitmap extension are present with correct layouts/ownership/relationships.
- Deployed DLMM, token and optional ATA/memo dependencies are executable, with canonical loader/ProgramData links and real ELF bytes.
- Final account plan remains consistent with the initial candidate plan.
- Captured Clock matches the final bank slot; its actual epoch determines transfer fees.

`executor::execute_probe_message` constructs a fresh LiteSVM 0.16, pins that
Clock, loads captured bytecode with original loaders, seeds captured accounts,
then executes the complete normalized transaction. A compute-budget instruction
sets 1,400,000 CU. The swap itself is the Token-2022-aware `swap2` instruction
from the [pinned official IDL](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/idls/dlmm.json).
The destination already exists, so no ATA creation executed in this result.

Observed transaction result: **success**, no error, **45,101 compute units**,
**10,000 lamports** transaction fee paid by the synthetic local fee payer.
Logs explicitly show `Instruction: Swap2`, Token-2022 `TransferChecked`, the
legacy-token CPI and DLMM event CPIs. The report retains their actual raw payloads,
full normalized message and watched post-account bytes, rather than an SDK quote.
The independently decoded `Swap2Evt` matches pool, original actor, direction,
full input, zero leftover, output and fee accounting.

## Exact state and token deltas

| Account / quantity | Before raw | After raw | Delta raw | Delta decimal base units |
| --- | ---: | ---: | ---: | ---: |
| Holder SPACEX source | 60,354 | 50,354 | -10,000 | -0.00001 SPACEX |
| Holder USDC destination | 24,545 | 30,610 | +6,065 | +0.006065 USDC |
| SPACEX vault public amount | 682,216,229 | 682,226,179 | +9,950 | +0.00000995 SPACEX |
| SPACEX vault withheld fee | 194,058 | 194,108 | +50 | +0.00000005 SPACEX |
| USDC vault public amount | 7,116,488 | 7,110,423 | -6,065 | -0.006065 USDC |
| Captured MM bin X reserves, summed | — | — | +9,948 | +0.000009948 SPACEX |
| Captured MM bin Y reserves, summed | — | — | -6,065 | -0.006065 USDC |

SPACEX token transfer fee: **50 raw**. DLMM LP/MM fee: **2 raw SPACEX**,
protocol fee **0 raw**, host fee **0 raw**, limit-order fee **0 raw**.
Exact conservation:

```text
10,000 source debit = 9,950 vault public increase + 50 vault withheld increase
 9,950 vault public increase = 9,948 MM bin reserve increase + 2 DLMM fee
 6,065 user USDC increase = 6,065 USDC vault decrease = 6,065 MM bin decrease
```

Pool protocol accumulator delta agrees with the actual zero protocol fee.
Both mint account bytes remain unchanged. The pool's active bin and variable-fee
state, oracle and active bin array change; the second bin array remains unchanged.
Every watched data change is recorded as exact old/new byte ranges with pre/post
hashes. Full post-state permits independent checking of token and protocol data.
No USD capital or externally sourced price is reported.

## Token-2022 behavior

The real deployed Token-2022 program applies the captured epoch-1036 transfer fee
schedule (50 basis points). The official SPL fee calculation independently agrees
with the 50 raw withheld increase. Withheld fees are separate from vault public
amount and are not counted twice. Permanent delegate is preserved but not used:
the original owner is the local assumed signer. The captured pause state is false
and hook program is null. Source/vault states are initialized and unfrozen.
Default-account state is preserved, with no fabricated/new SPACEX account.
Scaled UI configuration is retained but display conversions are not applied:
the reported amounts are raw token quantities divided by mint decimals.
Confidential-source approval/transfer is unsupported and never presumed.

## Lifecycle interpretation and architecture

The report attaches `LifecycleExecutionAssessment` to the selected original
Phase 4 entity ID. At the default hypothetical lifecycle time
`2026-09-17T20:00:00Z`, it remains `TransitionRequired / RequiresTransition`.
Its baseline Phase 4 impact and evidence are retained, and its newly tested
`SecondaryMarketExit` is `Succeeded`. Its explicit `OfficialTransition` remains
`NotTested`. Its venue-vault exitability is explicitly untested. The earlier
Phase 4 lifecycle execution field remains the baseline's `NotTested`; the wrapper
adds independent path-specific execution evidence rather than redefining that field.

`ExecutionProbe` supplies an execution plan and result reconciliation, while the
existing fresh-VM backend performs execution. The DLMM adapter owns protocol
rules; generic path/status/report/assessment models contain no issuer-specific
conditional. This extends the same Eplyx model: frozen state + proposed path +
execution consequence model → actual local post-state → exact state/token diff.
The lifecycle rule remains an external semantic input, not a token-program fact.

`--at` controls policy interpretation only. The VM Clock remains the captured
bank's `slot=447884621`, `epoch=1036`, `unix_timestamp=1789676472` even when a
different hypothetical lifecycle time is selected. Execution is not a newly
captured future/historical bank. Original snapshots are unchanged; Phase 5 uses
its own coherent final account batch instead of mixing earlier holder/pool balances.

## Evidence and replay artifacts

- [Probe specification](../probes/spacex-usdc-dlmm-exit.json): target, path, amount, rationale and snapshot/scenario/fixture fingerprints; portable relative fixture reference.
- [Captured execution fixture](../probes/spacex-usdc-dlmm-fixture.json): 5,490,496 bytes, all four complete RPC results and 22 final-batch accounts including deployed code.
- [Complete execution result](../reports/spacex-usdc-dlmm-exit-result.json): 83,212 bytes, preconditions, assumptions, account proofs, message, Clock, local fee payer, actual VM evidence, exact deltas and lifecycle assessment.
- [Execution architecture and CLI](lifecycle-phase-5-exitability.md): capture, offline replay, supported slice and limitations.

Capture used public standard RPC at `https://api.mainnet-beta.solana.com`:
genesis; initial pool slot `447884620`; executable headers slot `447884620`;
final route batch slot `447884621`. The final batch supplies every execution
account. Its addresses below resolve to record 3, `/value/<index>`, at that slot,
with runtime owners and raw hashes in `account_evidence`:

| Index | Account | Role |
| ---: | --- | --- |
| 0 | `124XHuTYUNnCf2ABCNcCEdQFpHQ7Y6MnPe9NeouDB1az` | Real SPACEX source |
| 1 | `2Q2teh8BvwLXup6u62CFtZ31Rxj3xhfzpoLa69mjmwwi` | Real initialized USDC destination |
| 2 | `2bfW1DbvLdmFfo76LeQsYM91onzLjpxS5VgLgwFZtssi` | Bin array index -2 |
| 3 | `3gvYRKWyXRR9xKWe1ZjPhLY5ZJRN7KDB4rFZFGoJfFk2` | Legacy token ProgramData |
| 4 | `4VSLTuneC2hvm82x4DH38w4HMn4mviorTrQKAJRYU6if` | Bitmap extension |
| 5 | `75yFZDjYhxGybJYxWrCFBZXmARrnDXQbriygrAL3y9SP` | Oracle |
| 6 | `82kPQkc7B8nZ23r1YNqNB3uA3cgYzwxP5partMjwdT6h` | USDC vault |
| 7 | `9PPwitu3fYPVzYby2BKFSdCF93psUCZjfdWznUX8bact` | Bin array index -1, used by swap |
| 8 | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` | Captured ATA executable |
| 9 | `BQbqzVpDkhFoCyV3YQR2LLJNpsVFzg9UhmASd9dLYsAz` | Canonical event authority |
| 10 | `D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6` | Original wallet-compatible owner |
| 11 | `DoU57AYuPFu2QU514RktNPG22QhApEjnKxnBcu4BHDTY` | Token-2022 ProgramData |
| 12 | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` | USDC mint |
| 13 | `HZcJwcJ2njPDxZtpPoKnF8v2w9QAx2rS7TdJPSRkbEhu` | DLMM ProgramData |
| 14 | `HgQRhiATjX9jTWh7QgLWnhCeR7PaBqoWVf4vSaL61YVv` | SPACEX vault |
| 15 | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | DLMM executable header |
| 16 | `MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr` | Captured memo executable |
| 17 | `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh` | Token-2022 mint |
| 18 | `SysvarC1ock11111111111111111111111111111111` | Actual captured Clock |
| 19 | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | Legacy token executable header |
| 20 | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | Token-2022 executable header |
| 21 | `v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc` | Pool |

DLMM ProgramData deployment slot: `423977638`; extracted ELF input SHA-256
`d296c6771cec945601027613ca637c1be6721044c5859009d41c453477844c1f`.
Token-2022 deployment slot: `427147035`; ELF input SHA-256
`0999dbf708971e723b08d1caafc988826a59c6001ed6dc02260da07defbe1469`.
Legacy token deployment slot: `419472000`; ELF input SHA-256
`8190d3f7ceb6cb7a7a8d8924bff89f9f611e15ce1f806f2b6237f3311a98f697`.
These hashes include the captured allocated ELF input bytes used by the VM.
Published IDL/layout provenance is not a claim of source-to-bytecode equivalence.

The runtime supplies native System/compute-budget/loader programs, transaction
instruction sysvar and its mainnet feature/default sysvar profile. The explicitly
recorded synthetic fee payer is the only seeded local account not from the batch.
The report's `local_accounts` carries its exact address and initial 1,000,000,000
lamports. The probe fixture supplies all externally captured route state and code.

Fixture SHA-256: `c523e3aa2d9d5fdbf2e38f890e037c868d1987a2fd3e42cd365ff0005b17f2ad`.
Spec SHA-256: `ab0f466c9c308fd440d3802135755084018ec3a59a7ebda7e3b36228c9321fe2`.
Result SHA-256: `62b0c38d09d41071d2bab6857853bd988bbda63c8e2cdf1161f94f7e121f2808`.
Adjacent `.sha256` files permit verification from repository root.

## Tests and validation

Completed against final code:

- `make test`: **159 passed, 0 failed, 0 ignored**, including both actual synthetic SBF builds and all previous tests.
- `make fmt-check`: passed.
- `make lint`: passed for all engine targets and both fixture versions.
- `git diff --check`: passed.
- Full captured production execution repeated offline: identical status, logs, CPI payloads and complete watched post-state.
- Canonical report file and CLI JSON stdout: byte-identical.
- Probe/result digests, lifecycle-path separation and earlier snapshot checksums: verified.

Suite breakdown: fixture v1 7, fixture v2 7, engine 47, Phase 2 13,
Phase 3 11, Phase 4 12, Phase 5 12, original scenario 2, upgrade integration 42,
interface 6. No execution tests were skipped or assertions weakened.

The 12 focused tests cover genuine controlled-capture success, insufficient
balance, wrong mint/pool/source-role rejection, missing bin/programdata,
Token-2022 fee accounting, user/vault/bin/fee reconciliation, explicit official
transition and vault-exit nonclaims, actual slippage failure with atomic rollback,
identical offline execution/message/state, report roundtrip and tamper detection,
fingerprints/context bounds, portable fixture loading and output protection.
The minimized earlier baseline fixture is explicitly test reconstruction of
genuine source/vault/authority bytes, not a verbatim production transcript.
Its execution state still comes from the unmodified full Phase 5 final batch.

## Phase 5 files changed

These 18 files were added or updated in this phase:

1. `AGENTS.md` — current path-specific execution scope and constraints.
2. `README.md` — execution result and offline workflow.
3. `engine/src/lib.rs` — exports the probe module.
4. `engine/src/main.rs` — `probe-capture` and offline `probe` commands.
5. `engine/src/executor.rs` — captured-message execution with full CPI/post-state evidence; original upgrade path unchanged.
6. `engine/src/probe/mod.rs` — generic taxonomy, probe/plan/report models, lifecycle assessment and replay validation.
7. `engine/src/probe/capture.rs` — bounded read-only fixture capture.
8. `engine/src/probe/meteora_dlmm.rs` — protocol preconditions, actual swap2 and exact state/event/MM reconciliation.
9. `engine/tests/execution_probe.rs` — 12 deterministic actual-execution tests.
10. `fixtures/spacex-dlmm-exit-baseline.json` — minimized earlier test provenance, explicitly reconstructed.
11. `probes/spacex-usdc-dlmm-exit.json` — portable probe specification.
12. `probes/spacex-usdc-dlmm-exit.sha256` — specification checksum.
13. `probes/spacex-usdc-dlmm-fixture.json` — complete real execution fixture and deployed programs.
14. `probes/spacex-usdc-dlmm-fixture.sha256` — fixture checksum.
15. `reports/spacex-usdc-dlmm-exit-result.json` — measured result and lifecycle assessment.
16. `reports/spacex-usdc-dlmm-exit-result.sha256` — result checksum.
17. `docs/lifecycle-phase-5-exitability.md` — architecture/CLI/evidence guide.
18. `docs/lifecycle-phase-5-production-report.md` — this final report.

No new dependency or existing test change was needed. Prior phase changes remain
uncommitted in the shared working tree. Work remains on `main`; no commit/push
was performed in this phase.

## Limits and smallest Phase 6 recommendation

This proves the selected amount's execution path in the captured local model
**under an assumed original owner signer**. Signature possession/authorization
and recent-blockhash freshness are not proved. The VM uses its pinned mainnet
feature profile and default rent/epoch/syscall configuration, not a full current
validator bank. RPC observations/hashes are not signed inclusion proofs. No
network inclusion, actual transfer, future liquidity or real funds movement occurs.

The probe does not prove an issuer successor transition, redemption, LP/vault
withdrawal, full 60,354-raw wallet exit, or exits for other entities/amounts/venues.
The simulated post-swap source balance is 50,354 raw; mainnet still has its
captured pre-state because nothing was broadcast. Lifecycle semantics remain
independent of the successful secondary-market path. No additional price feed,
USD valuation, successor integration, global routing or MintPause was introduced.

Smallest next product step: **one offline portfolio coverage report** joining the
Phase 4 full impact artifact with this typed Phase 5 result. Show precisely which
holder/amount has measured secondary-market evidence (10,000 raw), which source
amount remains untested (50,354 raw), and that all other entities and official
transition paths remain untested. Preserve context/signer assumptions and avoid
counting independent probes as simultaneous pool capacity. This adds a useful
Stocklana evidence view without another capture, venue, UI or mainnet action.
**Phase 6 had not been started when this Phase 5 report was written.**
The subsequent [Phase 6 coverage engine](lifecycle-phase-6-coverage.md) and
[portfolio assurance report](lifecycle-phase-6-production-report.md) track the new work.
