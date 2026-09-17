# Phase 2 production capture report

## 1. Asset used

SpaceX PreStocks (SPACEX), mint
`PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh`.

It was selected because the [official PreStocks SpaceX product page](https://prestocks.com/spacex)
links directly to that exact Solscan mint, and standard mainnet RPC successfully
returned the live mint and its full token-account enumeration. The mint's
on-chain TokenMetadata independently contains `SpaceX PreStocks`, `SPACEX` and
`https://prestocks.com/metadata/spacex.json`, with the matching mint address.
Mainnet identity was checked against live `getGenesisHash`:
`5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d`.
No alternate asset, synthetic holder dataset or indexer was substituted.
Off-chain identity provenance is configuration in `assets/prestocks-spacex.json`.

## 2. Production-state findings

Capture completed **2026-09-17T18:41:34Z**. Enumeration context:
**447865621**. Actual finalized read contexts span **447865619–447865805**.
This is a frozen multi-context observation set, not an atomic single-slot snapshot.

Token program: **Token-2022**,
`TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`.
Decimals: **9**. Mint supply: **8742515833967 raw units**, or
**8742.515833967 decimal base units**. UI scaling is recorded separately.

Mint and freeze authority both equal
`WV9PJN7XTmTLVwbutCLFxp8TyePee6Xq5mRq6Fti5Wc`.
The permanent delegate and the extension update/configuration authorities in
this capture also point to this address; its identity/control is not inferred.

| Mint extension detected | Captured configuration |
| --- | --- |
| TransferFeeConfig | Older: epoch 848, 0 bps, max fee 0. Newer: epoch 1032, 50 bps, max fee u64::MAX. Mint withheld amount 25844. Both fee authorities preserved. |
| ConfidentialTransferMint | autoApproveNewAccounts=false; auditor key absent; authority preserved. |
| DefaultAccountState | Initialized (raw state 1). |
| PermanentDelegate | Delegate address preserved above. This is mint-wide, independent of per-account delegate counts. |
| TransferHook | Extension present; programId=null. No configured hook program is inferred. Update authority preserved. |
| ConfidentialTransferFeeConfig | harvestToMintEnabled=true; authority, encryption public key and withheld ciphertext preserved. |
| MetadataPointer | Points to the mint itself; authority preserved. |
| TokenMetadata | Matching mint; SpaceX PreStocks / SPACEX / issuer URI; empty additional metadata; update authority preserved. |
| ScaledUiAmount | multiplier=1.0; newMultiplier=5.0; newMultiplierEffectiveTimestamp=1781065800; authority preserved. No lifecycle meaning or display conversion implemented. |
| Pausable | paused=false; authority preserved. |

No additional mint extension was present. Extension presence is decoded from
raw TLV, not assumed from the asset name.

| Account/entity observation | Count |
| --- | ---: |
| Token accounts / entities | 17957 |
| Distinct SPL owner authorities | 17950 |
| WalletCompatible | 12379 |
| ProgramOwnedAuthority | 165 |
| TokenMultisig | 0 |
| Unknown | 5413 |
| Active delegate / delegate present | 12 / 12 |
| Frozen | 0 |
| Uninitialized | 0 |
| Zero public balance, retained | 7802 |

The four entity types partition accounts. Delegated, frozen and zero-balance
properties overlap entity types. Unknown comprises 5221 accounts whose authority
account is absent, 191 special System-owned authorities, and one executable
authority. These are account counts, not counts of humans or unique beneficial
holders. ProgramOwnedAuthority proves runtime ownership of the authority account,
not the signing program of a PDA or a liquidity role.

Account extensions: TransferFeeAmount, TransferHookAccount and PausableAccount
on all 17957 accounts; ImmutableOwner on 17924 accounts. No
ConfidentialTransferAccount was present in this enumeration. The mint's
confidential configuration/ciphertext remains recorded without decryption.

Summed public raw balances: **8741534482051**. The observed supply difference
**981351916** equals the observed public account withheld fees **981326072**
plus mint withheld fees **25844**, exactly. This arithmetic reconciliation is
not a cryptographic enumeration-completeness proof or a claim of atomic contexts.
The generic snapshot warning preserves the visible balance/supply difference.

## 3. Architecture

`LifecycleStateSource` abstracts observation separately from change execution.
`SolanaTokenAssetSource<R: SolanaRpc>` captures any supported SPL/Token-2022 mint;
PreStocks is asset configuration. `HttpSolanaRpc` calls standard RPC directly.

The source reads chain identity, the initial mint, every token account using a
mint memcmp with no fixed size/balance filter, every owner authority in batches,
and the final mint. Full base64 account data and contextual RPC results are
retained. Normalized mint/entities reference transcript IDs and JSON Pointers
with their actual slots. Extensions use official SPL interfaces and TLV parsers.

`LifecycleSnapshot::load` replays the stored raw transcript offline and checks
all derived facts, query shapes, context bounds and evidence coverage.
Serialization is deterministic for a fixed capture. No LifecycleChange execution
or prohibited Phase 3 integrations were added.

[Detailed design, workflow and limitations](lifecycle-phase-2-discovery.md).

## 4. Files changed

Modified:

- `Cargo.lock`: locked HTTP/time/SPL discovery dependencies.
- `README.md`: Phase 2 scope and guide.
- `engine/Cargo.toml`: discovery dependencies.
- `engine/src/lib.rs`: export lifecycle module and document Phase 2.
- `engine/src/main.rs`: snapshot capture and offline snapshot-show CLI.
- `engine/tests/scenario.rs`: rustfmt line wrapping only; assertions unchanged.
- `engine/tests/upgrade_diff.rs`: rustfmt line wrapping only; assertions unchanged.

New:

- `assets/prestocks-spacex.json`: verified asset identity / expected mainnet program.
- `engine/src/lifecycle/mod.rs`: source abstraction, model, provenance, capture,
  conservative classification, deterministic normalization, persistence/replay.
- `engine/src/lifecycle/decode.rs`: mint/account/extension decoding.
- `engine/src/lifecycle/rpc.rs`: injectable standard RPC transport and bounded retries.
- `engine/tests/lifecycle_snapshot.rs`: 13 offline decoder/source/snapshot tests.
- `fixtures/token-2022-spacex-mint.json`: small genuine mainnet mint fixture,
  extracted from this snapshot's final mint evidence.
- `snapshots/spacex.json`: complete production snapshot and raw evidence.
- `snapshots/spacex.sha256`: SHA-256 integrity sidecar.
- `docs/lifecycle-phase-2-discovery.md`: architecture, CLI and limits.
- `docs/lifecycle-phase-2-production-report.md`: this report.

Generated fixture states and SBF artifacts remain ignored, as before.

## 5. Snapshot

[Production JSON](../snapshots/spacex.json): **53006043 bytes**, approximately
50.6 MiB including the full raw RPC transcript. Capture used only
`https://api.mainnet-beta.solana.com`. No private endpoint or API key was needed.

SHA-256:
`70d5c41cfc5deef048b5e13fcc4febe1b572d8f8c8abbacc49f767d9a5c0391f`.

```sh
shasum -a 256 -c snapshots/spacex.sha256
cargo run --locked -p eplyx-lifecycle-impact -- snapshot-show \
  --input snapshots/spacex.json
```

Offline production replay completed successfully in the network-restricted
sandbox. The saved snapshot is never overwritten by the capture CLI.

## 6. Tests

All final checks passed:

- `make test`: **124 passed, 0 failed, 0 ignored**: 7 fixture-program v1,
  7 fixture-program v2, 47 engine unit, 13 discovery, 2 Phase 1 scenario,
  42 ProgramUpgrade integration and 6 interface tests. Both SBF artifacts built
  successfully and differed. No execution tests were skipped.
- `make fmt-check`: passed for workspace and fixture program.
- `make lint`: passed with `-D warnings` for all engine targets and both
  fixture-program versions.
- `snapshot-show --input snapshots/spacex.json`: production evidence replay
  passed offline in the network-restricted sandbox.
- `shasum -a 256 -c snapshots/spacex.sha256`: passed.
- `git diff --check`: passed.

The required coverage maps to:

| Required test | Test coverage |
| --- | --- |
| Legacy SPL mint/account | legacy_mint_and_account_decoding |
| Token-2022 mint/extensions | token2022_mint_extensions_are_decoded_not_assumed; real_spacex_mint_fixture_decodes_all_live_extensions_and_embedded_metadata |
| Token-account normalization | delegated_frozen_zero_and_account_extensions_survive_normalization; exact_decimal_amounts_do_not_use_floats_or_overflow |
| Delegated/frozen/special states | delegated_frozen_zero_and_account_extensions_survive_normalization; uninitialized_accounts_are_preserved_and_zero_allowance_is_not_active; entity_classification_uses_authority_evidence_and_does_not_guess_pda_programs; spl_multisig_authority_preserves_threshold_and_signers |
| Deterministic serialization / offline reload | snapshot_json_roundtrip_and_offline_evidence_replay_are_deterministic |
| Unsupported/malformed fail explicitly | malformed_unsupported_and_wrong_mint_fail_explicitly; duplicate_extensions_and_invalid_marker_lengths_are_rejected; offline_replay_rejects_tampering_incomplete_evidence_and_bad_context |

The remaining test verifies RPC URL credentials never enter the retained origin.
Local deterministic test fixtures are not production holders or a fallback data
source. Existing Phase 1 and ProgramUpgrade tests retain every assertion.

## 7. Limitations

- RPC enumeration and account bytes are trusted provider observations, with no
  cryptographic inclusion or completeness proof. Some providers disable standard
  getProgramAccounts; that fails explicitly instead of falling back to a partial
  largest-account list or synthetic/indexed state.
- Individual queries have finalized slots but do not share an atomic bank.
  minContextSlot is a lower bound. The snapshot retains the whole interval and
  initial/final mint observations. A frozen file preserves those observations;
  it cannot retroactively make them simultaneous.
- Curve checks and runtime ownership do not prove human identity, signing keys,
  PDA signing program, protocol role, beneficial holdings or liquidity exposure.
- Confidential keys/configuration/ciphertext can be preserved, but encrypted
  amounts cannot be decrypted from public RPC. Visible balances describe only
  public account amounts. No current confidential-account extension was detected.
- Decimal UI fields use exact base units, not scaled/interest-bearing display
  transformations. The complete configurations are available to later phases.
- No historical reconstruction, last-write slots or closed account history is
  provided. Replay is offline re-normalization, not a historical RPC query.
- Embedded metadata is decoded; external pointer targets and hook integrations
  are not crawled in Phase 2. Raw pointers are retained for later discovery.
- Integrity sidecar/offline validation detects accidental changes or disagreement
  with evidence; it does not establish the truth of a provider response or prevent
  someone replacing both evidence and normalized data consistently.

## 8. Phase 3 recommendation

Add one read-only adapter for a single candidate liquidity pool for the same
SPACEX mint. Prove its program owner, pool layout, paired mint and vault authority
from on-chain accounts, then link the verified vault token accounts to snapshot
entities with raw evidence. Leave unproven accounts Unknown. This is the smallest
useful integration-exposure step; routing, LP reconstruction, exitability and
lifecycle simulation remain separate later work. Phase 3 has not begun.
