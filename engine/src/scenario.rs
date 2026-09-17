//! Change descriptions and dispatch, separate from the state being evaluated.
//!
//! Program upgrades borrow the existing binaries and use the existing execution
//! contract. Lifecycle changes are descriptions only:
//! no consequence model is implemented in Phase 1.

use anyhow::{bail, Result};
use solana_address::Address;

use crate::diff::StateDiff;
use crate::executor::ProgramVersion;
use crate::types::Fixture;

/// A change in program bytecode. State, transactions and dependencies remain
/// inputs to the execution model rather than fields of the change.
#[derive(Clone, Copy, Debug)]
pub struct ProgramUpgrade<'a> {
    pub baseline: &'a ProgramVersion,
    pub candidate: &'a ProgramVersion,
}

/// Placeholder for an asset lifecycle change.
///
/// This description carries no executable semantics, entitlement rule or
/// evidence claim. A later phase must define a consequence model and its state
/// inputs before a lifecycle change can be evaluated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleChange {
    pub description: String,
}

/// The proposed change, independent of its state inputs.
///
/// This is an in-memory API. Existing fixture and report schemas remain
/// unchanged; lifecycle results must not masquerade as upgrade reports.
#[derive(Clone, Debug)]
pub enum ChangeScenario<'a> {
    ProgramUpgrade(ProgramUpgrade<'a>),
    LifecycleChange(LifecycleChange),
}

impl<'a> ChangeScenario<'a> {
    pub fn program_upgrade(baseline: &'a ProgramVersion, candidate: &'a ProgramVersion) -> Self {
        Self::ProgramUpgrade(ProgramUpgrade {
            baseline,
            candidate,
        })
    }

    /// Compare a fixture using the execution model for this change.
    ///
    /// Both sides of a program upgrade get a fresh VM with identical state.
    pub fn compare_fixture(&self, fixture: &Fixture, program_id: &Address) -> Result<StateDiff> {
        match self {
            Self::ProgramUpgrade(upgrade) => {
                let baseline = crate::executor::execute(fixture, program_id, upgrade.baseline)?;
                let candidate = crate::executor::execute(fixture, program_id, upgrade.candidate)?;
                Ok(crate::diff::compare(fixture, baseline, candidate))
            }
            Self::LifecycleChange(_) => {
                bail!("LifecycleChange is not supported: no consequence model is implemented")
            }
        }
    }
}
