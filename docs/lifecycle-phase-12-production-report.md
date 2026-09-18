# Phase 12 issuer-notice pre-flight result

**PopulationRolloutReadiness = Incomplete**, reached from one real captured
PreStocks issuer notice through deterministic event/scenario generation and the
existing frozen-state assurance pipeline. Both mint addresses are independently
matched to Phase 9 raw account bytes. **OfficialTransition remains NotTested;
official mechanism Unknown; conversion ratio Unknown.** No source refresh,
network call, new VM execution or mainnet transaction occurs in these commands.

## 1. Source notice

Official source: `https://prestocks.com/spacex`. The unchanged historical artifact
is [the captured HTML](../evidence/lifecycle/prestocks-spacex-2026-09-17.html).
Retrieved time is retained from the original scenario's `/sources/0/captured_at`:
`2026-09-17T20:00:13Z`. Its exact UTF-8 content SHA-256 is
`3987e16a59ee881339038d284d0eebd60c51172670b6e030d6cffb9aa4a0ad82`.

[The new captured-source wrapper](../evidence/lifecycle/phase12/captured-issuer-notice.json)
embeds the exact old bytes and source URL, time, issuer and source type. Historical
HTTP status and MIME headers were not retained: `http_status=null` and
`content_type=null` explicitly preserve Unknown rather than fabricating transport
evidence. No duplicate download occurred. File-byte wrapper hash is `7ae96b029e825db09189e94895e07a93bbc5d21fd20423a7d8413cbe2d53193f`;
canonical parsed-document hash is `6ae746d91f052bdb472f79f410a53ca5353692060d0424b82b023d7f2b88b4dd`. JSON-pointer provenance uses the
latter, while artifact references use file-byte hashes. Digest integrity is relative
to the explicitly trusted historical capture and pinned inputs, not a signed
issuer attestation.

## 2. Extracted issuer assertions

Machine `og:site_name` and explorer-link `href` attributes provide issuer and
published mint identities. The rendered notice supplies bounded transition and
deadline grammar. Embedded Next.js script data does not contain these fields;
script strings are excluded. No fuzzy or model parsing runs.

The notice says SpaceX PreStocks tokens must be swapped into `$SPCXx` **or any
other token** before `11:59pm UTC on 12 March 2027`, or they will expire worthless.
This is an issuer assertion, not proof of exclusive conversion, zero market value
or technical non-exitability. Its separate “SpaceX has gone public” statement is
also an issuer assertion; no IPO fact is independently verified here.

All normalized semantic fields are listed below. Raw pointers are exact UTF-8
byte offsets into the unchanged HTML, with full exact snippets and digest in the
[event JSON](../reports/spacex-lifecycle-event.json). Unknowns point to the bounded
inspected notice rather than claiming absence everywhere. Metadata/identity
fields have additional exact JSON-pointer mappings in `field_provenance`.

