//! Presentation views over saved engine artifacts. Every analytical status here
//! is copied from an engine artifact or produced by the engine's own gate
//! evaluator. This module only selects, counts, joins and compares fields; it
//! never replays evidence or decides readiness.
use super::store::{Store, CAPTURE_LIMIT, REPORT_LIMIT, SEARCH_LIMIT, SMALL_LIMIT};
use crate::{
    conversion::{
        package,
        package_gate::{self, Policy},
        search::{self, Counterexample, SearchResult},
    },
    lifecycle::exposure::sha256,
    local_store::{counterexample_id, replay_inputs, Metadata, Reproduction, SavedCounterexample},
};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Downloadable run members: a fixed allowlist from public name to store path.
pub const ARTIFACTS: &[(&str, &str, &str, &str)] = &[
    (
        "report.json",
        "result/report.json",
        "application/json",
        "Engine report",
    ),
    (
        "report.md",
        "result/report.md",
        "text/plain; charset=utf-8",
        "Human report",
    ),
    (
        "metadata.json",
        "metadata.json",
        "application/json",
        "Local run metadata",
    ),
    (
        "bindings.json",
        "result/bindings.json",
        "application/json",
        "Evidence bindings",
    ),
    (
        "manifest.json",
        "package/eplyx.json",
        "application/json",
        "Package manifest",
    ),
    (
        "config.json",
        "package/config.json",
        "application/json",
        "Package execution config",
    ),
    (
        "search.json",
        "search/counterexamples.json",
        "application/json",
        "Counterexample search",
    ),
    (
        "authority.plan.json",
        "result/authority.plan.json",
        "application/json",
        "Authority plan",
    ),
    (
        "authority.report.json",
        "result/authority.report.json",
        "application/json",
        "Authority resolution",
    ),
    (
        "stress.plan.json",
        "result/stress.plan.json",
        "application/json",
        "Frozen stress plan",
    ),
    (
        "wallet.capture.json",
        "result/wallet.capture.json",
        "application/json",
        "Wallet capture",
    ),
    (
        "conversion.capture.json",
        "result/conversion.capture.json",
        "application/json",
        "Conversion capture",
    ),
    (
        "stress.cases.json",
        "result/stress.cases.json",
        "application/json",
        "Stress case captures",
    ),
    (
        "population.capture.json",
        "result/population.capture.json",
        "application/json",
        "Population capture",
    ),
];

pub fn artifact(name: &str) -> Option<(&'static str, &'static str)> {
    ARTIFACTS
        .iter()
        .find(|(public, ..)| *public == name)
        .map(|(_, path, kind, _)| (*path, *kind))
}

/// One run's small artifacts, parsed. Built from the local store or from the
/// exact bytes a cloud workspace received; both go through [`from_bytes`].
pub struct Run {
    id: String,
    metadata: Option<Metadata>,
    report: Option<Value>,
    manifest: Option<Value>,
    config: Option<Value>,
    search: Option<SearchResult>,
    search_sha256: Option<String>,
    problems: Vec<String>,
}

/// Exact bytes of a run's small members. Captures and program bytes are never
/// part of it; a missing member is `None`.
#[derive(Clone, Copy, Default)]
pub struct RunBytes<'a> {
    pub metadata: Option<&'a [u8]>,
    pub report: Option<&'a [u8]>,
    pub manifest: Option<&'a [u8]>,
    pub config: Option<&'a [u8]>,
    pub search: Option<&'a [u8]>,
}

impl Run {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn report(&self) -> Option<&Value> {
        self.report.as_ref()
    }

    pub fn search(&self) -> Option<&SearchResult> {
        self.search.as_ref()
    }

    pub fn search_sha256(&self) -> Option<&str> {
        self.search_sha256.as_deref()
    }

    pub fn problems(&self) -> &[String] {
        &self.problems
    }
}

fn parts(id: &str, member: &str) -> Vec<String> {
    std::iter::once("runs".to_owned())
        .chain(std::iter::once(id.to_owned()))
        .chain(member.split('/').map(str::to_owned))
        .collect()
}

fn read_json(
    store: &Store,
    id: &str,
    member: &str,
    limit: u64,
    problems: &mut Vec<String>,
) -> Option<Value> {
    let owned = parts(id, member);
    let borrowed: Vec<&str> = owned.iter().map(String::as_str).collect();
    match store.json(&borrowed, limit) {
        Ok(value) => value,
        Err(error) => {
            problems.push(format!("{member}: {error:#}"));
            None
        }
    }
}

fn read_member(
    store: &Store,
    id: &str,
    member: &str,
    limit: u64,
    problems: &mut Vec<String>,
) -> Option<Vec<u8>> {
    let owned = parts(id, member);
    let borrowed: Vec<&str> = owned.iter().map(String::as_str).collect();
    match store.read(&borrowed, limit) {
        Ok(bytes) => bytes,
        Err(error) => {
            problems.push(format!("{member}: {error:#}"));
            None
        }
    }
}

pub fn load(store: &Store, id: &str) -> Run {
    let mut problems = Vec::new();
    let metadata = read_member(store, id, "metadata.json", SMALL_LIMIT, &mut problems);
    let report = read_member(store, id, "result/report.json", REPORT_LIMIT, &mut problems);
    let manifest = read_member(store, id, "package/eplyx.json", SMALL_LIMIT, &mut problems);
    let config = read_member(store, id, "package/config.json", SMALL_LIMIT, &mut problems);
    let search = read_member(
        store,
        id,
        "search/counterexamples.json",
        SEARCH_LIMIT,
        &mut problems,
    );
    let bytes = RunBytes {
        metadata: metadata.as_deref(),
        report: report.as_deref(),
        manifest: manifest.as_deref(),
        config: config.as_deref(),
        search: search.as_deref(),
    };
    parse(id, bytes, problems)
}

/// Parse exact member bytes with the same identity checks as a local load.
pub fn from_bytes(id: &str, bytes: RunBytes) -> Run {
    parse(id, bytes, Vec::new())
}

