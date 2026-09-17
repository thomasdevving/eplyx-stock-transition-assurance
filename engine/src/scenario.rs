//! Change descriptions and dispatch, separate from the state being evaluated.
//!
//! Program upgrades borrow the existing binaries and use the existing execution
//! contract. Lifecycle changes use an external policy over frozen production state.

use crate::lifecycle::{
    consequence::{LifecycleConsequenceEvaluator, LifecycleImpactReport},
    policy::LifecycleScenario,
    LifecycleSnapshot,
};
use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

/// An asset lifecycle change description, independent of state and policy inputs.
///
/// This description carries no executable semantics, entitlement rule or
/// evidence claim. `compare_lifecycle` supplies an explicit policy and frozen world.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
    /// Apply external lifecycle semantics to two time views of one frozen world.
    pub fn compare_lifecycle(
        &self,
        snapshot: &LifecycleSnapshot,
        scenario: &LifecycleScenario,
        before: DateTime<Utc>,
        at: DateTime<Utc>,
    ) -> Result<LifecycleImpactReport> {
        match self {
            Self::LifecycleChange(change) => {
                anyhow::ensure!(
                    *change == scenario.change,
                    "lifecycle change differs from scenario specification"
                );
                LifecycleConsequenceEvaluator::evaluate(snapshot, scenario, before, at)
            }
            Self::ProgramUpgrade(_) => {
                bail!("ProgramUpgrade requires program execution, not lifecycle policy evaluation")
            }
        }
    }

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
                bail!("LifecycleChange is not supported for synthetic fixture execution: supply frozen production state and an explicit lifecycle policy to compare_lifecycle")
            }
        }
    }
}
