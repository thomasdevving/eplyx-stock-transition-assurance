//! Bounded operator requirements over independently verified package evidence.
//! Requirements observe exact results; they never create execution or issuer proof.
use super::package::InvariantDefinition;
use crate::{resolution::PathStatus, stress::authority};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const EVALUATION_VERSION: &str = "eplyx-package-invariants/v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Blocking,
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Satisfied,
    Violated,
    Indeterminate,
    NotApplicable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequiredPath {
    ReplacementConversion,
    OfficialTransition,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Result {
    pub invariant_id: String,
    pub invariant_type: String,
    pub severity: Severity,
    pub status: Status,
    pub scope: String,
    pub config: Value,
    pub evidence_refs: Vec<String>,
    pub explanation: String,
    pub evaluation_version: String,
}

pub struct Evidence<'a> {
    pub conversion: &'a Value,
    pub stress: &'a Value,
    pub authority: Option<&'a authority::Report>,
    pub population: &'a Value,
    pub conversion_capture_sha256: &'a str,
    pub stress_cases_sha256: &'a str,
    pub population_sha256: &'a str,
    pub authority_report_sha256: Option<&'a str>,
}

fn outcome(status: Status, explanation: impl Into<String>) -> (Status, String) {
    (status, explanation.into())
}

pub fn evaluate(definitions: &[InvariantDefinition], evidence: &Evidence<'_>) -> Vec<Result> {
    definitions
        .iter()
        .map(|definition| {
            let (status, explanation, refs) = match definition {
                InvariantDefinition::ConversionOutputMatches { .. } => {
                    let conversion = evidence.conversion;
                    let status = conversion["status"].as_str();
                    let reconciled = conversion["reconciliation"]["reconciled"].as_bool();
                    let (result, reason) = match (status, reconciled) {
                        (Some("Proven"), Some(true)) if conversion["execution_performed"] == true =>
                            outcome(Status::Satisfied, "Exact candidate VM output passed the existing conversion reconciliation."),
                        (Some("Failed"), _) if conversion["execution_performed"] == true =>
                            outcome(Status::Violated, "The exact candidate VM conversion failed."),
                        _ => outcome(Status::Indeterminate, "Exact reconciled candidate execution is unavailable."),
                    };
                    (result, reason, vec![evidence.conversion_capture_sha256.to_owned()])
                }
                InvariantDefinition::NoSelectedCaseFailed { .. } => {
                    let cases = evidence.stress["results"].as_array();
                    let selected = evidence.stress["selected_cases"].as_array();
                    let refs = vec![evidence.stress_cases_sha256.to_owned()];
                    match cases {
                        Some(cases) if !cases.is_empty() => {
                            let failed = cases.iter().filter(|c| c["status"] == "Failed").count();
                            if failed > 0 {
                                (Status::Violated, format!("{failed} exact selected stress cases failed."), refs)
                            } else if selected.is_some_and(|selected| {
                                let selected_ids: std::collections::BTreeSet<_> = selected.iter().filter_map(|case| case["case_id"].as_str()).collect();
                                let result_ids: std::collections::BTreeSet<_> = cases.iter().filter_map(|case| case["case_id"].as_str()).collect();
                                selected.len() == cases.len() && selected_ids.len() == selected.len() && result_ids == selected_ids
                            }) && cases.iter().all(|c| c["status"] == "Proven") {
                                (Status::Satisfied, format!("All {} exact selected stress cases were proven.", cases.len()), refs)
                            } else {
                                outcome(Status::Indeterminate, "The frozen stress selection lacks complete known successful outcomes.").pipe_refs(refs)
                            }
                        }
                        _ => outcome(Status::Indeterminate, "No exact stress case was selected and executed.").pipe_refs(refs),
                    }
                }
                InvariantDefinition::RequiredPathAvailable { path, .. } => {
                    let (path_status, reference) = match path {
                        RequiredPath::ReplacementConversion => (
                            evidence.conversion["status"].as_str(),
                            evidence.conversion_capture_sha256,
                        ),
                        RequiredPath::OfficialTransition => (
                            evidence.conversion["official_transition"].as_str(),
                            evidence.conversion_capture_sha256,
                        ),
                    };
                    let (result, reason) = match path_status {
                        Some("Proven") => outcome(Status::Satisfied, "The required exact path is proven."),
                        Some("Failed") => outcome(Status::Violated, "The required exact path failed in execution."),
                        _ => outcome(Status::Indeterminate, "The required exact path has no proven execution; unsupported and untested states remain unknown."),
                    };
                    (result, reason, vec![reference.to_owned()])
                }
                InvariantDefinition::AuthorityModelSupported { .. } => {
                    let mut refs = vec![evidence.population_sha256.to_owned()];
                    if let Some(digest) = evidence.authority_report_sha256 {
                        refs.push(digest.to_owned());
                    }
                    match evidence.authority {
                        Some(report) if !report.cases.is_empty() => {
                            let unsupported = report.cases.iter().filter(|case| {
                                matches!(case.resolution,
                                    authority::ResolutionStatus::ResolvedMultisig |
                                    authority::ResolutionStatus::ResolvedProtocolInternal |
                                    authority::ResolutionStatus::ResolvedNeedsPrivateAuthorization |
                                    authority::ResolutionStatus::ResolvedExecutionUnsupported)
                                    && case.conversion == PathStatus::Unsupported
                            }).count();
                            if unsupported > 0 {
                                (Status::Violated, format!("{unsupported} resolved selected authority cases explicitly lack a supported candidate execution path."), refs)
                            } else if report.cases.iter().all(|case| case.execution_supported && case.conversion == PathStatus::Proven) {
                                (Status::Satisfied, format!("All {} selected authority cases have proven supported execution.", report.cases.len()), refs)
                            } else {
                                outcome(Status::Indeterminate, "Selected authority control or candidate execution support remains unresolved.").pipe_refs(refs)
                            }
                        }
                        _ => outcome(Status::Indeterminate, "Authority resolution selected no exact cases or was not performed.").pipe_refs(refs),
                    }
                }
                InvariantDefinition::NoPositiveBalanceStranded { .. } => {
                    let complete = evidence.population["enumeration_completeness"] == "CompleteForQuery";
                    let count = evidence.population["counts"]["positive_balance_accounts_observed"].as_u64();
                    let (result, reason) = if complete && count == Some(0) {
                        outcome(Status::NotApplicable, "The complete observed population contains no positive-balance account.")
                    } else {
                        outcome(Status::Indeterminate, "No exhaustive account-bound transition or mobility path evidence exists for the observed positive-balance population.")
                    };
                    (result, reason, vec![evidence.population_sha256.to_owned()])
                }
            };
            Result {
                invariant_id: definition.id(),
                invariant_type: definition.kind().into(),
                severity: definition.severity(),
                status,
                scope: definition.scope().into(),
                config: definition.config(),
                evidence_refs: refs,
                explanation,
                evaluation_version: EVALUATION_VERSION.into(),
            }
        })
        .collect()
}