fn parse(id: &str, bytes: RunBytes, mut problems: Vec<String>) -> Run {
    let mut json = |member: &str, bytes: Option<&[u8]>| -> Option<Value> {
        let bytes = bytes?;
        match serde_json::from_slice::<Value>(bytes).context("invalid JSON") {
            Ok(value) => Some(value),
            Err(error) => {
                problems.push(format!("{member}: {error:#}"));
                None
            }
        }
    };
    let metadata = json("metadata.json", bytes.metadata);
    let report = json("result/report.json", bytes.report);
    let manifest = json("package/eplyx.json", bytes.manifest);
    let config = json("package/config.json", bytes.config);
    let metadata = metadata.and_then(|value| {
        serde_json::from_value::<Metadata>(value)
            .map_err(|error| problems.push(format!("metadata.json: {error}")))
            .ok()
    });
    let (search, search_sha256) = match bytes.search {
        Some(bytes) => match serde_json::from_slice::<SearchResult>(bytes) {
            Ok(result) => (Some(result), Some(sha256(bytes))),
            Err(error) => {
                problems.push(format!("search/counterexamples.json: {error}"));
                (None, Some(sha256(bytes)))
            }
        },
        None => (None, None),
    };
    if let Some(meta) = &metadata {
        if meta.run_id != id {
            problems.push("metadata run ID does not match its directory".into());
        }
        if let Some(report) = &report {
            if report["gate_outcome"].as_str() != Some(meta.gate_outcome.as_str()) {
                problems.push("metadata gate outcome differs from report.json".into());
            }
            if report["transition_package_sha256"].as_str()
                != Some(meta.transition_package_sha256.as_str())
            {
                problems.push("metadata package hash differs from report.json".into());
            }
        }
    }
    if let (Some(search), Some(report)) = (&search, &report) {
        if report["run_id"].as_str() != Some(search.parent_run.as_str())
            || report["transition_package_sha256"].as_str()
                != Some(search.transition_package_sha256.as_str())
        {
            problems.push("search artifact belongs to a different package run".into());
        }
    }
    Run {
        id: id.into(),
        metadata,
        report,
        manifest,
        config,
        search,
        search_sha256,
        problems,
    }
}

pub fn state(run: &Run) -> &'static str {
    // The CLI writes metadata.json last, so its absence means the preflight
    // never finished; unreadable artifacts are reported, never guessed around.
    if !run.problems.is_empty() {
        "Unreadable"
    } else if run.metadata.is_none() || run.report.is_none() {
        "Unfinished"
    } else {
        "Complete"
    }
}

fn status_counts<'a>(items: impl Iterator<Item = &'a Value>, field: &str) -> Value {
    let mut counts = BTreeMap::<String, u64>::new();
    for item in items {
        if let Some(status) = item[field].as_str() {
            *counts.entry(status.into()).or_default() += 1;
        }
    }
    json!(counts)
}

fn search_brief(run: &Run) -> Value {
    let Some(search) = &run.search else {
        return if run.search_sha256.is_some() {
            json!({"state": "Unreadable"})
        } else {
            Value::Null
        };
    };
    let observed = search
        .counterexamples
        .iter()
        .filter(|c| matches!(c, Counterexample::Observed { .. }))
        .count();
    json!({
        "state": "Recorded",
        "conclusion": search.conclusion,
        "observed": observed,
        "derived": search.counterexamples.len() - observed,
        "total": search.counterexamples.len(),
        "budget": search.budget,
        "has_derived_domain": search.derived_domain.is_some(),
        "sha256": run.search_sha256,
    })
}

pub fn summary(run: &Run) -> Value {
    let report = run.report.as_ref().unwrap_or(&Value::Null);
    let meta = run.metadata.as_ref();
    let manifest = run.manifest.as_ref().unwrap_or(&Value::Null);
    // The packaged execution config, read through the engine's own type.
    let config = run
        .config
        .clone()
        .and_then(|value| serde_json::from_value::<package::Config>(value).ok())
        .map_or(Value::Null, |c| {
            json!({
                "source_account": c.source_account,
                "public_owner": c.public_owner,
                "amount_decimal": c.amount_decimal,
                "reserve_funded_replacement_raw": c.reserve_funded_replacement_raw,
            })
        });
    let invariants = report["invariants"].as_array();
    json!({
        "id": run.id,
        "state": state(run),
        "problems": run.problems,
        "timestamp": meta.map(|m| m.timestamp.clone()),
        "run_source": meta.and_then(|m| m.run_source),
        "eplyx_version": meta.map(|m| m.eplyx_version.clone()),
        "engine_binary_sha256": meta.map(|m| m.engine_binary_sha256.clone()),
        "git": {
            "commit": meta.and_then(|m| m.git_commit.clone()),
            "branch": meta.and_then(|m| m.git_branch.clone()),
            "dirty": meta.and_then(|m| m.git_dirty),
        },
        "candidate_program_sha256": meta.map(|m| m.candidate_program_sha256.clone()).or_else(|| report["candidate_program_sha256"].as_str().map(str::to_owned)),
        "transition_package_sha256": meta.map(|m| m.transition_package_sha256.clone()).or_else(|| report["transition_package_sha256"].as_str().map(str::to_owned)),
        "config_sha256": report["config_sha256"],
        "gate": {
            "policy": report["gate_policy"].as_str().map(str::to_owned).or_else(|| meta.map(|m| m.gate_policy.clone())),
            "outcome": report["gate_outcome"].as_str().map(str::to_owned).or_else(|| meta.map(|m| m.gate_outcome.clone())),
            "reasons": report["gate_reasons"].as_array().map(Vec::len),
        },
        "conversion": report["conversion_result"]["status"],
        "readiness": {
            "candidate_plan": report["candidate_plan_readiness"],
            "conversion_stress": report["conversion_stress_readiness"]["status"],
            "population_rollout": report["population_rollout_readiness"]["status"],
            "analytical": report["declared_preflight_status"],
        },
        "official_transition": report["official_transition"],
        "stress": {
            "selected": report["exact_selected_cases"].as_array().map(Vec::len),
            "counts": report["selected_stress_counts"],
        },
        "population": {
            "token_accounts_observed": report["population_summary"]["counts"]["token_accounts_observed"],
            "positive_balance_accounts_observed": report["population_summary"]["counts"]["positive_balance_accounts_observed"],
        },
        "invariants": {
            "total": invariants.map(Vec::len),
            "counts": invariants.map_or(Value::Null, |items| status_counts(items.iter(), "status")),
        },
        "transition": {
            "source_mint": report["source_mint"].as_str().or(manifest["sourceMint"].as_str()),
            "replacement_mint": report["replacement_mint"].as_str().or(manifest["replacementMint"].as_str()),
            "adapter": report["adapter"].as_str().or(manifest["adapter"].as_str()),
            "terms": if report["terms"].is_null() { manifest["terms"].clone() } else { report["terms"].clone() },
            "proposed_reserve_raw": report["proposed_reserve_raw"].as_str().or(config["reserve_funded_replacement_raw"].as_str()),
            "effective_at": report["effective_at"].as_str().or(manifest["effectiveAt"].as_str()),
            "source_account": report["source_account"].as_str().or(config["source_account"].as_str()),
            "public_owner": report["public_owner"].as_str().or(config["public_owner"].as_str()),
            "amount_decimal": config["amount_decimal"],
        },
        "evaluated_at": report["evaluated_at"],
        "capture_timestamp": report["current_capture_timestamp"],
        "search": search_brief(run),
    })
}

