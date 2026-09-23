//! Fresh package pre-flight and fully offline replay using the existing M6/M7 paths.
use super::{current as conversion, invariants, package, package_gate};
use crate::{
    lifecycle::{current as wallet, exposure::sha256, rpc::HttpSolanaRpc},
    stress::{authority, execute, population, select, StressBudget},
};
use anyhow::{ensure, Context, Result};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

const VERSION: u32 = 1;
const WALLET_FILE: &str = "wallet.capture.json";
const CONVERSION_FILE: &str = "conversion.capture.json";
const POPULATION_FILE: &str = "population.capture.json";
const PLAN_FILE: &str = "stress.plan.json";
const CASES_FILE: &str = "stress.cases.json";
const AUTHORITY_PLAN_FILE: &str = "authority.plan.json";
const AUTHORITY_REPORT_FILE: &str = "authority.report.json";
const OFFLINE_VM_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Bindings {
    schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    gate_policy: Option<package_gate::Policy>,
    transition_package_sha256: String,
    candidate_program_sha256: String,
    config_sha256: String,
    plan_sha256: String,
    wallet_sha256: String,
    conversion_capture_sha256: String,
    population_sha256: String,
    stress_plan_sha256: String,
    cases_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authority_plan_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authority_report_sha256: Option<String>,
    run_id: String,
    check_id: String,
    stress_id: String,
    evaluated_at: String,
    budget: StressBudget,
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn read(root: &Path, file: &str, max: u64) -> Result<Vec<u8>> {
    let bytes = fs::read(root.join(file))?;
    ensure!(
        bytes.len() as u64 <= max,
        "pre-flight artifact exceeds bound"
    );
    Ok(bytes)
}

fn write(root: &Path, file: &str, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut out = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(file))?;
    out.write_all(bytes)?;
    out.sync_all()?;
    Ok(())
}

