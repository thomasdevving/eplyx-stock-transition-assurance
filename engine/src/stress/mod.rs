//! Bounded production-state conversion stress testing.
//!
//! One operator-supplied candidate conversion plan is executed against a bounded,
//! deterministically selected set of exact accounts drawn from a freshly captured
//! current population. A stress test answers "does this candidate plan also work
//! against the other production states that exist around this token right now?"
//! and it answers that question only for the exact entities it actually executed.
//!
//! Three separations are structural here and must stay that way:
//!
//! * A state shape prioritizes and describes tests. It is never a proof
//!   equivalence class: evidence belongs to one exact entity, amount, case plan,
//!   candidate program build and captured bank.
//! * Token-account enumeration completeness and authority-resolution completeness
//!   are independent facts. Not resolving every authority model does not mean the
//!   token accounts themselves were incompletely enumerated.
//! * Stress readiness is a finding under one explicit demonstration policy. It is
//!   not population-wide readiness and it is never an asset safety judgment.
pub mod authority;
pub mod classify;
pub mod execute;
pub mod population;
pub mod readiness;
pub mod select;
#[cfg(test)]
mod tests;

use crate::{
    conversion::ConversionPlan,
    expansion::{digest, Eligibility},
    lifecycle::{AuthorityObservation, EntityType, EvidenceRef},
    resolution::PathStatus,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Whether the token-account scan itself observed everything the query asked for.
///
/// This is about the `getProgramAccounts` enumeration only. Authority resolution
/// has its own independent axis; hitting the authority budget never downgrades
/// this value, because "we do not know how many token accounts exist" and "we
/// have every token account but not every authority model" are different facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EnumerationCompleteness {
    /// The scan returned, every row verified its mint and runtime program owner,
    /// and no response or decode bound was reached. Complete for this exact query
    /// at this context, and never a claim about any later slot.
    CompleteForQuery,
    /// The scan returned but a declared bound was reached, or some rows could not
    /// be verified or decoded. The trustworthy subset is retained.
    Partial,
    /// The provider errored, timed out or rate-limited the scan.
    Unavailable,
    /// The provider or the asset's token program cannot serve this enumeration.
    Unsupported,
}
impl EnumerationCompleteness {
    pub fn key(self) -> &'static str {
        match self {
            Self::CompleteForQuery => "CompleteForQuery",
            Self::Partial => "Partial",
            Self::Unavailable => "Unavailable",
            Self::Unsupported => "Unsupported",
        }
    }
}
/// Whether every recorded authority of the positive-balance population had its
/// own account inspected. Independent of [`EnumerationCompleteness`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuthorityResolutionCompleteness {
    Complete,
    Partial,
    NotPerformed,
}
impl AuthorityResolutionCompleteness {
    pub fn key(self) -> &'static str {
        match self {
            Self::Complete => "Complete",
            Self::Partial => "Partial",
            Self::NotPerformed => "NotPerformed",
        }
    }
}
/// Whether this entity's authority account was actually inspected in this capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityResolution {
    Resolved,
    /// Inside the population but outside the declared authority lookup budget.
    /// Unknown, never assumed wallet-compatible and never given an assumed signer.
    NotResolved,
}