pub fn run_summary(store: &Store, id: &str) -> Value {
    summary(&load(store, id))
}

/// Only scheme and host of a recorded provider origin; never a path, query or
/// user information, even if an older artifact carried one.
pub fn sanitize_origin(origin: &str) -> String {
    let Some((scheme, rest)) = origin.split_once("://") else {
        return "unrecorded".into();
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.rsplit('@').next().unwrap_or("");
    if host.is_empty() || !matches!(scheme, "http" | "https") {
        return "unrecorded".into();
    }
    format!("{scheme}://{host}")
}

fn strip(value: &Value, fields: &[&str]) -> Value {
    let mut value = value.clone();
    if let Some(object) = value.as_object_mut() {
        for field in fields {
            object.remove(*field);
        }
    }
    value
}

fn stress_cases(report: &Value) -> Value {
    let results: BTreeMap<&str, &Value> = report["stress_results"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|result| Some((result["case_id"].as_str()?, result)))
        .collect();
    let rows = report["exact_selected_cases"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|case| {
            let id = case["case_id"].as_str().unwrap_or("");
            let result = results.get(id).copied().unwrap_or(&Value::Null);
            let revalidation = &result["revalidation"];
            json!({
                "case_id": id,
                "token_account": case["token_account"],
                "authority": case["authority"],
                "selection_reason": case["selection_reason"],
                "selection_bucket": case["selection_bucket"],
                "discovery_amount_raw": case["discovery_amount_raw"],
                "status": result["status"],
                "reason": result["reason"],
                "classification": revalidation["classification"],
                "final_amount_raw": revalidation["final_amount_raw"],
                "ui_balance": revalidation["final_state"]["ui_balance"],
                "final_context_slot": revalidation["final_context_slot"],
                "changed_fields": revalidation["changed_fields"],
                "shape_preserved": revalidation["selection_shape_preserved"],
                "bucket_preserved": revalidation["selection_bucket_preserved"],
                "state_shape_sha256": case["state_shape_sha256"],
                "execution_plan_sha256": result["execution_plan_sha256"],
                "execution_fixture_sha256": result["execution_fixture_sha256"],
                "result_sha256": result["result_sha256"],
                "clock_slot": result["execution_context"]["clock_slot"],
                "coherence_status": result["execution_context"]["coherence_status"],
            })
        })
        .collect::<Vec<_>>();
    json!(rows)
}

fn cx_brief(c: &Counterexample) -> Value {
    let id = counterexample_id(c).ok();
    match c {
        Counterexample::Observed {
            observed_source_account,
            observed_amount_raw,
            failure_signature,
            ..
        } => json!({
            "id": id, "kind": "Observed", "key": semantic_key(c), "account": observed_source_account,
            "observed_amount_raw": observed_amount_raw, "failure": failure_signature,
        }),
        Counterexample::Derived {
            observed_source_account,
            observed_amount_raw,
            search_dimension,
            derived_value_raw,
            first_passing_value_raw,
            last_passing_value_raw,
            minimized,
            failure_signature,
            ..
        } => json!({
            "id": id, "kind": "Derived", "key": semantic_key(c), "account": observed_source_account,
            "observed_amount_raw": observed_amount_raw, "dimension": search_dimension,
            "derived_value_raw": derived_value_raw, "first_passing_value_raw": first_passing_value_raw,
            "last_passing_value_raw": last_passing_value_raw, "minimized": minimized, "failure": failure_signature,
        }),
    }
}

/// Stable across runs, unlike the content-addressed `cx_` ID which embeds the
/// package run: the observed account, plus the dimension for derived variants.
pub fn semantic_key(c: &Counterexample) -> String {
    match c {
        Counterexample::Observed {
            observed_source_account,
            ..
        } => format!("observed:{observed_source_account}"),
        Counterexample::Derived {
            observed_source_account,
            search_dimension,
            ..
        } => {
            format!("derived:{search_dimension:?}:{observed_source_account}")
        }
    }
}

fn gate_views(report: &Value, search: Option<&SearchResult>) -> Value {
    let saved: Option<package_gate::DeploymentGate> =
        serde_json::from_value(report["deployment_gate"].clone()).ok();
    let evaluate = |policy: Policy| match package_gate::evaluate(report, policy) {
        Ok(gate) => json!(gate),
        Err(error) => json!({"error": format!("{error:#}")}),
    };
    let with_search = |policy: Policy| {
        search.map(|result| {
            match package_gate::evaluate_with_counterexamples(report, policy, result) {
                Ok(gate) => json!(gate),
                Err(error) => json!({"error": format!("{error:#}")}),
            }
        })
    };
    let consistent = saved.as_ref().map(|gate| {
        package_gate::evaluate(report, gate.policy).is_ok_and(|expected| &expected == gate)
    });
    json!({
        "saved": report["deployment_gate"],
        "consistent_with_engine": consistent,
        "policies": [
            {"policy": "block-only", "preflight": evaluate(Policy::BlockOnly), "with_search": with_search(Policy::BlockOnly)},
            {"policy": "strict", "preflight": evaluate(Policy::Strict), "with_search": with_search(Policy::Strict)},
        ],
        "search_finding": search.map(search::gate_finding),
        "basis": "Evaluated by the engine's deployment gate over the saved report.json and, where recorded, the saved search result. The dashboard does not replay evidence; `eplyx reproduce` and the engine replay commands do.",
    })
}

fn command(label: &str, text: String) -> Value {
    json!({"label": label, "command": text})
}

/// Facts about a run that live outside its report: the provider origin from
/// the wallet capture, the evidence bindings and each artifact's size. A cloud
/// workspace supplies these from the synced document, without the provider.
pub struct DetailContext {
    pub provider: Option<Value>,
    pub bindings: Option<Value>,
    pub artifacts: Vec<Value>,
}

pub fn run_detail(store: &Store, id: &str, counterexample_files: &[Value]) -> Result<Value> {
    let run = load(store, id);
    let wallet = read_json(
        store,
        id,
        "result/wallet.capture.json",
        16 * SMALL_LIMIT,
        &mut Vec::new(),
    );
    let bindings = read_json(
        store,
        id,
        "result/bindings.json",
        SMALL_LIMIT,
        &mut Vec::new(),
    );
    let provider = wallet.as_ref().map(|w| {
        json!({
            "origin": sanitize_origin(w["rpc_origin"].as_str().unwrap_or("")),
            "cluster": w["selection"]["cluster"],
            "started_at": w["started_at"],
            "completed_at": w["completed_at"],
        })
    });
    let sizes = ARTIFACTS
        .iter()
        .map(|(_, path, ..)| {
            let owned = parts(id, path);
            let borrowed: Vec<&str> = owned.iter().map(String::as_str).collect();
            store
                .open_file(&borrowed, CAPTURE_LIMIT)
                .ok()
                .flatten()
                .map(|(_, len)| len)
        })
        .collect::<Vec<_>>();
    let context = DetailContext {
        provider,
        bindings,
        artifacts: artifact_rows(id, &sizes),
    };
    Ok(run_detail_for(&run, context, counterexample_files))
}

/// The artifact table for a run, from each allowlisted member's size in
/// [`ARTIFACTS`] order; `None` marks an absent member.
pub fn artifact_rows(id: &str, sizes: &[Option<u64>]) -> Vec<Value> {
    ARTIFACTS
        .iter()
        .zip(sizes.iter().chain(std::iter::repeat(&None)))
        .map(|((name, path, _, label), size)| {
            json!({"name": name, "label": label, "path": format!(".eplyx/runs/{id}/{path}"), "size": size})
        })
        .collect()
}

pub fn run_detail_for(run: &Run, context: DetailContext, counterexample_files: &[Value]) -> Value {
    let id = run.id.as_str();
    let mut detail = summary(run);
    let report = run.report.clone().unwrap_or(Value::Null);
    let manifest = run.manifest.clone().unwrap_or(Value::Null);
    let DetailContext {
        provider,
        bindings,
        artifacts,
    } = context;
    let saved_ids: BTreeSet<&str> = counterexample_files
        .iter()
        .filter(|c| c["parent_run"] == id)
        .filter_map(|c| c["id"].as_str())
        .collect();
    let unsupported_rows = report["unsupported_states"].as_array();
    let unsupported = json!({
        "shapes": unsupported_rows.map(Vec::len),
        "positive_balance_accounts": unsupported_rows.map(|rows| rows.iter().filter_map(|r| r["positive_balance_accounts"].as_u64()).sum::<u64>()),
        "rows": unsupported_rows.map(|rows| rows.iter().take(100).cloned().collect::<Vec<_>>()),
    });
    let search_detail = run.search.as_ref().map(|search| {
        let search_counterexamples = search
            .counterexamples
            .iter()
            .map(|c| {
                let mut brief = cx_brief(c);
                let saved = brief["id"]
                    .as_str()
                    .is_some_and(|cx| saved_ids.contains(cx));
                brief["saved"] = json!(saved);
                brief
            })
            .collect::<Vec<_>>();
        json!({
            "version": search.version,
            "conclusion": search.conclusion,
            "search_domain": search.search_domain,
            "derived_domain": search.derived_domain,
            "budget": search.budget,
            "observed_wave": search.observed_wave,
            "additional_waves": search.additional_waves,
            "trace": search.trace,
            "counterexamples": search_counterexamples,
            "saved_files": saved_ids.len(),
            "sha256": run.search_sha256,
        })
    });
    let base = format!(".eplyx/runs/{id}");
    let mut commands = vec![
        command("Summarize this run", format!("eplyx show {id}")),
        command("Replay the preflight offline", format!("eplyx-lifecycle replay-package-preflight {base}/package --result {base}/result")),
        command("Replay under the strict policy", format!("eplyx-lifecycle replay-package-preflight {base}/package --result {base}/result --gate strict")),
    ];
    if run.search.is_some() {
        commands.push(command("Replay the search offline", format!("eplyx-lifecycle replay-counterexample {base}/package --result {base}/result --search {base}/search")));
    } else {
        commands.push(command(
            "Search this run",
            format!("eplyx search --run {id}"),
        ));
    }
    let extra = json!({
        "release": {
            "program_id": manifest["candidateProgram"]["programId"],
            "packaged_artifact": manifest["candidateProgram"]["artifact"],
            "package_schema_version": manifest["schemaVersion"],
            "invariant_definitions": manifest["invariants"],
            "provenance": report["provenance"],
            "deployment_origin": report["deployment_origin"],
            "engine_run_id": report["run_id"],
        },
        "production": {
            "population_summary": report["population_summary"],
            "authority_control": strip(&report["non_standard_account_control"], &["cases"]),
            "authority_cases": report["non_standard_account_control"]["cases"],
            "unsupported_states": unsupported,
            "stress_rebinding": report["stress_rebinding"],
            "capture_timestamp": report["current_capture_timestamp"],
            "provider": provider,
        },
        "execution": {
            "result": report["conversion_result"],
            "terms": report["terms"],
            "proposed_reserve_raw": report["proposed_reserve_raw"],
            "source_account": report["source_account"],
            "public_owner": report["public_owner"],
            "note": "The engine reconciles source debit, burn, fee, ratio, reserve and replacement credit exactly during execution. report.json retains the reconciliation outcome, not each delta; offline replay re-executes them.",
        },
        "stress_detail": {
            "cases": stress_cases(&report),
            "readiness": report["conversion_stress_readiness"],
            "unresolved_cases": report["unresolved_cases"],
            "limitations": report["limitations"],
        },
        "search_detail": search_detail,
        "invariant_results": report["invariants"],
        "gate_detail": gate_views(&report, run.search.as_ref()),
        "evidence": {
            "artifacts": artifacts,
            "bindings": bindings,
            "hashes": {
                "transition_package_sha256": report["transition_package_sha256"],
                "candidate_program_sha256": report["candidate_program_sha256"],
                "config_sha256": report["config_sha256"],
                "refined_stress_world_sha256": report["refined_stress_world_sha256"],
                "conversion_plan_sha256": report["conversion_result"]["plan_sha256"],
                "conversion_fixture_sha256": report["conversion_result"]["execution_fixture_sha256"],
                "search_sha256": run.search_sha256,
                "engine_binary_sha256": run.metadata.as_ref().map(|m| m.engine_binary_sha256.clone()),
            },
            "commands": commands,
        },
    });
    if let (Some(target), Some(source)) = (detail.as_object_mut(), extra.as_object()) {
        target.extend(source.clone());
    }
    detail
}

fn load_counterexample(store: &Store, id: &str) -> Result<SavedCounterexample> {
    let file = format!("{id}.json");
    let bytes = store
        .read(&["counterexamples", &file], SMALL_LIMIT)?
        .context("counterexample file missing")?;
    serde_json::from_slice(&bytes).context("invalid saved counterexample")
}

pub fn counterexample_fields(saved: &SavedCounterexample, file_id: &str) -> Value {
    let computed = counterexample_id(&saved.counterexample).ok();
    let identity_verified = computed.as_deref() == Some(file_id) && saved.id == file_id;
    let replay_matches = saved
        .replay_inputs
        .as_ref()
        .is_none_or(|inputs| inputs == &replay_inputs(&saved.parent_run));
    let mut view = cx_brief(&saved.counterexample);
    let common = match &saved.counterexample {
        Counterexample::Observed {
            id,
            parent_run,
            transition_package_sha256,
            candidate_program_sha256,
            observed_state_digest,
            execution_plan_sha256,
            execution_fixture_sha256,
            provenance,
            limitations,
            ..
        } => json!({
            "engine_id": id, "package_run": parent_run, "transition_package_sha256": transition_package_sha256,
            "candidate_program_sha256": candidate_program_sha256, "observed_state_digest": observed_state_digest,
            "execution_plan_sha256": execution_plan_sha256, "execution_fixture_sha256": execution_fixture_sha256,
            "provenance": provenance, "limitations": limitations, "dimension": Value::Null, "minimized": Value::Null,
        }),
        Counterexample::Derived {
            id,
            parent_run,
            transition_package_sha256,
            candidate_program_sha256,
            observed_state_digest,
            execution_plan_sha256,
            execution_fixture_sha256,
            provenance,
            limitations,
            original_value_raw,
            signature_preserved,
            ..
        } => json!({
            "engine_id": id, "package_run": parent_run, "transition_package_sha256": transition_package_sha256,
            "candidate_program_sha256": candidate_program_sha256, "observed_state_digest": observed_state_digest,
            "execution_plan_sha256": execution_plan_sha256, "execution_fixture_sha256": execution_fixture_sha256,
            "provenance": provenance, "limitations": limitations, "original_value_raw": original_value_raw,
            "signature_preserved": signature_preserved,
        }),
    };
    if let (Some(target), Some(source)) = (view.as_object_mut(), common.as_object()) {
        target.extend(source.clone());
    }
    view["id"] = json!(file_id);
    view["claim"] = json!(saved.counterexample.claim());
    view["parent_run"] = json!(saved.parent_run);
    view["search_sha256"] = json!(saved.search_sha256);
    view["identity_verified"] = json!(identity_verified && replay_matches);
    view["state"] = json!(if identity_verified && replay_matches {
        "Valid"
    } else {
        "IdentityMismatch"
    });
    view["reproduce"] = json!(format!("eplyx reproduce {file_id}"));
    view
}

pub fn counterexample_summary(store: &Store, id: &str) -> Value {
    let mut view = match load_counterexample(store, id) {
        Ok(saved) => counterexample_fields(&saved, id),
        Err(error) => json!({"id": id, "state": "Unreadable", "problems": [format!("{error:#}")]}),
    };
    let file = format!("{id}.json");
    view["saved_at_ms"] = json!(store.modified_millis(&["counterexamples", &file]));
    view
}

/// Local reproduction history, copied from what `eplyx reproduce` recorded.
pub fn reproduction_summary(store: &Store, id: &str) -> Value {
    let file = format!("{id}.json");
    let parsed = store
        .read(&["reproductions", &file], SMALL_LIMIT)
        .and_then(|bytes| {
            let bytes = bytes.context("reproduction file missing")?;
            serde_json::from_slice::<Reproduction>(&bytes).context("invalid reproduction record")
        });
    match parsed {
        Ok(record) if record.id == id => {
            let mut value = json!(record);
            value["state"] = json!("Valid");
            value
        }
        Ok(_) => {
            json!({"id": id, "state": "IdentityMismatch", "problems": ["record ID does not match its file name"]})
        }
        Err(error) => json!({"id": id, "state": "Unreadable", "problems": [format!("{error:#}")]}),
    }
}

pub fn counterexample_detail(store: &Store, id: &str, summary: &Value) -> Result<Value> {
    let saved = load_counterexample(store, id)?;
    let parent = load(store, &saved.parent_run);
    counterexample_detail_for(&saved, summary, &parent)
}

pub fn counterexample_detail_for(
    saved: &SavedCounterexample,
    summary: &Value,
    parent: &Run,
) -> Result<Value> {
    let mut view = summary.clone();
    if let Counterexample::Derived {
        minimization_trace,
        original_failure_signature,
        ..
    } = &saved.counterexample
    {
        view["minimization_trace"] = json!(minimization_trace);
        view["original_failure_signature"] = json!(original_failure_signature);
    }
    view["replay_inputs"] = json!(saved.replay_inputs.as_ref().map(|inputs| json!({
        "package": format!(".eplyx/{}", inputs.package),
        "result": format!(".eplyx/{}", inputs.result),
        "search": format!(".eplyx/{}", inputs.search),
    })));
    view["search_artifact_matches"] =
        json!(parent.search_sha256.as_deref() == Some(saved.search_sha256.as_str()));
    view["search_context"] = json!(parent.search.as_ref().map(|search| json!({
        "conclusion": search.conclusion,
        "search_domain": search.search_domain,
        "derived_domain": search.derived_domain,
        "budget": search.budget,
        "total": search.counterexamples.len(),
    })));
    view["raw"] = serde_json::to_value(saved)?;
    Ok(view)
}

fn field(group: &str, label: &str, left: Value, right: Value) -> Value {
    json!({"group": group, "label": label, "changed": left != right, "left": left, "right": right})
}

/// Exact token accounts a run's search executed, with the engine outcome for
/// each: wave 0 from the report's stress results, then each additional wave.
fn executed_accounts(report: &Value, search: &SearchResult) -> BTreeMap<String, String> {
    let mut executed = BTreeMap::new();
    let wave_zero: BTreeMap<String, String> = stress_cases(report)
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| {
            Some((
                row["token_account"].as_str()?.to_owned(),
                row["status"].as_str().unwrap_or("Unknown").to_owned(),
            ))
        })
        .collect();
    for account in &search.observed_wave {
        executed.insert(
            account.clone(),
            wave_zero
                .get(account)
                .cloned()
                .unwrap_or_else(|| "Unknown".into()),
        );
    }
    for wave in &search.additional_waves {
        for (account, outcome) in wave.selected_exact_accounts.iter().zip(&wave.outcomes) {
            let outcome = serde_json::to_value(outcome)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_else(|| "Unknown".into());
            executed.insert(account.clone(), outcome);
        }
    }
    executed
}

