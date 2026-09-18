//! Official-mechanism discovery is distinct from generic token movement and proof.
pub mod research;

use crate::{probe::ExitPathType, resolution::PathStatus};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MechanismType {
    BurnMint,
    TransferClaim,
    Swap,
    Redemption,
    BackendMediated,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityRole {
    Holder,
    Issuer,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredSigner {
    pub address: String,
    pub role: AuthorityRole,
    pub possession_known: bool,
    pub assumed_locally: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EligibilityInput {
    KycBackend,
    PrivateEntitlement,
    BackendSignature,
    PublicStateIncomplete,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionScope {
    pub entity_id: String,
    pub mechanism_id: String,
    pub context_id: String,
    pub exact_input_raw: String,
    pub source_asset: String,
    pub destination_asset: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MechanismIdentity {
    pub issuer_semantics_bound: bool,
    pub observed_official_transaction: bool,
    pub exact_account_plan_verified: bool,
    pub source_destination_pair_observed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialTransitionMechanism {
    pub id: String,
    pub source_asset: String,
    pub destination_asset: String,
    pub mechanism_type: MechanismType,
    pub identity: MechanismIdentity,
    pub programs: Vec<String>,
    pub required_accounts: Vec<String>,
    pub required_signers: Vec<RequiredSigner>,
    pub eligibility_inputs: Vec<EligibilityInput>,
    pub onchain_evidence: Vec<String>,
    pub external_policy_evidence: Vec<String>,
    pub blocking_requirements: Vec<String>,
}
pub fn official_identity_established(m: &OfficialTransitionMechanism) -> bool {
    m.identity.issuer_semantics_bound
        && m.identity.observed_official_transaction
        && m.identity.exact_account_plan_verified
        && m.mechanism_type != MechanismType::Unknown
        && !m.programs.is_empty()
        && !m.required_accounts.is_empty()
        && !m.onchain_evidence.is_empty()
        && !m.external_policy_evidence.is_empty()
}
fn private_dependency(m: &OfficialTransitionMechanism) -> bool {
    m.required_signers
        .iter()
        .any(|s| s.role != AuthorityRole::Holder && (!s.possession_known || s.assumed_locally))
        || m.eligibility_inputs.iter().any(|i| {
            matches!(
                i,
                EligibilityInput::KycBackend
                    | EligibilityInput::PrivateEntitlement
                    | EligibilityInput::BackendSignature
            )
        })
}
/// Arithmetic alone cannot establish official mechanism identity or authorization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionTokenDeltas {
    pub source_before_raw: String,
    pub source_after_raw: String,
    pub source_effective_raw: String,
    pub source_fee_raw: String,
    pub successor_before_raw: String,
    pub successor_after_raw: String,
    pub documented_input_raw: String,
    pub documented_successor_credit_raw: String,
}
impl TransitionTokenDeltas {
    pub fn reconcile(&self) -> Result<()> {
        let n = |s: &str| -> Result<u64> {
            ensure!(
                !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()),
                "invalid raw amount"
            );
            Ok(s.parse()?)
        };
        let debit = n(&self.source_before_raw)?
            .checked_sub(n(&self.source_after_raw)?)
            .ok_or_else(|| anyhow::anyhow!("source did not debit"))?;
        let credit = n(&self.successor_after_raw)?
            .checked_sub(n(&self.successor_before_raw)?)
            .ok_or_else(|| anyhow::anyhow!("successor did not credit"))?;
        ensure!(
            debit > 0
                && credit > 0
                && debit == n(&self.documented_input_raw)?
                && credit == n(&self.documented_successor_credit_raw)?
                && u128::from(debit)
                    == u128::from(n(&self.source_effective_raw)?)
                        + u128::from(n(&self.source_fee_raw)?),
            "source/successor/documented terms do not reconcile"
        );
        Ok(())
    }
}
/// Deliberately opaque: discovery/JSON cannot construct verified execution.
/// No official execution adapter was established by this phase's investigation.
#[derive(Clone, Debug)]
pub struct VerifiedTransitionExecution {
    scope: TransitionScope,
    path_type: ExitPathType,
    vm_success: bool,
    issuer_authority_fabricated: bool,
    deltas: Option<TransitionTokenDeltas>,
    rollback_verified: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionAssessment {
    pub status: PathStatus,
    pub independent_execution_supported: bool,
    /// A bounded unsuccessful search must never produce Some(false).
    pub transition_exists_established: Option<bool>,
    pub execution_attempted: bool,
    pub reason: String,
}
pub fn assess(
    m: &OfficialTransitionMechanism,
    scope: &TransitionScope,
    execution: Option<&VerifiedTransitionExecution>,
) -> TransitionAssessment {
    let identified = official_identity_established(m);
    let private = private_dependency(m);
    let supported = identified && !private && m.eligibility_inputs.is_empty();
    let exact = execution.filter(|e| {
        e.path_type == ExitPathType::OfficialTransition
            && e.scope.entity_id == scope.entity_id
            && e.scope.mechanism_id == scope.mechanism_id
            && e.scope.context_id == scope.context_id
            && e.scope.exact_input_raw == scope.exact_input_raw
            && e.scope.source_asset == scope.source_asset
            && e.scope.destination_asset == scope.destination_asset
            && scope.mechanism_id == m.id
            && scope.source_asset == m.source_asset
            && scope.destination_asset == m.destination_asset
    });
    let (status, reason) = if !identified {
        (PathStatus::NotTested, "No independently verifiable official transition mechanism was established within the bounded investigation.")
    } else if private {
        (PathStatus::Unsupported, "Identified mechanism requires unavailable non-holder signing or private eligibility/backend state; no credential may be assumed.")
    } else if m
        .eligibility_inputs
        .contains(&EligibilityInput::PublicStateIncomplete)
    {
        (
            PathStatus::Indeterminate,
            "Identified public mechanism lacks required public execution state.",
        )
    } else if m.eligibility_inputs.contains(&EligibilityInput::Unknown) {
        (
            PathStatus::NotTested,
            "Official mechanism candidate retains unresolved eligibility requirements.",
        )
    } else if let Some(e) = exact {
        if e.issuer_authority_fabricated {
            (
                PathStatus::Unsupported,
                "Issuer authority was assumed; this cannot grant independent official proof.",
            )
        } else if !e.vm_success && e.rollback_verified {
            (
                PathStatus::Failed,
                "Exact supported official execution failed with verified rollback.",
            )
        } else if e.vm_success
            && e.deltas.as_ref().is_some_and(|d| {
                d.documented_input_raw == scope.exact_input_raw && d.reconcile().is_ok()
            })
        {
            (PathStatus::Proven, "Exact verified official execution succeeded and source/successor deltas match documented terms.")
        } else {
            (
                PathStatus::Indeterminate,
                "Execution success/failure or exact source/successor reconciliation is incomplete.",
            )
        }
    } else {
        (PathStatus::NotTested, "Official mechanism is identified but has no verified execution at the requested exact scope.")
    };
    TransitionAssessment {
        status,
        independent_execution_supported: supported,
        transition_exists_established: if identified { Some(true) } else { None },
        execution_attempted: exact.is_some(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
