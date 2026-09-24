# Transition package pre-flight

Package: `0958e335110b1185d77818ace1651cf8b8c24b7f7926397fd82a66441f8502e0`

Candidate program: `ce7b4d55310ca188e652951dd3102eea1af9c2469dd389a12ef11d6319ccf36d` (Proposed)

Source: `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh`
Replacement: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`

Selected stress cases: 10

CandidatePlanReadiness: **Blocked**
ConversionStressReadiness: **Blocked**
PopulationRolloutReadiness: **Incomplete**
OfficialTransition: **NotTested**
Analytical pre-flight status: **Blocked**

No funds moved. Holder signing was assumed locally; key possession and issuer binding remain unknown.

## Final-state stress evidence

Frozen identities: 10; executable at final capture: 10; locally tested: 10; no longer executable: 0; identity changed: 0. Discovery shapes preserved among executions: 10; discovery balance buckets preserved among executions: 10. Execution evidence applies only to the exact final account state and amount.

## Rollout invariants

- **Violated** conversion_output_matches (blocking, ExactCandidateConversion): The exact candidate VM conversion failed.
- **Violated** no_selected_case_failed (blocking, SelectedStressCases): 10 exact selected stress cases failed.

## Deployment gate

**BLOCKED** under `block-only`.

- candidate_plan_readiness is Blocked
- conversion_stress_readiness is Blocked
- population_rollout_readiness is Incomplete
- The candidate conversion failed in local simulation: InstructionError(1, Custom(13)). No mainnet funds moved and the watched accounts rolled back.
- 10 exact selected stress cases failed in local execution
- Authority control inspected for 20 selected non-wallet accounts; 3002 remain outside the bounded selection
- invariant inv-1e5b022d45b06f637691 (conversion_output_matches) is Violated: The exact candidate VM conversion failed.
- invariant inv-4281244c10dc3c203ffb (no_selected_case_failed) is Violated: 10 exact selected stress cases failed.

Replay offline: `eplyx-lifecycle replay-package-preflight <package> --result <result-directory>`

## Non-standard account control

Selected: 20; unresolved outside budget: 3002; proven program-mediated conversions: 0. Resolution does not provide authorization or conversion proof.
