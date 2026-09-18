# Phase 7 coverage expansion architecture

Phase 7 extends the frozen Phase 4 population and Phase 6 assurance contract.
It keeps population, current coverage, gaps, candidates, selection, execution
and updated coverage separate. The selector has no execution-result input.

The evidence unit is `(entity, fixture/source-state hash, exact input, path,
venue/context, captured Clock/programs, authority assumption, measured result)`.
`Proven` remains the Phase 6 derived classification: matching source balance
and a successful full exact-input execution on at least one requested path.
Other requested paths remain independent. Transfer proves movement to a real
token account; its amount coverage is not sale proceeds or exit liquidity.

## Selection contract

The serializable selector configuration is `lifecycle-gap-v1`. For eligible
candidates, score components are:

```text
100 × new positive-amount entity
+ floor(20 × min(marginal represented raw, 1,000,000,000) / 1,000,000,000)
+ 40 × new account class
+ 60 × new secondary-market venue
+ 80 × new path type
+ 30 × new entity/path/context tuple
+ 25 × new state shape
+  5 × executable-or-capture-required feasibility
- 1,000 × redundant context with zero marginal represented raw
```

These are development priorities, not probabilities, prices or safety scores.
The raw cap reduces concentration in large balances. Expected global amount
uses each entity's maximum across independent paths/venues; repeated context
requests can add context evidence with zero predicted global amount gain.
Scores are recomputed against virtual predicted coverage after each selection.
Expected gains remain hypothetical even if a selected execution later fails.

Positive wallet-compatible entities are ranked by `(raw balance, entity ID)`;
rank quartile is `floor(4 × rank / population size)`. The production configuration
first selects distinct sources from quartiles 0, 1 and 3, then fills the budget.
It permits six groups, three distinct sources, at most two groups per source,
and requires at least two path types when feasible. Within those explicit
constraints, candidates sort by decreasing score, decreasing marginal raw,
then increasing entity ID, context ID and path name. Missing buckets fail
selection rather than inventing accounts. Budget is capped at ten groups.

State shape fingerprints include authority classification, initialization,
freeze/delegation, sorted extension flags, observed authority runtime ownership,
executable/on-curve state, verified role and balance quartile. A sample never
proves its class peers. Authority counts and token-account counts are separate.

Eligibility distinguishes `ExecutableCandidate`, `CaptureRequired`,
`Unsupported` and `Invalid`. Unsupported/invalid candidates have no score.
Zero balances are excluded from positive-input candidates. Unsupported venue
contexts are recorded once, avoiding an unnecessary population cross product.
High-value unsupported class/path gaps remain capability-development guidance.

## Discovery, capture and execution

Discovery uses bounded finalized standard Solana RPC queries for the target
mint in either DLMM mint position. Raw successful responses and failures are
retained. Concrete candidate pools must pass the existing exposure adapter's
PDA, mint, vault, owner and reserve checks. Only one deterministically ordered
additional compatible venue is verified in this bounded run. A discovery
listing is not execution evidence; compatible reserves do not predict outputs.

The original Phase 3 graph supports one adapter run. Phase 7 retains that
contract: each execution world contains the unchanged original population and
one verified venue proof. An additional venue gets its own ephemeral world;
the original snapshot is never rewritten. The inventory hash binds its raw
verification evidence to selection and replay.

The plan is saved before route capture. Each selected group gets one coherent
finalized route-account batch, captured executable headers/ProgramData/ELF,
mint extensions, original authority, real source/destination and Clock. The
manifest binds group, fixture digest and plan digest. Failure records a gap;
selection and amount matrices are never repaired using execution knowledge.

Each positive matrix contains integer-floored 1%, 25%, 50% and full observed
balance, minimum one raw, deduplicated. Balance-plus-one is an invalid control
when arithmetic permits. If capture balance changes, the planned matrix stays
fixed; the control is not executed as a claimed insufficient-balance control
at a different balance. Zero balances have no invented positive case.