/// Every bound this milestone operates under, persisted into the frozen plan so
/// no runtime or resource limit is implicit. Server-controlled: the browser can
/// request a stress test but can never raise, lower or retune a budget.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StressBudget {
    pub budget_version: String,
    /// Response byte ceiling for the population scan.
    pub max_response_bytes: u64,
    /// Token-account rows this run will decode. Sized for known large current
    /// populations; reaching it makes enumeration Partial rather than silent.
    pub max_decoded_accounts: usize,
    /// Distinct positive-balance authorities whose accounts are inspected.
    pub max_authority_lookups: usize,
    pub authority_batch_size: usize,
    pub max_selected_cases: usize,
    pub executions_per_case: usize,
    pub rpc_requests_per_case: usize,
    pub max_concurrent_rpc_requests: usize,
    pub max_concurrent_vm_executions: usize,
    pub population_timeout_seconds: u64,
    pub case_timeout_seconds: u64,
    pub max_artifact_bytes: u64,
}
pub const BUDGET_VERSION: &str = "eplyx-conversion-stress-budget/v1";
impl Default for StressBudget {
    fn default() -> Self {
        Self {
            budget_version: BUDGET_VERSION.into(),
            max_response_bytes: 128 * 1024 * 1024,
            max_decoded_accounts: 100_000,
            max_authority_lookups: 40_000,
            authority_batch_size: 100,
            max_selected_cases: 10,
            executions_per_case: 1,
            rpc_requests_per_case: 5,
            max_concurrent_rpc_requests: 1,
            max_concurrent_vm_executions: 1,
            population_timeout_seconds: 900,
            case_timeout_seconds: 120,
            max_artifact_bytes: 192 * 1024 * 1024,
        }
    }
}
impl StressBudget {
    /// Server-side overrides only, each clamped to the validated range.
    ///
    /// The browser can ask for a stress test but never reaches this: the Node
    /// service passes no budget values through, so every bound here comes from
    /// the operator's own environment or from the defaults above.
    pub fn from_env() -> Self {
        let mut b = Self::default();
        let num = |key: &str| {
            std::env::var(key)
                .ok()
                .and_then(|v| v.trim().parse::<u64>().ok())
        };
        if let Some(v) = num("EPLYX_STRESS_MAX_ACCOUNTS") {
            b.max_decoded_accounts = (v as usize).clamp(1, 500_000);
        }
        if let Some(v) = num("EPLYX_STRESS_MAX_RESPONSE_MB") {
            b.max_response_bytes = v.saturating_mul(1024 * 1024).clamp(1, 512 * 1024 * 1024);
        }
        if let Some(v) = num("EPLYX_STRESS_MAX_AUTHORITIES") {
            b.max_authority_lookups = (v as usize).min(b.max_decoded_accounts);
        }
        if let Some(v) = num("EPLYX_STRESS_MAX_CASES") {
            b.max_selected_cases = (v as usize).clamp(1, 32);
        }
        if let Some(v) = num("EPLYX_STRESS_POPULATION_TIMEOUT_SECONDS") {
            b.population_timeout_seconds = v.clamp(1, 1800);
        }
        b.max_authority_lookups = b.max_authority_lookups.min(b.max_decoded_accounts);
        // The capture file holds the scan response, so it is always allowed to be
        // larger than one response by a fixed margin.
        b.max_artifact_bytes = b.max_response_bytes.saturating_add(64 * 1024 * 1024);
        b
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.budget_version == BUDGET_VERSION,
            "unknown stress budget version"
        );
        ensure!(
            self.max_response_bytes > 0
                && self.max_response_bytes <= 512 * 1024 * 1024
                && self.max_decoded_accounts > 0
                && self.max_decoded_accounts <= 500_000
                && self.max_authority_lookups <= self.max_decoded_accounts
                && self.authority_batch_size > 0
                && self.authority_batch_size <= 100,
            "population acquisition budget out of range"
        );
        ensure!(
            self.max_selected_cases > 0
                && self.max_selected_cases <= 32
                && self.executions_per_case == 1
                && self.rpc_requests_per_case == 5,
            "unsupported stress execution budget"
        );
        ensure!(
            self.max_concurrent_rpc_requests == 1 && self.max_concurrent_vm_executions == 1,
            "this milestone executes serially by design"
        );
        ensure!(
            self.population_timeout_seconds > 0
                && self.population_timeout_seconds <= 1800
                && self.case_timeout_seconds > 0
                && self.case_timeout_seconds <= 600
                && self.max_artifact_bytes > 0,
            "stress timeout or artifact budget out of range"
        );
        Ok(())
    }
}

