//! Operator-supplied candidate conversion plans.
//!
//! An operator states the conversion they intend to deploy; Eplyx executes exactly
//! that candidate mechanism against freshly captured current holder state. A proven
//! candidate conversion is evidence about the supplied plan under its declared
//! authority model, never evidence that an issuer defined, authorized or controls
//! that mechanism. OfficialTransition is a separate fact with separate requirements.
pub mod coherence;
pub mod current;
pub mod demo;
pub mod invariants;
pub mod package;
pub mod package_gate;
pub mod package_preflight;
pub mod search;
#[cfg(test)]
mod tests;

use crate::expansion::{canonical, digest};
use anyhow::{ensure, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_address::Address;

/// Where a lifecycle/conversion claim comes from. Only the operator-supplied class
/// has an executable adapter; an issuer-bound class would additionally need
/// independent evidence that this exact mechanism is the issuer's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanProvenance {
    /// A holder/visitor hypothesis about a future transition. Not executable.
    UserProposed,
    /// A candidate mechanism and configuration the operator intends to deploy.
    OperatorSupplied,
    /// An issuer-attested mechanism. No adapter and no accepted evidence exists here.
    IssuerVerified,
    /// A mechanism independently established from public issuer artifacts. None exists.
    PublicIssuerMechanism,
}
/// Registered candidate mechanisms. There is exactly one, and it ships with the
/// repository; no uploaded or browser-supplied program can be executed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MechanismId {
    EplyxDemoCandidateConversion,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rounding {
    Floor,
    Ceiling,
}
impl Rounding {
    pub fn code(self) -> u8 {
        match self {
            Self::Floor => 0,
            Self::Ceiling => 1,
        }
    }
}
/// How the old token leaves the holder under this candidate mechanism.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceConsumption {
    Burn,
}
/// How the replacement token reaches the holder under this candidate mechanism.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplacementDelivery {
    /// Release from a proposed candidate reserve. No issuer mint authority is used.
    ProposedReserveRelease,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateAuthority {
    /// A program-derived address of the candidate program, assumed locally.
    ProgramDerived,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmountMode {
    Full,
    Custom,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversionTerms {
    pub ratio_numerator: u64,
    pub ratio_denominator: u64,
    pub rounding: Rounding,
    /// Taken from the consumed source amount before the ratio. Distinct from any
    /// Token-2022 transfer fee on either asset.
    pub conversion_fee_bps: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityModel {
    pub holder_signs: bool,
    pub candidate_authority: CandidateAuthority,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReserveConfig {
    /// Raw replacement-token units the operator proposes to fund the reserve with.
    pub funded_replacement_raw: String,
}
/// The whole bounded, serializable conversion plan. No scripts, no account metas,
/// no transaction bytes and no program bytes cross this boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversionPlan {
    pub schema_version: u32,
    pub id: String,
    pub version: u32,
    pub provenance: PlanProvenance,
    pub mechanism: MechanismId,
    pub adapter_id: String,
    /// Operator-declared reference for the mechanism they intend to deploy.
    pub mechanism_ref: String,
    pub source_mint: String,
    pub replacement_mint: String,
    pub source_account: String,
    pub amount_mode: AmountMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_decimal: Option<String>,
    pub terms: ConversionTerms,
    pub authority_model: AuthorityModel,
    pub source_consumption: SourceConsumption,
    pub replacement_delivery: ReplacementDelivery,
    pub reserve: ReserveConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<DateTime<Utc>>,
}
pub const ADAPTER_ID: &str = "eplyx-demo-candidate-conversion/v1";

impl ConversionPlan {
    /// Structural validation only. It establishes nothing about the current chain.
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == 1 && self.version >= 1,
            "unsupported conversion plan schema"
        );
        ensure!(
            !self.id.is_empty() && self.id.len() <= 64,
            "conversion plan needs a bounded identifier"
        );
        ensure!(
            self.provenance == PlanProvenance::OperatorSupplied,
            "only an OperatorSupplied candidate plan has an executable adapter; issuer-bound provenance requires independent evidence that does not exist"
        );
        ensure!(
            self.mechanism == MechanismId::EplyxDemoCandidateConversion
                && self.adapter_id == ADAPTER_ID,
            "unknown candidate mechanism; only the registered repository mechanism runs"
        );
        ensure!(
            !self.mechanism_ref.is_empty() && self.mechanism_ref.len() <= 200,
            "declare the candidate mechanism reference"
        );
        for address in [
            &self.source_mint,
            &self.replacement_mint,
            &self.source_account,
        ] {
            let _: Address = address.parse().context("invalid plan address")?;
        }
        ensure!(
            self.source_mint != self.replacement_mint,
            "the replacement mint must differ from the source mint"
        );
        ensure!(
            self.source_account != self.source_mint && self.source_account != self.replacement_mint,
            "the selected source account cannot be a mint"
        );
        ensure!(
            self.source_consumption == SourceConsumption::Burn
                && self.replacement_delivery == ReplacementDelivery::ProposedReserveRelease,
            "unsupported candidate source/replacement design"
        );
        ensure!(
            self.authority_model.holder_signs
                && self.authority_model.candidate_authority == CandidateAuthority::ProgramDerived,
            "this mechanism requires holder signing and a program-derived candidate authority; no issuer key may be assumed"
        );
        ensure!(
            self.terms.ratio_numerator > 0
                && self.terms.ratio_denominator > 0
                && self.terms.conversion_fee_bps <= 10_000,
            "conversion terms require a positive ratio and a fee within 0-10000 bps"
        );
        let funded: u64 = self
            .reserve
            .funded_replacement_raw
            .parse()
            .context("invalid proposed reserve funding")?;
        ensure!(
            funded.to_string() == self.reserve.funded_replacement_raw,
            "noncanonical proposed reserve funding"
        );
        match self.amount_mode {
            AmountMode::Full => ensure!(
                self.amount_decimal.is_none(),
                "full balance cannot include a custom amount"
            ),
            AmountMode::Custom => {
                let amount = self
                    .amount_decimal
                    .as_deref()
                    .context("custom amount required")?;
                ensure!(
                    !amount.is_empty() && amount.len() <= 280,
                    "invalid custom amount"
                );
            }
        }
        ensure!(
            self.deadline.is_none()
                || self.effective_at.is_none()
                || self.deadline > self.effective_at,
            "a candidate deadline must follow its effective time"
        );
        Ok(())
    }
    pub fn sha256(&self) -> Result<String> {
        digest(self)
    }
    pub fn to_json(&self) -> Result<String> {
        canonical(self)
    }
}

/// The exact expected arithmetic of one conversion, mirroring the candidate program.
/// Computing it never creates evidence: it only checks what the program actually did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversionExpectation {
    pub consumed_raw: String,
    pub conversion_fee_raw: String,
    pub convertible_raw: String,
    pub replacement_gross_raw: String,
}
pub fn expected_output(consumed: u64, terms: &ConversionTerms) -> Result<ConversionExpectation> {
    ensure!(
        terms.ratio_numerator > 0
            && terms.ratio_denominator > 0
            && terms.conversion_fee_bps <= 10_000,
        "invalid conversion terms"
    );
    let consumed_u128 = u128::from(consumed);
    let fee = consumed_u128
        .checked_mul(u128::from(terms.conversion_fee_bps))
        .context("conversion fee overflow")?
        / 10_000;
    let base = consumed_u128
        .checked_sub(fee)
        .context("fee exceeds input")?;
    let numerator = base
        .checked_mul(u128::from(terms.ratio_numerator))
        .context("conversion ratio overflow")?;
    let denominator = u128::from(terms.ratio_denominator);
    let output = match terms.rounding {
        Rounding::Floor => numerator / denominator,
        Rounding::Ceiling => {
            numerator
                .checked_add(denominator - 1)
                .context("conversion rounding overflow")?
                / denominator
        }
    };
    ensure!(
        output <= u128::from(u64::MAX) && fee <= u128::from(u64::MAX),
        "conversion output exceeds raw integer range"
    );
    Ok(ConversionExpectation {
        consumed_raw: consumed.to_string(),
        conversion_fee_raw: fee.to_string(),
        convertible_raw: base.to_string(),
        replacement_gross_raw: output.to_string(),
    })
}

