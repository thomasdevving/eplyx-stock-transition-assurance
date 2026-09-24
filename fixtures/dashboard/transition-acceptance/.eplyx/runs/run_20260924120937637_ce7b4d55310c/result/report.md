# Transition package pre-flight

Package: `8b1e9639fbca1aea8a3ff5bd83c21f3cca6c55e8155a326f461cc677487fed46`

Candidate program: `ce7b4d55310ca188e652951dd3102eea1af9c2469dd389a12ef11d6319ccf36d` (Proposed)

Source: `PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh`
Replacement: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`

Selected stress cases: 10

CandidatePlanReadiness: **Ready**
ConversionStressReadiness: **Incomplete**
PopulationRolloutReadiness: **Incomplete**
OfficialTransition: **NotTested**
Analytical pre-flight status: **Incomplete**

No funds moved. Holder signing was assumed locally; key possession and issuer binding remain unknown.

## Final-state stress evidence

Frozen identities: 10; executable at final capture: 10; locally tested: 10; no longer executable: 0; identity changed: 0. Discovery shapes preserved among executions: 10; discovery balance buckets preserved among executions: 10. Execution evidence applies only to the exact final account state and amount.

## Rollout invariants

- **Satisfied** conversion_output_matches (blocking, ExactCandidateConversion): Exact candidate VM output passed the existing conversion reconciliation.
- **Satisfied** no_selected_case_failed (blocking, SelectedStressCases): All 10 exact selected stress cases were proven.

## Deployment gate

**PASS WITH WARNINGS** under `block-only`.

- conversion_stress_readiness is Incomplete
- population_rollout_readiness is Incomplete
- Authority control inspected for 20 selected non-wallet accounts; 3002 remain outside the bounded selection

Replay offline: `eplyx-lifecycle replay-package-preflight <package> --result <result-directory>`

## Non-standard account control

Selected: 20; unresolved outside budget: 3002; proven program-mediated conversions: 0. Resolution does not provide authorization or conversion proof.
