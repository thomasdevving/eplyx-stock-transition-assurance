# Phase 5: a captured secondary-market exit probe

Phase 5 extends classification with actual local execution. It implements one
bounded Meteora DLMM X-to-Y `swap2` path, using the already verified SPACEX/USDC
venue and a wallet-compatible holder. The deployed DLMM, Token-2022 and legacy
token programs execute in the existing fresh LiteSVM backend. A quote is never
used as an execution result. No transaction is submitted to mainnet.

## Generic architecture

`probe/mod.rs` defines `ExecutionProbeSpec`, `ExecutionProbe`, concrete precondition
evidence, an execution plan, deterministic token/data deltas and `ExitabilityReport`.
`ExecutionProbe::build_execution` supplies validated accounts, programs, Clock,
normalized message and explicit assumptions. `executor::execute_probe_message`
loads the captured SBF bytes into a fresh VM and executes the complete transaction.
`ExecutionProbe::classify_result` independently reconciles state and actual event
CPI payloads. Protocol-specific account planning, instruction construction and
decoding live exclusively in `probe/meteora_dlmm.rs`.

The path taxonomy distinguishes `OfficialTransition`, `SecondaryMarketExit`,
`Redemption`, `Withdrawal`, `Transfer` and `Unknown`. This phase implements only
`SecondaryMarketExit`. Path results are `Succeeded`, `Failed` or `Indeterminate`.
`Succeeded` requires both a successful actual VM transaction and exact economic
reconciliation. A failed actual transaction with proven preconditions is `Failed`;
missing/unproven prerequisites or runtime setup/reconciliation blockers yield
`Indeterminate`. Invalid artifact bindings fail loading rather than being trusted.
No result asserts that all possible exits fail or that an exposure is stranded.

`LifecycleExecutionAssessment` retains the selected Phase 4 entity impact as its
baseline, then attaches the independent tested path and status for that same
entity. Its original lifecycle execution field remains `NotTested`, as does the
explicit `official_transition` field. The new `secondary_market_exit` field
records actual market-exit evidence. This does not replace lifecycle policy,
promise an issuer successor, modify previous frozen reports, or test LP/vault
withdrawals. `venue_vault_exitability_tested` is explicitly false.

## Captured state and instruction

The official [pinned DLMM IDL](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/idls/dlmm.json)
defines `swap2`, its account order and `Swap2Evt`. The
[official PDA helpers](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/ts-client/src/dlmm/helpers/derive.ts)
define oracle, event authority and bin-array derivation. The published bitmap
initialization instruction supplies its PDA seed. Instruction data contains the
8-byte discriminator, little-endian raw input/minimum output and an empty Borsh
remaining-hook slice vector. Active hooks are explicitly unsupported and produce
an indeterminate precondition result; they are never silently omitted.

The bounded supported slice is the earlier verified PermissionlessV2 pool,
Token-2022 X and legacy-token Y, internal-bitmap active index, up to four initialized
arrays in the X-to-Y direction, and an MM-only reconciled fill. Other routing,
extended-bitmap traversal or order-fill accounting is not inferred. A transaction
that executes but exceeds the supported reconciliation scope retains execution
evidence and is classified `Indeterminate`.

Capture uses four standard finalized RPC observations:

1. Genesis hash, matching the earlier snapshot.
2. Pool account, with `minContextSlot` at least the previously verified venue.
3. Five executable program headers, at least the initial pool context.
4. One final contextual batch containing every planned route/source/destination/
   authority/auxiliary/program/ProgramData/Clock account, at least the header context.

The final plan is revalidated from the final pool bytes. ProgramData addresses
must remain linked, canonical under the upgradeable loader, and contain valid
older deployment metadata/ELF. Programs retain their actual runtime loaders.
The Clock slot must match the final bank context. Pool/vault/mint PDAs and
ownership, source owner/amount/state, destination state, bins, oracle, optional
bitmap, mint pause/hook configuration and executable dependencies are checked.