fn summary(
    package: &package::ValidatedPackage,
    bindings: &Bindings,
    conversion: &Value,
    stress: &Value,
    authority: Option<&authority::Report>,
) -> Value {
    let selected: Vec<Value> = stress["selected_cases"].as_array().into_iter().flatten().map(|c| json!({
        "case_id": c["case_id"], "entity_id": c["entity_id"], "token_account": c["token_account"],
        "authority": c["authority"], "selected_amount_raw": c["selected_amount_raw"],
        "case_plan_sha256": c["case_plan_sha256"], "state_shape_sha256": c["state_shape_sha256"],
        "selection_reason": c["selection_reason"],
    })).collect();
    let outcomes: Vec<Value> = stress["results"].as_array().into_iter().flatten().map(|r| {
        let mut outcome = json!({
            "case_id": r["case_id"], "entity_id": r["entity_id"], "status": r["status"],
            "reason": r["reason"], "execution_performed": r["execution_performed"],
            "execution_fixture_sha256": r["execution_fixture_sha256"], "result_sha256": r["result_sha256"],
        });
        if let Some(context) = r["detail"].get("execution_context") {
            outcome["execution_context"] = context.clone();
        }
        if let Some(revalidation) = r["detail"].get("revalidation") {
            outcome["revalidation"] = revalidation.clone();
            outcome["execution_plan_sha256"] = r["detail"]["execution_plan_sha256"].clone();
            outcome["resolved_case_plan_sha256"] = r["detail"]["resolved_case_plan_sha256"].clone();
        }
        outcome
    }).collect();
    let failed: Vec<Value> = outcomes
        .iter()
        .filter(|r| r["status"] == "Failed")
        .cloned()
        .collect();
    let unresolved: Vec<Value> = outcomes
        .iter()
        .filter(|r| r["status"] != "Proven" && r["status"] != "Failed")
        .cloned()
        .collect();
    let unsupported: Vec<Value> = stress["unsupported_summary"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|u| {
            json!({
                "state_shape_sha256": u["state_shape_sha256"],
                "positive_balance_accounts": u["positive_balance_accounts"],
                "reason": u["reason"],
            })
        })
        .collect();
    let candidate = if conversion["status"] == "Proven" {
        "Ready"
    } else if conversion["status"] == "Failed" {
        "Blocked"
    } else {
        "Incomplete"
    };
    let stress_status = stress["readiness"]["status"]
        .as_str()
        .unwrap_or("Incomplete");
    let declared = if candidate == "Blocked" || stress_status == "Blocked" {
        "Blocked"
    } else if candidate == "Ready" && stress_status == "Ready" {
        "Ready"
    } else {
        "Incomplete"
    };
    let mut report = json!({
        "schema_version": VERSION,
        "transition_package_sha256": bindings.transition_package_sha256,
        "candidate_program_sha256": bindings.candidate_program_sha256,
        "config_sha256": bindings.config_sha256,
        "adapter": package::ADAPTER,
        "adapter_version": VERSION,
        "deployment_origin": "Proposed",
        "provenance": "OperatorSupplied",
        "source_mint": package.manifest.source_mint,
        "replacement_mint": package.manifest.replacement_mint,
        "source_account": package.config.source_account,
        "public_owner": package.config.public_owner,
        "effective_at": package.manifest.effective_at,
        "proposed_reserve_raw": package.config.reserve_funded_replacement_raw,
        "terms": package.manifest.terms,
        "current_capture_timestamp": stress["population_capture"]["acquisition"]["completed_at"],
        "evaluated_at": bindings.evaluated_at,
        "run_id": bindings.run_id,
        "exact_selected_cases": selected,
        "population_summary": stress["population_summary"],
        "conversion_result": {
            "status": conversion["status"],
            "reason": conversion["reason"],
            "plan_sha256": conversion["plan_sha256"],
            "execution_fixture_sha256": conversion["execution_fixture_sha256"],
            "execution_performed": conversion["execution_performed"],
            "reconciliation": {"reconciled": conversion["reconciliation"]["reconciled"]},
        },
        "stress_results": outcomes,
        "failed_cases": failed,
        "unresolved_cases": unresolved,
        "unsupported_states": unsupported,
        "candidate_plan_readiness": candidate,
        "conversion_stress_readiness": {
            "status": stress["readiness"]["status"],
            "policy_sha256": stress["readiness"]["policy_sha256"],
            "findings_count": stress["readiness"]["findings"].as_array().map_or(0, Vec::len),
        },
        "official_transition": "NotTested",
        "population_rollout_readiness": {
            "status": stress["population_rollout_readiness"]["status"],
            "policy_sha256": stress["population_rollout_readiness"]["policy_sha256"],
        },
        "declared_preflight_status": declared,
        "limitations": [
            "OperatorSupplied package identity does not establish issuer authorization or an official transition.",
            "Only exact selected accounts, amounts, captured banks and locally assumed signing can receive conversion proof.",
            "Stress sampling does not establish population rollout readiness.",
            "No funds moved"
        ],
        "funds_moved": false,
    });
    if let Some(context) = conversion.get("execution_context") {
        report["conversion_result"]["execution_context"] = context.clone();
    }
    if let Some(revalidation) = conversion.get("mint_revalidated") {
        report["conversion_result"]["mint_revalidated"] = revalidation.clone();
        report["conversion_result"]["source_revalidated"] =
            conversion["source_revalidated"].clone();
        report["conversion_result"]["discovery_amount_raw"] =
            conversion["discovery_amount_raw"].clone();
        report["conversion_result"]["final_execution_amount_raw"] =
            conversion["amount_raw"].clone();
    }
    if let Some(rebinding) = stress["selection_plan"].get("rebinding") {
        report["stress_rebinding"] = rebinding.clone();
        for (selected, case) in report["exact_selected_cases"]
            .as_array_mut()
            .into_iter()
            .flatten()
            .zip(stress["selected_cases"].as_array().into_iter().flatten())
        {
            selected["selection_bucket"] = case["balance_bucket"].clone();
            selected["discovery_amount_raw"] = case["observed_balance_raw"].clone();
            selected["amount_policy"] = case["amount_policy"].clone();
        }
    }
    if let Some(authority) = authority {
        report["non_standard_account_control"] = serde_json::to_value(authority).unwrap();
        report["refined_stress_world_sha256"] = json!(crate::expansion::digest(&(
            &bindings.population_sha256,
            &authority.plan_sha256,
            &bindings.stress_plan_sha256,
        ))
        .unwrap());
    }
    report
}