Every transaction uses a fresh LiteSVM through `execute_probe_message`, deployed
bytecode and captured Clock. Secondary-market execution uses real DLMM `swap2`.
Transfer uses the official Token-2022 `TransferChecked` instruction, a distinct
real observed recipient, and the retained original owner locally assumed to
sign. No private key or signature authorization is established. Its adapter
rejects incompatible owners/mints, freeze, active pause/hook, confidential
account support and required incoming memo support outside this slice.
Permanent delegate and ordinary delegation are preserved but unused.

Successful transfers reconcile source debit with destination public credit
plus destination withheld transfer fee, check the official captured-epoch fee,
unchanged source withheld fee and unchanged mint bytes. Successful swaps retain
user/vault/bin/event/LP/protocol reconciliation. Failed executed transactions
must roll back every watched token/protocol account; native fee-payer transaction
fees are separate. Preconditions do not masquerade as executed failures.

Current captures are `CurrentFinalizedProduction`, not a historical Phase 4 or
Phase 5 bank. Policy time stays independent. Matching source public balance
permits the existing conditional population-amount join; mismatches have zero
earlier-population amount credit while retaining exact current-state execution.
Neither matching quantity nor hashes are signed inclusion proofs.

## Delta and storage

Results are stored once as `results/<SHA-256>.json`. An execution index carries
digest-bound references, statuses and fixture hashes. Public coverage update
checks input fingerprints, regenerates the immutable plan, checks fixtures and
results, and fresh-replays every result before merging. A claimed success must
have real successful execution and exact reconciled debit.

The schema-2 delta references the unchanged complete Phase 6 report, applies an
explicit Transfer capability override (`Unsupported` to `Untested` for wallet
accounts), and contains changed entity rows, before/after aggregates, expected
and realized group gains, evidence references and recomputed remaining gaps.
Capability adds no positive amount. Baseline + override + changed rows provides
the complete updated view without duplicating the 61 MB baseline or VM reports.

Amount aggregation is a per-path maximum and then a per-entity maximum, never
a sum across independent probes. Only exact successful points are measured;
the numeric maximum does not assert continuous interval feasibility. Venue
counts include directly successful exact pool contexts only. Every group resets
state; these results cannot be summed as simultaneous venue capacity or proceeds.
`OfficialTransition` remains `NotTested`; unsupported does not mean impossible.

## CLI and replay

Use the checked-in inventory for deterministic offline selection. Only
`venue-discover` and `capture-plan` require RPC. All output paths must be new.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- venue-discover \
  --snapshot snapshots/spacex-exposure.json \
  --coverage reports/spacex-lifecycle-coverage.json \
  --rpc https://api.mainnet-beta.solana.com --out /tmp/venues.json

# Supply these common arguments to each command below:
# --snapshot snapshots/spacex-exposure.json
# --impact reports/spacex-transition-impact.json
# --coverage reports/spacex-lifecycle-coverage.json
# --baseline-plan probes/spacex-lifecycle-coverage-plan.json
# --inventory probes/spacex-phase7-venues-verified.json

# coverage-expand --selector-config probes/spacex-phase7-selector-config.json
#                 --out-plan <new-plan.json>
# capture-plan --plan <plan.json> --rpc <RPC> --out-dir <new-capture-dir>
# execute-plan --plan <plan.json> --captures <capture-manifest.json>
#              --out-dir <new-result-dir>
# coverage-update --plan <plan.json> --captures <capture-manifest.json>
#                 --results <execution-index.json> --out <new-delta.json>
```

Canonical JSON is pretty printed with one final newline, no generation clock.
Saved JSON and stdout are byte-identical. Strict model parsing rejects unknown
fields; changed selection, fingerprints, fixture/result bytes and context
bindings fail closed. Content digests are checked before execution/merge.

See the [production report](lifecycle-phase-7-production-report.md) for actual
outcomes, exact deltas, performance, storage costs and remaining gaps. Run
`python3 scripts/test-phase7-mutations.py` on an isolated writable checkout to
inject the ten named faults and require assertion failures; it restores exact
source bytes and refuses to overwrite its saved mutation report.
