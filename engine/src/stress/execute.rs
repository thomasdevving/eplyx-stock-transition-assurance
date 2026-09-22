//! Per-case bounded read-only capture, offline execution of the registered
//! candidate mechanism, and aggregation without extrapolation.
//!
//! Each selected case starts from its own freshly captured bank and runs the same
//! operator-supplied plan at its own exact source account and frozen amount. Cases
//! are never applied sequentially to one another, and independent case outputs are
//! never summed into rollout capacity. Execution reuses the Milestone 6 adapter:
//! there is no second execution engine and no second reconciliation.
use super::{
    assert_no_proof_inheritance, classify,
    population::PopulationObservation,
    select::{eligibility_key, StressTestPlan},
    sum_once, CaseResult, ConversionStressTestResult, CoverageSummary, SelectedCase, ShapeCoverage,
    StressBudget, UnsupportedState, COVERAGE_NOTE, EXECUTOR_BOUNDARY, MAX_SHAPE_EXAMPLES,
    SHAPE_PROOF_SCOPE,
};
use crate::{
    conversion::demo,
    executor,
    expansion::{digest, Eligibility},
    lifecycle::{current::Observation, exposure::sha256, rpc::SolanaRpc, RpcEvidence},
    probe::{meteora_dlmm as shared, ProbeClock, ProbeMessage},
    resolution::PathStatus,
};
use anyhow::{ensure, Context, Result};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;