fn markdown(report: &Value) -> String {
    let analytical_label = if report.get("deployment_gate").is_some() {
        "Analytical pre-flight status"
    } else {
        "Declared pre-flight policy" // Preserve historical Milestone 8 Markdown bytes.
    };
    let mut output = format!(
        "# Transition package pre-flight\n\nPackage: `{}`\n\nCandidate program: `{}` (Proposed)\n\nSource: `{}`\nReplacement: `{}`\n\nSelected stress cases: {}\n\nCandidatePlanReadiness: **{}**\nConversionStressReadiness: **{}**\nPopulationRolloutReadiness: **{}**\nOfficialTransition: **NotTested**\n{}: **{}**\n\nNo funds moved. Holder signing was assumed locally; key possession and issuer binding remain unknown.\n",
        report["transition_package_sha256"].as_str().unwrap_or(""),
        report["candidate_program_sha256"].as_str().unwrap_or(""),
        report["source_mint"].as_str().unwrap_or(""),
        report["replacement_mint"].as_str().unwrap_or(""),
        report["exact_selected_cases"].as_array().map_or(0, Vec::len),
        report["candidate_plan_readiness"].as_str().unwrap_or("Incomplete"),
        report["conversion_stress_readiness"]["status"].as_str().unwrap_or("Incomplete"),
        report["population_rollout_readiness"]["status"].as_str().unwrap_or("Incomplete"),
        analytical_label,
        report["declared_preflight_status"].as_str().unwrap_or("Incomplete"),
    );
    if let Some(rebinding) = report.get("stress_rebinding") {
        output.push_str(&format!(
            "\n## Final-state stress evidence\n\nFrozen identities: {}; executable at final capture: {}; locally tested: {}; no longer executable: {}; identity changed: {}. Discovery shapes preserved among executions: {}; discovery balance buckets preserved among executions: {}. Execution evidence applies only to the exact final account state and amount.\n",
            rebinding["selected_identities"].as_u64().unwrap_or(0),
            rebinding["executable_current_state"].as_u64().unwrap_or(0)
                + rebinding["selection_state_changed_but_executable"]
                    .as_u64()
                    .unwrap_or(0),
            rebinding["exact_final_state_executions"]
                .as_u64()
                .unwrap_or(0),
            rebinding["no_longer_executable"].as_u64().unwrap_or(0),
            rebinding["identity_changed"].as_u64().unwrap_or(0),
            rebinding["discovery_shapes_preserved_at_execution"]
                .as_u64()
                .unwrap_or(0),
            rebinding["discovery_buckets_preserved_at_execution"]
                .as_u64()
                .unwrap_or(0),
        ));
    }
    if let Some(findings) = report.get("invariants").and_then(Value::as_array) {
        output.push_str("\n## Rollout invariants\n\n");
        for finding in findings {
            output.push_str(&format!(
                "- **{}** {} ({}, {}): {}\n",
                finding["status"].as_str().unwrap_or("Indeterminate"),
                finding["invariant_type"].as_str().unwrap_or("unknown"),
                finding["severity"].as_str().unwrap_or("unknown"),
                finding["scope"].as_str().unwrap_or("unknown"),
                finding["explanation"].as_str().unwrap_or("")
            ));
        }
    }
    if let Some(gate) = report.get("deployment_gate") {
        output.push_str(&format!(
            "\n## Deployment gate\n\n**{}** under `{}`.\n\n",
            match gate["outcome"].as_str().unwrap_or("") {
                "Pass" => "PASS",
                "Warn" => "PASS WITH WARNINGS",
                _ => "BLOCKED",
            },
            gate["policy"].as_str().unwrap_or("")
        ));
        for reason in gate["reasons"].as_array().into_iter().flatten() {
            output.push_str(&format!("- {}\n", reason.as_str().unwrap_or("")));
        }
        output.push_str("\nReplay offline: `eplyx-lifecycle replay-package-preflight <package> --result <result-directory>`\n");
    }
    if let Some(control) = report.get("non_standard_account_control") {
        let selected = control["coverage"]["cases_selected"].as_u64().unwrap_or(0);
        let remaining = control["coverage"]["unselected_non_wallet_accounts"]
            .as_u64()
            .unwrap_or(0);
        match report["population_summary"]["enumeration_completeness"].as_str() {
            Some(completeness) if completeness != "CompleteForQuery" => {
                output.push_str(&format!(
                    "\n## Non-standard account control\n\nSelected from observed accounts: {selected}; further observed accounts outside budget: {remaining}; enumeration: {completeness}. Full population is unknown. Proven program-mediated conversions: 0. Resolution does not provide authorization or conversion proof.\n"
                ));
            }
            _ => output.push_str(&format!(
                "\n## Non-standard account control\n\nSelected: {selected}; unresolved outside budget: {remaining}; proven program-mediated conversions: 0. Resolution does not provide authorization or conversion proof.\n"
            )),
        }
    }
    output
}

