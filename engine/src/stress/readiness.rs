//! Candidate Conversion Stress Readiness: one explicit demonstration policy
//! evaluated by the existing requirement engine.
//!
//! This adds a policy and its facts, not a second readiness engine. Four scopes
//! stay distinct and none of them overwrites another:
//!
//! * `SelectedEntityReadiness` — one wallet's exact mobility tests.
//! * `CandidatePlanReadiness` — one account, one supplied plan.
//! * `ConversionStressReadiness` — this bounded run, under its own policy.
//! * `PopulationRolloutReadiness` — every positive-balance account, separately
//!   evaluated here and never satisfied by a bounded sample.
//!
//! Ready means only that this policy's declared requirements held under its
//! stated scope. It is not an asset safety judgment.
use super::{
    population::PopulationObservation, select::StressTestPlan, CaseResult, SelectedCase,
    ShapeCoverage,
};
use crate::{
    expansion::Eligibility,
    lifecycle::{
        exposure::sha256,
        policy::{AssetLifecyclePolicy, LifecycleStatus},
    },
    readiness::{
        evaluate, ConversionFact, EvaluatedScope, EvidenceReference, EvidenceScope,
        LifecycleReadinessPolicy, PolicyType, PopulationEntityFact, PopulationEvidence,
        ReadinessRequirement, RequirementCondition, RequirementTarget, StateShape, StressEvidence,
        VerifiedReadinessEvidence,
    },
    resolution::{ArtifactRef, PathStatus, SignerAssumption},
};
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const STRESS_POLICY_ID: &str = "eplyx-conversion-stress-readiness-v1";
pub const POPULATION_POLICY_ID: &str = "eplyx-conversion-population-rollout-v1";
/// This milestone selects no lifecycle event. Both sides of the neutral policy
/// are Unknown so that nothing here can be read as a lifecycle assertion.
pub const NO_LIFECYCLE_EVENT: &str = "no-lifecycle-event/conversion-stress/v1";

fn neutral_lifecycle(asset_mint: &str) -> AssetLifecyclePolicy {
    AssetLifecyclePolicy {
        asset_mint: asset_mint.into(),
        effective_at: chrono::DateTime::UNIX_EPOCH,
        before: LifecycleStatus::Unknown,
        after: LifecycleStatus::Unknown,
        deadline: None,
        successor: None,
    }
}
fn scenario_digest() -> String {
    sha256(NO_LIFECYCLE_EVENT.as_bytes())
}

/// The exact scope of one stress case. Built once and used for both the policy
/// requirement and the evidence fact, so the two can only match exactly.
fn case_scope(
    asset_mint: &str,
    plan: &StressTestPlan,
    case: &SelectedCase,
    result: &CaseResult,
) -> EvidenceScope {
    let rebound = plan.schema_version == 2;
    EvidenceScope {
        asset_mint: asset_mint.into(),
        entity_id: case.entity_id.clone(),
        state_shape: StateShape::DirectTokenAccount,
        authority: case.authority.clone(),
        exact_amount_raw: Some(if rebound {
            result.detail["execution_plan"]["final_amount_raw"]
                .as_str()
                .unwrap_or(&case.selected_amount_raw)
                .to_string()
        } else {
            case.selected_amount_raw.clone()
        }),
        range: None,
        bps_to_remove: None,
        venue: None,
        context_id: Some(case.case_id.clone()),
        captured_slot: None,
        clock: None,
        capture_context: Some(
            crate::expansion::pipeline::CaptureContext::CurrentFinalizedProduction,
        ),
        captured_state_sha256: Some(if rebound {
            result.detail["execution_plan"]["final_capture_digest"]
                .as_str()
                .unwrap_or(&plan.population_capture_sha256)
                .to_string()
        } else {
            plan.population_capture_sha256.clone()
        }),
        fixture_sha256: result.execution_fixture_sha256.clone(),
        source_before_raw: Some(if rebound {
            result.detail["revalidation"]["final_amount_raw"]
                .as_str()
                .unwrap_or(&case.observed_balance_raw)
                .to_string()
        } else {
            case.observed_balance_raw.clone()
        }),
        scenario_sha256: scenario_digest(),
    }
}

