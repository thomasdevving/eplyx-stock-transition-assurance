# Phase 2: production token-asset discovery

Phase 2 adds observation only. It does not execute LifecycleChange, simulate
migration or introduce corporate-action rules, liquidity integrations or risk
scores. Existing ProgramUpgrade behavior and tests remain in place.

## Capture and replay

```sh
cargo run --locked -p eplyx-lifecycle-impact -- snapshot \
  --mint PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh \
  --rpc https://api.mainnet-beta.solana.com \
  --asset-config assets/prestocks-spacex.json \
  --out snapshots/spacex.json

# Entirely offline: re-decode and verify all normalized facts from raw evidence.
cargo run --locked -p eplyx-lifecycle-impact -- snapshot-show \
  --input snapshots/spacex.json
```

Output paths must not exist: frozen evidence is never overwritten. Omit
`--asset-config` to query an arbitrary SPL Token or Token-2022 mint. A configuration
adds an off-chain name, identity-verification sources and optional expected chain
and program. No PreStocks logic lives in the source, decoder or snapshot model.
RPC endpoints/credentials are provided at invocation, not hard-coded. Only the
provider origin is retained, without userinfo, paths or query parameters.

## Architecture

`LifecycleStateSource::capture(AssetDescriptor)` is independent of
`ChangeScenario`. `SolanaTokenAssetSource<R: SolanaRpc>` implements it;
`HttpSolanaRpc` uses standard HTTP JSON-RPC. The injectable transport enables
deterministic tests without production RPC access.

The acquisition transcript is:

1. `getGenesisHash`: identify the network and enforce configured expectations.
2. `getAccountInfo` with full base64 data: validate initialized mint, discover
   runtime token program and decimals.
3. Contextual `getProgramAccounts` against that token program, filtering the
   mint at offset zero. There is deliberately **no fixed dataSize filter**;
   Token-2022 token accounts can have different lengths. No balance filter
   discards zero accounts. Unexpected, unsupported or malformed accounts fail.
4. `getMultipleAccounts` in sorted batches of at most 100: inspect every distinct
   SPL owner authority. Absent owner accounts are retained as null evidence.
5. A final `getAccountInfo` for the mint: retain any configuration drift and
   use this read for the normalized mint. A program/decimals change fails.

Every state read uses finalized commitment. After enumeration, authority/mint
reads carry minContextSlot equal to the enumeration context. All actual slots,
requests and complete raw result values are stored. Public RPC requests are paced
and rate-limit/server errors have bounded backoff. Unsupported enumeration is an
explicit error, never a fallback to largest accounts, fake holders or an indexer.

## Model and provenance

`LifecycleSnapshot` contains schema version, asset identity, capture timestamp,
enumeration slot, source/context range, mint configuration, sorted entities,
summary and RPC evidence. `LifecycleEntity` represents one token account, not one
person; owner authorities may repeat. It includes all normalized account fields,
extensions, authority observation and classification reason.

Each evidence reference has a transcript ID, JSON Pointer into the raw result
and observation slot. For example, `/value/12/account` points to the actual
base64 token account; `/value/7` in an authority query can point to an account
or null. Runtime Token program ownership is distinct from the SPL owner
authority stored in account data.

The original result ordering is preserved as raw evidence, while normalized
entities are sorted by token-account address and extensions by numeric type.
Struct field order and BTreeMap summaries make JSON serialization deterministic
for a fixed capture. A new capture is naturally different as state/time changes.
Base balances, allowances and supply use decimal strings, avoiding JSON consumer
u64 precision loss. Official extension config serialization can contain integer
JSON numbers; consumers must use a lossless parser. The raw bytes are authoritative.

Loading replays and checks the full transcript offline: query shape, commitment,
context bounds, account lengths, program IDs, mint membership, complete authority
coverage, duplicate accounts and all derived fields. It also rejects unknown
schema versions. This detects disagreement with the stored evidence; it is not a
cryptographic proof that an RPC provider or someone editing the entire file is
honest. Capture timestamps and off-chain identity sources are descriptive metadata.

## Decoder correctness

Base states use the official SPL Pack layouts; Token-2022 states use official
StateWithExtensions / StateWithExtensionsMut and TLV extension accessors. Each
known extension is decoded, including markers, variable-length TokenMetadata,
group extensions and confidential state. Unknown type IDs, incorrect base type,
duplicate extensions, incorrect payload lengths and nonzero trailing data fail
explicitly. Uninitialized token accounts are retained where their official layout
is valid; uninitialized mints cannot be captured as supported live assets.

`raw_balance` is the public account amount. `ui_balance` is its exact decimal
base-unit representation, with a stated basis. ScaledUiAmount and
InterestBearingConfig are reported but no timestamp-dependent display transform
is applied. Supply is likewise decimal base units. This prevents a display
multiplier from being mistaken for economic simulation or exact public supply.

## Conservative classification

The mutually exclusive classifications are:

- WalletCompatible: on-curve authority, existing non-executable System account,
  empty data. This does not prove a human holder, key ownership or current signer.
- ProgramOwnedAuthority: authority account has a non-System runtime owner.
  It does not prove which program signs for a PDA or identify its protocol role.
- TokenMultisig: valid initialized SPL multisig authority, with signer threshold
  and signer addresses retained.
- Unknown: absent authority account, executable authority, off-curve System
  address, or other unproven special state.

Account-level delegate presence, active positive allowance, frozen,
uninitialized and zero-balance counts are independent and overlap these types.
PermanentDelegate is mint-wide authority configuration, not evidence of an
account-level allowance. No category claims DirectHolder, LiquidityPosition or
program signing control without proof. All live zero-balance accounts are retained.

## Limits

Standard RPC is trusted for enumeration and context; it supplies no account
inclusion/completeness proof. Providers can restrict getProgramAccounts. Use a
provider that supports the standard method if needed; no indexer is required by
the implementation. Closed/deleted token accounts are not current state.

Multiple finalized RPC contexts are an observation interval, **not an atomic
single-slot snapshot**. minContextSlot is a lower bound, not a historical read
selector. The snapshot explicitly labels consistency and retains initial/final
mint drift. Supply mismatches are reported without inventing missing holders;
withheld fees, confidential balances and context drift can contribute.

Confidential configurations, approvals, counters, public keys and ciphertext are
retained. No encrypted balances are decrypted, and visible amounts cannot prove
full beneficial holdings or confidential circulating supply. Neither public RPC
nor curve membership proves human identity, custody, PDA signing program, DEX
pool role, transferability, redemption rights or lifecycle semantics.

No historical reconstruction is attempted. RPC context slots record when data
was observed, not each account's last-write slot. Snapshot replay restores the
captured observations without pretending to access past chain state.

## Smallest Phase 3 step

For this same mint, add one explicitly scoped, read-only protocol adapter that
verifies a candidate liquidity pool from its on-chain program owner, pool
layout, mint pair and vault authority. Link proven vault token accounts back to
snapshot entities, retain raw account evidence, and mark everything else Unknown.
Do not add routing, LP reconstruction, exitability probes or lifecycle simulation
in that first step. Phase 3 is not implemented here.