fn with_gate(mut report: Value, policy: package_gate::Policy) -> Result<Value> {
    let gate = package_gate::evaluate(&report, policy)?;
    let mut counts = serde_json::Map::new();
    for result in report["stress_results"].as_array().into_iter().flatten() {
        if let Some(status) = result["status"].as_str() {
            let count = counts.entry(status).or_insert_with(|| json!(0));
            *count = json!(count.as_u64().unwrap_or(0) + 1);
        }
    }
    report["selected_stress_counts"] = Value::Object(counts);
    report["failures"] = report["failed_cases"].clone();
    report["gate_policy"] = policy.name().into();
    report["gate_outcome"] = serde_json::to_value(gate.outcome)?;
    report["gate_reasons"] = serde_json::to_value(&gate.reasons)?;
    report["deployment_gate"] = serde_json::to_value(gate)?;
    Ok(report)
}

fn evaluate(package: &package::ValidatedPackage, root: &Path, b: &Bindings) -> Result<Value> {
    ensure!(
        b.schema_version == VERSION
            && b.transition_package_sha256 == package.transition_package_sha256
            && b.candidate_program_sha256 == package.program_sha256
            && b.config_sha256 == package.config_sha256,
        "package identity changed"
    );
    b.budget.validate()?;
    let expected_plan = package.conversion_plan()?;
    ensure!(
        expected_plan.sha256()? == b.plan_sha256,
        "package plan identity changed"
    );
    let wallet_bytes = read(root, WALLET_FILE, 10 * 1024 * 1024)?;
    ensure!(
        sha256(&wallet_bytes) == b.wallet_sha256,
        "wallet capture changed"
    );
    let wallet_capture: wallet::Capture = serde_json::from_slice(&wallet_bytes)?;
    let wallet_result = wallet::evaluate(&wallet_capture)?;
    ensure!(
        wallet_capture.asset.mint == package.manifest.source_mint
            && wallet_capture
                .selection
                .as_ref()
                .and_then(|s| s.public_owner.as_deref())
                == Some(package.config.public_owner.as_str())
            && wallet_result["wallet_observation"]["token_accounts"]
                .as_array()
                .is_some(),
        "wallet scope changed"
    );
    let conversion_bytes = read(root, CONVERSION_FILE, 64 * 1024 * 1024)?;
    let verified = conversion::replay(
        &conversion_bytes,
        &b.run_id,
        &b.check_id,
        &b.wallet_sha256,
        &b.conversion_capture_sha256,
        &b.plan_sha256,
        &package.program,
        &b.candidate_program_sha256,
    )?;
    let conversion_value = verified.value();
    ensure!(
        conversion_value["plan"] == serde_json::to_value(&expected_plan)?,
        "conversion plan differs from package"
    );
    let population_bytes = read(root, POPULATION_FILE, b.budget.max_artifact_bytes)?;
    let observation = population::evaluate_bytes(&population_bytes, &b.budget)?;
    let stress_plan_bytes = read(root, PLAN_FILE, b.budget.max_artifact_bytes)?;
    let cases_bytes = read(root, CASES_FILE, b.budget.max_artifact_bytes)?;
    let stress = execute::replay(
        &population_bytes,
        &stress_plan_bytes,
        &cases_bytes,
        &b.stress_id,
        &b.run_id,
        &b.population_sha256,
        &b.stress_plan_sha256,
        &b.cases_sha256,
        &package.program,
        &b.candidate_program_sha256,
        &b.budget,
        &b.evaluated_at,
    )?;
    let stress_value = serde_json::to_value(stress.value())?;
    ensure!(
        stress_value["candidate_plan"] == serde_json::to_value(expected_plan)?,
        "stress plan differs from package"
    );
    let authority = match (&b.authority_plan_sha256, &b.authority_report_sha256) {
        (Some(plan_sha), Some(report_sha)) => {
            let plan_bytes = read(root, AUTHORITY_PLAN_FILE, 4 * 1024 * 1024)?;
            let report_bytes = read(root, AUTHORITY_REPORT_FILE, 4 * 1024 * 1024)?;
            ensure!(
                sha256(&plan_bytes) == *plan_sha && sha256(&report_bytes) == *report_sha,
                "authority resolution artifact digest mismatch"
            );
            let resolution_plan: authority::Plan = serde_json::from_slice(&plan_bytes)?;
            let saved: authority::Report = serde_json::from_slice(&report_bytes)?;
            let capture: population::Capture = serde_json::from_slice(&population_bytes)?;
            let reconstructed = authority::resolve(&resolution_plan, &observation, &capture)?;
            ensure!(
                saved == reconstructed,
                "saved authority resolution differs from offline replay"
            );
            Some(reconstructed)
        }
        (None, None) => None, // Historical Milestone 8/9 package reports.
        _ => anyhow::bail!("incomplete authority resolution binding"),
    };
    let mut report = summary(
        package,
        b,
        conversion_value,
        &stress_value,
        authority.as_ref(),
    );
    if let Some(definitions) = &package.manifest.invariants {
        let evidence = invariants::Evidence {
            conversion: conversion_value,
            stress: &stress_value,
            authority: authority.as_ref(),
            population: &stress_value["population_summary"],
            conversion_capture_sha256: &b.conversion_capture_sha256,
            stress_cases_sha256: &b.cases_sha256,
            population_sha256: &b.population_sha256,
            authority_report_sha256: b.authority_report_sha256.as_deref(),
        };
        report["invariants"] = serde_json::to_value(invariants::evaluate(definitions, &evidence))?;
        report["invariant_schema_version"] = json!(package::INVARIANT_SCHEMA_VERSION);
    }
    match b.gate_policy {
        Some(policy) => with_gate(report, policy),
        None => Ok(report), // Milestone 8 report replay is byte-for-byte compatible.
    }
}