pub const BUNDLE_KIND: &str = "conversion-stress-cases";
pub const RESULT_KIND: &str = "current-conversion-stress";

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}
fn config(slot: u64) -> Value {
    json!({"encoding":"base64","commitment":"finalized","minContextSlot":slot})
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseCapture {
    pub case_id: String,
    pub entity_id: String,
    pub token_account: String,
    pub case_plan_sha256: String,
    pub started_at: String,
    pub completed_at: String,
    pub observations: Vec<Observation>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureBundle {
    pub schema_version: u32,
    pub kind: String,
    pub stress_id: String,
    pub run_id: String,
    pub stress_plan_sha256: String,
    pub population_capture_sha256: String,
    pub rpc_origin: String,
    pub started_at: String,
    pub completed_at: String,
    pub cases: Vec<CaseCapture>,
}

fn record(
    rpc: &impl SolanaRpc,
    records: &mut Vec<Observation>,
    method: &str,
    params: Value,
) -> Option<Value> {
    let started_at = now();
    let (result, error) = match rpc.call(method, params.clone()) {
        Ok(v) => (Some(v), None),
        Err(e) => (None, Some(e.to_string())),
    };
    records.push(Observation {
        method: method.into(),
        params,
        started_at,
        completed_at: now(),
        result: result.clone(),
        error,
    });
    result
}

/// Exactly five bounded read-only requests for one case, in the same shape the
/// Milestone 6 adapter validates. An incomplete acquisition leaves the case
/// unexecuted; it never becomes a Failed conversion.
fn acquire_case(
    case: &SelectedCase,
    source_program: &str,
    rpc: &impl SolanaRpc,
) -> Vec<Observation> {
    let mut records = vec![];
    let plan = &case.case_plan;
    let attempt = (|| -> Option<()> {
        record(rpc, &mut records, "getGenesisHash", json!([]))?;
        let source_mint = record(
            rpc,
            &mut records,
            "getAccountInfo",
            json!([plan.source_mint, config(case.discovery_slot)]),
        )?;
        let source_slot = shared::slot(&source_mint).ok()?;
        let replacement = record(
            rpc,
            &mut records,
            "getAccountInfo",
            json!([plan.replacement_mint, config(source_slot)]),
        )?;
        let replacement_slot = shared::slot(&replacement).ok()?;
        let replacement_program = replacement["value"]["owner"].as_str()?.to_string();
        let mut programs = vec![
            source_program.to_string(),
            replacement_program.clone(),
            shared::ATA_PROGRAM.into(),
        ];
        programs.sort();
        programs.dedup();
        let headers = record(
            rpc,
            &mut records,
            "getMultipleAccounts",
            json!([programs, config(replacement_slot)]),
        )?;
        let programdata = demo::programdata_addresses(headers["value"].as_array()?).ok()?;
        let overlay = demo::derive(
            &case.case_plan_sha256,
            &case.authority,
            &plan.replacement_mint,
            &replacement_program,
        )
        .ok()?;
        let addresses = demo::address_plan(
            plan,
            &overlay,
            &case.authority,
            source_program,
            &replacement_program,
            &programdata,
        );
        record(
            rpc,
            &mut records,
            "getMultipleAccounts",
            json!([addresses, config(shared::slot(&headers).ok()?)]),
        )?;
        Some(())
    })();
    let _ = attempt;
    records
}

/// Capture fresh current state for every selected case, in frozen plan order.
/// A case whose acquisition fails keeps its partial record and is never dropped.
pub fn capture_cases(
    plan: &StressTestPlan,
    plan_sha256: &str,
    source_program: &str,
    rpc: &impl SolanaRpc,
) -> Result<CaptureBundle> {
    ensure!(
        plan.sha256()? == plan_sha256,
        "the frozen stress plan digest does not match the plan being captured for"
    );
    let total = plan.selected.len();
    let mut cases = Vec::with_capacity(total);
    let started_at = now();
    for (index, case) in plan.selected.iter().enumerate() {
        eprintln!(
            "CURRENT_STAGE:Revalidating current state for case {}/{total}",
            index + 1
        );
        let case_started = now();
        let observations = acquire_case(case, source_program, rpc);
        cases.push(CaseCapture {
            case_id: case.case_id.clone(),
            entity_id: case.entity_id.clone(),
            token_account: case.token_account.clone(),
            case_plan_sha256: case.case_plan_sha256.clone(),
            started_at: case_started,
            completed_at: now(),
            observations,
        });
    }
    Ok(CaptureBundle {
        schema_version: 1,
        kind: BUNDLE_KIND.into(),
        stress_id: plan.stress_id.clone(),
        run_id: plan.run_id.clone(),
        stress_plan_sha256: plan_sha256.into(),
        population_capture_sha256: plan.population_capture_sha256.clone(),
        rpc_origin: rpc.origin(),
        started_at,
        completed_at: now(),
        cases,
    })
}

pub fn save<T: Serialize>(value: &T, path: &Path) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(&serde_json::to_vec(value)?)?;
    file.sync_all()?;
    Ok(())
}

fn case_result(
    case: &SelectedCase,
    capture: &CaseCapture,
    observation: &PopulationObservation,
    plan: &StressTestPlan,
    program: &[u8],
    program_sha256: &str,
) -> Result<CaseResult> {
    let mint = observation
        .mint_config
        .as_ref()
        .context("stress execution requires a decoded current mint")?;
    let entity = observation
        .entities
        .iter()
        .find(|e| e.token_account == case.token_account)
        .context("the selected case is not an entity of this population capture")?;
    // The assumed local signer is granted in exactly one place, by authority
    // model alone. A program-owned, multisig or unresolved authority never gets it.
    let assumed_signer =
        classify::assumed_local_signer(&entity.authority_model, entity.authority_resolution);
    let amount: u64 = case.selected_amount_raw.parse()?;

    let mut detail = json!({
        "population_capture_sha256": plan.population_capture_sha256,
        "state_shape_sha256": case.state_shape_sha256,
        "selection_reason": case.selection_reason,
        "selection_detail": case.selection_detail,
        "amount_policy": case.amount_policy,
        "amount_capped": case.amount_capped,
        "observed_balance_at_population_capture_raw": case.observed_balance_raw,
        "authority_model": entity.authority_model,
        "authority_resolution": entity.authority_resolution,
        "authority_classification_reason": entity.classification_reason,
        "scope": "Only this exact source account, source mint, frozen amount, replacement mint, case plan version, candidate program build, proposed overlay, captured bank and assumed local signing. No other account, amount, plan, state shape or population member inherits this evidence.",
    });
    let acquisition = json!({
        "started_at": capture.started_at,
        "completed_at": capture.completed_at,
        "commitment": "finalized",
        "discovery_slot": case.discovery_slot,
        "requests_performed": capture.observations.len(),
        "request_budget": plan.budget.rpc_requests_per_case,
        "consistency": "The population enumeration and this case check are separate finalized observations. The final captured batch is authoritative for this case's execution.",
    });

    let finish = |detail: Value,
                  status: PathStatus,
                  reason: Option<String>,
                  executed: bool,
                  fixture: Option<String>|
     -> Result<CaseResult> {
        let mut result = CaseResult {
            case_id: case.case_id.clone(),
            entity_id: case.entity_id.clone(),
            token_account: case.token_account.clone(),
            authority: case.authority.clone(),
            authority_model: entity.authority_model.clone(),
            state_shape_sha256: case.state_shape_sha256.clone(),
            shape_label: case.shape_label.clone(),
            balance_bucket: case.balance_bucket,
            selection_reason: case.selection_reason,
            selected_amount_raw: case.selected_amount_raw.clone(),
            selected_amount_decimal: case.selected_amount_decimal.clone(),
            case_plan_sha256: case.case_plan_sha256.clone(),
            candidate_program_sha256: program_sha256.into(),
            status,
            reason,
            execution_performed: executed,
            local_execution_performed: executed,
            signer_assumed_locally: executed && assumed_signer,
            signer_possession_known: false,
            candidate_authority_assumed_locally: true,
            issuer_binding_established: false,
            official_transition: PathStatus::NotTested,
            funds_moved: false,
            execution_fixture_sha256: fixture,
            acquisition: acquisition.clone(),
            detail,
            result_sha256: String::new(),
        };
        // The digest covers the whole case result with an empty digest slot, so
        // replay recomputes exactly the same value.
        result.result_sha256 = digest(&result)?;
        Ok(result)
    };

    ensure!(
        assumed_signer,
        "a selected stress case must be an executable wallet-compatible authority; no other authority model may receive an assumed local signer"
    );
    if capture.observations.len() != plan.budget.rpc_requests_per_case
        || capture.observations.iter().any(|r| r.error.is_some())
    {
        return finish(
            detail,
            PathStatus::Indeterminate,
            Some("Required public state for this case could not be captured within the bounded request budget. No execution was attempted and no outcome is claimed.".into()),
            false,
            None,
        );
    }
    let evidence: Vec<RpcEvidence> = capture
        .observations
        .iter()
        .enumerate()
        .map(|(id, r)| RpcEvidence {
            id,
            method: r.method.clone(),
            params: r.params.clone(),
            result: r.result.clone().unwrap(),
        })
        .collect();
    ensure!(
        evidence[0].result.as_str() == Some(observation.acquisition.genesis_hash.as_str()),
        "a stress case was captured on a different chain than its population"
    );
    let context = demo::ConversionContext {
        genesis_hash: observation.acquisition.genesis_hash.clone(),
        minimum_slot: case.discovery_slot,
        owner: case.authority.clone(),
        source_program: mint.token_program.clone(),
        source_decimals: mint.decimals,
        amount,
    };
    let built = match demo::build(
        &case.case_plan,
        &case.case_plan_sha256,
        &context,
        &evidence,
        program,
    ) {
        Ok(built) => built,
        Err(error) => {
            let text = format!("{error:#}");
            let unsupported = text.contains("Unsupported: ");
            return finish(
                detail,
                if unsupported {
                    PathStatus::Unsupported
                } else {
                    PathStatus::Indeterminate
                },
                Some(format!(
                    "The candidate conversion precondition could not be established for this production state: {text}"
                )),
                false,
                None,
            );
        }
    };
    // Whether this exact account still matches the frozen population observation.
    // Reported either way; the execution fixture remains authoritative.
    detail["source_revalidated"] = json!({
        "population_balance_raw": case.observed_balance_raw,
        "execution_bank_balance_raw": built.source_before_raw,
        "unchanged_since_population_capture": built.source_before_raw == case.observed_balance_raw,
        "execution_fixture_is_authoritative": true,
    });
    let p = &built.plan;
    detail["destination"] = built.overlay.destination.clone().into();
    detail["proposed_overlay"] = json!({
        "origin": "Proposed",
        "addresses": built.overlay,
        "reserve_funded_replacement_raw": case.case_plan.reserve.funded_replacement_raw,
        "note": "Deterministically derived from the candidate program, this case's plan digest and the observed replacement identity. None of it is observed mainnet state and none of it is issuer controlled."
    });
    detail["observed_state"] = json!({
        "origin": "Observed",
        "source_balance_before_raw": built.source_before_raw,
        "source_decimals": built.source_decimals,
        "replacement_decimals": built.replacement_decimals,
        "replacement_token_program": built.replacement_program,
        "holder_replacement_account_existed": built.destination_existed,
    });
    detail["fixture_accounts"] = serde_json::to_value(&built.accounts)?;
    detail["expected"] = serde_json::to_value(&built.expectation)?;
    detail["clock"] = serde_json::to_value(ProbeClock::from(&p.clock))?;
    detail["message"] = serde_json::to_value(ProbeMessage::from(&p.message))?;
    detail["account_evidence"] = serde_json::to_value(&p.account_evidence)?;
    detail["account_plan"] = evidence[4].params[0].clone();
    detail["deployed_programs"] = json!(p
        .programs
        .iter()
        .map(|loaded| {
            let id = loaded.program_id.to_string();
            json!({"program": id, "loader": loaded.loader.to_string(),
                "code_sha256": sha256(&loaded.bytes),
                "origin": if id == demo::PROGRAM_ID { "Proposed" } else { "Observed" },
                "deployed_on_mainnet": id != demo::PROGRAM_ID})
        })
        .collect::<Vec<_>>());
    detail["assumptions"] = serde_json::to_value(&p.assumptions)?;
    detail["preconditions"] = serde_json::to_value(&p.preconditions)?;
    detail["runtime_profile"] = json!({"backend":"LiteSVM 0.16","features":"pinned mainnet/default profile","signature_verification":false,"recent_blockhash_verification":false,"synthetic_fee_payer":shared::payer().to_string(),"validator_bank_reproduction":false,"network_access":false});
    let fixture = digest(&json!({
        "case_plan_sha256": case.case_plan_sha256,
        "candidate_program_sha256": built.candidate_program_sha256,
        "accounts": built.accounts,
        "clock": ProbeClock::from(&p.clock),
        "message": ProbeMessage::from(&p.message),
        "account_plan": evidence[4].params[0],
    }))?;
    detail["execution_fixture_sha256"] = fixture.clone().into();

    demo::assert_candidate_program_identity(&p.programs, program, program_sha256)?;
    let execution = executor::execute_probe_message(
        &p.accounts,
        &p.watch,
        p.clock.clone(),
        &p.programs,
        p.message.clone(),
    )?;
    detail["execution"] = serde_json::to_value(&execution)?;
    let deltas = match demo::reconcile(&case.case_plan, &built, amount, &execution) {
        Ok(deltas) => deltas,
        Err(error) => {
            return finish(
                detail,
                PathStatus::Indeterminate,
                Some(format!("The candidate instruction ran for this production state, but exact reconciliation could not be established: {error:#}")),
                true,
                Some(fixture),
            )
        }
    };
    let status = if !deltas.reconciled {
        PathStatus::Indeterminate
    } else if execution.success {
        PathStatus::Proven
    } else {
        PathStatus::Failed
    };
    let reason = if !deltas.reconciled {
        Some("The candidate instruction ran, but exact source, supply, ratio, rounding, fee or replacement deltas did not reconcile. No conversion proof was granted for this production state.".to_string())
    } else if !execution.success {
        Some(format!(
            "The candidate conversion failed in local simulation for this production state: {}. No mainnet funds moved and the watched accounts rolled back.",
            execution
                .error
                .clone()
                .unwrap_or_else(|| "instruction error".into())
        ))
    } else {
        None
    };
    detail["reconciliation"] = serde_json::to_value(&deltas)?;
    detail["rollback_verified"] = json!(!execution.success && deltas.reconciled);
    finish(detail, status, reason, true, Some(fixture))
}

/// Opaque verified stress test. Deliberately not `Deserialize`: only actual local
/// executions plus exact reconciliation and the structural invariants can build
/// one, so a serialized Proven row is never accepted as evidence.
#[derive(Serialize)]
pub struct VerifiedConversionStressTest {
    result: ConversionStressTestResult,
}
impl VerifiedConversionStressTest {
    pub fn value(&self) -> &ConversionStressTestResult {
        &self.result
    }
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self.result)?)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn replay(
    population_bytes: &[u8],
    plan_bytes: &[u8],
    bundle_bytes: &[u8],
    stress_id: &str,
    run_id: &str,
    population_sha256: &str,
    plan_sha256: &str,
    bundle_sha256: &str,
    program: &[u8],
    program_sha256: &str,
    budget: &StressBudget,
    evaluated_at: &str,
) -> Result<VerifiedConversionStressTest> {
    ensure!(
        sha256(population_bytes) == population_sha256
            && sha256(plan_bytes) == plan_sha256
            && sha256(bundle_bytes) == bundle_sha256,
        "stress input digest mismatch"
    );
    ensure!(
        sha256(program) == program_sha256,
        "candidate program digest mismatch"
    );
    eprintln!("CURRENT_STAGE:Verifying population capture");
    let observation = super::population::evaluate_bytes(population_bytes, budget)?;
    ensure!(
        observation.capture_sha256 == population_sha256
            && observation.run_id == run_id
            && observation.stress_id == stress_id,
        "population capture identity mismatch"
    );
    let plan: StressTestPlan = serde_json::from_slice(plan_bytes)?;
    ensure!(
        plan.schema_version == 1
            && plan.kind == super::select::PLAN_KIND
            && plan.stress_id == stress_id
            && plan.run_id == run_id
            && plan.population_capture_sha256 == population_sha256
            && plan.candidate_program_sha256 == program_sha256
            && plan.sha256()? == plan_sha256,
        "stress plan binding mismatch"
    );
    eprintln!("CURRENT_STAGE:Verifying frozen selection plan");
    // Recompute the entire pre-execution selection. Classifications, selected
    // cases, amounts and reasons cannot have been edited after the fact.
    plan.validate(&observation, &plan.candidate_plan, program_sha256)?;

    let bundle: CaptureBundle = serde_json::from_slice(bundle_bytes)?;
    ensure!(
        bundle.schema_version == 1
            && bundle.kind == BUNDLE_KIND
            && bundle.stress_id == stress_id
            && bundle.run_id == run_id
            && bundle.stress_plan_sha256 == plan_sha256
            && bundle.population_capture_sha256 == population_sha256
            && bundle.cases.len() == plan.selected.len(),
        "stress case bundle binding mismatch"
    );
    let bundle_start = chrono::DateTime::parse_from_rfc3339(&bundle.started_at)?;
    let bundle_end = chrono::DateTime::parse_from_rfc3339(&bundle.completed_at)?;
    ensure!(
        bundle_start <= bundle_end,
        "invalid stress capture interval"
    );

    let mut results = Vec::with_capacity(plan.selected.len());
    for (index, case) in plan.selected.iter().enumerate() {
        let capture = &bundle.cases[index];
        ensure!(
            capture.case_id == case.case_id
                && capture.entity_id == case.entity_id
                && capture.token_account == case.token_account
                && capture.case_plan_sha256 == case.case_plan_sha256,
            "a captured case is not the frozen case at this position; selected cases may not be reordered or replaced"
        );
        ensure!(
            capture.observations.len() <= budget.rpc_requests_per_case,
            "a stress case exceeded its declared request budget"
        );
        let start = chrono::DateTime::parse_from_rfc3339(&capture.started_at)?;
        let end = chrono::DateTime::parse_from_rfc3339(&capture.completed_at)?;
        ensure!(
            bundle_start <= start && start <= end && end <= bundle_end,
            "invalid case acquisition interval"
        );
        for r in &capture.observations {
            let a = chrono::DateTime::parse_from_rfc3339(&r.started_at)?;
            let b = chrono::DateTime::parse_from_rfc3339(&r.completed_at)?;
            ensure!(
                start <= a && a <= b && b <= end && r.result.is_some() != r.error.is_some(),
                "invalid case acquisition record"
            );
        }
        eprintln!(
            "CURRENT_STAGE:Running candidate conversion locally for case {}/{}",
            index + 1,
            plan.selected.len()
        );
        results.push(case_result(
            case,
            capture,
            &observation,
            &plan,
            program,
            program_sha256,
        )?);
    }
    eprintln!("CURRENT_STAGE:Aggregating stress coverage");
    aggregate(
        observation,
        plan,
        bundle,
        results,
        program_sha256,
        evaluated_at,
    )
}