/// One freshly observed current token account. No human holder is inferred, a
/// zero balance is not exposure, and an unknown confidential balance is not zero.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StressEntity {
    pub entity_id: String,
    pub token_account: String,
    pub mint: String,
    pub token_program: String,
    pub authority: String,
    pub authority_model: EntityType,
    pub authority_resolution: AuthorityResolution,
    pub authority_observation: AuthorityObservation,
    pub classification_reason: String,
    pub state: crate::lifecycle::decode::TokenAccountState,
    pub token_account_evidence: EvidenceRef,
    pub authority_evidence: Option<EvidenceRef>,
}
impl StressEntity {
    pub fn balance(&self) -> Result<u64> {
        Ok(self.state.raw_balance.parse()?)
    }
}
/// A row the scan returned that this decoder could not turn into a supported
/// token account. Retained separately; never counted as a zero balance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UndecodedAccount {
    pub address: String,
    pub reason: String,
    pub raw_data_sha256: Option<String>,
    pub evidence: EvidenceRef,
}

/// Why the deterministic selector chose an exact entity. Recorded before any
/// execution, so the plan shows what each case was meant to cover.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionReason {
    /// First executable case for a distinct discovered state shape.
    NewStateShape,
    /// First executable case in a balance bucket not yet covered anywhere.
    NewBalanceBucket,
    /// Remaining budget, highest observed public balance first.
    HighestRemainingBalance,
}

/// One exact test case, frozen before capture and execution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedCase {
    pub case_id: String,
    pub selection_order: usize,
    pub selection_reason: SelectionReason,
    pub selection_detail: String,
    pub entity_id: String,
    pub token_account: String,
    pub authority: String,
    pub authority_model: EntityType,
    pub state_shape_sha256: String,
    pub shape_label: String,
    pub balance_bucket: u8,
    pub observed_balance_raw: String,
    pub discovery_slot: u64,
    pub selected_amount_raw: String,
    pub selected_amount_decimal: String,
    pub amount_policy: String,
    /// Explicit: this milestone never silently caps a selected amount.
    pub amount_capped: bool,
    /// The operator's plan with this exact source account and amount. Same terms,
    /// same mechanism, same replacement asset; a distinct digest per case.
    pub case_plan: ConversionPlan,
    pub case_plan_sha256: String,
}
pub const FULL_BALANCE_POLICY: &str = "FullObservedPublicBalance: the entire public balance observed for this exact account in the frozen population capture";
pub const FULL_AT_FINAL_POLICY: &str = "FullAtFinalCapture: freeze the selected account and policy before execution, then use its entire positive public balance from the verified final account batch";

/// A group of entities sharing execution-relevant characteristics.
///
/// Deliberately carries no status: a shape is exercised, never proven. Execution
/// outcomes live on exact cases and are joined into a separate coverage view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShapeGroup {
    pub state_shape_sha256: String,
    pub shape_label: String,
    pub dimensions: classify::ShapeDimensions,
    pub eligibility: Eligibility,
    pub eligibility_reason: String,
    pub entities_in_shape: usize,
    pub represented_raw: String,
    pub balance_buckets: BTreeMap<u8, usize>,
    /// Bounded example listing, highest observed balance first. Not a selection.
    pub highest_balance_entities: Vec<String>,
    pub entities_selected: usize,
    pub selected_entity_ids: Vec<String>,
}
pub const MAX_SHAPE_EXAMPLES: usize = 10;

/// One exact executed, or explicitly not executed, case.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaseResult {
    pub case_id: String,
    pub entity_id: String,
    pub token_account: String,
    pub authority: String,
    pub authority_model: EntityType,
    pub state_shape_sha256: String,
    pub shape_label: String,
    pub balance_bucket: u8,
    pub selection_reason: SelectionReason,
    pub selected_amount_raw: String,
    pub selected_amount_decimal: String,
    pub case_plan_sha256: String,
    pub candidate_program_sha256: String,
    pub status: PathStatus,
    pub reason: Option<String>,
    pub execution_performed: bool,
    pub local_execution_performed: bool,
    pub signer_assumed_locally: bool,
    pub signer_possession_known: bool,
    pub candidate_authority_assumed_locally: bool,
    pub issuer_binding_established: bool,
    pub official_transition: PathStatus,
    pub funds_moved: bool,
    pub execution_fixture_sha256: Option<String>,
    pub acquisition: serde_json::Value,
    pub detail: serde_json::Value,
    pub result_sha256: String,
}

