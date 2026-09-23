//! A deployment decision over verified analytical findings, never new evidence.
use anyhow::{ensure, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Policy {
    #[default]
    BlockOnly,
    Strict,
}

impl Policy {
    pub fn name(self) -> &'static str {
        match self {
            Self::BlockOnly => "block-only",
            Self::Strict => "strict",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Pass,
    Warn,
    Block,
}

impl Outcome {
    pub fn exit_code(self) -> u8 {
        if self == Self::Block {
            3
        } else {
            0
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "PASS WITH WARNINGS",
            Self::Block => "BLOCKED",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentGate {
    pub policy: Policy,
    pub outcome: Outcome,
    pub analytical_readiness: String,
    pub reasons: Vec<String>,
}

pub fn evaluate(report: &Value, policy: Policy) -> Result<DeploymentGate> {
    let fields = [
        (
            "candidate_plan_readiness",
            &report["candidate_plan_readiness"],
        ),
        (
            "conversion_stress_readiness",
            &report["conversion_stress_readiness"]["status"],
        ),
        (
            "population_rollout_readiness",
            &report["population_rollout_readiness"]["status"],
        ),
    ];
    let mut blocked = false;
    let mut incomplete = false;
    let mut reasons = Vec::new();
    for (name, value) in fields {
        let status = value.as_str().unwrap_or("");
        ensure!(
            matches!(status, "Ready" | "Incomplete" | "Blocked"),
            "invalid analytical readiness: {name}"
        );
        if status == "Blocked" {
            blocked = true;
            reasons.push(format!("{name} is Blocked"));
        } else if status == "Incomplete" {
            incomplete = true;
            reasons.push(format!("{name} is Incomplete"));
        }
    }
    if report["candidate_plan_readiness"] == "Blocked" {
        if let Some(reason) = report["conversion_result"]["reason"].as_str() {
            reasons.push(reason.into());
        }
    }
    let failed = report["failed_cases"].as_array().map_or(0, Vec::len);
    if failed > 0 {
        reasons.push(format!(
            "{failed} exact selected stress cases failed in local execution"
        ));
    }
    if let Some(coverage) = report
        .get("non_standard_account_control")
        .and_then(|r| r.get("coverage"))
    {
        let selected = coverage["cases_selected"].as_u64().unwrap_or(0);
        let remaining = coverage["unselected_non_wallet_accounts"]
            .as_u64()
            .unwrap_or(0);
        match report["population_summary"]["enumeration_completeness"].as_str() {
            Some(completeness) if completeness != "CompleteForQuery" => {
                reasons.push(format!(
                    "Authority control inspected for {selected} selected non-wallet accounts among observed accounts; {remaining} more were observed outside the bounded selection. Enumeration is {completeness}, so the full population is unknown"
                ));
            }
            _ => reasons.push(format!(
                "Authority control inspected for {selected} selected non-wallet accounts; {remaining} remain outside the bounded selection"
            )),
        }
    }
    ensure!(
        report["official_transition"] == "NotTested",
        "unexpected official transition claim"
    );
    ensure!(
        report["funds_moved"] == false,
        "unexpected funds movement claim"
    );
    let analytical = report["declared_preflight_status"].as_str().unwrap_or("");
    let expected_analytical = if report["candidate_plan_readiness"] == "Blocked"
        || report["conversion_stress_readiness"]["status"] == "Blocked"
    {
        "Blocked"
    } else if report["candidate_plan_readiness"] == "Ready"
        && report["conversion_stress_readiness"]["status"] == "Ready"
    {
        "Ready"
    } else {
        "Incomplete"
    };
    ensure!(
        analytical == expected_analytical,
        "analytical readiness is inconsistent"
    );
    let outcome = if blocked || (policy == Policy::Strict && incomplete) {
        Outcome::Block
    } else if incomplete {
        Outcome::Warn
    } else {
        Outcome::Pass
    };
    Ok(DeploymentGate {
        policy,
        outcome,
        analytical_readiness: analytical.into(),
        reasons,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn report(candidate: &str, stress: &str, population: &str) -> Value {
        json!({
            "candidate_plan_readiness": candidate,
            "conversion_stress_readiness": {"status": stress},
            "population_rollout_readiness": {"status": population},
            "declared_preflight_status": if candidate == "Blocked" || stress == "Blocked" {"Blocked"} else if candidate == "Ready" && stress == "Ready" {"Ready"} else {"Incomplete"},
            "official_transition": "NotTested",
            "funds_moved": false,
        })
    }

    #[test]
    fn block_only_warns_on_incomplete_and_blocks_explicit_failure() {
        assert_eq!(
            evaluate(&report("Ready", "Ready", "Ready"), Policy::BlockOnly)
                .unwrap()
                .outcome,
            Outcome::Pass
        );
        assert_eq!(
            evaluate(
                &report("Ready", "Incomplete", "Incomplete"),
                Policy::BlockOnly
            )
            .unwrap()
            .outcome,
            Outcome::Warn
        );
        assert_eq!(
            evaluate(
                &report("Blocked", "Blocked", "Incomplete"),
                Policy::BlockOnly
            )
            .unwrap()
            .outcome,
            Outcome::Block
        );
        assert!(evaluate(&report("Invalid", "Ready", "Ready"), Policy::BlockOnly).is_err());
    }

    #[test]
    fn strict_blocks_incomplete_without_mutating_analytical_evidence() {
        let analytical = report("Ready", "Incomplete", "Incomplete");
        let original = analytical.clone();
        assert_eq!(
            evaluate(&report("Ready", "Ready", "Ready"), Policy::Strict)
                .unwrap()
                .outcome,
            Outcome::Pass
        );
        assert_eq!(
            evaluate(&analytical, Policy::Strict).unwrap().outcome,
            Outcome::Block
        );
        assert_eq!(
            evaluate(&analytical, Policy::BlockOnly).unwrap().outcome,
            Outcome::Warn
        );
        assert_eq!(analytical, original);
        assert_eq!(analytical["official_transition"], "NotTested");
        assert_eq!(
            analytical["population_rollout_readiness"]["status"],
            "Incomplete"
        );
        assert!(
            evaluate(&report("Blocked", "Blocked", "Incomplete"), Policy::Strict)
                .unwrap()
                .outcome
                == Outcome::Block
        );
        assert!(evaluate(&report("Invalid", "Ready", "Ready"), Policy::Strict).is_err());
    }
}