pub fn exit_code(report: &Value) -> Result<u8> {
    if report.get("deployment_gate").is_some() {
        let saved: package_gate::DeploymentGate =
            serde_json::from_value(report["deployment_gate"].clone())?;
        let expected = package_gate::evaluate(report, saved.policy)?;
        ensure!(
            saved == expected
                && report["gate_policy"] == saved.policy.name()
                && report["gate_outcome"] == serde_json::to_value(saved.outcome)?
                && report["gate_reasons"] == serde_json::to_value(&saved.reasons)?,
            "deployment gate result mismatch"
        );
        return Ok(expected.outcome.exit_code());
    }
    match report["declared_preflight_status"].as_str() {
        Some("Ready") => Ok(0),
        Some("Blocked") => Ok(3),
        Some("Incomplete") => Ok(4),
        _ => anyhow::bail!("invalid analytical result"),
    }
}

pub fn run(
    package_directory: &Path,
    output_directory: &Path,
    gate_policy: package_gate::Policy,
) -> Result<Value> {
    // Validate every package byte before the first RPC request.
    let package = package::load(package_directory)?;
    let plan = package.conversion_plan()?;
    let budget = StressBudget::from_env();
    budget.validate()?;
    fs::create_dir(output_directory).context("output directory must not already exist")?;
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".into());
    let run_id = format!(
        "pkg-{}-{}",
        &package.transition_package_sha256[..12],
        Utc::now().timestamp_millis()
    );
    let check_id = format!("{run_id}-conversion");
    let stress_id = format!("{run_id}-stress");
    let selection = wallet::InspectionSelection {
        cluster: "solana-mainnet".into(),
        mint: package.manifest.source_mint.clone(),
        reference: None,
        sample_accounts: false,
        public_owner: Some(package.config.public_owner.clone()),
    };
    let wallet_capture = wallet::capture_selected(selection, &HttpSolanaRpc::bounded(&rpc_url)?)?;
    wallet::save(&wallet_capture, &output_directory.join(WALLET_FILE))?;
    let wallet_bytes = read(output_directory, WALLET_FILE, 10 * 1024 * 1024)?;
    let population_capture = population::capture(
        plan.source_mint.clone(),
        run_id.clone(),
        stress_id.clone(),
        budget.clone(),
        &HttpSolanaRpc::bounded_population(
            &rpc_url,
            budget.max_response_bytes,
            budget.population_timeout_seconds,
        )?,
    )?;
    population::save(&population_capture, &output_directory.join(POPULATION_FILE))?;
    let population_bytes = read(output_directory, POPULATION_FILE, budget.max_artifact_bytes)?;
    let observed = population::evaluate_bytes(&population_bytes, &budget)?;
    let authority_plan = authority::plan(&observed)?;
    write(
        output_directory,
        AUTHORITY_PLAN_FILE,
        crate::expansion::canonical(&authority_plan)?.as_bytes(),
    )?;
    // The plan is already durable and immutable before the first control adapter runs.
    let authority_result = authority::resolve(&authority_plan, &observed, &population_capture)?;
    write(
        output_directory,
        AUTHORITY_REPORT_FILE,
        crate::expansion::canonical(&authority_result)?.as_bytes(),
    )?;
    let stress_plan = select::build(&observed, &plan, &package.program_sha256, &now())?;
    stress_plan.save(&output_directory.join(PLAN_FILE))?;
    // Freeze population and the exact stress selection before any execution-bank
    // recapture. The standalone wallet plan uses the same generic capture rule.
    let conversion_capture = conversion::capture(
        String::from_utf8(wallet_bytes.clone())?,
        plan.clone(),
        run_id.clone(),
        check_id.clone(),
        &HttpSolanaRpc::bounded_execution(&rpc_url)?,
    )?;
    conversion::save(&conversion_capture, &output_directory.join(CONVERSION_FILE))?;
    let mint = observed
        .mint_config
        .as_ref()
        .context("current mint unavailable")?;
    let cases = execute::capture_cases(
        &stress_plan,
        &stress_plan.sha256()?,
        &mint.token_program,
        &HttpSolanaRpc::bounded_execution(&rpc_url)?,
    )?;
    execute::save(&cases, &output_directory.join(CASES_FILE))?;
    let b = Bindings {
        schema_version: VERSION,
        gate_policy: Some(gate_policy),
        transition_package_sha256: package.transition_package_sha256.clone(),
        candidate_program_sha256: package.program_sha256.clone(),
        config_sha256: package.config_sha256.clone(),
        plan_sha256: plan.sha256()?,
        wallet_sha256: sha256(&wallet_bytes),
        conversion_capture_sha256: sha256(&read(
            output_directory,
            CONVERSION_FILE,
            64 * 1024 * 1024,
        )?),
        population_sha256: sha256(&population_bytes),
        stress_plan_sha256: stress_plan.sha256()?,
        cases_sha256: sha256(&read(
            output_directory,
            CASES_FILE,
            budget.max_artifact_bytes,
        )?),
        authority_plan_sha256: Some(sha256(&read(
            output_directory,
            AUTHORITY_PLAN_FILE,
            4 * 1024 * 1024,
        )?)),
        authority_report_sha256: Some(sha256(&read(
            output_directory,
            AUTHORITY_REPORT_FILE,
            4 * 1024 * 1024,
        )?)),
        run_id,
        check_id,
        stress_id,
        evaluated_at: now(),
        budget,
    };
    write(
        output_directory,
        "bindings.json",
        crate::expansion::canonical(&b)?.as_bytes(),
    )?;
    // The child receives exact saved evidence and no RPC URL or unrelated
    // environment secrets. A hung candidate VM cannot hang the capture process.
    let mut child = Command::new(std::env::current_exe()?)
        .arg("finish-package-preflight")
        .arg(package_directory)
        .arg("--result")
        .arg(output_directory)
        .env_clear()
        .spawn()
        .context("could not start isolated offline VM worker")?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            ensure!(status.success(), "offline VM worker failed");
            break;
        }
        if start.elapsed() >= OFFLINE_VM_TIMEOUT {
            child.kill()?;
            child.wait()?;
            anyhow::bail!("offline VM worker exceeded 120-second deadline");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    // Rebinding reports include exact per-case final-state diagnostics. Keep
    // this worker handoff bounded while allowing the ten selected case rows.
    let report: Value =
        serde_json::from_slice(&read(output_directory, "report.json", 4 * 1024 * 1024)?)?;
    ensure!(
        report["transition_package_sha256"] == b.transition_package_sha256,
        "worker package identity mismatch"
    );
    Ok(report)
}

