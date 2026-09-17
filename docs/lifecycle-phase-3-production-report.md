# Phase 3 production exposure report

## Verified integration

**Meteora DLMM**, `LbPair` state version **1**, pair type
**PermissionlessV2** (enum 3). This is not DAMM or DAMM v2.

Pool: `v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc`.
Runtime owning program: `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`.
Pool/vault authority: `v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc`.

The external candidate locator is recorded in
`assets/prestocks-spacex-dlmm.json`; it supplies only an address. It is not part
of verification. The pool was selected because it is a live candidate whose
SPACEX reserve and raw pool-authority bytes are already in the Phase 2 capture,
allowing a complete, minimal historical-evidence link without historical RPC.
This does not claim it is the largest/primary market or discover all venues.

Evidence was verified at finalized **slot 447877573**; capture completed
**2026-09-17T19:44:01Z**. Initial candidate and final verification reads happened
to share that slot. One final `getMultipleAccounts` contains the pool, both vaults,
both mints and the executable protocol program account.

Verification requires:

1. Exact DLMM runtime owner, 904-byte pool layout and published LbPair discriminator
   `[33,11,49,98,181,101,177,13]`; supported version/type and valid relevant fields.
2. Exact lifecycle mint membership in decoded pool mint fields; distinct paired
   mints and vaults.
3. Pool PDA from `[base_key, sorted mint 1, sorted mint 2]` under the DLMM program.
   Captured base key: `EULoJHLdTDkhEWmFN8XCiTHyXqu4GQPXcU5HbwnLL8a7`.
   Derived pool and stored bump **254** both match.
4. Canonical reserve PDAs from `[pool address, mint]` under that program, equal
   to decoded reserve fields and the queried token-account addresses.
5. Both mint runtime token programs match pool token-program flags **[1,0]**.
   Vaults decode with the matching SPL/Token-2022 layout, carry the correct mint,
   are initialized, and have the pool PDA as their SPL owner authority.
6. The protocol program account is executable under a supported Solana loader.
7. The saved Phase 2 SPACEX vault and its saved pool-authority account bytes prove
   the same pool/mint/reserve/program role. The old and current bytes are retained.