/// Join exact case evidence with population facts. Nothing here extrapolates: a
/// per-entity outcome is never widened to its shape, its bucket or the population,
/// and independent conversion outputs are never summed into available capacity.
fn aggregate(
    observation: PopulationObservation,
    plan: StressTestPlan,
    bundle: CaptureBundle,
    results: Vec<CaseResult>,
    program_sha256: &str,
    evaluated_at: &str,
) -> Result<VerifiedConversionStressTest> {
    // One balance per exact entity id, so nothing is double counted across views.
    let balances: BTreeMap<String, u64> = observation
        .positive_entities()
        .map(|e| Ok((e.entity_id.clone(), e.balance()?)))
        .collect::<Result<_>>()?;
    let pick = |ids: &BTreeSet<String>| -> BTreeMap<String, u64> {
        balances
            .iter()
            .filter(|(id, _)| ids.contains(*id))
            .map(|(id, b)| (id.clone(), *b))
            .collect()
    };
    let selected_ids: BTreeSet<String> =
        plan.selected.iter().map(|c| c.entity_id.clone()).collect();
    let proven_ids: BTreeSet<String> = results
        .iter()
        .filter(|r| r.status == PathStatus::Proven)
        .map(|r| r.entity_id.clone())
        .collect();

    let mut shape_coverage = Vec::with_capacity(plan.state_shapes.len());
    for shape in &plan.state_shapes {
        let executed: Vec<&CaseResult> = results
            .iter()
            .filter(|r| r.state_shape_sha256 == shape.state_shape_sha256 && r.execution_performed)
            .collect();
        let executed_ids: BTreeSet<String> = executed.iter().map(|r| r.entity_id.clone()).collect();
        shape_coverage.push(ShapeCoverage {
            state_shape_sha256: shape.state_shape_sha256.clone(),
            shape_label: shape.shape_label.clone(),
            eligibility: shape.eligibility,
            entities_in_shape: shape.entities_in_shape,
            entities_selected: shape.entities_selected,
            entities_executed: executed_ids.len(),
            executed_entity_ids: executed_ids.iter().cloned().collect(),
            entities_untested: shape.entities_in_shape - executed_ids.len(),
            represented_raw: shape.represented_raw.clone(),
            tested_raw: sum_once(&pick(&executed_ids)),
            proof_scope: SHAPE_PROOF_SCOPE.into(),
        });
    }

    // Production states the current executor cannot exercise, plus any state that
    // a selected case discovered to be unsupported at execution time.
    let mut unsupported_summary: Vec<UnsupportedState> = plan
        .state_shapes
        .iter()
        .filter(|s| s.eligibility != Eligibility::ExecutableCandidate)
        .map(|s| UnsupportedState {
            state_shape_sha256: s.state_shape_sha256.clone(),
            shape_label: s.shape_label.clone(),
            eligibility: s.eligibility,
            reason: s.eligibility_reason.clone(),
            positive_balance_accounts: s.entities_in_shape,
            represented_raw: s.represented_raw.clone(),
            highest_balance_entities: s
                .highest_balance_entities
                .iter()
                .take(MAX_SHAPE_EXAMPLES)
                .cloned()
                .collect(),
            boundary: EXECUTOR_BOUNDARY.into(),
        })
        .collect();
    for r in results
        .iter()
        .filter(|r| r.status == PathStatus::Unsupported)
    {
        let shape = plan
            .state_shapes
            .iter()
            .find(|s| s.state_shape_sha256 == r.state_shape_sha256);
        unsupported_summary.push(UnsupportedState {
            state_shape_sha256: r.state_shape_sha256.clone(),
            shape_label: r.shape_label.clone(),
            eligibility: Eligibility::Unsupported,
            reason: r.reason.clone().unwrap_or_else(|| {
                "The selected case for this state shape was unsupported at execution time.".into()
            }),
            positive_balance_accounts: shape.map(|s| s.entities_in_shape).unwrap_or(1),
            represented_raw: shape
                .map(|s| s.represented_raw.clone())
                .unwrap_or_else(|| r.selected_amount_raw.clone()),
            highest_balance_entities: vec![r.token_account.clone()],
            boundary: EXECUTOR_BOUNDARY.into(),
        });
    }
    unsupported_summary.sort_by(|a, b| {
        b.positive_balance_accounts
            .cmp(&a.positive_balance_accounts)
            .then_with(|| a.state_shape_sha256.cmp(&b.state_shape_sha256))
    });

    let unsupported_authority_classes = plan
        .state_shapes
        .iter()
        .filter(|s| s.eligibility != Eligibility::ExecutableCandidate)
        .map(|s| s.dimensions.authority_model.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let shape_entities = |e: Eligibility| -> usize {
        plan.state_shapes
            .iter()
            .filter(|s| s.eligibility == e)
            .map(|s| s.entities_in_shape)
            .sum()
    };
    let coverage_summary = CoverageSummary {
        accounts_observed: observation.summary.token_accounts_observed,
        positive_balance_accounts_observed: observation.summary.positive_balance_accounts_observed,
        zero_balance_accounts_observed: observation.summary.zero_balance_accounts_observed,
        undecodable_rows_observed: observation.summary.undecodable_rows_observed,
        exact_accounts_selected: plan.selected.len(),
        exact_accounts_executed: results.iter().filter(|r| r.execution_performed).count(),
        exact_accounts_proven: count(&results, PathStatus::Proven),
        exact_accounts_failed: count(&results, PathStatus::Failed),
        exact_accounts_indeterminate: count(&results, PathStatus::Indeterminate),
        exact_accounts_unsupported: count(&results, PathStatus::Unsupported),
        population_public_balance_raw: sum_once(&balances),
        tested_public_balance_raw: sum_once(&pick(&selected_ids)),
        proven_public_balance_raw: sum_once(&pick(&proven_ids)),
        state_shapes_discovered: plan.state_shapes.len(),
        state_shapes_executable: plan
            .state_shapes
            .iter()
            .filter(|s| s.eligibility == Eligibility::ExecutableCandidate)
            .count(),
        state_shapes_with_executed_case: shape_coverage
            .iter()
            .filter(|s| s.entities_executed > 0)
            .count(),
        unsupported_authority_classes,
        unsupported_positive_balance_accounts: shape_entities(Eligibility::Unsupported),
        capture_required_positive_balance_accounts: shape_entities(Eligibility::CaptureRequired),
        note: COVERAGE_NOTE.into(),
    };

    // Structural invariants run before any readiness is evaluated.
    assert_no_proof_inheritance(&plan.selected, &results, &shape_coverage)?;

    let (readiness, population_rollout_readiness) = super::readiness::evaluate_stress(
        &observation,
        &plan,
        &results,
        &shape_coverage,
        evaluated_at,
    )?;

    let failures: Vec<CaseResult> = results
        .iter()
        .filter(|r| r.status == PathStatus::Failed)
        .cloned()
        .collect();

    let result = ConversionStressTestResult {
        schema_version: 1,
        kind: RESULT_KIND.into(),
        stress_id: plan.stress_id.clone(),
        run_id: plan.run_id.clone(),
        asset_mint: plan.asset_mint.clone(),
        population_capture: json!({
            "capture_sha256": observation.capture_sha256,
            "acquisition": observation.acquisition,
            "enumeration": observation.enumeration,
            "authority_resolution": observation.authority_resolution,
            "mint": observation.mint_config,
            "mint_slot": observation.mint_slot,
            "undecoded_rows": observation.undecoded,
            "budget": observation.budget,
            "historical_population_used": false,
            "note": "A new bounded read-only acquisition of current state. No historical population, balance or classification was imported or substituted.",
        }),
        candidate_plan_sha256: plan.candidate_plan_sha256.clone(),
        candidate_plan: plan.candidate_plan.clone(),
        candidate_mechanism: json!({
            "id": demo::PROGRAM_ID,
            "name": "Eplyx Demo Candidate Conversion",
            "adapter_id": crate::conversion::ADAPTER_ID,
            "revision": demo::REVISION,
            "artifact": demo::ARTIFACT,
            "program_sha256": program_sha256,
            "loader": demo::LOADER,
            "origin": "Proposed",
            "registered_by": "This repository",
            "deployed_on_mainnet": false,
            "issuer_mechanism": false,
            "program_id_preimage": demo::PROGRAM_PREIMAGE,
            "note": "A candidate mechanism the operator intends to deploy. It is not deployed on any cluster, holds no issuer authority and is not any issuer's mechanism for this or any other asset.",
        }),
        classifier: json!({
            "version": plan.classifier_version,
            "classification_sha256": plan.classification_sha256,
            "eligibility_counts": plan.eligibility_counts,
            "positive_balance_entities_classified": plan.positive_balance_entities,
            "scope": "State shapes are computed over the positive-balance token accounts observed in this capture. A shape groups execution-relevant characteristics; it is not a risk score and not a proof equivalence class.",
            "eligibility_keys": [
                eligibility_key(&Eligibility::ExecutableCandidate),
                eligibility_key(&Eligibility::CaptureRequired),
                eligibility_key(&Eligibility::Unsupported),
                eligibility_key(&Eligibility::Invalid),
            ],
        }),
        selection_plan: json!({
            "stress_plan_sha256": plan.sha256()?,
            "selector_version": plan.selector_version,
            "ordering_rule": plan.ordering_rule,
            "selection_strategy": plan.selection_strategy,
            "buckets": plan.buckets,
            "budget": plan.budget,
            "frozen_at": plan.frozen_at,
            "frozen_before_execution": true,
            "case_capture": {
                "started_at": bundle.started_at,
                "completed_at": bundle.completed_at,
                "rpc_origin": bundle.rpc_origin,
                "cases": bundle.cases.len(),
            },
        }),
        population_summary: json!({
            "counts": observation.summary,
            "enumeration_completeness": plan.enumeration_completeness,
            "authority_resolution_completeness": plan.authority_resolution_completeness,
            "independent_axes": crate::stress::population::AUTHORITY_INDEPENDENCE,
        }),
        state_shapes: plan.state_shapes.clone(),
        selected_cases: plan.selected.clone(),
        results,
        coverage_summary,
        shape_coverage,
        unsupported_summary,
        failures,
        readiness,
        official_transition: PathStatus::NotTested,
        population_rollout_readiness,
        execution_performed: true,
        funds_moved: false,
        authorization: false,
        limitations: vec![
            "Evidence is exact to each tested entity, amount, case plan, candidate program build and captured bank. No tested account establishes anything about its peers.".into(),
            "State-shape coverage records which classes were exercised. It never means every account sharing a shape is proven.".into(),
            "Unsupported and indeterminate outcomes are executor and evidence boundaries. They are not proof that those accounts cannot convert, and they are never collapsed into failure.".into(),
            "A proven candidate conversion is evidence about the supplied plan under its declared authority model. OfficialTransition remains NotTested and no issuer binding is established.".into(),
            "Independent case outputs are never summed into rollout capacity or available liquidity, and each entity's balance is counted once.".into(),
            "Holder signing is assumed locally for wallet-compatible authorities only; key possession stays unknown. No mainnet transaction was constructed, signed or submitted and no funds moved.".into(),
            "Refreshing current state creates a new stress world. This result does not carry over to any later capture.".into(),
        ],
    };
    Ok(VerifiedConversionStressTest { result })
}

fn count(results: &[CaseResult], status: PathStatus) -> usize {
    results.iter().filter(|r| r.status == status).count()
}
