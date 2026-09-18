# Phase 9 official transition mechanism evidence

The official-transition investigation consumes frozen public research, the
original population/policy and a digest-bound Phase 8 resolution. It deepens
only OfficialTransition. No holder, venue or transfer coverage is expanded.

## Reusable model

`OfficialTransitionMechanism` records source/destination assets, mechanism type,
candidate programs and observed account/signature requirements, eligibility
inputs, on-chain references, external policy references and blocking gaps.
`MechanismIdentity` separates issuer-bound semantics, observed official use,
verified account plan and mere source/successor co-occurrence.

Mechanism types are BurnMint, TransferClaim, Swap, Redemption, BackendMediated
and Unknown. A sampled swap can be a Swap candidate while official identity
remains unestablished. Candidate account/signature lists describe the observed
transaction; they are not a replacement plan for the canonical holder.

`TransitionScope` keeps entity, mechanism, context, input and both assets
separate. `TransitionAssessment` preserves exact status, current independent
execution support, execution-attempt flag and nullable existence evidence.
Unestablished official identity produces NotTested and existence `null`.
An identified mechanism with evidenced private/issuer dependencies produces
Unsupported. Public state incompleteness produces Indeterminate. Unknown
eligibility remains NotTested; it is not inferred KYC or backend dependence.

`VerifiedTransitionExecution` is opaque and has no public/deserialization
constructor. The production investigation supplies no execution receipt and no
official VM adapter. Discovery JSON cannot inject success or select Proven.
The generic execution assessment requires exact entity/mechanism/context/amount/
asset/path matching, no fabricated issuer authority, successful execution and
exact source/successor/documented-term reconciliation. Failed requires failed
execution and verified rollback. These gates have controlled unit tests; their
test receipts are not real official transactions or production evidence.

Holder signing may be explicitly assumed locally. Non-holder signing and
private KYC, backend signatures or entitlements cannot be assumed. Successor
mint metadata, a burn, MintTo, TransferChecked or DEX output cannot independently
establish issuer authorization or completion semantics.

## Offline research adapter

`transition::research` checks all artifact digests and schema/input identity.
It parses only the recorded public RPC methods, counts requested/returned
signature entries, unique transaction selections and retries, and enforces the
manifest's finite bounds. Transport/RPC errors remain explicit. Recovered
transactions retain their original successful record reference; failed first
requests are not erased or counted as additional transaction observations.

The latest captured bytes per address determine mint/program observations.
Mint verification uses the existing official SPL/Token-2022 decoder, retaining
all extensions and precise supply. On-chain metadata is separate from external
metadata documents and issuer webpage assertions. Creation slot remains null
unless independently established; oldest returned signature is not creation.

Transaction observations retain all parsed account keys, signer flags, outer/
inner instructions, programs, logs, errors and exact token deltas. Account
classifications distinguish observed signer, captured public executable,
verified PDA and unknown. Observed signer does not mean Eplyx has that key.
Unknown accounts are not relabeled issuer-controlled or holder-controlled.

Upgradeable program observations check captured executable/ProgramData loader
links, deployment slot, upgrade authority and allocated ELF input digest. Native
or legacy loader accounts retain their actual available evidence without a
fabricated ProgramData link. Squads multisig/vault and supported observed DLMM
pool/reserve PDAs are derived and checked. Those administrative/market protocol
facts do not confer official conversion identity. Published layouts are not a
source-to-bytecode equivalence claim.

Current account/program records and historical transaction samples are distinct
banks. They are not coherent transaction pre-state or an official execution
fixture. No transaction is reconstructed for execution unless mechanism identity,
semantics, account plan and permitted credentials are actually established.

The adapter validates the original Phase 8 resolution through fresh replay,
builds a new discovery manifest and resolves its OfficialTransition row through
the existing generic `LifecyclePathResolver`. The other four rows remain exactly
equal to Phase 8. Population, policy time, signer assumptions and baseline
execution field remain unchanged. Historical artifacts are never overwritten.

## CLI and determinism

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- investigate-transition \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --entity 741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs \
  --research probes/spacex-official-transition-research.json
```

Text is default; `--format json` emits complete canonical JSON.
`--out <new-path>` saves the investigation, `--out-resolution <new-path>` saves
the five-path matrix, and `--out-discovery <new-path>` saves its discovery input.
All are canonical JSON regardless of display format. Existing/identical output
paths are rejected before replay; writes use the existing create-new protection.
Absolute inputs work from another directory and artifact references resolve
relative to the research manifest. No RPC flag or generation timestamp exists.
Saved JSON and JSON stdout are identical. Frozen retrieval times/slots identify
the captured observations, not the time of replay.

The [production report](lifecycle-phase-9-production-report.md) records the exact
six-address/31-transaction boundary and NotTested outcome. Reproduce the seven
actual fault injections with `python3 scripts/test-phase9-mutations.py` on an
isolated writable checkout with fresh output paths. The runner refuses to
overwrite logs/results, requires a named assertion failure and restores exact
core bytes. Compiler failures do not count as caught mutations.