Decoder rules are pinned to official SDK revision
`576919e3e4368e542c402f000b4264724f7f23ec`:
[account IDL](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/idls/dlmm.json),
[PDA helpers](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/ts-client/src/dlmm/helpers/derive.ts),
[token-program mapping](https://github.com/MeteoraAg/dlmm-sdk/blob/576919e3e4368e542c402f000b4264724f7f23ec/commons/src/extensions/lb_pair.rs).

These prove the relationship from supplied on-chain account evidence, without
website labels. PDA derivation is cryptographic address verification; RPC account
observations are still trusted provider data, not cryptographic inclusion proofs.

## Exposure found

| Asset | Mint | Canonical vault | Public raw balance | Decimal base units |
| --- | --- | --- | ---: | ---: |
| SPACEX PreStocks | `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh` | `HgQRhiATjX9jTWh7QgLWnhCeR7PaBqoWVf4vSaL61YVv` | 682216229 | 0.682216229 |
| Paired mint (USDC) | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` | `82kPQkc7B8nZ23r1YNqNB3uA3cgYzwxP5partMjwdT6h` | 7116488 | 7.116488 |

SPACEX uses Token-2022, 9 decimals. The paired mint uses legacy SPL Token,
6 decimals. The paired address matches [Circle's official Solana USDC address](https://developers.circle.com/stablecoins/usdc-contract-addresses);
that off-chain label is separate from the on-chain mint/vault relationship proof.

The SPACEX vault has TransferFeeAmount (withheld amount **194058 raw units**),
ImmutableOwner, TransferHookAccount (transferring=false) and PausableAccount.
The USDC vault has no Token-2022 extensions. Withheld fees are retained separately
from the public base balance, not represented as withdrawable LP liquidity.

All ten SPACEX mint extensions remain decoded in the current mint evidence,
including permanent delegate, confidential configuration, fees, scaling, pause
configuration and metadata. Public account balances can be read normally for both
vaults. The SPACEX vault has no ConfidentialTransferAccount extension; no encrypted
balances are recovered. Scaled UI display transformations are not applied, so the
SPACEX amount above is explicitly **decimal base units**, not a scaled display
amount or a lifecycle conversion quantity.

This is vault custody exposure, not effective trade depth, withdrawable liquidity,
LP ownership, a price/value estimate or a proof of exitability.

## Phase 2 refinement

Exact linked entity:

`solana-token-account:HgQRhiATjX9jTWh7QgLWnhCeR7PaBqoWVf4vSaL61YVv`.

Original classification remains **ProgramOwnedAuthority**. An attached refinement
adds **LiquidityVault**, verified program-controlled under the DLMM pool PDA.
Its evidence includes the old vault/pool bytes and current pool/vault/mint proof.
Program control is not exclusive: mint-wide permanent delegate and other recorded
Token-2022 authorities still exist.

Phase 2 balance: **678320992** raw units. Current vault balance: **682216229**.
Both are retained with their distinct observation slots; no holder amount is
silently rewritten.

| Graph observation | Count |
| --- | ---: |
| Retained Phase 2 token accounts | 17957 |
| Retained wallet-compatible classifications | 12379 |
| Verified program-controlled lifecycle vaults | 1 |
| Verified protocol pools | 1 |
| Target vaults linked to parent entities | 1 |
| Phase 2 entities refined | 1 |
| Previously Unknown entities refined | **0** |
| Unknown classifications remaining | 5413 |
| Other program-owned authorities without a verified integration | 164 |
| Unresolved entities (5413 + 164) | 5577 |
| Retained active delegations | 12 |
| Retained frozen accounts | 0 |
| Evidence-backed graph edges | 17977 |

The original Unknown accounts are not force-classified. Original entity types,
balances, evidence and Phase 2 capture metadata are unchanged. The graph represents
all existing holding edges and active delegates, plus the proven pool structures.
The current pool context does not refresh the whole older holder distribution.

## Architecture and snapshot

`ExposureAdapter` separates protocol identity, account planning and proof from
`LifecycleStateSource`. Only `MeteoraDlmmAdapter` is implemented. Generic discovery
calls standard RPC; graph normalization does not contain DLMM layout/seed rules.

`LifecycleSnapshot.exposures` holds `ExposureGraph`, protocol exposures, one record
per parent account, explicit refinements, sorted adjacency edges, adapter-run
transcripts and summary. Each edge carries addresses, runtime owner, actual slot,
raw-result JSON Pointer, SHA-256 of account bytes and adapter/decoder identity.
No graph database or generic indexing infrastructure was introduced.

The enriched snapshot uses explicit **schema 2**; the graph has schema 1. Previous
schema-1 snapshots default to no graph and serialize byte-stably. Offline load
replays Phase 2, verifies the retained adapter transcript and regenerates every
hash, link, refinement, edge and count. Version/field mismatches or unsupported
adapter revisions fail. Existing graphs are never silently replaced.

[Exposure snapshot](../snapshots/spacex-exposure.json): **73665608 bytes**
(approximately 70.3 MiB), including all original Phase 2 evidence.
[Checksum sidecar](../snapshots/spacex-exposure.sha256):
`6802afb871035a0883a04196c7c9542d021d97514f43748b8f63747ea3ce016e`.

The parent `snapshots/spacex.json` remains unchanged, SHA-256
`70d5c41cfc5deef048b5e13fcc4febe1b572d8f8c8abbacc49f767d9a5c0391f`.
The graph stores that same parent canonical JSON hash.

```sh
cargo run --locked -p eplyx-lifecycle-impact -- exposure \
  --snapshot snapshots/spacex.json --adapter meteora-dlmm \
  --pool v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc \
  --rpc https://api.mainnet-beta.solana.com \
  --out snapshots/spacex-exposure-new.json

cargo run --locked -p eplyx-lifecycle-impact -- snapshot-show \
  --input snapshots/spacex-exposure.json
```

[Detailed architecture and workflow](lifecycle-phase-3-exposure.md).

## Files changed in Phase 3

Modified:

- `Cargo.lock`: direct SHA-256 dependency locked; existing discovery dependencies retained.
- `engine/Cargo.toml`: direct `sha2` dependency.
- `engine/src/lifecycle/mod.rs`: optional exposure field, explicit schema-1/2 validation,
  schema-1 serialization compatibility and graph replay on load.
- `engine/src/main.rs`: exposure capture CLI, adapter selection and graph summary on offline show.
- `README.md`: current Phase 3 scope and guide/report links.

New:

- `engine/src/lifecycle/exposure/mod.rs`: generic adapter boundary, capture orchestration,
  evidence, graph/links/refinements, deterministic replay and summaries.
- `engine/src/lifecycle/exposure/meteora_dlmm.rs`: the single bounded protocol adapter.
- `engine/tests/lifecycle_exposure.rs`: eleven focused network-free tests.
- `fixtures/meteora-dlmm-layout.json`: reduced official pinned account/type IDL fixture.
- `fixtures/meteora-dlmm-spacex.json`: deterministic minimized test transcript from real
  captured account bytes, plus the actual three-call Phase 3 transcript. Its reduced
  Phase 2 query lists are explicitly test reconstruction, not an original RPC dump.
- `assets/prestocks-spacex-dlmm.json`: untrusted candidate address and external locator provenance.
- `snapshots/spacex-exposure.json`: actual complete enriched production snapshot.
- `snapshots/spacex-exposure.sha256`: snapshot integrity sidecar.
- `docs/lifecycle-phase-3-exposure.md`: design, CLI, proof rules, versioning and limits.
- `docs/lifecycle-phase-3-production-report.md`: this report.

No additional edits were made to Phase 1/2 tests. Pre-existing uncommitted Phase 2
files remain part of the workspace; the list above describes this phase's delta.
Generated synthetic fixture states and SBF artifacts retain their ignored status.

## Tests

All required checks passed:

- `make test`: **135 passed, 0 failed, 0 ignored**. 7 fixture-program v1,
  7 fixture-program v2, 47 engine unit, 13 Phase 2 discovery, 11 Phase 3 exposure,
  2 Phase 1 scenario, 42 ProgramUpgrade integration and 6 interface tests.
  Both actual SBF program versions built successfully; execution tests were not skipped.
- `make fmt-check`: passed for workspace and fixture program.
- `make lint`: passed with `-D warnings` for all engine targets and both fixture versions.
- Actual schema-2 production `snapshot-show`: passed offline in the network-restricted sandbox.
- Both production SHA-256 sidecars: passed, including unchanged Phase 2 checksum.
- `git diff --check`: passed.

| Required coverage | Tests |
| --- | --- |
| Valid known state → exposure | genuine_dlmm_state_normalizes_pool_both_assets_authority_and_balances; decoder_offsets_and_discriminator_match_pinned_official_zero_copy_idl |
| Candidate lacks target | candidate_without_target_mint_is_rejected |
| Fake/malformed relationship never classified | malformed_pool_wrong_owner_discriminator_pda_reserve_or_version_never_classifies; fake_vault_mint_authority_program_extension_and_missing_accounts_are_rejected |
| Vault links to Phase 2 | vault_link_retains_phase2_identity_classification_balance_and_historical_pool_proof; absent_phase2_vault_and_unproven_phase2_pool_role_cannot_be_refined |
| Deterministic serialization | schema1_compatibility_and_schema2_serialization_roundtrip_are_deterministic |
| Identical offline graph replay | offline_replay_rebuilds_the_identical_graph_without_rpc |
| Existing Phase 1/2 behavior | Unchanged scenario/discovery/upgrade suites passed in make test |

Additional tests cover query shape/single contextual batch, missing/cross-chain/old
context evidence, unsupported adapter version and tampered proofs. No core test
requires network access. Test reconstruction is clearly marked, never used as
production fallback data.

## Investigation answers and limitations

1. **Product/program:** DLMM, PermissionlessV2 pool, owning program
   `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`. Other Meteora products are unsupported.
2. **Vault linkage:** decoded reserve fields + canonical `[pool,mint]` PDAs + vault
   mint/program fields + SPL authority equal to the cryptographically derived pool PDA.
3. **Token-2022 complication:** account lengths/extensions differ from legacy tokens;
   flag 1 must select Token-2022. Shared official extension decoding preserves fees,
   pause marker/hook state, mint delegate/freeze/configuration and UI-scaling context.
4. **Balance visibility:** both vault public balances are normally readable. Separate
   withheld fees are preserved. Confidential balances cannot be decrypted; no
   confidential-account extension was detected on this particular SPACEX vault.
5. **Entire on-chain relationship:** all pool/authority/vault/mint assertions are
   verified from on-chain bytes and PDA math, with the published protocol program
   identity/layout as the adapter's explicit trust boundary. External labels are unused.
6. **Unknown refinement:** exactly zero previously Unknown entities; one existing
   ProgramOwnedAuthority becomes a verified LiquidityVault refinement.
7. **Proof limits:** standard RPC supplies no signed inclusion or completeness proof.
   Checksums bind retained bytes, not ledger truth. Current protocol evidence shares one
   finalized slot; older Phase 2 holder observations do not become contemporaneous.
8. **Coverage limits:** exactly one candidate, not all venues/integrations. Other
   program-owned authorities/Unknown entities stay unresolved. Human identity,
   beneficial ownership, LP positions, reserve economic components, effective depth,
   transfer feasibility, exitability, prices and lifecycle effects are not inferred.
9. **Protocol implementation limit:** DLMM on-chain source is not open sourced;
   [Meteora's developer guide](https://github.com/MeteoraAg/docs/blob/main/developer-guides/dlmm/index.mdx)
   directs integration to the published IDL/SDK. This adapter does not attest deployed
   source/bytecode equivalence or audit execution. It supports exactly the checked
   904-byte state-version-1 PermissionlessV2 layout, failing explicitly on other types.
10. **Historical limit:** old pool/vault role proof uses bytes already in Phase 2.
    No historical RPC queries, reconstruction or account last-write slots are added.
    If old role/identity cannot be proven, the candidate cannot refine that snapshot.

## Phase 4 recommendation

Introduce one proposed, snapshot-only **MintPause LifecycleChange** for this
SPACEX asset, driven by its actually present Pausable configuration
(`paused=false` → proposed `true`). Give the change an explicit authority/config
precondition and a versioned consequence model. First produce the proposed mint
configuration diff and evidence-linked list of affected holdings/delegates/the
verified pool vault. Use fixtures to define expected state and consequence behavior
before widening semantics to issuer lifecycle events. No live transactions,
conversion/deadline rules, swaps, withdrawals, valuation or exitability are needed
for that first model. This is a next-phase proposal; Phase 4 has not begun.