trait WithRefs {
    fn pipe_refs(self, refs: Vec<String>) -> (Status, String, Vec<String>);
}
impl WithRefs for (Status, String) {
    fn pipe_refs(self, refs: Vec<String>) -> (Status, String, Vec<String>) {
        (self.0, self.1, refs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn evidence<'a>(
        conversion: &'a Value,
        stress: &'a Value,
        population: &'a Value,
    ) -> Evidence<'a> {
        Evidence {
            conversion,
            stress,
            authority: None,
            population,
            conversion_capture_sha256: "conversion-digest",
            stress_cases_sha256: "cases-digest",
            population_sha256: "population-digest",
            authority_report_sha256: None,
        }
    }

    fn finding(
        definition: InvariantDefinition,
        conversion: Value,
        stress: Value,
        population: Value,
    ) -> Result {
        evaluate(&[definition], &evidence(&conversion, &stress, &population)).remove(0)
    }

    #[test]
    fn conversion_output_uses_reconciled_vm_evidence_only() {
        let definition = InvariantDefinition::ConversionOutputMatches {
            severity: Severity::Blocking,
        };
        let stress = json!({"results": []});
        let population = json!({});
        let proven = json!({"status":"Proven", "execution_performed":true, "reconciliation":{"reconciled":true}});
        assert_eq!(
            finding(
                definition.clone(),
                proven.clone(),
                stress.clone(),
                population.clone()
            )
            .status,
            Status::Satisfied
        );
        let mut untrusted = proven;
        untrusted["reconciliation"]["reconciled"] = json!(false);
        assert_eq!(
            finding(
                definition.clone(),
                untrusted,
                stress.clone(),
                population.clone()
            )
            .status,
            Status::Indeterminate
        );
        let failed = json!({"status":"Failed", "execution_performed":true, "reconciliation":{"reconciled":true}});
        assert_eq!(
            finding(
                definition.clone(),
                failed,
                stress.clone(),
                population.clone()
            )
            .status,
            Status::Violated
        );
        let indeterminate = json!({"status":"Indeterminate", "execution_performed":false});
        assert_eq!(
            finding(definition, indeterminate, stress, population).status,
            Status::Indeterminate
        );
    }