/// Internal, offline-only child entry point. It never reads RPC environment or
/// makes a provider request; actual VM execution and reconciliation are repeated.
pub fn finish(package_directory: &Path, output_directory: &Path) -> Result<()> {
    let package = package::load(package_directory)?;
    let b: Bindings = serde_json::from_slice(&read(output_directory, "bindings.json", 32 * 1024)?)?;
    let report = evaluate(&package, output_directory, &b)?;
    write(
        output_directory,
        "report.json",
        crate::expansion::canonical(&report)?.as_bytes(),
    )?;
    write(output_directory, "report.md", markdown(&report).as_bytes())?;
    Ok(())
}

pub fn replay(package_directory: &Path, output_directory: &Path) -> Result<Value> {
    let package = package::load(package_directory)?;
    let b: Bindings = serde_json::from_slice(&read(output_directory, "bindings.json", 32 * 1024)?)?;
    let report = evaluate(&package, output_directory, &b)?;
    ensure!(
        crate::expansion::canonical(&report)?.as_bytes()
            == read(output_directory, "report.json", 128 * 1024 * 1024)?,
        "saved report differs from offline replay"
    );
    ensure!(
        markdown(&report).as_bytes() == read(output_directory, "report.md", 64 * 1024)?,
        "saved human report differs from offline replay"
    );
    Ok(report)
}

