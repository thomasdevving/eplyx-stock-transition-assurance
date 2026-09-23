//! Strict, offline validation of an operator-supplied transition package.
//!
//! A package is a proposal. Validation never grants conversion or issuer proof.
use super::{
    demo,
    invariants::{RequiredPath, Severity},
    AmountMode, AuthorityModel, CandidateAuthority, ConversionPlan, ConversionTerms, MechanismId,
    PlanProvenance, ReplacementDelivery, ReserveConfig, Rounding, SourceConsumption, ADAPTER_ID,
};
use crate::lifecycle::exposure::sha256;
use anyhow::{ensure, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use solana_address::Address;
use std::path::{Component, Path, PathBuf};

pub const ADAPTER: &str = "fixed_ratio_conversion_v1";
pub const VERSION: u32 = 1;
pub const INVARIANT_PACKAGE_VERSION: u32 = 2;
pub const INVARIANT_SCHEMA_VERSION: u32 = 1;
pub const MAX_INVARIANTS: usize = 16;
pub const MAX_MANIFEST_BYTES: u64 = 16 * 1024;
pub const MAX_CONFIG_BYTES: u64 = 16 * 1024;
pub const MAX_PROGRAM_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CandidateProgram {
    pub program_id: String,
    pub artifact: String,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Terms {
    pub numerator: String,
    pub denominator: String,
    pub rounding: String,
    pub fee_bps: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: u32,
    pub source_mint: String,
    pub replacement_mint: String,
    pub adapter: String,
    pub candidate_program: CandidateProgram,
    pub terms: Terms,
    pub effective_at: DateTime<Utc>,
    pub config: String,
    pub config_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invariant_schema_version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invariants: Option<Vec<InvariantDefinition>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum InvariantDefinition {
    ConversionOutputMatches {
        severity: Severity,
    },
    NoSelectedCaseFailed {
        severity: Severity,
    },
    RequiredPathAvailable {
        severity: Severity,
        path: RequiredPath,
    },
    AuthorityModelSupported {
        severity: Severity,
    },
    NoPositiveBalanceStranded {
        severity: Severity,
    },
}

impl InvariantDefinition {
    pub fn severity(&self) -> Severity {
        match self {
            Self::ConversionOutputMatches { severity }
            | Self::NoSelectedCaseFailed { severity }
            | Self::RequiredPathAvailable { severity, .. }
            | Self::AuthorityModelSupported { severity }
            | Self::NoPositiveBalanceStranded { severity } => *severity,
        }
    }
    pub fn kind(&self) -> &'static str {
        match self {
            Self::ConversionOutputMatches { .. } => "conversion_output_matches",
            Self::NoSelectedCaseFailed { .. } => "no_selected_case_failed",
            Self::RequiredPathAvailable { .. } => "required_path_available",
            Self::AuthorityModelSupported { .. } => "authority_model_supported",
            Self::NoPositiveBalanceStranded { .. } => "no_positive_balance_stranded",
        }
    }
    pub fn scope(&self) -> &'static str {
        match self {
            Self::ConversionOutputMatches { .. }
            | Self::RequiredPathAvailable {
                path: RequiredPath::ReplacementConversion,
                ..
            } => "ExactCandidateConversion",
            Self::RequiredPathAvailable {
                path: RequiredPath::OfficialTransition,
                ..
            } => "ExactOfficialTransition",
            Self::NoSelectedCaseFailed { .. } => "SelectedStressCases",
            Self::AuthorityModelSupported { .. } => "SelectedAuthorityCases",
            Self::NoPositiveBalanceStranded { .. } => "ObservedPositiveBalancePopulation",
        }
    }
    pub fn config(&self) -> serde_json::Value {
        match self {
            Self::RequiredPathAvailable { path, .. } => serde_json::json!({"path": path}),
            _ => serde_json::json!({}),
        }
    }
    pub fn id(&self) -> String {
        let identity = crate::expansion::canonical(self).expect("typed invariant is serializable");
        format!("inv-{}", &sha256(identity.as_bytes())[..20])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Config {
    pub public_owner: String,
    pub source_account: String,
    pub amount_decimal: Option<String>,
    pub reserve_funded_replacement_raw: String,
}

pub struct ValidatedPackage {
    pub manifest: Manifest,
    pub config: Config,
    pub program: Vec<u8>,
    pub program_sha256: String,
    pub config_sha256: String,
    pub transition_package_sha256: String,
}

fn bounded_read(path: &Path, max: u64) -> Result<Vec<u8>> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("missing package file {}", path.display()))?;
    ensure!(
        metadata.is_file() && metadata.len() <= max,
        "package file exceeds its size bound"
    );
    let bytes = std::fs::read(path)?;
    ensure!(
        bytes.len() as u64 <= max,
        "package file exceeds its size bound"
    );
    Ok(bytes)
}

fn member(root: &Path, name: &str) -> Result<PathBuf> {
    let path = Path::new(name);
    ensure!(
        !name.is_empty()
            && !path.is_absolute()
            && path
                .components()
                .all(|part| matches!(part, Component::Normal(_))),
        "package path must be a relative member without traversal"
    );
    let joined = root.join(path);
    let resolved = joined.canonicalize().context("missing package member")?;
    ensure!(
        resolved.starts_with(root),
        "package member escapes package root"
    );
    Ok(resolved)
}

fn canonical_decimal(input: &str) -> Result<u64> {
    let parsed: u64 = input.parse().context("invalid integer term")?;
    ensure!(parsed.to_string() == input, "integer term is not canonical");
    Ok(parsed)
}

fn valid_digest(input: &str) -> bool {
    input.len() == 64
        && input
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Only Solana BPF ELF can enter the VM. Native ELF and arbitrary host formats
/// are refused before any acquisition or execution.
fn validate_sbf(bytes: &[u8]) -> Result<()> {
    ensure!(bytes.len() >= 64, "candidate is not a supported SBF ELF");
    ensure!(
        &bytes[..4] == b"\x7fELF" && bytes[4] == 2 && bytes[5] == 1,
        "candidate is not a 64-bit little-endian ELF"
    );
    ensure!(
        u16::from_le_bytes([bytes[16], bytes[17]]) == 3,
        "candidate must be a loadable shared ELF"
    );
    ensure!(
        matches!(u16::from_le_bytes([bytes[18], bytes[19]]), 247 | 263),
        "candidate is not Solana SBF/BPF"
    );
    Ok(())
}

impl Manifest {
    pub fn conversion_terms(&self) -> Result<ConversionTerms> {
        let rounding = match self.terms.rounding.as_str() {
            "floor" => Rounding::Floor,
            "ceiling" => Rounding::Ceiling,
            _ => anyhow::bail!("unsupported rounding rule"),
        };
        let terms = ConversionTerms {
            ratio_numerator: canonical_decimal(&self.terms.numerator)?,
            ratio_denominator: canonical_decimal(&self.terms.denominator)?,
            rounding,
            conversion_fee_bps: self.terms.fee_bps,
        };
        ensure!(
            terms.ratio_numerator > 0
                && terms.ratio_denominator > 0
                && terms.conversion_fee_bps <= 10_000,
            "invalid conversion terms"
        );
        // All u64 source amounts must fit in the adapter's u128 intermediate.
        ensure!(
            u128::from(u64::MAX)
                .checked_mul(u128::from(terms.ratio_numerator))
                .is_some(),
            "ratio can overflow adapter arithmetic"
        );
        Ok(terms)
    }
}

pub fn load(directory: &Path) -> Result<ValidatedPackage> {
    let root = directory
        .canonicalize()
        .context("package directory missing")?;
    ensure!(root.is_dir(), "package root is not a directory");
    let manifest_bytes = bounded_read(&member(&root, "eplyx.json")?, MAX_MANIFEST_BYTES)?;
    let mut manifest: Manifest =
        serde_json::from_slice(&manifest_bytes).context("invalid package manifest")?;
    match manifest.schema_version {
        VERSION => ensure!(
            manifest.invariant_schema_version.is_none() && manifest.invariants.is_none(),
            "schema 1 cannot declare invariants"
        ),
        INVARIANT_PACKAGE_VERSION => {
            ensure!(
                manifest.invariant_schema_version == Some(INVARIANT_SCHEMA_VERSION),
                "unsupported invariant schema"
            );
            let definitions = manifest
                .invariants
                .as_mut()
                .context("schema 2 requires invariants")?;
            ensure!(
                !definitions.is_empty() && definitions.len() <= MAX_INVARIANTS,
                "invariant count exceeds bounds"
            );
            let mut keys = definitions
                .iter()
                .map(|definition| {
                    crate::expansion::canonical(&(definition.kind(), definition.config()))
                })
                .collect::<Result<Vec<_>>>()?;
            keys.sort();
            ensure!(
                keys.windows(2).all(|pair| pair[0] != pair[1]),
                "duplicate invariant definition"
            );
            definitions.sort_by_key(|definition| {
                crate::expansion::canonical(definition).expect("typed invariant is serializable")
            });
        }
        _ => anyhow::bail!("unsupported package schema"),
    }
    ensure!(manifest.adapter == ADAPTER, "unsupported package adapter");
    let source: Address = manifest
        .source_mint
        .parse()
        .context("invalid source mint")?;
    let replacement: Address = manifest
        .replacement_mint
        .parse()
        .context("invalid replacement mint")?;
    ensure!(
        source != replacement,
        "source and replacement mints must differ"
    );
    let _ = manifest.conversion_terms()?;
    ensure!(
        manifest.candidate_program.program_id.parse::<Address>()?
            == demo::PROGRAM_ID.parse::<Address>()?,
        "program ID is not registered for this adapter"
    );
    ensure!(
        valid_digest(&manifest.candidate_program.sha256),
        "invalid program SHA-256"
    );
    let program = bounded_read(
        &member(&root, &manifest.candidate_program.artifact)?,
        MAX_PROGRAM_BYTES,
    )?;
    validate_sbf(&program)?;
    let program_sha256 = sha256(&program);
    ensure!(
        program_sha256 == manifest.candidate_program.sha256,
        "candidate program SHA-256 mismatch"
    );
    // The registry fixes the instruction/account ABI and program ID. The
    // operator may supply a new candidate build; only actual VM execution and
    // exact reconciliation can grant evidence for its hash.
    let config_bytes = bounded_read(&member(&root, &manifest.config)?, MAX_CONFIG_BYTES)?;
    ensure!(
        valid_digest(&manifest.config_sha256),
        "invalid config SHA-256"
    );
    let config_sha256 = sha256(&config_bytes);
    ensure!(
        config_sha256 == manifest.config_sha256,
        "package config SHA-256 mismatch"
    );
    let config: Config = serde_json::from_slice(&config_bytes).context("invalid package config")?;
    let _: Address = config
        .public_owner
        .parse()
        .context("invalid public owner")?;
    let _: Address = config
        .source_account
        .parse()
        .context("invalid source account")?;
    let _ = canonical_decimal(&config.reserve_funded_replacement_raw)?;
    if let Some(amount) = &config.amount_decimal {
        ensure!(
            !amount.is_empty() && amount.len() <= 280,
            "invalid amount length"
        );
    }
    let canonical = crate::expansion::canonical(&manifest)?;
    let identity = crate::expansion::canonical(&serde_json::json!({
        "schema_version": manifest.schema_version,
        "adapter": ADAPTER,
        "manifest": canonical,
        "program_sha256": program_sha256,
        "config_sha256": config_sha256,
    }))?;
    let transition_package_sha256 = sha256(identity.as_bytes());
    Ok(ValidatedPackage {
        manifest,
        config,
        program,
        program_sha256,
        config_sha256,
        transition_package_sha256,
    })
}

impl ValidatedPackage {
    pub fn conversion_plan(&self) -> Result<ConversionPlan> {
        let plan = ConversionPlan {
            schema_version: 1,
            id: format!("pkg-{}", &self.transition_package_sha256[..20]),
            version: 1,
            provenance: PlanProvenance::OperatorSupplied,
            mechanism: MechanismId::EplyxDemoCandidateConversion,
            adapter_id: ADAPTER_ID.into(),
            mechanism_ref: format!("{}@{}", ADAPTER, self.program_sha256),
            source_mint: self.manifest.source_mint.clone(),
            replacement_mint: self.manifest.replacement_mint.clone(),
            source_account: self.config.source_account.clone(),
            amount_mode: if self.config.amount_decimal.is_some() {
                AmountMode::Custom
            } else {
                AmountMode::Full
            },
            amount_decimal: self.config.amount_decimal.clone(),
            terms: self.manifest.conversion_terms()?,
            authority_model: AuthorityModel {
                holder_signs: true,
                candidate_authority: CandidateAuthority::ProgramDerived,
            },
            source_consumption: SourceConsumption::Burn,
            replacement_delivery: ReplacementDelivery::ProposedReserveRelease,
            reserve: ReserveConfig {
                funded_replacement_raw: self.config.reserve_funded_replacement_raw.clone(),
            },
            effective_at: Some(self.manifest.effective_at),
            deadline: None,
        };
        plan.validate()?;
        Ok(plan)
    }
}