fn conversion_facts(
    asset_mint: &str,
    plan: &StressTestPlan,
    results: &[CaseResult],
) -> Vec<ConversionFact> {
    plan.selected
        .iter()
        .zip(results)
        .map(|(case, result)| ConversionFact {
            scope: case_scope(asset_mint, plan, case, result),
            status: result.status,
            provenance: case.case_plan.provenance,
            plan_sha256: if plan.schema_version == 2 {
                result.detail["resolved_case_plan_sha256"].as_str()
                    .unwrap_or(&case.case_plan_sha256).to_string()
            } else { case.case_plan_sha256.clone() },
            program_sha256: result.candidate_program_sha256.clone(),
            replacement_mint: case.case_plan.replacement_mint.clone(),
            destination: result.detail["destination"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            execution_attempted: result.execution_performed,
            reconciled: result.status == PathStatus::Proven
                || result.detail["rollback_verified"] == json!(true),
            rollback_verified: result.detail["rollback_verified"].as_bool(),
            holder_signer: SignerAssumption {
                authority: case.authority.clone(),
                signer_possession_known: false,
                signer_assumed_locally: result.signer_assumed_locally,
                wording: "The recorded holder authority is assumed locally to sign. Key possession and authorization remain unknown.".into(),
            },
            candidate_authority_assumed_locally: true,
            issuer_binding_established: false,
            evidence_ids: vec![result.case_id.clone()],
            reason: result
                .reason
                .clone()
                .unwrap_or_else(|| "Exact stress case executed against its own freshly captured bank.".into()),
        })
        .collect()
}

fn stress_evidence(
    plan: &StressTestPlan,
    results: &[CaseResult],
    shapes: &[ShapeCoverage],
    observation: &PopulationObservation,
) -> StressEvidence {
    let status = |s: PathStatus| results.iter().filter(|r| r.status == s).count();
    let ids = |s: PathStatus| {
        results
            .iter()
            .filter(|r| r.status == s)
            .map(|r| r.case_id.clone())
            .collect::<Vec<_>>()
    };
    let executable: Vec<&ShapeCoverage> = shapes
        .iter()
        .filter(|s| s.eligibility == Eligibility::ExecutableCandidate)
        .collect();
    let unsupported_positive: usize = plan
        .state_shapes
        .iter()
        .filter(|s| s.eligibility == Eligibility::Unsupported)
        .map(|s| s.entities_in_shape)
        .sum();
    let capture_required: usize = plan
        .state_shapes
        .iter()
        .filter(|s| s.eligibility == Eligibility::CaptureRequired)
        .map(|s| s.entities_in_shape)
        .sum();
    StressEvidence {
        stress_id: plan.stress_id.clone(),
        population_capture_sha256: plan.population_capture_sha256.clone(),
        stress_plan_sha256: plan.sha256().unwrap_or_default(),
        candidate_plan_sha256: plan.candidate_plan_sha256.clone(),
        candidate_program_sha256: plan.candidate_program_sha256.clone(),
        enumeration_completeness: plan.enumeration_completeness.key().into(),
        authority_resolution_completeness: plan.authority_resolution_completeness.key().into(),
        positive_balance_accounts: observation.summary.positive_balance_accounts_observed,
        selected_cases: plan.selected.len(),
        executed_cases: results.iter().filter(|r| r.execution_performed).count(),
        proven_cases: status(PathStatus::Proven),
        failed_cases: status(PathStatus::Failed),
        indeterminate_cases: status(PathStatus::Indeterminate),
        unsupported_cases: status(PathStatus::Unsupported),
        executable_shapes: executable.len(),
        executable_shapes_with_executed_case: executable
            .iter()
            .filter(|s| s.entities_executed > 0)
            .count(),
        uncovered_executable_shapes: executable
            .iter()
            .filter(|s| s.entities_executed == 0)
            .map(|s| s.state_shape_sha256.clone())
            .collect(),
        unsupported_positive_balance_accounts: unsupported_positive,
        capture_required_positive_balance_accounts: capture_required,
        failed_case_ids: ids(PathStatus::Failed),
        indeterminate_case_ids: ids(PathStatus::Indeterminate),
        unsupported_case_ids: ids(PathStatus::Unsupported),
        evidence_ids: results.iter().map(|r| r.case_id.clone()).collect(),
    }
}

fn population_evidence(
    observation: &PopulationObservation,
    proven: &[&CaseResult],
) -> Result<PopulationEvidence> {
    let proven_ids: std::collections::BTreeSet<&str> =
        proven.iter().map(|r| r.entity_id.as_str()).collect();
    let mut entities = Vec::new();
    let mut represented: BTreeMap<String, u64> = BTreeMap::new();
    let mut covered: BTreeMap<String, u64> = BTreeMap::new();
    for e in observation.positive_entities() {
        let balance = e.balance()?;
        let proven_here = proven_ids.contains(e.entity_id.as_str());
        represented.insert(e.entity_id.clone(), balance);
        if proven_here {
            covered.insert(e.entity_id.clone(), balance);
        }
        entities.push(PopulationEntityFact {
            entity_id: e.entity_id.clone(),
            account_type: crate::expansion::type_key(&e.authority_model),
            balance_raw: balance.to_string(),
            proven_full_amount_paths: vec![],
            proven_candidate_conversion: proven_here,
        });
    }
    let represented_raw = super::sum_once(&represented);
    let covered_raw = super::sum_once(&covered);
    let without = represented_raw.parse::<u128>()? - covered_raw.parse::<u128>()?;
    Ok(PopulationEvidence {
        token_account_entities: observation.summary.token_accounts_observed,
        positive_balance_entities: observation.summary.positive_balance_accounts_observed,
        distinct_owner_authorities: observation.summary.distinct_recorded_authorities,
        entities_with_measured_amount: proven_ids.len(),
        represented_amount_raw: represented_raw,
        covered_amount_raw: covered_raw,
        without_evidence_raw: without.to_string(),
        account_types: BTreeMap::new(),
        execution_status_counts: BTreeMap::new(),
        evidence_ids: vec![observation.capture_sha256.clone()],
        entities,
    })
}

fn requirement(
    id: &str,
    label: &str,
    target: RequirementTarget,
    condition: RequirementCondition,
    rollout_assumption: &str,
    remediation: &str,
) -> ReadinessRequirement {
    ReadinessRequirement {
        id: id.into(),
        label: label.into(),
        target,
        required: true,
        condition,
        rollout_assumption: rollout_assumption.into(),
        remediation_requirement: remediation.into(),
    }
}

/// Evaluate both the stress policy and the separate population rollout policy.
/// They are returned as two independent findings and never merged.
pub fn evaluate_stress(
    observation: &PopulationObservation,
    plan: &StressTestPlan,
    results: &[CaseResult],
    shapes: &[ShapeCoverage],
    evaluated_at: &str,
) -> Result<(Value, Value)> {
    let asset_mint = &plan.asset_mint;
    let scenario_sha256 = scenario_digest();
    let lifecycle = neutral_lifecycle(asset_mint);
    let facts = conversion_facts(asset_mint, plan, results);
    let stress = stress_evidence(plan, results, shapes, observation);
    let stress_target = RequirementTarget::StressRun {
        stress_id: plan.stress_id.clone(),
    };

    let mut requirements = vec![
        requirement(
            "evidence-isolation",
            "Exact evidence isolation across entities, state shapes and amounts",
            stress_target.clone(),
            RequirementCondition::EvidenceIsolation,
            "This run's exact executed cases only; no tested entity or shape stands in for its peers",
            "Keep per-case evidence bound to its exact entity, amount, case plan, candidate program build and captured bank",
        ),
        requirement(
            "population-acquisition",
            "Fresh population discovery met the declared completeness on both independent axes",
            stress_target.clone(),
            RequirementCondition::PopulationAcquisition {
                required_enumeration: "CompleteForQuery".into(),
                required_authority_resolution: "Complete".into(),
                max_unsupported_positive_balance_accounts: 0,
            },
            "Only what this capture actually observed; an incomplete acquisition is never treated as an empty remainder",
            "Obtain a provider that can serve a complete filtered enumeration and resolve every recorded authority, then recapture and rerun",
        ),
        requirement(
            "supported-shape-coverage",
            "Every discovered executable state shape has at least one exact executed case",
            stress_target.clone(),
            RequirementCondition::SupportedShapeCoverage,
            "Shape coverage records which classes were exercised, never that their members are proven",
            "Execute at least one exact case in each uncovered executable state shape, or implement the missing authority mechanism",
        ),
        requirement(
            "selected-case-outcomes",
            "No selected executable case failed, and every selected case is proven",
            stress_target.clone(),
            RequirementCondition::SelectedCaseOutcomes {
                allow_failed: false,
                allow_indeterminate: false,
                allow_unsupported: false,
            },
            "Only the exact selected cases and their exact frozen amounts",
            "Fix the candidate mechanism for the failing production state, or recapture the state that could not be established, then rerun the stress test",
        ),
    ];
    for (case, fact) in plan.selected.iter().zip(&facts) {
        requirements.push(requirement(
            &format!("candidate-conversion-{}", case.case_id),
            &format!(
                "Candidate conversion proven for exact account {} at its full observed balance",
                case.token_account
            ),
            RequirementTarget::Entity {
                entity_id: case.entity_id.clone(),
            },
            RequirementCondition::CandidateConversion {
                scope: Box::new(fact.scope.clone()),
                plan_sha256: fact.plan_sha256.clone(),
                program_sha256: plan.candidate_program_sha256.clone(),
                replacement_mint: case.case_plan.replacement_mint.clone(),
            },
            "This exact account, amount, case plan, candidate program build and captured bank only",
            "Execute this exact case successfully and reconcile it exactly; no other account or amount can supply it",
        ));
    }

    let stress_policy = LifecycleReadinessPolicy {
        schema_version: 1,
        id: STRESS_POLICY_ID.into(),
        policy_type: PolicyType::DemoAssurancePolicy,
        not_issuer_policy: true,
        description: "Explicit demonstration stress policy for one bounded candidate-conversion run against freshly captured current production state. Ready means only that these declared requirements held for the exact cases executed. It is not population readiness, not an issuer rule and not an asset safety judgment.".into(),
        asset_mint: asset_mint.clone(),
        scenario_sha256: scenario_sha256.clone(),
        evaluated_scope: EvaluatedScope::ConversionStressReadiness,
        evidence_manifest: ArtifactRef {
            file: format!("stress:{}", plan.stress_id),
            sha256: plan.population_capture_sha256.clone(),
        },
        requirements,
    };

    let proven: Vec<&CaseResult> = results
        .iter()
        .filter(|r| r.status == PathStatus::Proven)
        .collect();
    let evidence_refs = vec![EvidenceReference {
        id: plan.stress_id.clone(),
        artifact: ArtifactRef {
            file: format!("{}.population.json", plan.stress_id),
            sha256: plan.population_capture_sha256.clone(),
        },
        description: "Freshly captured current population for this stress run".into(),
    }];
    let build_evidence = |population: PopulationEvidence| VerifiedReadinessEvidence {
        asset_mint: asset_mint.clone(),
        scenario_sha256: scenario_sha256.clone(),
        lifecycle_event: lifecycle.clone(),
        policy_evaluated_at: evaluated_at.into(),
        path_facts: vec![],
        conversion_facts: facts.clone(),
        complete_exits: vec![],
        population,
        evidence_refs: evidence_refs.clone(),
        isolation_verified: true,
        stress: Some(stress.clone()),
    };

    let stress_report = evaluate(
        &stress_policy,
        &build_evidence(population_evidence(observation, &[])?),
    )?;

    let population_policy = LifecycleReadinessPolicy {
        schema_version: 1,
        id: POPULATION_POLICY_ID.into(),
        policy_type: PolicyType::DemoAssurancePolicy,
        not_issuer_policy: true,
        description: "Population rollout readiness for the supplied candidate conversion. It requires every positive-balance account observed in this capture to have its own proven conversion at its own full amount, so a bounded stress sample can never satisfy it.".into(),
        asset_mint: asset_mint.clone(),
        scenario_sha256: scenario_sha256.clone(),
        evaluated_scope: EvaluatedScope::PopulationRolloutReadiness,
        evidence_manifest: ArtifactRef {
            file: format!("stress:{}", plan.stress_id),
            sha256: plan.population_capture_sha256.clone(),
        },
        requirements: vec![requirement(
            "population-conversion-coverage",
            "Every positive-balance account observed in this capture has its own proven candidate conversion",
            RequirementTarget::Rollout,
            RequirementCondition::PopulationConversionCoverage {
                plan_sha256: plan.candidate_plan_sha256.clone(),
                program_sha256: plan.candidate_program_sha256.clone(),
            },
            "Whole observed population; bounded sampled evidence explicitly does not satisfy it",
            "Execute and reconcile the candidate conversion for every positive-balance account, or narrow the rollout to the exact accounts that were proven",
        )],
    };
    let population_report = evaluate(
        &population_policy,
        &build_evidence(population_evidence(observation, &proven)?),
    )?;

    let stress_value = json!({
        "scope": "ConversionStressReadiness",
        "question": "Does this supplied candidate conversion plan also work against the other production states that currently exist around this token?",
        "status": stress_report.overall_status,
        "policy": stress_report.policy,
        "policy_sha256": stress_report.policy_semantic_sha256,
        "findings": stress_report.findings,
        "entity_readiness": stress_report.entity_readiness,
        "prevented_rollout_conditions": stress_report.prevented_rollout_conditions,
        "lifecycle_event_selected": false,
        "official_transition_established": false,
        "population_readiness": null,
        "authorization": false,
        "not_asset_safety": "Ready under this policy means its declared requirements held for the exact cases executed. It is not a safety, value or entitlement judgment about the asset or about any untested account.",
    });
    let population_value = json!({
        "scope": "PopulationRolloutReadiness",
        "question": "Is the whole observed positive-balance population ready for this candidate conversion?",
        "status": population_report.overall_status,
        "policy": population_report.policy,
        "policy_sha256": population_report.policy_semantic_sha256,
        "findings": population_report.findings,
        "rollout_readiness": population_report.rollout_readiness,
        "population_summary": population_report.population_summary,
        "authorization": false,
        "note": "Evaluated separately from the stress policy and never satisfied by a bounded sample. The single-account CandidatePlanReadiness and SelectedEntityReadiness gates are unchanged by this run.",
    });
    Ok((stress_value, population_value))
}