/// Verify the saved report first, then re-evaluate the same analytical findings
/// under a requested policy without changing the saved evidence or report.
pub fn replay_with_policy(
    package_directory: &Path,
    output_directory: &Path,
    policy: Option<package_gate::Policy>,
) -> Result<Value> {
    let saved = replay(package_directory, output_directory)?;
    if let Some(policy) = policy {
        let mut analytical = saved;
        if let Some(object) = analytical.as_object_mut() {
            for field in [
                "deployment_gate",
                "gate_policy",
                "gate_outcome",
                "gate_reasons",
            ] {
                object.remove(field);
            }
        }
        with_gate(analytical, policy)
    } else {
        Ok(saved)
    }
}

#[cfg(test)]
mod gate_tests {
    use super::*;

    #[test]
    fn gate_policy_preserves_analytical_evidence_and_official_boundary() {
        let analytical = json!({
            "candidate_plan_readiness": "Ready",
            "conversion_stress_readiness": {"status": "Incomplete"},
            "population_rollout_readiness": {"status": "Incomplete"},
            "declared_preflight_status": "Incomplete",
            "official_transition": "NotTested",
            "funds_moved": false,
            "stress_results": [], "failed_cases": [],
        });
        let original = analytical.clone();
        let block_only = with_gate(analytical.clone(), package_gate::Policy::BlockOnly).unwrap();
        let strict = with_gate(analytical, package_gate::Policy::Strict).unwrap();
        for (name, value) in original.as_object().unwrap() {
            assert_eq!(block_only[name], *value);
            assert_eq!(strict[name], *value);
        }
        assert_eq!(block_only["gate_outcome"], "Warn");
        assert_eq!(strict["gate_outcome"], "Block");
        assert_eq!(block_only["official_transition"], "NotTested");
        assert_eq!(
            strict["population_rollout_readiness"]["status"],
            "Incomplete"
        );
    }