/// Exact entity and amount coverage. Every denominator is stated; none of these
/// is a score and none of them may be combined into one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageSummary {
    pub accounts_observed: usize,
    pub positive_balance_accounts_observed: usize,
    pub zero_balance_accounts_observed: usize,
    pub undecodable_rows_observed: usize,
    pub exact_accounts_selected: usize,
    pub exact_accounts_executed: usize,
    pub exact_accounts_proven: usize,
    pub exact_accounts_failed: usize,
    pub exact_accounts_indeterminate: usize,
    pub exact_accounts_unsupported: usize,
    pub population_public_balance_raw: String,
    pub tested_public_balance_raw: String,
    pub proven_public_balance_raw: String,
    pub state_shapes_discovered: usize,
    pub state_shapes_executable: usize,
    pub state_shapes_with_executed_case: usize,
    pub unsupported_authority_classes: usize,
    pub unsupported_positive_balance_accounts: usize,
    pub capture_required_positive_balance_accounts: usize,
    pub note: String,
}
pub const COVERAGE_NOTE: &str = "Separate denominators over one freshly captured population. Public balances are unscaled base token units attributed to each exact entity once. Independent case outputs are never summed into rollout capacity, and no denominator is a safety score.";

/// Shape-level coverage, kept strictly separate from entity-level evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShapeCoverage {
    pub state_shape_sha256: String,
    pub shape_label: String,
    pub eligibility: Eligibility,
    pub entities_in_shape: usize,
    pub entities_selected: usize,
    pub entities_executed: usize,
    /// Exactly the entities this shape's evidence applies to. Never its members.
    pub executed_entity_ids: Vec<String>,
    pub entities_untested: usize,
    pub represented_raw: String,
    pub tested_raw: String,
    pub proof_scope: String,
}
pub const SHAPE_PROOF_SCOPE: &str = "Evidence applies only to the exact executed entity ids listed here, at their exact tested amounts, case plans, candidate program build and captured banks. Every other member of this shape is untested.";

/// A production state the current executor cannot exercise. These are product
/// findings about the rollout, never proof that the accounts cannot convert.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnsupportedState {
    pub state_shape_sha256: String,
    pub shape_label: String,
    pub eligibility: Eligibility,
    pub reason: String,
    pub positive_balance_accounts: usize,
    pub represented_raw: String,
    pub highest_balance_entities: Vec<String>,
    pub boundary: String,
}
pub const EXECUTOR_BOUNDARY: &str = "Executor and evidence boundary of the current candidate conversion adapter. It is not proof that these accounts cannot convert, and no fake wallet signer is substituted for their real authority mechanism.";