Execution uses the **final Phase 5 batch**, not older balances mixed with newer
bins or programs. Earlier Phase 2/3 bytes supply provenance and entity/venue
identity links. All four RPC results remain in the fixture with exact request
parameters/context slots. Program ELF inputs, including allocated trailing bytes,
are extracted from ProgramData after its fixed loader metadata; no SDK/math stub
or embedded default token program substitutes for those captured dependencies.

The current USDC ATA is real and already initialized. If a future captured
destination is absent, the adapter constructs a real idempotent ATA-create
instruction using the captured ATA program and local fee payer. It never seeds
a fake token account or invents liquidity. Source balances and protocol state
are always the captured bytes. The only synthetic seeded account is a named,
deterministic local fee payer with 1,000,000,000 lamports.

## Token-2022 and exact deltas

Raw `u64` token amounts and signed `i128` deltas are canonical strings in JSON.
Mint decimals provide base-unit display, excluding scaled/interest-bearing UI
transformations and external USD valuation. The actual Token-2022 transfer CPI
applies epoch transfer fees and account/mint extension rules. The independent
official SPL transfer-fee calculation must match the observed vault-withheld delta.
Permanent delegate is not used: the original owner is the assumed message signer.
Captured source/vault initialization/frozen states and inactive mint pause/hook
configuration are checked. Default-account state is preserved; no SPACEX token
account is fabricated or newly initialized. Confidential-source transfer approval
is outside this probe and cannot be presumed.

Result evidence includes preconditions, account pointers/slots/hashes, full message
header/keys/compiled instructions, exact VM Clock, local seeded accounts, compute,
transaction fee, logs, raw inner instructions (including actual swap event), full
watched post-account bytes, token deltas and exact changed byte ranges. Raw token
and withheld-fee conservation, event amount/direction/actor/output/fee evidence,
pool protocol accumulator and MM bin reserves must reconcile. DLMM trading fees
remain inside the relevant vault public amount and are not counted again as
additional token movement. Failed atomic transactions require zero token/withheld
deltas; the local fee payer may still pay the transaction fee.

## Developer workflow

Captured artifacts already ship in the repository. Replay entirely offline:

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- probe \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --probe probes/spacex-usdc-dlmm-exit.json \
  --out /tmp/new-spacex-usdc-dlmm-exit-result.json
```

`--at` chooses lifecycle semantics only, defaulting to the policy effective time.
It does not advance the execution Clock or fetch another bank. The report binds
snapshot, scenario and fixture hashes; `validate` reruns the same VM and compares
the complete result. Output files must not already exist. Inspect the JSON status:
producing a report is distinct from the path's success/failure/indeterminate status.

For a **new** read-only fixture, select new output paths:

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- probe-capture \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --rpc https://api.mainnet-beta.solana.com --amount-raw 10000 \
  --entity-id solana-token-account:124XHuTYUNnCf2ABCNcCEdQFpHQ7Y6MnPe9NeouDB1az \
  --fixture-out /tmp/new-probe/fixture.json --probe-out /tmp/new-probe/probe.json
```

The selected account must be an existing wallet-compatible entity. Without
`--entity-id`, capture selects the first matching positive holder large enough
for the amount; it is not a discovery scan. Keep the fixture beneath the probe
directory so its stored path is relative and replayable on another machine.
If state or executable dependencies cannot be proved, a probe cannot claim exit
success. No RPC/API client exists in the replay adapter or transaction executor.

## Assumptions and limits

Signature verification and recent-blockhash checks are disabled only locally.
Original owner addresses and transaction signer privileges are retained; this
is not proof of wallet key possession or authorization. LiteSVM 0.16's pinned
mainnet feature profile and its default rent/epoch/syscall environment are used.
The captured Clock and actual programs/account state are exact inputs, but this
is not a full validator-bank, network-inclusion or future/historical reproduction.
Standard RPC and local digests do not supply signed chain inclusion proofs.

A successful bounded secondary-market exit does not prove official transition,
redemption, LP withdrawal, full-wallet exit, routes for other actors, future
liquidity or market value. The lifecycle requirement remains independently
applicable. No issuer backend, successor integration, burn/mint/migration,
global routing, price feed, frontend, risk scoring or mainnet action is introduced.