fn search_conditions(
    left: Option<&SearchResult>,
    right: Option<&SearchResult>,
) -> (bool, Vec<String>) {
    match (left, right) {
        (None, None) => (
            false,
            vec!["Neither run has a recorded counterexample search.".into()],
        ),
        (None, Some(_)) => (
            false,
            vec!["Run A has no recorded counterexample search.".into()],
        ),
        (Some(_), None) => (
            false,
            vec!["Run B has no recorded counterexample search.".into()],
        ),
        (Some(a), Some(b)) => {
            let mut differences = Vec::new();
            if a.version != b.version {
                differences.push(format!(
                    "Search version differs: {} → {}",
                    a.version, b.version
                ));
            }
            let maxima = |x: &SearchResult| {
                (
                    x.budget.max_observed_executions,
                    x.budget.max_boundary_executions,
                    x.budget.max_minimization_executions,
                )
            };
            if maxima(a) != maxima(b) {
                differences.push("Search budget limits differ.".into());
            }
            if a.search_domain != b.search_domain {
                differences.push("Observed search domain description differs.".into());
            }
            match (&a.derived_domain, &b.derived_domain) {
                (None, None) => {}
                (None, Some(_)) => differences.push("Run A had no derived search domain (no failing seed); run B searched a derived domain.".into()),
                (Some(_), None) => differences.push("Run A searched a derived domain; run B had no derived search domain (no failing seed).".into()),
                (Some(x), Some(y)) if x != y => differences.push("Derived search domain differs (seed account, bounds or captured replacement supply).".into()),
                _ => {}
            }
            (differences.is_empty(), differences)
        }
    }
}