| Event field | Value | Classification | Source pointer |
| --- | --- | --- | --- |
| `/issuer` | PreStocks | IssuerAsserted | `/raw_content/utf8-bytes/80845/80854` |
| `/event_type` | SuccessorTransition | Derived | `/raw_content/utf8-bytes/27269/28083` |
| `/effective_at` | Unknown | Unknown | `/raw_content/utf8-bytes/27269/28083` |
| `/deadline` | 2027-03-12T23:59:00Z | IssuerAsserted | `/raw_content/utf8-bytes/27269/28083` |
| `/deadline_wording` | 11:59pm UTC on 12 March 2027 | IssuerAsserted | `/raw_content/utf8-bytes/27269/28083` |
| `/alternate_destination` | any other token | IssuerAsserted | `/raw_content/utf8-bytes/27269/28083` |
| `/conversion_ratio` | Unknown | Unknown | `/raw_content/utf8-bytes/27269/28083` |
| `/official_mechanism` | Unknown | Unknown | `/raw_content/utf8-bytes/27269/28083` |
| `/official_execution_status` | NotTested | Unknown | `/raw_content/utf8-bytes/27269/28083` |
| `/source_asset/name` | SpaceX PreStocks | IssuerAsserted | `/raw_content/utf8-bytes/27269/28083` |
| `/source_asset/symbol` | Unknown | Unknown | `/raw_content/utf8-bytes/27269/28083` |
| `/source_asset/issuer_asserted_mint` | PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh | IssuerAsserted, OnChainVerified | `/raw_content/utf8-bytes/11599/11667`; `/records/4/response/result/value/39` |
| `/successor_asset/name` | Unknown | Unknown | `/raw_content/utf8-bytes/27269/28083` |
| `/successor_asset/symbol` | SPCXx | IssuerAsserted | `/raw_content/utf8-bytes/27487/27493` |
| `/successor_asset/issuer_asserted_mint` | Xs3oZwbHvqis4NYcf4YKWmEia2eC84wSiVrcYcTqpH8 | IssuerAsserted, OnChainVerified | `/raw_content/utf8-bytes/27331/27399`; `/records/4/response/result/value/47` |
| `/issuer_assertions/public_listing` | ⚠️ SpaceX has gone public! | IssuerAsserted | `/raw_content/utf8-bytes/27212/27242` |
| `/issuer_assertions/transition_and_expiry` | SpaceX PreStocks tokens must be swapped into $SPCXx or any other token before 11:59pm UTC on 12 March 2027, or they will expire worthless. Learn more | IssuerAsserted | `/raw_content/utf8-bytes/27269/28083` |

## 3. On-chain identity verification

Both asserted mint addresses are separately checked against unchanged
[Phase 9 raw RPC reconstruction](../evidence/transition/phase9/rpc-account-reconstruction.json),
file SHA `ab4d10bbcd1df05d6461d23031aea507e8bc0eda236e316415c9a92b45d9684b`.
Each check binds requested address/index, response bank, runtime owner, exact raw
hash, initialized non-executable mint and independently re-decoded full mint
configuration to the original verification report.

| Asset | Issuer-asserted and observed mint | RPC pointer | Captured slot | Observed metadata |
| --- | --- | --- | ---: | --- |
| Source | `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh` | `/records/4/response/result/value/39` | 448067723 | SpaceX PreStocks / SPACEX |
| Successor reference | `Xs3oZwbHvqis4NYcf4YKWmEia2eC84wSiVrcYcTqpH8` | `/records/4/response/result/value/47` | 448067723 | SpaceX xStock / SPCXx |

The adapter alone emits Unknown identity. Only independent raw-account verification
adds OnChainVerified. Captured initialized mint/configuration matching does not
prove legal affiliation, private entitlement, conversion or signed inclusion.
The notice does not publish source symbol SPACEX in this banner; its observed
on-chain symbol is retained separately, without relabeling it as issuer text.

## 4. Unknowns

The notice does not establish a precise lifecycle effective timestamp, conversion
ratio, official program/account plan, eligibility/backend/KYC conditions, signing
access, successful successor conversion, current liquidity or complete LP exit.
Mechanism remains Unknown, ratio is null and official execution is NotTested.
The exact deadline wording is retained separately from the explicit demo choice
to use the exclusive start of the stated minute. No fresh bank or source is
claimed. Native unwind still leaves accrued fees and a retained position.

## 5. Generated lifecycle event

[Canonical event](../reports/spacex-lifecycle-event.json), SHA `b8f91dd0aba091787d8a1368e3c5b0887778f72732e7e9d977ba8f4d7edf3824`.
The value-only semantic digest tests irrelevant markup/attribute ordering stability;
it never substitutes for source/event integrity. Exact whole-event regeneration
checks digest, extracted values, provenance pointers, classifications and both
independent identities. [The actual text summary](../reports/spacex-lifecycle-event.txt)
starts the demo with the real-world notice and its evidence boundary.

## 6. Generated scenario

[Generated LifecycleScenario](../scenarios/spacex-transition-ingested.json),
SHA `bcf333bff6d3d116aff97003887857a5164a26dd427ab00a6c21b8de0a83a534`, and [field binding](../reports/spacex-notice-scenario-binding.json).
It uses the existing LifecycleChange schema and generic consequence evaluator.
Source asset, TransitionRequired assertion, deadline and successor reference come
from captured material with independent identity checks. The new successor is
identity only. No successor-execution adapter or converted entitlement is added.