/// Structural invariants that keep sampled evidence from becoming population or
/// state-shape proof. Every one is checked in aggregation and again in offline
/// replay, so a serialized result can never assert more than it measured.
pub fn assert_no_proof_inheritance(
    selected: &[SelectedCase],
    results: &[CaseResult],
    shapes: &[ShapeCoverage],
) -> Result<()> {
    let selected_ids: BTreeSet<&str> = selected.iter().map(|c| c.entity_id.as_str()).collect();
    let selected_cases: BTreeSet<&str> = selected.iter().map(|c| c.case_id.as_str()).collect();
    ensure!(
        selected_ids.len() == selected.len() && selected_cases.len() == selected.len(),
        "a stress plan cannot select the same entity or case identity twice"
    );
    ensure!(
        results.len() == selected.len(),
        "every selected case keeps exactly one result; unsuccessful cases are never removed or replaced"
    );
    for (case, result) in selected.iter().zip(results) {
        ensure!(
            case.case_id == result.case_id
                && case.entity_id == result.entity_id
                && case.token_account == result.token_account
                && case.selected_amount_raw == result.selected_amount_raw
                && case.case_plan_sha256 == result.case_plan_sha256
                && case.state_shape_sha256 == result.state_shape_sha256,
            "a stress result is bound to its exact frozen case; results may not be re-pointed after execution"
        );
    }
    let proven: BTreeSet<&str> = results
        .iter()
        .filter(|r| r.status == PathStatus::Proven)
        .map(|r| r.entity_id.as_str())
        .collect();
    ensure!(
        proven.iter().all(|id| selected_ids.contains(id)),
        "only an exact selected, executed and reconciled entity can be Proven"
    );
    for r in results {
        if r.status == PathStatus::Proven {
            ensure!(
                r.execution_performed && r.local_execution_performed,
                "Proven requires an actual local execution of the registered candidate program"
            );
        }
        ensure!(
            r.official_transition == PathStatus::NotTested
                && !r.issuer_binding_established
                && !r.funds_moved
                && !r.signer_possession_known,
            "a candidate conversion stress case never establishes issuer binding, signer possession, fund movement or an official transition"
        );
    }
    for shape in shapes {
        ensure!(
            shape.executed_entity_ids.len() == shape.entities_executed
                && shape.entities_executed <= shape.entities_selected
                && shape.entities_selected <= shape.entities_in_shape
                && shape.entities_untested == shape.entities_in_shape - shape.entities_executed,
            "shape coverage counts must stay consistent with exact executed entities"
        );
        ensure!(
            shape
                .executed_entity_ids
                .iter()
                .all(|id| selected_ids.contains(id.as_str())),
            "a state shape cannot report evidence for entities that were never selected"
        );
        ensure!(
            shape.proof_scope == SHAPE_PROOF_SCOPE,
            "shape coverage must carry its explicit non-inheritance scope"
        );
    }
    Ok(())
}

/// Raw public balance is attributed to an exact entity id exactly once, so an
/// entity appearing in several analytical views is never counted twice.
pub fn sum_once(balances: &BTreeMap<String, u64>) -> String {
    balances
        .values()
        .fold(0u128, |a, b| a + u128::from(*b))
        .to_string()
}

/// The whole stress-test artifact. Built by aggregation and by offline replay,
/// never deserialized into existence from a stored status.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ConversionStressTestResult {
    pub schema_version: u32,
    pub kind: String,
    pub stress_id: String,
    pub run_id: String,
    pub asset_mint: String,
    pub population_capture: serde_json::Value,
    pub candidate_plan: ConversionPlan,
    pub candidate_plan_sha256: String,
    pub candidate_mechanism: serde_json::Value,
    pub classifier: serde_json::Value,
    pub selection_plan: serde_json::Value,
    pub population_summary: serde_json::Value,
    pub state_shapes: Vec<ShapeGroup>,
    pub selected_cases: Vec<SelectedCase>,
    pub results: Vec<CaseResult>,
    pub coverage_summary: CoverageSummary,
    pub shape_coverage: Vec<ShapeCoverage>,
    pub unsupported_summary: Vec<UnsupportedState>,
    pub failures: Vec<CaseResult>,
    pub readiness: serde_json::Value,
    pub official_transition: PathStatus,
    pub population_rollout_readiness: serde_json::Value,
    pub execution_performed: bool,
    pub funds_moved: bool,
    pub authorization: bool,
    pub limitations: Vec<String>,
}

pub fn entity_id(population_sha256: &str, token_account: &str) -> String {
    format!("current-stress:{population_sha256}:{token_account}")
}
pub fn result_digest<T: Serialize>(v: &T) -> Result<String> {
    digest(v)
}