fn counterexample_diff(a: &Run, b: &Run) -> Value {
    let (comparable, differences) = search_conditions(a.search.as_ref(), b.search.as_ref());
    let index = |run: &Run| -> BTreeMap<String, Value> {
        let mut map = BTreeMap::new();
        for c in run.search.iter().flat_map(|s| &s.counterexamples) {
            let brief = cx_brief(c);
            let mut key = semantic_key(c);
            let mut n = 1;
            while map.contains_key(&key) {
                n += 1;
                key = format!("{}#{n}", semantic_key(c));
            }
            map.insert(key, brief);
        }
        map
    };
    let (left, right) = (index(a), index(b));
    let executed = |run: &Run| match (&run.report, &run.search) {
        (Some(report), Some(search)) => executed_accounts(report, search),
        _ => BTreeMap::new(),
    };
    let (left_executed, right_executed) = (executed(a), executed(b));
    let keys: BTreeSet<&String> = left.keys().chain(right.keys()).collect();
    let mut counts = BTreeMap::<&str, u64>::new();
    let mut items = Vec::new();
    for key in keys {
        let (l, r) = (left.get(key), right.get(key));
        let account = l.or(r).and_then(|v| v["account"].as_str()).unwrap_or("");
        let observed = key.starts_with("observed:");
        let (status, note) = match (l, r) {
            (Some(l), Some(r)) => {
                let boundary = |v: &Value| {
                    (
                        v["failure"]["instruction_error"].clone(),
                        v["derived_value_raw"].clone(),
                        v["first_passing_value_raw"].clone(),
                        v["last_passing_value_raw"].clone(),
                    )
                };
                if boundary(l) != boundary(r) {
                    (
                        "changed",
                        "Found in both runs; the failure signature or boundary values differ."
                            .to_string(),
                    )
                } else if l["observed_amount_raw"] != r["observed_amount_raw"] {
                    (
                        "persistent",
                        "Found in both runs with the same failure; the observed amount differs."
                            .into(),
                    )
                } else {
                    (
                        "persistent",
                        "Found in both runs with the same failure.".into(),
                    )
                }
            }
            (None, Some(_)) => {
                let tested_in_a = if observed {
                    left_executed.get(account).is_some_and(|o| o != "Failed")
                } else {
                    comparable
                };
                if comparable && tested_in_a {
                    (
                        "new",
                        "Run A searched under equivalent conditions and did not find this failure."
                            .into(),
                    )
                } else if observed && tested_in_a {
                    ("only_right", "Run A executed this account without failure, but its search conditions differ.".into())
                } else {
                    (
                        "only_right",
                        "Run A did not test this state under equivalent search conditions.".into(),
                    )
                }
            }
            (Some(_), None) => {
                let retested = if observed {
                    right_executed.get(account).is_some_and(|o| o != "Failed")
                } else {
                    comparable
                };
                if b.search.is_none() {
                    (
                        "only_left",
                        "Run B has no recorded search, so this failure was not re-tested.".into(),
                    )
                } else if comparable && retested {
                    ("resolved", "Run B re-executed this state under equivalent search conditions without failure.".into())
                } else if observed && retested {
                    ("not_reproduced", "Run B executed the same token account without failure, but search conditions differ, so this is not reported as resolved. Its captured state may differ.".into())
                } else {
                    ("only_left", "Absent from run B, which did not re-test this state under equivalent conditions.".into())
                }
            }
            (None, None) => continue,
        };
        *counts.entry(status).or_default() += 1;
        items.push(json!({"key": key, "kind": if observed {"Observed"} else {"Derived"}, "account": account, "status": status, "note": note, "left": l, "right": r}));
    }
    json!({
        "left_total": left.len(),
        "right_total": right.len(),
        "comparable": comparable,
        "differences": differences,
        "counts": counts,
        "items": items,
        "identity": "Matched by observed token account (and search dimension for derived variants). Local cx_ IDs embed the package run, so they never match across runs.",
    })
}

