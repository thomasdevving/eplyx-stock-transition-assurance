//! File formats of a project's local `.eplyx/` run store, shared by the `eplyx`
//! CLI that writes them and the read-only dashboard that presents them. These
//! are local bookkeeping records around engine artifacts, never new evidence.
use crate::{conversion::search, expansion::canonical, lifecycle::exposure::sha256};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub schema_version: u32,
    pub run_id: String,
    pub timestamp: String,
    pub eplyx_version: String,
    pub engine_binary_sha256: String,
    pub git_commit: Option<String>,
    pub git_branch: Option<String>,
    pub git_dirty: Option<bool>,
    pub candidate_program_sha256: String,
    pub transition_package_sha256: String,
    pub gate_policy: String,
    pub gate_outcome: String,
    /// Where the run was produced. Absent on schema 1 metadata written before
    /// the field existed; never inferred for those runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_source: Option<RunSource>,
}

/// Metadata schema that carries `run_source`.
pub const METADATA_VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunSource {
    /// `eplyx preflight` on a developer machine.
    Local,
    /// `eplyx preflight` with a non-empty `CI` environment variable.
    Ci,
    /// Reserved for runs brought in from another store; nothing writes it yet.
    Imported,
}

impl RunSource {
    pub fn detect() -> Self {
        match std::env::var("CI") {
            Ok(value)
                if !value.is_empty() && value != "0" && !value.eq_ignore_ascii_case("false") =>
            {
                Self::Ci
            }
            _ => Self::Local,
        }
    }
}

/// One local `eplyx reproduce` attempt. History only: it records what the
/// offline replay concluded and never substitutes for re-running it.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reproduction {
    pub schema_version: u32,
    pub id: String,
    pub counterexample_id: String,
    pub parent_run: Option<String>,
    pub search_sha256: Option<String>,
    pub timestamp: String,
    pub outcome: ReproductionOutcome,
    /// The saved counterexample, including its failure signature and
    /// rollback, was found unchanged in the offline VM replay.
    pub failure_signature_matched: bool,
    pub error: Option<String>,
    pub eplyx_version: String,
    pub engine_binary_sha256: String,
    /// The RPC environment was removed before replay; no provider was used.
    pub no_rpc: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReproductionOutcome {
    Reproduced,
    Failed,
}

pub const REPRODUCTION_VERSION: u32 = 1;

/// `repro_<UTC millis>_<cx digest>`: sortable, and safe as a local ID.
pub fn reproduction_id(timestamp: &str, counterexample: &str) -> String {
    format!(
        "repro_{timestamp}_{}",
        counterexample.strip_prefix("cx_").unwrap_or(counterexample)
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedCounterexample {
    pub schema_version: u32,
    pub id: String,
    pub parent_run: String,
    pub search_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_inputs: Option<ReplayInputs>,
    pub counterexample: search::Counterexample,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReplayInputs {
    pub package: String,
    pub result: String,
    pub search: String,
}

/// Store-relative replay inputs; always forward slashes on every platform.
pub fn replay_inputs(run: &str) -> ReplayInputs {
    ReplayInputs {
        package: format!("runs/{run}/package"),
        result: format!("runs/{run}/result"),
        search: format!("runs/{run}/search"),
    }
}

/// Content-addressed local counterexample ID. The engine counterexample embeds
/// its package run, so the same failing account in two runs has two IDs.
pub fn counterexample_id(counterexample: &search::Counterexample) -> Result<String> {
    Ok(format!(
        "cx_{}",
        &sha256(canonical(counterexample)?.as_bytes())[..24]
    ))
}

/// Local IDs are lowercase ASCII, digits and underscores behind a fixed prefix,
/// so they can never carry a separator, dot, drive letter or encoded byte.
pub fn is_safe_id(id: &str, prefix: &str) -> bool {
    id.starts_with(prefix)
        && id.len() > prefix.len()
        && id.len() <= 100
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

pub fn safe_id(id: &str, prefix: &str) -> Result<()> {
    ensure!(is_safe_id(id, prefix), "invalid local ID");
    Ok(())
}
