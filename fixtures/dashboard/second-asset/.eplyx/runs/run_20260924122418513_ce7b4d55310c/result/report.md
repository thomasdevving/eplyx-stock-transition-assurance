# Transition package pre-flight

Package: `0d5c60ceaabd4f0dcea3aeab29b58c49f0e9b5b08869c75914b77832c7f21d7a`

Candidate program: `ce7b4d55310ca188e652951dd3102eea1af9c2469dd389a12ef11d6319ccf36d` (Proposed)

Source: `PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF`
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
- Authority control inspected for 20 selected non-wallet accounts; 12484 remain outside the bounded selection

Replay offline: `eplyx-lifecycle replay-package-preflight <package> --result <result-directory>`

## Non-standard account control

Selected: 20; unresolved outside budget: 12484; proven program-mediated conversions: 0. Resolution does not provide authorization or conversion proof.
