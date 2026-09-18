# Phase 10 protocol-position withdrawal

Phase 10 introduces one production state shape: a real Meteora DLMM PositionV2.
It executes one native liquidity-principal removal against frozen deployed code
and account state, with the encoded original owner assumed to sign locally.
A pool vault establishes protocol custody, not LP ownership or withdrawal rights.

## Evidence and model

`position::ProtocolPosition` records position ID, protocol, pool, encoded owner,
authority model, two assets, bin range, all 70 liquidity share entries, current
principal exposure, pending/accrued fee accounting and original raw fingerprints.
Lifecycle status comes from the external scenario at its explicit effective-time
view, independently of the captured VM Clock.

`WithdrawalProbe` specifies the exact position, pool, authority, range, fraction
in basis points and compute limit. `WithdrawalScope` also binds the exact fixture
hash. `VerifiedWithdrawal` has no public or deserialization constructor. Only the
adapter's fresh execution and reconciliation can provide a production proof.
Generic path resolution checks the complete scope and Withdrawal path type;
neither direct-holder evidence nor serialized report assertions grant proof.

The smallest protocol slice is in `position::meteora_dlmm`: PositionV2 with an
8120-byte layout, at most 70 bins, an initialized captured bitmap, existing
owner destination ATAs, and an encoded direct owner. PDA/program-controlled or
private authority requirements are outside this adapter; no admin is impersonated.
Absent public dependencies cause explicit Indeterminate preconditions. The
adapter never seeds replacement bins, reserves, positions or token accounts.

## Layout and withdrawal provenance

The [pinned official IDL](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/idls/dlmm.json)
and [official SDK builder](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/ts-client/src/dlmm/index.ts)
define PositionV2, bin/account relationships, `remove_liquidity_by_range2` and
remaining accounts. Complete source copies and their URL/revision/hash provenance
are retained in [the raw evidence directory](../evidence/withdrawal/phase10/).
IDL provenance is not a source-to-bytecode equivalence claim.

PositionV2 encodes its pool at offset 8, owner at 40, and 70 u128 shares at 72.
Its lower/upper bins are at 7912/7916. Fee/reward accounting, operator, fee owner,
lock point and version are retained. Withdrawal uses the encoded owner rather
than a vault's SPL authority. Position ownership is encoded directly; this tested
model does not use a fabricated LP token/NFT or require a position PDA inference.

Each 10136-byte bin array has a verified PDA, discriminator, index and pool link.
It contains 70 bins of 144 bytes at offset 56. Principal exposure is calculated
with checked integer arithmetic per bin:

```text
principal side = floor(bin side public amount × position shares / bin supply)
removed shares = floor(position shares × requested bps / 10000)
removed principal = floor(bin side public amount × removed shares / bin supply)
```

Accrued fees use the pinned SDK's Q64 fee checkpoint calculation and captured
pending amounts. They are distinct from principal. Estimates are never execution
claims. Unrepresentable checked products stop rather than inventing a value.

The native instruction has discriminator `[204,2,195,145,53,145,145,205]`, then
little-endian i32 lower, i32 upper, u16 fraction and a zero-length Borsh
`RemainingAccountsInfo.slices` vector. No active transfer hook is assumed;
canonical writable bin arrays follow the fixed accounts. The report retains all
metas, signer/writable privileges and the complete normalized message. A
1,400,000-CU instruction precedes removal. No claim, close or sale is appended.

## Capture and replay

Acquisition was bounded to one already verified pool. A finalized
`getProgramAccounts` query filters the PositionV2 discriminator and pool at offset
8. Exactly one position was returned. A genesis check, initial pool/position/
owner/program-header batch and final complete route batch were captured using
public standard RPC. Every execution account and deployed bytecode comes from
the final batch. The initial response identifies the plan; earlier snapshots and
discovery are provenance rather than mixed execution balances.

Upgradeable program headers must link to canonical loader ProgramData PDAs.
The adapter verifies their owner, state, deployment slot and complete ELF bytes.
Clock must match the final account context. The runtime supplies native programs,
instruction sysvar and its mainnet feature/default sysvar profile. The one local
synthetic fee payer is recorded separately with initial 1,000,000,000 lamports.
It provides fees only; no liquidity, protocol authority or token value is seeded.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- discover-position \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --discovery evidence/withdrawal/phase10/position-discovery.json \
  --fixture evidence/withdrawal/phase10/execution-capture.json

cargo run --locked -q -p eplyx-lifecycle-impact -- probe-withdrawal \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --discovery evidence/withdrawal/phase10/position-discovery.json \
  --position probes/spacex-dlmm-position.json \
  --fixture evidence/withdrawal/phase10/execution-capture.json
```

Both commands are offline. Text is the default; `--format json --out <new-path>`
saves canonical pretty JSON with a final newline, identical to JSON stdout.
Existing outputs are rejected before validation/execution. Absolute CLI input
paths work from other directories. No RPC environment is required. Captured
acquisition time is evidence; no wall-clock report-generation timestamp is added.

Replay validates snapshot/scenario, raw fixture/discovery fingerprints, re-decodes
position/accounting facts and compares the complete position artifact before
executing. A saved result cannot promote itself to verified evidence. Wrong
position/pool/authority/range, changed amounts and missing accounts fail explicitly.

## Reconciliation and status

Successful transactions must satisfy exact position-share and bin-supply removal
for every position bin, exact bin-principal changes, and token conservation:

```text
reserve public debit = sum(bin principal decrease)
reserve public debit = user public credit + destination withheld increase
withheld increase = official SPL captured-epoch transfer fee
```

Mint bytes and reserve withheld amounts remain unchanged. Pool protocol-fee
accumulators do not change. Bin prices, fee/order fields and peer bins outside
the position remain unchanged. Position authority/range stays bound; its fee
checkpoints update and accrued fees remain unclaimed. Claimed-fee/reward totals
are checked unchanged. Exact data ranges/hashes and complete watched post-accounts
are retained. Full removal is recognized only when every position share is zero.

Proven requires actual fresh VM success and these checks. Failed requires an
actual VM failure with rollback of every watched external account. The fee payer
may still pay transaction fees. Missing public state is Indeterminate; unsupported
private/authority capabilities are Unsupported. No execution is invented for
precondition failures.

Withdrawal evidence grants neither OfficialTransition nor Redemption. Direct
SPL Transfer and market sale are not this protocol-position operation and are
NotApplicable to the tested position path; separately acting on withdrawn tokens
requires separate evidence. Full principal removal does not prove fee collection,
position closure or complete lifecycle readiness.

## Validation and boundaries

The [production report](lifecycle-phase-10-production-report.md) records the
actual suite and mutation results. [The mutation runner](../scripts/test-phase10-mutations.py)
injects eight dangerous code faults, requires a named assertion failure rather
than a compiler failure, records each log and restores exact original source.
It protects existing mutation outputs.

This fixture proves one exact position/range/fraction in one frozen bank under
assumed original owner signing. It proves no other positions, venues, future
liquidity, private-key possession, transaction inclusion or issuer entitlement.
Public RPC hashes are observations, not signed inclusion proofs. The VM is a
captured-account local model, not a complete validator bank. No mainnet
transaction occurs and historical Phase 1–9 evidence remains preserved.