    #[test]
    fn selected_stress_failure_and_unknown_are_kept_separate() {
        let definition = InvariantDefinition::NoSelectedCaseFailed {
            severity: Severity::Blocking,
        };
        let conversion = json!({});
        let population = json!({});
        for (statuses, expected) in [
            (vec!["Proven", "Proven"], Status::Satisfied),
            (vec!["Proven", "Failed"], Status::Violated),
            (vec!["Proven", "Indeterminate"], Status::Indeterminate),
            (vec![], Status::Indeterminate),
        ] {
            let cases: Vec<_> = statuses
                .iter()
                .enumerate()
                .map(|(index, status)| json!({"case_id": format!("case-{index}"), "status":status}))
                .collect();
            let selected: Vec<_> = (0..statuses.len())
                .map(|index| json!({"case_id": format!("case-{index}")}))
                .collect();
            let result = finding(
                definition.clone(),
                conversion.clone(),
                json!({"selected_cases":selected, "results": cases}),
                population.clone(),
            );
            assert_eq!(result.status, expected);
            assert_eq!(result.scope, "SelectedStressCases");
        }
        let incomplete = json!({
            "selected_cases": [{"case_id":"case-0"}, {"case_id":"case-1"}],
            "results": [{"case_id":"case-0", "status":"Proven"}]
        });
        assert_eq!(
            finding(definition, conversion, incomplete, population).status,
            Status::Indeterminate
        );
    }

    #[test]
    fn replacement_proof_cannot_satisfy_official_path() {
        let conversion = json!({"status":"Proven", "official_transition":"NotTested"});
        let stress = json!({});
        let population = json!({});
        let replacement = InvariantDefinition::RequiredPathAvailable {
            severity: Severity::Blocking,
            path: RequiredPath::ReplacementConversion,
        };
        let official = InvariantDefinition::RequiredPathAvailable {
            severity: Severity::Blocking,
            path: RequiredPath::OfficialTransition,
        };
        assert_eq!(
            finding(
                replacement,
                conversion.clone(),
                stress.clone(),
                population.clone()
            )
            .status,
            Status::Satisfied
        );
        assert_eq!(
            finding(official, conversion, stress, population).status,
            Status::Indeterminate
        );
    }