fn stress_count(report: &Value, status: &str) -> Value {
    json!(report["selected_stress_counts"][status]
        .as_u64()
        .unwrap_or(0))
}

pub fn compare(store: &Store, left: &str, right: &str) -> Result<Value> {
    compare_runs(&load(store, left), &load(store, right))
}

/// Milestone 16 comparison semantics over two parsed runs, wherever their
/// bytes came from. The cloud workspace calls this same function.
pub fn compare_runs(a: &Run, b: &Run) -> Result<Value> {
    ensure!(
        a.report.is_some() && b.report.is_some(),
        "both runs need a readable report.json to compare"
    );
    let (ra, rb) = (a.report.as_ref().unwrap(), b.report.as_ref().unwrap());
    let (ma, mb) = (
        a.manifest.clone().unwrap_or(Value::Null),
        b.manifest.clone().unwrap_or(Value::Null),
    );
    let (sa, sb) = (summary(a), summary(b));
    let inputs = vec![
        field(
            "Candidate",
            "Candidate program hash",
            ra["candidate_program_sha256"].clone(),
            rb["candidate_program_sha256"].clone(),
        ),
        field(
            "Candidate",
            "Transition package hash",
            ra["transition_package_sha256"].clone(),
            rb["transition_package_sha256"].clone(),
        ),
        field(
            "Candidate",
            "Execution config hash",
            ra["config_sha256"].clone(),
            rb["config_sha256"].clone(),
        ),
        field(
            "Candidate",
            "Adapter",
            ra["adapter"].clone(),
            rb["adapter"].clone(),
        ),
        field(
            "Candidate",
            "Program ID",
            ma["candidateProgram"]["programId"].clone(),
            mb["candidateProgram"]["programId"].clone(),
        ),
        field(
            "Transition",
            "Source asset",
            ra["source_mint"].clone(),
            rb["source_mint"].clone(),
        ),
        field(
            "Transition",
            "Replacement asset",
            ra["replacement_mint"].clone(),
            rb["replacement_mint"].clone(),
        ),
        field(
            "Transition",
            "Ratio numerator",
            ra["terms"]["numerator"].clone(),
            rb["terms"]["numerator"].clone(),
        ),
        field(
            "Transition",
            "Ratio denominator",
            ra["terms"]["denominator"].clone(),
            rb["terms"]["denominator"].clone(),
        ),
        field(
            "Transition",
            "Rounding",
            ra["terms"]["rounding"].clone(),
            rb["terms"]["rounding"].clone(),
        ),
        field(
            "Transition",
            "Conversion fee (bps)",
            ra["terms"]["feeBps"].clone(),
            rb["terms"]["feeBps"].clone(),
        ),
        field(
            "Transition",
            "Effective at",
            ra["effective_at"].clone(),
            rb["effective_at"].clone(),
        ),
        field(
            "Execution",
            "Proposed replacement reserve (raw)",
            ra["proposed_reserve_raw"].clone(),
            rb["proposed_reserve_raw"].clone(),
        ),
        field(
            "Execution",
            "Source account",
            ra["source_account"].clone(),
            rb["source_account"].clone(),
        ),
        field(
            "Execution",
            "Public owner",
            ra["public_owner"].clone(),
            rb["public_owner"].clone(),
        ),
        field(
            "Execution",
            "Configured amount",
            sa["transition"]["amount_decimal"].clone(),
            sb["transition"]["amount_decimal"].clone(),
        ),
        field(
            "Policy",
            "Declared invariants",
            ma["invariants"].clone(),
            mb["invariants"].clone(),
        ),
        field(
            "Policy",
            "Gate policy",
            ra["gate_policy"].clone(),
            rb["gate_policy"].clone(),
        ),
    ];
    let population = |r: &Value, name: &str| r["population_summary"]["counts"][name].clone();
    let results = vec![
        field(
            "Production",
            "Token accounts observed",
            population(ra, "token_accounts_observed"),
            population(rb, "token_accounts_observed"),
        ),
        field(
            "Production",
            "Positive balances",
            population(ra, "positive_balance_accounts_observed"),
            population(rb, "positive_balance_accounts_observed"),
        ),
        field(
            "Production",
            "Authority resolution",
            ra["population_summary"]["authority_resolution_completeness"].clone(),
            rb["population_summary"]["authority_resolution_completeness"].clone(),
        ),
        field(
            "Production",
            "Enumeration",
            ra["population_summary"]["enumeration_completeness"].clone(),
            rb["population_summary"]["enumeration_completeness"].clone(),
        ),
        field(
            "Execution",
            "Candidate conversion",
            ra["conversion_result"]["status"].clone(),
            rb["conversion_result"]["status"].clone(),
        ),
        field(
            "Execution",
            "Candidate plan readiness",
            ra["candidate_plan_readiness"].clone(),
            rb["candidate_plan_readiness"].clone(),
        ),
        field(
            "Stress",
            "Selected cases",
            json!(ra["exact_selected_cases"].as_array().map_or(0, Vec::len)),
            json!(rb["exact_selected_cases"].as_array().map_or(0, Vec::len)),
        ),
        field(
            "Stress",
            "Proven",
            stress_count(ra, "Proven"),
            stress_count(rb, "Proven"),
        ),
        field(
            "Stress",
            "Failed",
            stress_count(ra, "Failed"),
            stress_count(rb, "Failed"),
        ),
        field(
            "Stress",
            "Indeterminate",
            stress_count(ra, "Indeterminate"),
            stress_count(rb, "Indeterminate"),
        ),
        field(
            "Stress",
            "Unsupported",
            stress_count(ra, "Unsupported"),
            stress_count(rb, "Unsupported"),
        ),
        field(
            "Stress",
            "Stress readiness",
            ra["conversion_stress_readiness"]["status"].clone(),
            rb["conversion_stress_readiness"]["status"].clone(),
        ),
        field(
            "Readiness",
            "Population rollout readiness",
            ra["population_rollout_readiness"]["status"].clone(),
            rb["population_rollout_readiness"]["status"].clone(),
        ),
        field(
            "Readiness",
            "Official transition",
            ra["official_transition"].clone(),
            rb["official_transition"].clone(),
        ),
    ];
    let invariant_map = |r: &Value| -> BTreeMap<String, Value> {
        r["invariants"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|i| Some((i["invariant_id"].as_str()?.to_owned(), i.clone())))
            .collect()
    };
    let (ia, ib) = (invariant_map(ra), invariant_map(rb));
    let invariant_ids: BTreeSet<&String> = ia.keys().chain(ib.keys()).collect();
    let invariants = invariant_ids.into_iter().map(|id| {
        let (l, r) = (ia.get(id), ib.get(id));
        let kind = l.or(r).map_or(Value::Null, |i| i["invariant_type"].clone());
        let status = |v: Option<&Value>| v.map_or(Value::Null, |i| i["status"].clone());
        json!({"invariant_id": id, "invariant_type": kind, "severity": l.or(r).map_or(Value::Null, |i| i["severity"].clone()),
               "left": status(l), "right": status(r), "changed": status(l) != status(r),
               "left_explanation": l.map(|i| i["explanation"].clone()), "right_explanation": r.map(|i| i["explanation"].clone())})
    }).collect::<Vec<_>>();
    let reasons = |r: &Value| -> BTreeSet<String> {
        r["gate_reasons"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|x| x.as_str().map(str::to_owned))
            .collect()
    };
    let (ga, gb) = (reasons(ra), reasons(rb));
    let meta = |run: &Run, f: fn(&Metadata) -> Value| run.metadata.as_ref().map_or(Value::Null, f);
    let git = vec![
        field(
            "Git",
            "Run source",
            meta(a, |m| json!(m.run_source)),
            meta(b, |m| json!(m.run_source)),
        ),
        field(
            "Git",
            "Commit",
            meta(a, |m| json!(m.git_commit)),
            meta(b, |m| json!(m.git_commit)),
        ),
        field(
            "Git",
            "Branch",
            meta(a, |m| json!(m.git_branch)),
            meta(b, |m| json!(m.git_branch)),
        ),
        field(
            "Git",
            "Uncommitted changes",
            meta(a, |m| json!(m.git_dirty)),
            meta(b, |m| json!(m.git_dirty)),
        ),
        field(
            "Tooling",
            "Eplyx version",
            meta(a, |m| json!(m.eplyx_version)),
            meta(b, |m| json!(m.eplyx_version)),
        ),
        field(
            "Tooling",
            "Engine binary hash",
            meta(a, |m| json!(m.engine_binary_sha256)),
            meta(b, |m| json!(m.engine_binary_sha256)),
        ),
    ];
    Ok(json!({
        "left": sa,
        "right": sb,
        "inputs": inputs,
        "results": results,
        "invariants": invariants,
        "gate": {
            "left": {"outcome": ra["gate_outcome"], "policy": ra["gate_policy"]},
            "right": {"outcome": rb["gate_outcome"], "policy": rb["gate_policy"]},
            "changed": ra["gate_outcome"] != rb["gate_outcome"] || ra["gate_policy"] != rb["gate_policy"],
            "reasons_added": gb.difference(&ga).collect::<Vec<_>>(),
            "reasons_removed": ga.difference(&gb).collect::<Vec<_>>(),
            "reasons_kept": ga.intersection(&gb).count(),
        },
        "git": git,
        "counterexamples": counterexample_diff(a, b),
        "causality": "Differences are listed side by side. Eplyx does not infer which input change caused which result change.",
    }))
}