[Explicit demo configuration](../scenarios/spacex-demo-evaluation.json) retains
`2026-09-17T20:00:00Z` as the hypothetical effective/evaluation boundary,
Before=Active, exclusive minute cutoff and external NoIssuerEntitlement expiry
interpretation. None is silently promoted to an issuer/on-chain fact. Scenario
captured_at identifies the original source capture; no generation timestamp enters
outputs. Later mint/proof observation banks remain separate contexts.

## 7. Provenance classification

IssuerAsserted records published words/links; OnChainVerified records separately
decoded captured mint bytes; DemoConfigured records the hypothetical boundary and
interpretive choices; Derived records deterministic mapping/parsing; Unknown
records facts the bounded source does not establish. Every serialized scenario
leaf has a mapping in the binding, including metadata/support arrays. Mints have
IssuerAsserted + OnChainVerified; deadline mapping has IssuerAsserted + Derived +
DemoConfigured precision interpretation. The evaluation boundary is exclusively
DemoConfigured. Semantic facts and execution proof remain distinct.

## 8. Readiness result

[Complete new impact](../reports/spacex-notice-transition-impact.json) evaluates all
17,957 frozen token-account entities under the generated scenario. No LP or vault
observation is added to holder totals. [The path compatibility view](../reports/spacex-notice-path-resolution.json)
retains the already validated direct/LP matrices and their exact original contexts.

The generated scenario has a different hash, ID/provenance and additive successor
identity. An explicit compatibility bridge requires **exact asset, effective
boundary, before/after statuses and deadline/after status** against the original
scenario. Only independently verified successor identity is additive, with Unknown
mechanism, absent ratio and NotTested official execution. The selected direct-holder
impact must match its original path entity/type/public amount/status/classification
and policy evaluation time. Changed economic semantics fail closed; no old proof
hash is rewritten or treated as a proof at a new bank.

The original Phase 11 manifest reader verifies all frozen measurements. The
unchanged readiness evaluator receives the **unchanged assurance policy and original
proofs**. [Notice readiness JSON](../reports/spacex-notice-readiness.json) is
byte-identical to Phase 11 readiness, SHA
`fb2b093170d5b4c85461c2d83b4d99569b2d80d9f6b62c3783bbfd2a843a4953`.
The wrapper explicitly binds generated impact/event/scenario to this compatibility
result; it does not re-attest old transactions against the new scenario hash.

| Required condition | Result |
| --- | --- |
| Exact direct-holder mobility | Satisfied |
| Native LP principal unwind | Satisfied |
| Exact evidence isolation | Satisfied |
| Direct and LP official transition | IncompleteEvidence |
| Complete LP exit, including fees and closure | IncompleteEvidence |
| Population actionability | IncompleteEvidence |

LP accrued fees remain 126,543 raw SPACEX / 113,735 raw USDC; collection/closure
stay NotTested. Four positive entities have full represented-amount evidence;
10,151 positive entities lack matching requested-path evidence. The optional
historical failed route remains Informational. Independent banks/routes are not
simultaneous capacity or proceeds. [Full notice pre-flight](../reports/spacex-notice-preflight.json)
and [actual CLI summary](../reports/spacex-notice-preflight.txt) record Incomplete,
exit 4. Notice ingestion adds no readiness or execution assurance by itself.

## 9. Tamper tests

Changed raw digest, linked mint, normalized deadline, exact pointer, provenance
classification or generated scenario fails verification. Missing required fields
and missing/mismatched independent chain evidence fail. Irrelevant markup changes
require explicit new source generation, preserve value semantics where appropriate
and still change the source integrity hash. Deserializing event JSON alone never
constructs VerifiedLifecycleEvent. The checked compatibility bridge also rejects
changed economic semantics rather than weakening the existing policy.

## 10. Architecture and deterministic CLI

