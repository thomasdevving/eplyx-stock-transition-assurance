# Phase 13: deterministic counterfactual lifecycle analysis

One `FrozenCounterfactualWorld` loads the requested snapshot/scenario through the
existing trusted readiness-policy manifest. That reader verifies original exact
holder/position/coverage/measurement bindings without a VM. Position discovery
then independently decodes the original captured pre-execution position, compares
it with the stored withdrawal report's original position, and never uses simulated
post-withdrawal state. Public capture banks remain independently identified.

`CounterfactualScenario` specifies identity, label, evaluation time and exact
lifecycle-policy hash. The three canonical views use effective time minus one
nanosecond, effective time, and the explicit inclusive deadline. There is no
scenario generator or alternate consequence engine. Unknown semantics stay unknown;
custom views cannot replace the policy. Empty/duplicate view IDs are rejected.

The existing consequence evaluator supplies every population classification and
summary. Each view evaluates before=after=view time; changed-meaning counts belong
to the explicit comparisons rather than the evaluator's within-view summary.
Its checked public entry remains checked. A crate-private entry shares
the same implementation while reusing the canonical snapshot hash after immutable
world ownership and pinned validation; repeated times do not renormalize unchanged
RPC bytes. The shared public-exposure classifier also classifies the decoded LP
principal without counting it as another holder or deduplicating it against vault
capital. Protocol observations retain their own exact amount/byte/slot evidence.

Historical holder and position matrices are retained once, with original full
contexts and digests. Each view adds separate path implications: status stays
historical; `lifecycle_relevant` only describes consideration under the economic
policy. Relevant NotTested/Unsupported paths are still evidence gaps, not available
execution routes. Transfer, sale, withdrawal, redemption and conversion cannot
inherit one another's proof.

The exact readiness policy is evaluated using the original verified evidence.
That report is retained as `frozen_readiness`, including original evaluation time,
requirements, scopes and `PreflightFailureMode` causal records. Each time view adds
`PreEvent`, `Applicable` or `Unknown` gate applicability. Active semantics have no
immediate lifecycle-action requirement; this is not a Ready assurance finding.
Known non-Active exposure activates the unchanged policy result and references
its failure-mode IDs. Unknown applicability grants no readiness status. Passing a
deadline alone cannot create a policy violation or promote Incomplete to Blocked.

The canonical JSON includes one sorted population-entity ledger. View classification
groups and comparison change groups reference zero-based indices into that ledger.
The UI only resolves indices, with no lifecycle computation. Selected holder,
protocol-vault and position rows give exact observations and interpretations.
Comparisons explicitly carry boundary pointers, changed selected/protocol exposure
pairs, lifecycle states, path relevance pairs, active readiness-finding IDs and
change taxonomy. Unchanged capture facts are stated separately.

Production identity hashes the base snapshot, augmented exposure snapshot,
position fixture/discovery and raw position. Historical proof, scenario, lifecycle
policy, readiness policy and nested evidence artifacts have separate exact bindings.
A comparison rejects different production digests, public-balance hashes/selected
amounts, selected/protocol byte hashes/banks, policy hashes, entity populations or
historical proof results. `validate` regenerates the full report from the trusted
immutable world; editing claims while retaining declared digests is not sufficient.
This is digest integrity relative to pinned evidence, not signed inclusion proof.

## Offline replay

```sh
cargo run --locked -q -p eplyx-lifecycle-impact -- compare-scenarios \
  --snapshot snapshots/spacex-exposure.json \
  --scenario scenarios/spacex-transition.json \
  --readiness-policy policies/stocklana-spacex-preflight-v1.json \
  --format json --out /tmp/spacex-counterfactual.json
cmp /tmp/spacex-counterfactual.json reports/spacex-counterfactual-lifecycle.json
```

Saved JSON and JSON stdout are canonical and identical, with a trailing newline.
The command returns 0 for valid analysis (even when applicable assurance is
Incomplete), 2 for invalid/non-comparable inputs or an existing protected output.
This analysis command does not approve rollout. Readiness CLI codes remain unchanged.
Outputs use the existing create-new writer and never overwrite frozen artifacts.
Inputs work as absolute paths from outside the repository; no RPC environment is
needed. Tests include actual CLI replay and protected-output verification.

No RPC/HTTP, capture, holder/pool sampling, new execution, official retry, redemption,
notice ingestion, price source, valuation, stochastic simulation or UI is introduced.
The required full regression suite still runs the existing execution tests.
