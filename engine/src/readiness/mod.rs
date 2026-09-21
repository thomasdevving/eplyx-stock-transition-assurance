//! Explicit assurance policy + frozen, scope-checked evidence -> deterministic pre-flight.
//! This is separate from economic lifecycle policy and never performs execution or RPC.
pub mod current;
pub mod evidence;

use crate::{
    expansion::{canonical, digest},
    probe::ExitPathType,
    resolution::{ArtifactRef, PathStatus, SignerAssumption},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadinessStatus {
    Ready,
    Blocked,
    Incomplete,
}
impl ReadinessStatus {
    pub fn exit_code(self) -> u8 {
        match self {
            Self::Ready => 0,
            Self::Blocked => 3,
            Self::Incomplete => 4,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingEffect {
    Blocking,
    IncompleteEvidence,
    Satisfied,
    Informational,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluatedScope {
    DemoEntityReadiness,
    PopulationRolloutReadiness,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyType {
    DemoAssurancePolicy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateShape {
    DirectTokenAccount,
    ProtocolPosition,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceScope {
    pub asset_mint: String,
    pub entity_id: String,
    pub state_shape: StateShape,
    pub authority: String,
    pub exact_amount_raw: Option<String>,
    pub range: Option<[i32; 2]>,
    pub bps_to_remove: Option<u16>,
    pub venue: Option<String>,
    pub context_id: Option<String>,
    pub captured_slot: Option<u64>,
    pub clock: Option<crate::probe::ProbeClock>,
    pub capture_context: Option<crate::expansion::pipeline::CaptureContext>,
    pub captured_state_sha256: Option<String>,
    pub fixture_sha256: Option<String>,
    pub source_before_raw: Option<String>,
    pub scenario_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathCondition {
    pub path_type: ExitPathType,
    pub scope: EvidenceScope,
    pub accepted_statuses: Vec<PathStatus>,
    pub blocking_statuses: Vec<PathStatus>,
    pub allow_local_signer_assumption: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RequirementCondition {
    Path {
        any_of: Vec<PathCondition>,
    },
    CompletePositionExit {
        scope: Box<EvidenceScope>,
        forbid_remaining_fees: bool,
    },
    PopulationCoverage {
        any_of_paths: Vec<ExitPathType>,
    },
    /// An operator-supplied candidate conversion plan, proven by actual execution
    /// at this exact scope. Deliberately a different slot from the lifecycle path
    /// matrix: a candidate plan is never an issuer-defined official transition.
    CandidateConversion {
        scope: Box<EvidenceScope>,
        plan_sha256: String,
        program_sha256: String,
        replacement_mint: String,
    },
    EvidenceIsolation,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RequirementTarget {
    Entity { entity_id: String },
    Rollout,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadinessRequirement {
    pub id: String,
    pub label: String,
    pub target: RequirementTarget,
    pub required: bool,
    pub condition: RequirementCondition,
    pub rollout_assumption: String,
    pub remediation_requirement: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleReadinessPolicy {
    pub schema_version: u32,
    pub id: String,
    pub policy_type: PolicyType,
    pub not_issuer_policy: bool,
    pub description: String,
    pub asset_mint: String,
    pub scenario_sha256: String,
    pub evaluated_scope: EvaluatedScope,
    pub evidence_manifest: ArtifactRef,
    pub requirements: Vec<ReadinessRequirement>,
}
fn normalize_paths(v: &mut Vec<ExitPathType>) {
    v.sort_by_key(|p| crate::resolution::path_name(*p));
    v.dedup();
}
impl LifecycleReadinessPolicy {
    pub fn normalized(&self) -> Result<Self> {
        let mut p = self.clone();
        ensure!(
            p.schema_version == 1
                && p.not_issuer_policy
                && !p.id.is_empty()
                && !p.description.is_empty(),
            "explicit non-issuer demonstration assurance policy required"
        );
        ensure!(
            p.requirements.iter().any(|r| r.required),
            "assurance policy requires a nonempty required scope"
        );
        p.requirements.sort_by(|a, b| a.id.cmp(&b.id));
        ensure!(
            p.requirements
                .iter()
                .map(|r| &r.id)
                .collect::<BTreeSet<_>>()
                .len()
                == p.requirements.len(),
            "duplicate requirement id"
        );
        for r in &mut p.requirements {
            ensure!(
                !r.id.is_empty()
                    && !r.label.is_empty()
                    && !r.rollout_assumption.is_empty()
                    && !r.remediation_requirement.is_empty(),
                "unexplained assurance requirement"
            );
            match &mut r.condition {
                RequirementCondition::Path { any_of } => {
                    ensure!(!any_of.is_empty(), "empty path alternatives");
                    for c in any_of.iter_mut() {
                        ensure!(
                            c.scope.asset_mint == p.asset_mint
                                && c.scope.scenario_sha256 == p.scenario_sha256,
                            "requirement asset/scenario mismatch"
                        );
                        ensure!(
                            matches!(&r.target,RequirementTarget::Entity{entity_id} if *entity_id==c.scope.entity_id),
                            "path target/entity mismatch"
                        );
                        ensure!(
                            c.path_type != ExitPathType::Unknown
                                && c.accepted_statuses == [PathStatus::Proven],
                            "this assurance gate accepts independently measured Proven paths only"
                        );
                        ensure!(
                            c.blocking_statuses.iter().all(|s| *s != PathStatus::Proven),
                            "accepted Proven evidence cannot also be a forbidden status"
                        );
                        c.blocking_statuses.sort_by_key(|s| format!("{s:?}"));
                        c.blocking_statuses.dedup();
                        validate_scope(&c.scope)?;
                    }
                    any_of.sort_by_cached_key(|c| canonical(c).expect("serializable condition"));
                    any_of.dedup();
                }
                RequirementCondition::CompletePositionExit { scope, .. } => {
                    validate_scope(scope)?;
                    ensure!(
                        scope.asset_mint == p.asset_mint
                            && scope.scenario_sha256 == p.scenario_sha256
                            && scope.state_shape == StateShape::ProtocolPosition
                            && matches!(&r.target,RequirementTarget::Entity{entity_id} if *entity_id==scope.entity_id),
                        "complete-exit target/scope mismatch"
                    );
                }
                RequirementCondition::PopulationCoverage { any_of_paths } => {
                    ensure!(
                        r.target == RequirementTarget::Rollout
                            && !any_of_paths.is_empty()
                            && !any_of_paths.contains(&ExitPathType::Unknown),
                        "invalid population requirement"
                    );
                    normalize_paths(any_of_paths);
                }
                RequirementCondition::CandidateConversion {
                    scope,
                    plan_sha256,
                    program_sha256,
                    replacement_mint,
                } => {
                    validate_scope(scope)?;
                    ensure!(
                        scope.asset_mint == p.asset_mint
                            && scope.scenario_sha256 == p.scenario_sha256
                            && matches!(&r.target,RequirementTarget::Entity{entity_id} if *entity_id==scope.entity_id),
                        "candidate-conversion target/scope mismatch"
                    );
                    ensure!(
                        plan_sha256.len() == 64
                            && program_sha256.len() == 64
                            && !replacement_mint.is_empty()
                            && *replacement_mint != p.asset_mint,
                        "a candidate-conversion requirement must pin its plan, candidate program and distinct replacement asset"
                    );
                }
                RequirementCondition::EvidenceIsolation => {}
            }
        }
        if p.evaluated_scope == EvaluatedScope::PopulationRolloutReadiness {
            ensure!(
                p.requirements.iter().any(|r| r.required
                    && matches!(r.condition, RequirementCondition::PopulationCoverage { .. })),
                "population rollout policy must explicitly require population coverage"
            );
        } else {
            ensure!(
                p.requirements
                    .iter()
                    .all(|r| r.target != RequirementTarget::Rollout),
                "entity-only policy cannot imply rollout readiness"
            );
        }
        Ok(p)
    }
}
fn validate_scope(s: &EvidenceScope) -> Result<()> {
    ensure!(
        !s.entity_id.is_empty() && !s.authority.is_empty(),
        "exact entity and authority required"
    );
    for n in [&s.exact_amount_raw, &s.source_before_raw]
        .into_iter()
        .flatten()
    {
        let v = n.parse::<u64>()?;
        ensure!(v.to_string() == *n, "noncanonical raw amount");
    }
    if let Some(clock) = &s.clock {
        ensure!(
            s.captured_slot == Some(clock.slot),
            "scope clock/slot mismatch"
        );
    }
    if let Some(r) = s.range {
        ensure!(r[0] <= r[1], "invalid exact range");
    }
    if let Some(b) = s.bps_to_remove {
        ensure!(b > 0 && b <= 10000, "invalid exact fraction");
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceReference {
    pub id: String,
    pub artifact: ArtifactRef,
    pub description: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathFact {
    pub scope: EvidenceScope,
    pub path_type: ExitPathType,
    pub status: PathStatus,
    pub execution_attempted: bool,
    pub reconciled: bool,
    pub rollback_verified: Option<bool>,
    pub signer: SignerAssumption,
    pub evidence_ids: Vec<String>,
    pub reason: String,
}
/// One executed operator-supplied candidate conversion, with every distinction
/// this gate must not blur: provenance, digests, assumed authorities and the
/// separate question of issuer binding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionFact {
    pub scope: EvidenceScope,
    pub status: PathStatus,
    pub provenance: crate::conversion::PlanProvenance,
    pub plan_sha256: String,
    pub program_sha256: String,
    pub replacement_mint: String,
    pub destination: String,
    pub execution_attempted: bool,
    pub reconciled: bool,
    pub rollback_verified: Option<bool>,
    pub holder_signer: SignerAssumption,
    pub candidate_authority_assumed_locally: bool,
    pub issuer_binding_established: bool,
    pub evidence_ids: Vec<String>,
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompleteExitFact {
    pub scope: EvidenceScope,
    pub principal_unwind: PathStatus,
    pub fee_collection: PathStatus,
    pub position_closure: PathStatus,
    pub residual_fees_raw: BTreeMap<String, String>,
    pub principal_removed_raw: BTreeMap<String, String>,
    pub owner_received_raw: BTreeMap<String, String>,
    pub destination_withheld_raw: BTreeMap<String, String>,
    pub evidence_ids: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PopulationEntityFact {
    pub entity_id: String,
    pub account_type: String,
    pub balance_raw: String,
    pub proven_full_amount_paths: Vec<ExitPathType>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PopulationEvidence {
    pub token_account_entities: usize,
    pub positive_balance_entities: usize,
    pub distinct_owner_authorities: usize,
    pub entities_with_measured_amount: usize,
    pub represented_amount_raw: String,
    pub covered_amount_raw: String,
    pub without_evidence_raw: String,
    pub account_types: BTreeMap<String, crate::coverage::CoverageAggregate>,
    pub execution_status_counts: BTreeMap<String, usize>,
    pub evidence_ids: Vec<String>,
    pub entities: Vec<PopulationEntityFact>,
}
/// A published measurement is consumed only after input digests and scope relationships are checked.
/// No public/deserialization constructor; readiness cannot grant new execution proof.
pub struct VerifiedReadinessEvidence {
    pub(super) asset_mint: String,
    pub(super) scenario_sha256: String,
    pub(super) lifecycle_event: crate::lifecycle::policy::AssetLifecyclePolicy,
    pub(super) policy_evaluated_at: String,
    pub(super) path_facts: Vec<PathFact>,
    pub(super) conversion_facts: Vec<ConversionFact>,
    pub(super) complete_exits: Vec<CompleteExitFact>,
    pub(super) population: PopulationEvidence,
    pub(super) evidence_refs: Vec<EvidenceReference>,
    pub(super) isolation_verified: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessFinding {
    pub requirement_id: String,
    pub label: String,
    pub required: bool,
    pub scope: RequirementTarget,
    pub effect: FindingEffect,
    pub expected: String,
    pub observed: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub reason: String,
    pub remediation_requirement: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightFailureMode {
    pub requirement_id: String,
    pub scope: RequirementTarget,
    pub assumption: String,
    pub observed: Vec<String>,
    pub effect: FindingEffect,
    pub evidence_ids: Vec<String>,
    pub preflight_effect: String,
    pub real_incident_claimed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityReadiness {
    pub entity_id: String,
    pub status: ReadinessStatus,
    pub requirement_ids: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PopulationReadiness {
    pub status: ReadinessStatus,
    pub positive_entities_required: usize,
    pub exact_entities_satisfied: usize,
    pub exhaustive_execution: bool,
    pub unresolved_account_types: BTreeMap<String, usize>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleReadinessReport {
    pub schema_version: u32,
    pub policy: LifecycleReadinessPolicy,
    pub policy_semantic_sha256: String,
    pub asset_mint: String,
    pub lifecycle_event: crate::lifecycle::policy::AssetLifecyclePolicy,
    pub policy_evaluated_at: String,
    pub evaluated_scope: EvaluatedScope,
    pub overall_status: ReadinessStatus,
    pub requirements: Vec<ReadinessRequirement>,
    pub findings: Vec<ReadinessFinding>,
    pub entity_readiness: Vec<EntityReadiness>,
    pub rollout_readiness: Option<PopulationReadiness>,
    pub path_evidence: Vec<PathFact>,
    pub position_exit_evidence: Vec<CompleteExitFact>,
    pub population_summary: PopulationSummary,
    pub prevented_rollout_conditions: Vec<PreflightFailureMode>,
    pub evidence_refs: Vec<EvidenceReference>,
    pub limitations: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PopulationSummary {
    pub token_account_entities: usize,
    pub positive_balance_entities: usize,
    pub distinct_owner_authorities: usize,
    pub entities_with_measured_amount: usize,
    pub represented_amount_raw: String,
    pub covered_amount_raw: String,
    pub without_evidence_raw: String,
    pub account_types: BTreeMap<String, crate::coverage::CoverageAggregate>,
    pub execution_status_counts: BTreeMap<String, usize>,
}
fn aggregate_effects<'a>(effects: impl Iterator<Item = &'a FindingEffect>) -> ReadinessStatus {
    let v = effects.copied().collect::<Vec<_>>();
    if v.contains(&FindingEffect::Blocking) {
        ReadinessStatus::Blocked
    } else if v.contains(&FindingEffect::IncompleteEvidence) {
        ReadinessStatus::Incomplete
    } else {
        ReadinessStatus::Ready
    }
}
fn matches_scope(f: &EvidenceScope, c: &EvidenceScope) -> bool {
    f == c
}
fn accepted_path(f: &PathFact, c: &PathCondition) -> bool {
    c.accepted_statuses.contains(&f.status)
        && f.execution_attempted
        && f.reconciled
        && f.signer.authority == c.scope.authority
        && (f.signer.signer_possession_known
            || (c.allow_local_signer_assumption && f.signer.signer_assumed_locally))
}
fn population_satisfied(p: &PopulationEvidence, paths: &[ExitPathType]) -> usize {
    p.entities
        .iter()
        .filter(|e| {
            e.balance_raw != "0"
                && e.proven_full_amount_paths
                    .iter()
                    .any(|path| paths.contains(path))
        })
        .count()
}
fn evaluate_requirement(
    r: &ReadinessRequirement,
    e: &VerifiedReadinessEvidence,
) -> Result<ReadinessFinding> {
    let mut observed = vec![];
    let mut ids = vec![];
    let (effect, expected, reason) = match &r.condition {
        RequirementCondition::Path { any_of } => {
            let mut satisfied = false;
            let mut blocking = false;
            for c in any_of {
                let candidates = e
                    .path_facts
                    .iter()
                    .filter(|f| {
                        f.scope.asset_mint == c.scope.asset_mint && f.path_type == c.path_type
                    })
                    .collect::<Vec<_>>();
                for f in candidates {
                    let exact = matches_scope(&f.scope, &c.scope);
                    observed.push(format!(
                        "{:?} {:?}; entity {}; context {:?}; amount {:?}; exact scope {}. {}",
                        f.path_type,
                        f.status,
                        f.scope.entity_id,
                        f.scope.context_id,
                        f.scope.exact_amount_raw,
                        exact,
                        f.reason
                    ));
                    ids.extend(f.evidence_ids.clone());
                    if exact {
                        satisfied |= accepted_path(f, c);
                        blocking |= c.blocking_statuses.contains(&f.status)
                            && (f.status != PathStatus::Failed
                                || (f.execution_attempted && f.rollback_verified == Some(true)));
                    }
                }
            }
            let effect = if satisfied {
                FindingEffect::Satisfied
            } else if blocking {
                FindingEffect::Blocking
            } else {
                FindingEffect::IncompleteEvidence
            };
            (effect,"At least one declared exact path must be Proven, reconciled and compatible with the policy's signer assumptions.","Path identity, asset/entity/authority/amount/range/venue/fixture/bank/scenario must match; missing or incompatible proof is not execution failure.")
        }
        RequirementCondition::CompletePositionExit {
            scope,
            forbid_remaining_fees,
        } => {
            let fact = e
                .complete_exits
                .iter()
                .find(|f| matches_scope(&f.scope, scope));
            let effect = if let Some(f) = fact {
                observed.push(format!("Principal unwind {:?}; fee collection {:?}; position closure {:?}; residual fee ledger {:?}.",f.principal_unwind,f.fee_collection,f.position_closure,f.residual_fees_raw));
                ids.extend(f.evidence_ids.clone());
                let residual = f.residual_fees_raw.values().any(|v| v != "0");
                if *forbid_remaining_fees && residual {
                    FindingEffect::Blocking
                } else if f.principal_unwind == PathStatus::Proven
                    && f.fee_collection == PathStatus::Proven
                    && f.position_closure == PathStatus::Proven
                    && !residual
                {
                    FindingEffect::Satisfied
                } else {
                    FindingEffect::IncompleteEvidence
                }
            } else {
                FindingEffect::IncompleteEvidence
            };
            (effect,"Exact complete position exit requires proven principal unwind, proven fee collection/closure and no residual fee exposure.","Native principal withdrawal cannot establish unexecuted fee collection or position closure; residual exposure blocks only when explicitly forbidden by policy.")
        }
        RequirementCondition::PopulationCoverage { any_of_paths } => {
            let n = population_satisfied(&e.population, any_of_paths);
            observed.push(format!("{} of {} positive token-account entities have full represented-amount evidence on the requested paths; {} total observed accounts.",n,e.population.positive_balance_entities,e.population.token_account_entities));
            ids.extend(e.population.evidence_ids.clone());
            (if n==e.population.positive_balance_entities{FindingEffect::Satisfied}else{FindingEffect::IncompleteEvidence},"Every positive observed token-account entity must have its own full represented-amount proof on a requested path.","Sampled entities, protocol observations and venue reserves cannot establish peer-holder or population-wide actionability.")
        }
        RequirementCondition::CandidateConversion {
            scope,
            plan_sha256,
            program_sha256,
            replacement_mint,
        } => {
            let mut satisfied = false;
            for f in e
                .conversion_facts
                .iter()
                .filter(|f| f.scope.asset_mint == scope.asset_mint)
            {
                let exact = matches_scope(&f.scope, scope)
                    && f.plan_sha256 == *plan_sha256
                    && f.program_sha256 == *program_sha256
                    && f.replacement_mint == *replacement_mint;
                observed.push(format!(
                    "Candidate conversion {:?} under {:?} provenance; entity {}; amount {:?}; replacement {}; plan {}; candidate program {}; exact scope {}. Issuer binding established: {}. {}",
                    f.status, f.provenance, f.scope.entity_id, f.scope.exact_amount_raw,
                    f.replacement_mint, f.plan_sha256, f.program_sha256, exact,
                    f.issuer_binding_established, f.reason
                ));
                ids.extend(f.evidence_ids.clone());
                if exact {
                    satisfied |= f.status == PathStatus::Proven
                        && f.provenance == crate::conversion::PlanProvenance::OperatorSupplied
                        && f.execution_attempted
                        && f.reconciled
                        && (f.holder_signer.signer_possession_known
                            || f.holder_signer.signer_assumed_locally);
                }
            }
            (if satisfied{FindingEffect::Satisfied}else{FindingEffect::IncompleteEvidence},
             "The declared candidate conversion plan must have actually executed and reconciled at this exact account, amount, replacement asset, plan version and candidate program build.",
             "This is evidence about the supplied candidate plan under its declared authority model only. It never establishes an issuer-defined official transition, an issuer relationship or any authorization.")
        }
        RequirementCondition::EvidenceIsolation => {
            observed.push(format!(
                "Evidence identity/scope isolation verified: {}.",
                e.isolation_verified
            ));
            ids.extend(e.evidence_refs.iter().map(|r| r.id.clone()));
            (if e.isolation_verified{FindingEffect::Satisfied}else{FindingEffect::IncompleteEvidence},"No path/entity/venue/amount/fee/closure proof escalation.","The gate consumes only pinned measurements with explicit source and scope bindings; no readiness finding grants new execution evidence.")
        }
    };
    observed.sort();
    observed.dedup();
    if observed.is_empty() {
        observed.push("No matching scoped evidence.".into());
    }
    ids.sort();
    ids.dedup();
    Ok(ReadinessFinding {
        requirement_id: r.id.clone(),
        label: r.label.clone(),
        required: r.required,
        scope: r.target.clone(),
        effect: if r.required {
            effect
        } else {
            FindingEffect::Informational
        },
        expected: expected.into(),
        observed,
        evidence_ids: ids,
        reason: reason.into(),
        remediation_requirement: r.remediation_requirement.clone(),
    })
}
pub fn evaluate(
    policy: &LifecycleReadinessPolicy,
    e: &VerifiedReadinessEvidence,
) -> Result<LifecycleReadinessReport> {
    let policy = policy.normalized()?;
    ensure!(
        policy.asset_mint == e.asset_mint && policy.scenario_sha256 == e.scenario_sha256,
        "assurance policy/evidence asset or economic scenario mismatch"
    );
    let findings = policy
        .requirements
        .iter()
        .map(|r| evaluate_requirement(r, e))
        .collect::<Result<Vec<_>>>()?;
    let overall_status =
        aggregate_effects(findings.iter().filter(|f| f.required).map(|f| &f.effect));
    let entity_ids = policy
        .requirements
        .iter()
        .filter(|r| r.required)
        .filter_map(|r| match &r.target {
            RequirementTarget::Entity { entity_id } => Some(entity_id.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let entity_readiness = entity_ids
        .into_iter()
        .map(|id| {
            let rows = findings
                .iter()
                .filter(|f| {
                    f.scope
                        == RequirementTarget::Entity {
                            entity_id: id.clone(),
                        }
                })
                .collect::<Vec<_>>();
            EntityReadiness {
                entity_id: id,
                status: aggregate_effects(rows.iter().filter(|f| f.required).map(|f| &f.effect)),
                requirement_ids: rows.iter().map(|f| f.requirement_id.clone()).collect(),
            }
        })
        .collect();
    let rollout_readiness = if policy.evaluated_scope == EvaluatedScope::PopulationRolloutReadiness
    {
        let pop_conditions = policy
            .requirements
            .iter()
            .filter_map(|r| match &r.condition {
                RequirementCondition::PopulationCoverage { any_of_paths } if r.required => {
                    Some(any_of_paths)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let n = e
            .population
            .entities
            .iter()
            .filter(|entity| {
                entity.balance_raw != "0"
                    && pop_conditions.iter().all(|paths| {
                        entity
                            .proven_full_amount_paths
                            .iter()
                            .any(|p| paths.contains(p))
                    })
            })
            .count();
        Some(PopulationReadiness {
            status: overall_status,
            positive_entities_required: e.population.positive_balance_entities,
            exact_entities_satisfied: n,
            exhaustive_execution: n == e.population.positive_balance_entities,
            unresolved_account_types: {
                let mut counts = BTreeMap::new();
                for entity in &e.population.entities {
                    if entity.balance_raw != "0"
                        && !pop_conditions.iter().all(|paths| {
                            entity
                                .proven_full_amount_paths
                                .iter()
                                .any(|p| paths.contains(p))
                        })
                    {
                        *counts.entry(entity.account_type.clone()).or_insert(0) += 1;
                    }
                }
                counts
            },
        })
    } else {
        None
    };
    let prevented_rollout_conditions=findings.iter().filter(|f|f.effect!=FindingEffect::Satisfied).map(|f|{
        let r=policy.requirements.iter().find(|r|r.id==f.requirement_id).unwrap();
        PreflightFailureMode{requirement_id:r.id.clone(),scope:r.target.clone(),assumption:r.rollout_assumption.clone(),observed:f.observed.clone(),effect:f.effect,evidence_ids:f.evidence_ids.clone(),preflight_effect:"Eplyx would prevent a rollout from relying on this unverified, incompatible or contradicted assumption. This report is not a claim that a real rollout was blocked or a loss prevented.".into(),real_incident_claimed:false}
    }).collect();
    let mut path_evidence = e.path_facts.clone();
    path_evidence.sort_by_cached_key(|f| canonical(f).expect("serializable fact"));
    let mut complete = e.complete_exits.clone();
    complete.sort_by(|a, b| a.scope.entity_id.cmp(&b.scope.entity_id));
    let mut refs = e.evidence_refs.clone();
    refs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(LifecycleReadinessReport{schema_version:1,policy_semantic_sha256:digest(&policy)?,asset_mint:e.asset_mint.clone(),lifecycle_event:e.lifecycle_event.clone(),policy_evaluated_at:e.policy_evaluated_at.clone(),evaluated_scope:policy.evaluated_scope,overall_status,requirements:policy.requirements.clone(),policy,findings,entity_readiness,rollout_readiness,path_evidence,position_exit_evidence:complete,
        population_summary:PopulationSummary{token_account_entities:e.population.token_account_entities,positive_balance_entities:e.population.positive_balance_entities,distinct_owner_authorities:e.population.distinct_owner_authorities,entities_with_measured_amount:e.population.entities_with_measured_amount,represented_amount_raw:e.population.represented_amount_raw.clone(),covered_amount_raw:e.population.covered_amount_raw.clone(),without_evidence_raw:e.population.without_evidence_raw.clone(),account_types:e.population.account_types.clone(),execution_status_counts:e.population.execution_status_counts.clone()},prevented_rollout_conditions,evidence_refs:refs,
        limitations:vec!["This is a demonstration assurance policy, not an issuer's internal rollout policy, a subjective risk score or an asset safety judgment.".into(),"No VM execution or RPC is performed by the readiness gate. It verifies declared published measurement digests and bindings, then evaluates scoped evidence; it creates no new execution proof.".into(),"Ready means only that this policy's required conditions are met under its declared captured-state/signer assumptions; key possession, freshness/inclusion and future liquidity remain unproved.".into(),"The two state shapes retain their distinct captured banks. Entity actionability, population assurance, issuer lifecycle completion, principal unwind and full position exit are separate conclusions.".into(),"Earlier population adapter Unsupported statuses do not prove issuer/private blockers. Missing scoped evidence remains Incomplete unless an explicit policy blocker is evidenced.".into()]})
}
impl LifecycleReadinessReport {
    pub fn to_json(&self) -> Result<String> {
        canonical(self)
    }
    pub fn render_text(&self) -> String {
        let mut s=format!("LIFECYCLE PRE-FLIGHT\nAsset: {}\nPolicy: {} (DemoAssurancePolicy; not issuer policy)\nEvaluated scope: {:?}\nOverall readiness: {:?}\n\n",self.asset_mint,self.policy.id,self.evaluated_scope,self.overall_status);
        for f in &self.findings {
            s.push_str(&format!(
                "{:?}: {} [{}]\n",
                f.effect, f.label, f.requirement_id
            ));
            let requirement = self
                .requirements
                .iter()
                .find(|r| r.id == f.requirement_id)
                .expect("finding requirement");
            if let RequirementCondition::Path { any_of } = &requirement.condition {
                for c in any_of {
                    let exact = self
                        .path_evidence
                        .iter()
                        .find(|p| p.path_type == c.path_type && matches_scope(&p.scope, &c.scope));
                    let observed = exact.or_else(|| {
                        self.path_evidence.iter().find(|p| {
                            p.path_type == c.path_type && p.scope.entity_id == c.scope.entity_id
                        })
                    });
                    if let Some(p) = observed {
                        s.push_str(&format!(
                            "  {:?}: {:?}; requested raw {:?}; exact scope {}; bank {:?}.\n",
                            p.path_type,
                            p.status,
                            c.scope.exact_amount_raw,
                            exact.is_some(),
                            p.scope.captured_slot
                        ));
                        if exact.is_none() || p.status == PathStatus::Failed {
                            s.push_str(&format!("  {}\n", p.reason));
                        }
                    } else {
                        s.push_str("  No matching scoped evidence.\n");
                    }
                }
            } else {
                for o in &f.observed {
                    s.push_str(&format!("  {o}\n"));
                }
            }
        }
        s.push_str(&format!(
            "\nPopulation: {} accounts; {} positive; {} entities with measured amount.\n",
            self.population_summary.token_account_entities,
            self.population_summary.positive_balance_entities,
            self.population_summary.entities_with_measured_amount
        ));
        for f in &self.position_exit_evidence {
            s.push_str(&format!("Principal removed by mint (raw): {:?}; public owner credit: {:?}; destination withheld: {:?}.\n",f.principal_removed_raw,f.owner_received_raw,f.destination_withheld_raw));
            s.push_str(&format!(
                "Position {}: principal {:?}; fees {:?}; closure {:?}; remaining {:?}.\n",
                f.scope.entity_id,
                f.principal_unwind,
                f.fee_collection,
                f.position_closure,
                f.residual_fees_raw
            ));
        }
        s.push_str(
            "\nNo real incident/loss is claimed. No mainnet action or new execution occurs.\n",
        );
        s
    }
}
#[cfg(test)]
mod tests;