    #[test]
    fn resolved_custody_does_not_escalate_readiness_or_change_gate_policy() {
        let analytical = json!({
            "candidate_plan_readiness": "Ready",
            "conversion_stress_readiness": {"status": "Incomplete"},
            "population_rollout_readiness": {"status": "Incomplete"},
            "declared_preflight_status": "Incomplete",
            "official_transition": "NotTested",
            "funds_moved": false,
            "stress_results": [], "failed_cases": [],
            "non_standard_account_control": {"coverage": {
                "cases_selected": 20,
                "unselected_non_wallet_accounts": 3000,
                "resolved": {"ResolvedProtocolInternal": {"accounts": 1}}
            }}
        });
        let block_only = with_gate(analytical.clone(), package_gate::Policy::BlockOnly).unwrap();
        let strict = with_gate(analytical, package_gate::Policy::Strict).unwrap();
        assert_eq!(block_only["gate_outcome"], "Warn");
        assert_eq!(strict["gate_outcome"], "Block");
        assert_eq!(
            block_only["conversion_stress_readiness"]["status"],
            "Incomplete"
        );
        assert_eq!(
            block_only["population_rollout_readiness"]["status"],
            "Incomplete"
        );
        assert_eq!(block_only["official_transition"], "NotTested");
    }

    #[test]
    fn unavailable_enumeration_does_not_report_zero_remaining_population() {
        let report = with_gate(
            json!({
                "candidate_plan_readiness": "Ready",
                "conversion_stress_readiness": {"status": "Incomplete"},
                "population_rollout_readiness": {"status": "Incomplete"},
                "declared_preflight_status": "Incomplete",
                "official_transition": "NotTested",
                "funds_moved": false,
                "stress_results": [], "failed_cases": [],
                "population_summary": {"enumeration_completeness": "Unavailable"},
                "non_standard_account_control": {"coverage": {
                    "cases_selected": 0,
                    "unselected_non_wallet_accounts": 0
                }}
            }),
            package_gate::Policy::BlockOnly,
        )
        .unwrap();
        assert!(report["gate_reasons"][2]
            .as_str()
            .unwrap()
            .contains("full population is unknown"));
        assert!(markdown(&report).contains("Full population is unknown"));
    }
}
