# Phase 3: one proven production integration

Phase 3 enriches a frozen Phase 2 snapshot with a deterministic exposure graph.
It observes custody relationships only. It does not execute LifecycleChange,
simulate swaps/withdrawals, estimate exitability, reconstruct LP positions,
introduce deadlines/corporate actions or use prices/risk scores/UI.

## Workflow

```sh
cargo run --locked -p eplyx-lifecycle-impact -- exposure \
  --snapshot snapshots/spacex.json \
  --adapter meteora-dlmm \
  --pool v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc \
  --rpc https://api.mainnet-beta.solana.com \
  --out snapshots/spacex-exposure.json

# Fully offline verification of both Phase 2 state and the exposure graph:
cargo run --locked -p eplyx-lifecycle-impact -- snapshot-show \
  --input snapshots/spacex-exposure.json

shasum -a 256 -c snapshots/spacex.sha256
shasum -a 256 -c snapshots/spacex-exposure.sha256
```

`--adapter meteora` is an alias for `meteora-dlmm`; exactly one adapter is
implemented. The pool argument is an untrusted candidate, never a product label
accepted as proof. `assets/prestocks-spacex-dlmm.json` records the first candidate
and its external locator source as data; it is not used as verification evidence.
No global protocol/pool scan, third-party indexer, API key or private endpoint is
needed. Output must not exist. A schema-2 graph cannot be silently replaced;
start another capture from the original schema-1 snapshot.

## Model and boundary

`LifecycleSnapshot.exposures` is an optional `ExposureGraph`. Its original
entities, classifications, capture metadata, mint and raw Phase 2 transcript are
retained. Generic graph/source code lives in `lifecycle/exposure/mod.rs`.
Protocol decoding and rules live in `lifecycle/exposure/meteora_dlmm.rs`.

`ExposureAdapter` supplies:

- identity, version, pinned decoder revision and program ID;
- a verified required-account plan from candidate pool bytes;
- protocol-specific proof and normalized `LiquidityExposure` output.

`discover` orchestrates the generic standard-RPC capture; `normalize_graph`
rebuilds the graph deterministically. The registry currently contains one adapter
and accepts exactly one candidate run. This is a boundary for later adapters,
not an indexer. Meteora rules never enter the mint/account discovery decoder.

`LiquidityExposure` contains protocol/product/program/pool/authority, both asset
mints and vaults, public balances and complete mint/account extensions, decoded
custody-relevant pool fields, adapter identity, verification slot and evidence.
LP/position model is explicitly null: none is discovered or reconstructed.

`AccountExposure` points to every parent entity and preserves its original type.
Only a proven vault receives `ClassificationRefinement {LiquidityVault,
program_controlled, protocol_exposure_id, evidence}`. Observed runtime program
ownership elsewhere does not turn into verified signing control.

Normalized adjacency is represented by sorted `ExposureEdge` records:

- lifecycle asset → each Phase 2 holding token account;
- account → active delegate / frozen state where actually present;
- pool → owning executable protocol program;
- pool → PDA authority;
- pool → canonical reserve vault;
- vault → mint;
- each pool asset → pool.

Every edge includes a rule/reason and raw evidence references with account
address, runtime owner, slot, decoder and SHA-256 of decoded account bytes. Phase
2 baseline types are reproducible from the parent entity/evidence, without
claiming human identity. The graph does not duplicate token balances into an
aggregate circulating-supply ledger.

## Protocol verification

The supported slice is **Meteora DLMM**, program
`LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`, 904-byte `LbPair`, state version 1,
pair type PermissionlessV2 (enum 3). Other DLMM pair types/state versions,
DAMM/DAMM v2 and different account lengths fail explicitly. They are not routed
through a fallback heuristic.

The decoder is pinned to official SDK revision
`576919e3e4368e542c402f000b4264724f7f23ec`:

- [Published account IDL](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/idls/dlmm.json)
- [Official pool and reserve PDA helpers](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/ts-client/src/dlmm/helpers/derive.ts)
- [Official token-program flag mapping](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/commons/src/extensions/lb_pair.rs)

The published zero-copy layout has explicit padding. The bounded Rust layout
reads the whole 896-byte body after the discriminator. Custody-relevant fields
are typed; fee/reward/bin regions are opaque fixed-width fields with all bytes
preserved in raw evidence. An upstream reduced IDL fixture checks full length,
relevant offsets, discriminator and enum ordering. No fee or pricing model is
introduced by parsing those bytes.