/// Join run, counterexample and reproduction summaries into the dashboard's
/// lists. Runs arrive oldest first (sorted by run ID, which starts with its
/// UTC timestamp) and are numbered in that order; reproductions arrive newest
/// first. Returns runs newest first and counterexamples by parent run.
pub fn assemble(
    run_summaries: Vec<Value>,
    counterexample_summaries: Vec<Value>,
    reproductions_newest_first: &[Value],
) -> (Vec<Value>, Vec<Value>) {
    let history: Vec<&Value> = reproductions_newest_first
        .iter()
        .filter(|r| r["state"] == "Valid")
        .collect();
    let mut files: Vec<Value> = counterexample_summaries
        .into_iter()
        .map(|mut file| {
            let mine: Vec<&&Value> = history
                .iter()
                .filter(|r| r["counterexample_id"] == file["id"])
                .collect();
            file["reproductions"] = json!({
                "count": mine.len(),
                "succeeded": mine.iter().filter(|r| r["outcome"] == "Reproduced").count(),
                "failed": mine.iter().filter(|r| r["outcome"] == "Failed").count(),
                "last_timestamp": mine.first().map(|r| r["timestamp"].clone()),
                "last_outcome": mine.first().map(|r| r["outcome"].clone()),
                "history": mine.iter().take(50).collect::<Vec<_>>(),
            });
            file
        })
        .collect();
    let mut runs: Vec<Value> = run_summaries
        .into_iter()
        .enumerate()
        .map(|(position, mut run)| {
            let id = run["id"].as_str().unwrap_or("").to_owned();
            let mine = files.iter().filter(|c| c["parent_run"] == id.as_str());
            let (mut total, mut observed) = (0, 0);
            for c in mine {
                total += 1;
                observed += usize::from(c["kind"] == "Observed");
            }
            run["number"] = json!(position + 1);
            run["saved_counterexamples"] =
                json!({"total": total, "observed": observed, "derived": total - observed});
            run
        })
        .collect();
    runs.reverse();
    for file in &mut files {
        let parent = runs.iter().find(|r| r["id"] == file["parent_run"]);
        file["parent"] = parent.map_or(Value::Null, |run| {
            json!({
                "number": run["number"], "gate": run["gate"]["outcome"], "timestamp": run["timestamp"],
                "source_mint": run["transition"]["source_mint"], "replacement_mint": run["transition"]["replacement_mint"],
                "candidate_program_sha256": run["candidate_program_sha256"],
            })
        });
    }
    let number = |c: &Value| c["parent"]["number"].as_u64().unwrap_or(0);
    files.sort_by(|a, b| {
        number(b)
            .cmp(&number(a))
            .then_with(|| a["kind"].as_str().cmp(&b["kind"].as_str()))
            .then_with(|| a["id"].as_str().cmp(&b["id"].as_str()))
    });
    (runs, files)
}