    #[test]
    fn bounded_sample_never_satisfies_population_requirement() {
        let definition = InvariantDefinition::NoPositiveBalanceStranded {
            severity: Severity::Blocking,
        };
        let conversion = json!({"status":"Proven"});
        let stress = json!({"results":[{"status":"Proven"}]});
        let complete = json!({"enumeration_completeness":"CompleteForQuery", "counts":{"positive_balance_accounts_observed":1}});
        assert_eq!(
            finding(
                definition.clone(),
                conversion.clone(),
                stress.clone(),
                complete
            )
            .status,
            Status::Indeterminate
        );
        let partial = json!({"enumeration_completeness":"Unavailable", "counts":{"positive_balance_accounts_observed":0}});
        assert_eq!(
            finding(
                definition.clone(),
                conversion.clone(),
                stress.clone(),
                partial
            )
            .status,
            Status::Indeterminate
        );
        let empty = json!({"enumeration_completeness":"CompleteForQuery", "counts":{"positive_balance_accounts_observed":0}});
        assert_eq!(
            finding(definition, conversion, stress, empty).status,
            Status::NotApplicable
        );
    }

    #[test]
    fn unperformed_authority_resolution_is_indeterminate() {
        let result = finding(
            InvariantDefinition::AuthorityModelSupported {
                severity: Severity::Warning,
            },
            json!({}),
            json!({}),
            json!({}),
        );
        assert_eq!(result.status, Status::Indeterminate);
        assert_eq!(result.scope, "SelectedAuthorityCases");
    }

    #[test]
    fn unresolved_authority_is_not_supported_and_explicit_unsupported_is_violated() {
        use crate::lifecycle::EntityType;
        use authority::{
            Case, ControlPath, Coverage, InvocationKind, Report, ResolutionStatus, Selection,
        };
        use std::collections::BTreeMap;
        let selection = Selection {
            token_account: "selected-account".into(),
            recorded_authority: "recorded-authority".into(),
            initial_classification: EntityType::Unknown,
            observed_balance_raw: "1".into(),
            control_type_hint: "unknown".into(),
            selection_reason: "test".into(),
        };
        let case = Case {
            selection,
            authority_exists: false,
            on_curve: false,
            runtime_owner: None,
            executable: None,
            data_len: None,
            raw_data_sha256: None,
            lamports: None,
            resolution: ResolutionStatus::Unresolved,
            conversion: PathStatus::Indeterminate,
            execution_supported: false,
            signer_assumed_locally: false,
            reason: "unknown".into(),
            control_path: ControlPath {
                token_account: "selected-account".into(),
                recorded_authority: "recorded-authority".into(),
                controller_program: None,
                controller_state: None,
                pda_derivation: None,
                invocation_kind: InvocationKind::Unsupported,
                required_accounts: vec![],
                required_signers: vec![],
                multisig_threshold: None,
                multisig_members: vec![],
                preconditions: vec![],
                evidence: vec![],
            },
        };
        let mut report = Report {
            resolver_version: "test".into(),
            plan_sha256: "plan".into(),
            population_digest: "population".into(),
            cases: vec![case],
            coverage: Coverage {
                positive_balance_accounts_observed: 1,
                cases_selected: 1,
                population_initial: BTreeMap::new(),
                initial: BTreeMap::new(),
                resolved: BTreeMap::new(),
                unselected_non_wallet_accounts: 0,
                unselected_non_wallet_raw: "0".into(),
            },
        };
        let conversion = json!({});
        let stress = json!({});
        let population = json!({});
        let definition = InvariantDefinition::AuthorityModelSupported {
            severity: Severity::Blocking,
        };
        {
            let mut observed = evidence(&conversion, &stress, &population);
            observed.authority = Some(&report);
            assert_eq!(
                evaluate(std::slice::from_ref(&definition), &observed)[0].status,
                Status::Indeterminate
            );
        }
        report.cases[0].resolution = ResolutionStatus::ResolvedNeedsPrivateAuthorization;
        report.cases[0].conversion = PathStatus::Unsupported;
        let mut observed = evidence(&conversion, &stress, &population);
        observed.authority = Some(&report);
        assert_eq!(
            evaluate(&[definition], &observed)[0].status,
            Status::Violated
        );
    }
}
