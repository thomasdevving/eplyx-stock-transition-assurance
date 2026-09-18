# One issuer notice, offline lifecycle ingestion

Phase 12 adds exactly one adapter and event type: the captured official PreStocks
SpaceX page and `SuccessorTransition`. No HTTP/RPC, VM, discovery, polling, model
inference or notification runs in this workflow.

`LifecycleEventSource` produces generic `NormalizedLifecycleEvent` values from
`CapturedSourceDocument`. `prestocks::CapturedIssuerNotice` reads machine `href`
and `og:site_name` attributes and the exact rendered notice grammar. Captured
Next.js script data does not carry the required notice fields, so scripts are
excluded. Attribute ordering, quoting and irrelevant markup do not supply
semantics. Missing or ambiguous identities, notice, symbol or deadline fail.

Each semantic field carries a classification and exact raw UTF-8 byte range/text
or parsed-JSON pointer with its source digest. Metadata fields have a separate
`field_provenance` map. Unknown fields point to the bounded inspected notice;
Unknown means that this notice does not establish the fact. It is not a claim
that the fact cannot be established elsewhere. Original raw HTML bytes are
unchanged. The new source wrapper embeds them and retains its historical URL/time
references. HTTP status/MIME were not retained; both are null, explicitly Unknown.

## Trust and identity

`probes/spacex-notice-workflow.json` pins source, demo configuration, old identity
report and original assurance policy. All its nested artifact references resolve
against the workflow's directory (`probes/`), including the wrapper's raw and
historical references. The policy's own evidence manifest resolves against its
policy directory, as before. Absolute workflow paths therefore work from another
working directory with no network environment.

The adapter alone returns `IdentityStatus::Unknown`. The workflow separately
loads each Phase 9 raw account by verified artifact digest and pointer, checks the
RPC requested address/index and bank, re-decodes initialized mint bytes, and
compares runtime owner, raw hash and full mint configuration. Only this step adds
OnChainVerified. This verifies captured mint identity/configuration; it does not
prove legal issuer affiliation, economic entitlement or a conversion mechanism.
Raw RPC observations are not signed inclusion proofs.

Artifact references use file-byte SHA-256. `captured_document_sha256` and JSON
pointer provenance use canonical parsed-document SHA-256 (`serde_json` pretty
serialization plus newline). They can differ from the wrapper's original file
format. Raw-content hash always covers exact UTF-8 bytes. The event's semantic
hash excludes layout/digest metadata; it is used only to test extraction stability,
never as an integrity substitute.

## Generate and verify

`VerifiedLifecycleEvent` has no public/deserialization constructor. Normalized
JSON is freshly regenerated from pinned source and chain evidence and compared
in full before scenario generation. Scenario and `ScenarioBinding` are freshly
regenerated before preflight. Changing raw bytes, mint, date, pointer, taxonomy or
scenario leaves fails. A genuinely new source requires an explicit new captured
artifact and workflow digest; it cannot silently replace the historical source.

The generated `LifecycleScenario` uses the existing generic lifecycle schema.
After=TransitionRequired comes from the transition assertion. Source and successor
mints retain IssuerAsserted and OnChainVerified. The deadline literal is issuer
wording; parsing it and choosing the exclusive start of its stated minute is
Derived plus explicit DemoConfigured interpretation. “Expire worthless” maps to
NoIssuerEntitlement only as an external-policy demo interpretation, never zero
market value or technical non-exitability. Before=Active and the evaluation
boundary are DemoConfigured. The notice supplies no precise effective time,
conversion ratio, official program/plan or signing access.

## Frozen proof compatibility

The generated scenario differs in digest, ID, provenance and additive verified
successor identity from the old hand-authored scenario. The orchestration checks
exact source asset, effective boundary, before/after statuses and deadline/after
status against the original frozen proof scenario. It allows only a verified
successor identity addition with Unknown mechanism, absent ratio and NotTested
official execution. Different economic semantics fail closed; new assurance
bindings would be needed. Existing proof hashes are never rewritten.

The existing consequence evaluator runs on the entire frozen snapshot with the
new scenario. Its direct-holder impact is checked against the already validated
exact original path resolution. The LP matrix remains its separately captured
position context and is not inserted into population totals. `NoticeResolutionReuse`
retains original exact paths and both scenario hashes; it is a compatibility view,
not a new execution attestation. The original Phase 11 manifest reader verifies
all measured artifacts, and the **unchanged** `readiness::evaluate` receives the
**unchanged** policy and original evidence. This gate result is identical to
Phase 11. No path or requirement is silently respecified.

## Commands

Summary and one-command replay:

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- ingest-notice \
  --workflow probes/spacex-notice-workflow.json
cargo run --locked -q -p eplyx-lifecycle-impact -- preflight-from-notice \
  --workflow probes/spacex-notice-workflow.json
```

Explicit stages, with unused output paths:

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- ingest-notice \
  --workflow probes/spacex-notice-workflow.json --format json \
  --out /tmp/new-event.json
cargo run --locked -q -p eplyx-lifecycle-impact -- scenario-from-event \
  --workflow probes/spacex-notice-workflow.json --event /tmp/new-event.json \
  --format json --out /tmp/new-scenario.json --out-binding /tmp/new-binding.json
cargo run --locked -q -p eplyx-lifecycle-impact -- preflight-from-notice \
  --workflow probes/spacex-notice-workflow.json --event /tmp/new-event.json \
  --scenario /tmp/new-scenario.json --binding /tmp/new-binding.json --format json \
  --out /tmp/new-preflight.json --out-impact /tmp/new-impact.json \
  --out-resolution /tmp/new-resolution.json --out-readiness /tmp/new-readiness.json
```

The generated scenario retains the repository's portable scenario convention.
The notice commands verify its artifact relationships against the pinned workflow,
so explicit stages can save/reload in another directory as shown above. A separate
standalone `impact --scenario` command uses `LifecycleScenario::load` and resolves
references against the scenario's directory: use the published scenario in
`scenarios/` or preserve that repository layout. Absolute workflow paths work from
another directory. Editing paths changes the scenario and fails regeneration.

Ingest/scenario success exits 0. Preflight exits Ready 0, Blocked 3, Incomplete 4;
invalid input/protected output exits 2. Decision reports still emit on 3/4.
JSON stdout equals saved canonical JSON. All requested output paths are checked
before writes and `create_new` protects existing files. An I/O failure during
multi-file output can leave earlier newly created files; no existing file is
replaced. No wall-clock generation timestamp enters outputs.