/// The Overview/Project payload shared by the local and cloud dashboards.
pub fn project_payload(
    project: Value,
    context: Value,
    store: Value,
    runs: &[Value],
    files: &[Value],
    reproductions: &[Value],
    ignored: usize,
) -> Value {
    let latest = runs.iter().find(|r| r["state"] == "Complete").cloned();
    json!({
        "project": project,
        "context": context,
        "store": store,
        "stats": stats(runs, files, reproductions, ignored),
        "latest": latest,
        "recent_runs": runs.iter().take(6).collect::<Vec<_>>(),
        "recent_counterexamples": files.iter().take(6).collect::<Vec<_>>(),
        "gate_history": runs.iter().take(30).map(|r| json!({"id": r["id"], "number": r["number"], "outcome": r["gate"]["outcome"], "timestamp": r["timestamp"]})).collect::<Vec<_>>(),
    })
}

/// Local usage statistics from `.eplyx/` only; nothing is sent anywhere.
pub fn stats(
    runs: &[Value],
    counterexamples: &[Value],
    reproductions: &[Value],
    ignored: usize,
) -> Value {
    let valid: Vec<&Value> = reproductions
        .iter()
        .filter(|r| r["state"] == "Valid")
        .collect();
    let latest_reproduction = valid.iter().filter_map(|r| r["timestamp"].as_str()).max();
    let outcome = |name: &str| runs.iter().filter(|r| r["gate"]["outcome"] == name).count();
    let timestamps: Vec<&str> = runs
        .iter()
        .filter_map(|r| r["timestamp"].as_str())
        .collect();
    let branches: BTreeSet<&str> = runs
        .iter()
        .filter_map(|r| r["git"]["branch"].as_str())
        .collect();
    let mut kinds = Map::new();
    for kind in ["Observed", "Derived"] {
        kinds.insert(
            kind.into(),
            json!(counterexamples.iter().filter(|c| c["kind"] == kind).count()),
        );
    }
    json!({
        "runs": runs.len(),
        "preflights": runs.iter().filter(|r| r["state"] == "Complete").count(),
        "unfinished_or_unreadable": runs.iter().filter(|r| r["state"] != "Complete").count(),
        "searches": runs.iter().filter(|r| r["search"]["state"] == "Recorded").count(),
        "counterexamples_saved": counterexamples.len(),
        "counterexample_kinds": kinds,
        "passed": outcome("Pass"),
        "warned": outcome("Warn"),
        "blocked": outcome("Block"),
        "offline_reproductions": valid.len(),
        "reproductions_succeeded": valid.iter().filter(|r| r["outcome"] == "Reproduced").count(),
        "reproductions_failed": valid.iter().filter(|r| r["outcome"] == "Failed").count(),
        "latest_reproduction": latest_reproduction,
        "offline_reproductions_note": "Recorded by `eplyx reproduce` in .eplyx/reproductions/ from Milestone 16 onward; earlier reproductions were not recorded.",
        "run_sources": {
            "local": runs.iter().filter(|r| r["run_source"] == "local").count(),
            "ci": runs.iter().filter(|r| r["run_source"] == "ci").count(),
            "imported": runs.iter().filter(|r| r["run_source"] == "imported").count(),
            "not_recorded": runs.iter().filter(|r| r["run_source"].is_null()).count(),
        },
        "first_run": timestamps.iter().min(),
        "latest_run": timestamps.iter().max(),
        "branches": branches,
        "ignored_store_entries": ignored,
    })
}