A pool must have the exact runtime owner, discriminator, length, supported
version/type, valid flag fields and bin-step seed. It must contain the lifecycle
mint, with two distinct mints and reserves. For PermissionlessV2:

```text
pool PDA = find_program_address(
    [base_key/preset parameter key, sorted mint 1, sorted mint 2], DLMM program
)
reserve PDA = find_program_address([pool address, reserve mint], DLMM program)
```

Pool address and stored bump must match. Both reserves must match their canonical
PDAs. The two pool token-program flags must agree with the actual mint owners.
Each vault must decode under that mint's correct token program, contain the
correct mint, be initialized and have the pool PDA as its SPL owner authority.
The protocol program account must be executable under a supported runtime loader.
On-chain ownership, discriminator and PDA/mint/vault relations together supply
proof; no symbols, labels or account-owner appearance substitute for these checks.

## Capture consistency and links

Only three RPC calls are retained:

1. `getGenesisHash`, which must match the parent snapshot's chain.
2. `getAccountInfo` for the candidate, finalized and bounded by the parent's
   maximum observed slot. Decode and validate an account plan.
3. One finalized `getMultipleAccounts` for pool, X vault, Y vault, X mint, Y mint
   and protocol program. Its context must be at least the initial candidate slot.

The final pool is decoded again; its required account plan must exactly match the
batch. All current relationship and balance evidence comes from that one final
context. Changing relationships, missing/malformed accounts, unsupported layouts
and wrong programs fail before any new snapshot is saved.

The lifecycle vault must exist in the Phase 2 entities. Its saved SPL owner and
mint must agree. Its Phase 2 authority-account bytes must themselves decode as the
same pool with identical mint/reserve/program relationships and canonical PDA
proof. This uses raw bytes already captured in Phase 2, not historical RPC. Both
old vault/pool references and current verification references are attached to the
refinement. If the link or old role cannot be proven, capture fails rather than
fabricating a link or reclassifying an unproven historical account.

Old and current balances are separate strings in `SnapshotVaultLink`; drift is
reported. Neither the frozen holder distribution nor original classification is
overwritten. The original file's canonical SHA-256 binds the graph to its parent.

## Versioning and deterministic replay

Schema 1 remains unchanged in serialization: `exposures=None` is defaulted on
load and omitted on save. Schema 2 requires a graph. Unexpected version/field
presence combinations fail. The graph has its own explicit schema version 1.

`LifecycleSnapshot::validate` first replays Phase 2 evidence, then replays each
adapter transcript using the supported pinned decoder and reconstructs every
exposure, link, hash, refinement, sorted edge and summary. Normalized facts must
match the result exactly. Original schema-1 files remain loadable and their
canonical JSON is byte-stable; no migration of their meaning is performed.

Evidence order remains the acquisition transcript. Account exposure records and
edges are sorted, as are the existing parent entities/extensions. A fixed snapshot
has deterministic serialization/replay. Another live capture is naturally a new
observation/time. SHA-256 and replay detect disagreement or accidental change;
they are not signed Solana state inclusion/completeness proofs.

## Summary interpretation and limits

WalletCompatible is retained Phase 2 compatibility, not direct beneficial
ownership. Verified program-controlled counts only linked, canonical protocol
vaults. Unknown remaining and runtime-program-owned-without-integration are
separate; unresolved entities sums all accounts for which neither wallet
compatibility nor the selected integration was proven. Delegated/frozen counts
refer to the retained Phase 2 state and overlap these categories.

Exactly one supplied pool is checked. The graph cannot prove that other venues,
vaults, custody structures, integrations or positions do not exist. Public vault
amounts include multiple economic components and are not a proof of withdrawable
LP reserves or executable liquidity. Fees, delegates, freeze state and mint-wide
extensions stay visible without transfer probes. In SPACEX, PermanentDelegate
also prevents interpreting pool control as exclusive control.

Token-2022 public balances read normally using the shared correct decoder.
Confidential configuration/ciphertext is preserved; encrypted amounts cannot be
decrypted from public RPC. Decimal base units deliberately exclude scaled or
interest-bearing display transforms. No lifecycle conversion interpretation is
attached to those configurations.

All current pool/balance facts share one final RPC context. Earlier Phase 2 holder
facts still have their original contexts. No combined atomic snapshot, account
last-write slot or historical reconstruction is claimed. RPC/provider trust and
the official SDK/program identity mapping remain assumptions; proving PDA
addresses is cryptographic math, not a Merkle inclusion proof. The deployed DLMM
source is not public, so the adapter does not claim source-to-bytecode equivalence
or audit swap/execution behavior.