Generic `notice/` contains source/event/taxonomy models, opaque verified event,
scenario binding and offline orchestration. Issuer grammar lives only in
`prestocks::CapturedIssuerNotice`. No new dependency is added. Existing consequence,
resolution evidence and readiness components retain their responsibilities; no
evaluator logic is copied. Original proof contexts and assurance policy stay intact.

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- ingest-notice \
  --workflow probes/spacex-notice-workflow.json
cargo run --locked -q -p eplyx-lifecycle-impact -- preflight-from-notice \
  --workflow probes/spacex-notice-workflow.json
```

`scenario-from-event` supports explicit stages. JSON stdout and saved JSON match;
existing outputs are protected. Preflight codes remain Ready 0 / Blocked 3 /
Incomplete 4 / invalid or protected input/output 2. Absolute workflow paths work
from another directory without RPC configuration. See [architecture/CLI guide](lifecycle-phase-12-notice.md)
for provenance hash conventions, portable explicit commands and proof boundaries.

## 11. Tests and actual code mutations

Completed against final source and published artifacts:

- `make test`: **313 passed, 0 failed, 0 ignored**; both synthetic SBF versions built.
- `make fmt-check`: passed for engine/interface and fixture programs.
- `make lint`: passed for all engine targets and both fixture versions.
- `git diff --check`: passed.
- Final offline pipeline replay: event-derived scenario, complete impact, path compatibility view and readiness byte-identical to published artifacts; expected Incomplete exit **4**.
- Explicit three-stage CLI flow passed from a temporary directory with absolute workflow input, generated event/scenario/binding reload, byte-identical JSON stdout/saved outputs and existing-output protection.
- **All 8 final actual executable-code mutations caught by named assertions**, with final source restored byte-for-byte; no compiler failure counts as a caught fault.
- All **50 historical checksum files**: **302 matching entries**, **20** allowed entries for four shared scope/export/CLI files; **zero unexpected changes**.

Suite breakdown: fixture v1 7, fixture v2 7, engine 156, expansion 6,
probe 12, lifecycle consequence 12, coverage 16, exposure 11, notice 2,
readiness 3, resolution 5, snapshot 13, official transition 3,
protocol position 10, scenario 2, upgrade integration 42 and interface 6.
Older execution regressions perform their original frozen-fixture VM replays;
notice ingestion/orchestration itself performs none. No earlier test assertion
is weakened, ignored or skipped.

The 21 focused unit tests cover source digest, exact successor/date provenance,
missing required fields, source-only Unknown identity, independently confirmed
raw mint identities, mismatched/missing chain evidence, wording/Unknown mechanism/
unknown ratio, exclusive demo-time taxonomy, deterministic scenario generation,
attribute-order/markup stability, altered date/mint/pointer rejection, manual
scenario/binding tampering, alternate-destination/assertion boundaries, script
exclusion, strict parsing and rejection of changed economic semantics before
frozen-proof reuse. Two integration tests consume unchanged production evidence,
reproduce the final full pipeline and exact Phase 11 readiness, exercise the
portable explicit CLI stages and reject tampered inputs/protected outputs.

| Injected executable fault | Named catching assertion | Actual result |
| --- | --- | --- |
| Trust successor address/metadata without raw on-chain verification | `notice::tests::missing_chain_artifact_cannot_grant_identity` | Caught; [log 1](../reports/phase12-mutations-final/01.txt) |
| Let issuer swap wording prove OfficialTransition | `notice::tests::issuer_wording_never_proves_official_transition` | Caught; [log 2](../reports/phase12-mutations-final/02.txt) |
| Relabel demo timestamp as issuer assertion | `notice::tests::demo_time_never_becomes_issuer_assertion` | Caught; [log 3](../reports/phase12-mutations-final/03.txt) |
| Ignore source raw-content digest mismatch | `notice::tests::source_digest_mismatch_is_rejected` | Caught; [log 4](../reports/phase12-mutations-final/04.txt) |
| Accept altered normalized deadline without source regeneration | `notice::tests::altered_deadline_without_regeneration_is_rejected` | Caught; [log 5](../reports/phase12-mutations-final/05.txt) |
| Infer invented conversion ratio from unrelated page data | `notice::tests::unrelated_data_never_infers_conversion_ratio` | Caught; [log 6](../reports/phase12-mutations-final/06.txt) |
| Convert unknown mechanism into an executable Swap | `notice::tests::unknown_mechanism_stays_unknown` | Caught; [log 7](../reports/phase12-mutations-final/07.txt) |
| Bypass unchanged required readiness findings | `notice_cannot_bypass_existing_readiness_requirements` | Caught; [log 8](../reports/phase12-mutations-final/08.txt) |

[The final mutation record](../reports/spacex-phase12-mutation-results-final.json)
binds all eight assertion-failure logs to the exact restored final core/adapter/
workflow hashes. Development runs are retained separately: an earlier eight-fault
campaign and one failed verification attempt with a compiler error. The latter
records **zero caught faults** and is excluded from final assurance. The final
campaign has eight assertion failures, not build failures. The CLI is rebuilt
from restored final source so no mutated executable is left as the working binary.

[The validation record](../reports/spacex-phase12-validation.json) binds final
source/output/mutation hashes and [complete logs](../reports/phase12-validation/).
[The full Phase 12 file list](../reports/spacex-phase12-files.txt) and
[artifact checksums](../reports/spacex-phase12-artifacts.sha256) cover all added/
updated files and retained validation evidence. Earlier checksum files are intact.

| Final artifact | SHA-256 |
| --- | --- |
| [Workflow](../probes/spacex-notice-workflow.json) | `4b0c76e4fc9bd5cf0641ffbba73dcf39d3b6d8c234184939633628cd9e13f7f3` |
| [Event](../reports/spacex-lifecycle-event.json) | `b8f91dd0aba091787d8a1368e3c5b0887778f72732e7e9d977ba8f4d7edf3824` |
| [Generated scenario](../scenarios/spacex-transition-ingested.json) | `bcf333bff6d3d116aff97003887857a5164a26dd427ab00a6c21b8de0a83a534` |
| [Scenario field binding](../reports/spacex-notice-scenario-binding.json) | `71cd214301284000cd9d16d49183eb9cdeb5dca67e8c86b9694fa4a985b19705` |
| [Generated impact](../reports/spacex-notice-transition-impact.json) | `d4ca7245e30f5a59dada7eae1ce503cdea718a17101befb12a262fe42a894a25` |
| [Frozen path compatibility view](../reports/spacex-notice-path-resolution.json) | `f4c2f99c07fed9b2ca4bd88c60767a5fe2c690daa5c4333b008a31b20608068f` |
| [Notice pre-flight](../reports/spacex-notice-preflight.json) | `053a47d6da73ab3009fa09b09e389908bc2f49e0558a086211aa9e119b91890e` |
| [Final mutations](../reports/spacex-phase12-mutation-results-final.json) | `ac74ab53e2cf11bbe304e00cc6ee3037cd80775413286b50d4318f40d890607f` |

## 12. Product conclusion

Stocklana can now begin with this real issuer notice, explain exactly what was
published, match both referenced mints to independent frozen observations, derive
an explicit lifecycle change and reach the same production-state pre-flight
result. Broader readiness stays Incomplete because actual required proof gaps
remain. Notice semantics never fabricate an issuer mechanism, converted funds,
key possession, peer assurance or technical non-exitability.

Work remains on main in the shared uncommitted tree. No commit/push, new capture or probe campaign,
official retry, redemption, price/valuation, issuer
credentials, crawler, automatic discovery, polling, model parsing, feed,
notification or UI is introduced. Earlier artifacts and assertions remain intact;
only four shared scope/export/CLI files receive additive Phase 12 edits.

## 13. Smallest Phase 13 recommendation

Add one read-only interactive Stocklana demo over the published backend artifacts:
notice → field provenance → generated scenario → population impact → exact direct/LP
paths → readiness requirements and remaining gaps. Let users inspect proof context,
issuer/on-chain/demo distinctions and retained fees. Reuse deterministic outputs;
no new capture, scoring, execution or issuer integration is needed. **Phase 13 has
not been started.**