/// Whether an account in the execution bank is freshly captured current production
/// state or part of the operator's proposed rollout. Proposed state is never
/// described as observed, and observed state is never synthesized.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountOrigin {
    Observed,
    Proposed,
    /// A typed local mutation whose captured parent remains identified by its RPC pointer.
    DerivedForSearch,
    /// A local search mutation of a proposal; it never has an RPC parent account.
    DerivedProposedForSearch,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureAccount {
    pub address: String,
    pub origin: AccountOrigin,
    pub role: String,
    pub runtime_owner: String,
    pub lamports: u64,
    pub executable: bool,
    pub data_len: usize,
    pub data_sha256: String,
    /// Present only for observed accounts: the exact captured RPC pointer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc_record: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<u64>,
    pub derivation: Option<String>,
}
/// Every account's origin must be structurally consistent with its evidence.
pub fn validate_origins(accounts: &[FixtureAccount]) -> Result<()> {
    for a in accounts {
        match a.origin {
            AccountOrigin::Observed => ensure!(
                a.rpc_record.is_some()
                    && a.pointer.is_some()
                    && a.slot.is_some()
                    && a.derivation.is_none(),
                "an observed account must carry its exact captured RPC pointer"
            ),
            AccountOrigin::Proposed => ensure!(
                a.rpc_record.is_none()
                    && a.pointer.is_none()
                    && a.slot.is_none()
                    && a.derivation.is_some(),
                "a proposed account must have no captured RPC evidence and an explicit derivation"
            ),
            AccountOrigin::DerivedForSearch => ensure!(
                a.rpc_record.is_some()
                    && a.pointer.is_some()
                    && a.slot.is_some()
                    && a.derivation.is_some(),
                "a derived search account must retain its observed parent and typed derivation"
            ),
            AccountOrigin::DerivedProposedForSearch => ensure!(
                a.rpc_record.is_none()
                    && a.pointer.is_none()
                    && a.slot.is_none()
                    && a.derivation.is_some(),
                "a derived proposed account cannot claim RPC evidence"
            ),
        }
    }
    Ok(())
}

/// Opaque verified candidate conversion.
///
/// Deliberately not `Deserialize`: only a validated plan, an actual local VM
/// execution of the registered candidate program and exact reconciliation can
/// construct one. A serialized status is never accepted as proof.
#[derive(Serialize)]
pub struct VerifiedReplacementConversion {
    result: Value,
}
impl VerifiedReplacementConversion {
    pub(crate) fn new(result: Value) -> Self {
        Self { result }
    }
    pub fn value(&self) -> &Value {
        &self.result
    }
}
