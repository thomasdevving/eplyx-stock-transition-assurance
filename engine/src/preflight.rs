//! Prospective orchestration. Current bytes, proposed semantics and execution remain separate.
use crate::{
    expansion::{canonical, digest},
    lifecycle::{
        consequence::classify_public_exposure, current as wallet, decode, exposure::sha256,
        policy::*,
    },
    probe::current as execution,
    readiness,
    resolution::{self, PathStatus},
};
use anyhow::{ensure, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_address::Address;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub source: String,
    pub successor_mint: Option<String>,
    pub effective_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub post_deadline: Option<PostDeadline>,
    pub assurance: Assurance,
    pub check_ids: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PostDeadline {
    TransitionStillRequired,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Assurance {
    Mobility,
    FullTransition,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckEvidence {
    pub id: String,
    pub capture_sha256: String,
    pub result_sha256: String,
    pub engine_sha256: String,
    pub capture: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inputs {
    pub run_id: String,
    pub preflight_id: String,
    pub created_at: DateTime<Utc>,
    pub engine_sha256: String,
    pub wallet_capture: String,
    pub wallet_sha256: String,
    pub request: Request,
    pub successor_capture: Option<String>,
    pub checks: Vec<CheckEvidence>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub schema_version: u32,
    pub inputs: Inputs,
    pub scenario: Value,
    pub scenario_sha256: String,
}

fn account(input: &Inputs) -> Result<(wallet::Capture, Value, decode::TokenAccountState, Value)> {
    ensure!(
        input.wallet_capture.len() <= 10 * 1024 * 1024
            && sha256(input.wallet_capture.as_bytes()) == input.wallet_sha256,
        "preflight wallet digest mismatch"
    );
    let capture: wallet::Capture = serde_json::from_str(&input.wallet_capture)?;
    ensure!(
        capture.schema_version == 3,
        "preflight requires current wallet capture"
    );
    let observed = wallet::evaluate(&capture)?;
    let row = observed["wallet_observation"]["token_accounts"]
        .as_array()
        .context("wallet accounts unavailable")?
        .iter()
        .find(|a| a["address"] == input.request.source)
        .context("selected current account missing")?;
    let state: decode::TokenAccountState = serde_json::from_value(row["state"].clone())?;
    ensure!(
        state.owner
            == capture
                .selection
                .as_ref()
                .and_then(|s| s.public_owner.as_ref())
                .context("owner missing")?
                .as_str()
            && state.mint == capture.asset.mint,
        "preflight entity identity mismatch"
    );
    let raw = capture.observations[3]
        .result
        .as_ref()
        .context("lookup missing")?["value"]
        .as_array()
        .context("account bytes missing")?
        .iter()
        .find(|v| v["pubkey"] == input.request.source)
        .context("selected account bytes missing")?["account"]
        .clone();
    Ok((capture, observed, state, raw))
}
pub fn validate(input: &Inputs) -> Result<()> {
    let (capture, _, _, _) = account(input)?;
    let r = &input.request;
    let _: Address = r.source.parse()?;
    ensure!(
        !input.run_id.is_empty() && !input.preflight_id.is_empty(),
        "missing run identity"
    );
    ensure!(
        r.effective_at > input.created_at
            && input.created_at
                >= DateTime::parse_from_rfc3339(&capture.completed_at)?.with_timezone(&Utc),
        "proposed effective time must be in the future at creation"
    );
    ensure!(
        r.deadline.is_some() == r.post_deadline.is_some()
            && r.deadline.is_none_or(|d| d > r.effective_at),
        "deadline needs a supported semantic rule and must follow effective time"
    );
    if let Some(mint) = &r.successor_mint {
        let _: Address = mint.parse().context("invalid replacement mint address")?;
        ensure!(
            mint != &capture.asset.mint,
            "replacement must differ from current mint"
        );
    }
    ensure!(
        r.check_ids.len() <= 4
            && r.check_ids.iter().collect::<BTreeSet<_>>().len() == r.check_ids.len(),
        "choose at most four distinct current checks"
    );
    Ok(())
}
fn scenario(input: &Inputs) -> Result<Value> {
    validate(input)?;
    let (capture, _, state, raw) = account(input)?;
    let r = &input.request;
    let mut supports = vec![
        "/policy/effective_at".into(),
        "/policy/before".into(),
        "/policy/after".into(),
    ];
    if r.deadline.is_some() {
        supports.push("/policy/deadline".into());
    }
    if r.successor_mint.is_some() {
        supports.push("/policy/successor".into());
    }
    let lifecycle=LifecycleScenario {schema_version:1,scenario_type:LifecycleScenarioType::LifecycleChange,id:input.preflight_id.clone(),scenario_version:"current-proposed/v1".into(),captured_at:input.created_at,
        change:crate::scenario::LifecycleChange{description:"User-proposed replacement-token transition against unchanged currently captured state".into()},
        policy:AssetLifecyclePolicy{asset_mint:capture.asset.mint.clone(),effective_at:r.effective_at,before:LifecycleStatus::Active,after:LifecycleStatus::TransitionRequired,
            deadline:r.deadline.map(|at|LifecycleDeadline{at,after:LifecycleStatus::PostDeadlineTransitionRequired}),
            successor:r.successor_mint.as_ref().map(|mint|SuccessorAsset{mint:mint.clone(),description:"User-proposed association only; issuer relationship, ratio and mechanism unknown".into()})},
        sources:vec![LifecycleSource{id:"user-proposed".into(),kind:LifecycleSourceKind::ScenarioAssumption,reference:format!("current-run:{}",input.run_id),description:"UserProposed, not issuer asserted or on-chain policy".into(),captured_at:input.created_at,supports,artifact:None,content_sha256:None}]};
    lifecycle.validate()?;
    Ok(
        json!({"scenario_id":input.preflight_id,"scenario_version":"current-proposed/v1","source":"UserProposed","created_for_run":input.run_id,
        "created_at":input.created_at,"wallet_capture_sha256":input.wallet_sha256,"source_asset":capture.asset.mint,"public_owner":state.owner,
        "focused_account":r.source,"current_balance_raw":state.raw_balance,"account_sha256":digest(&raw)?,
        "successor_asset":r.successor_mint,"successor_capture_sha256":input.successor_capture.as_ref().map(|s|sha256(s.as_bytes())),
        "execution_evidence":input.checks.iter().map(|c|json!({"id":c.id,"capture_sha256":c.capture_sha256,"result_sha256":c.result_sha256,"engine_sha256":c.engine_sha256})).collect::<Vec<_>>(),
        "lifecycle":lifecycle,"provenance":"User-provided hypothetical policy; current mint observations and local checks are separate evidence",
        "issuer_relationship_verified":false,"conversion_ratio":null,"official_authorization":false}),
    )
}
pub fn prepare(input: Inputs) -> Result<Bundle> {
    let scenario = scenario(&input)?;
    Ok(Bundle {
        schema_version: 1,
        scenario_sha256: digest(&scenario)?,
        inputs: input,
        scenario,
    })
}
pub fn replay(
    bytes: &[u8],
    run: &str,
    id: &str,
    wallet_hash: &str,
    scenario_hash: &str,
    bundle_hash: &str,
) -> Result<Value> {
    ensure!(
        bytes.len() <= 192 * 1024 * 1024 && sha256(bytes) == bundle_hash,
        "preflight bundle digest mismatch"
    );
    let bundle: Bundle = serde_json::from_slice(bytes)?;
    let input = &bundle.inputs;
    ensure!(
        bundle.schema_version == 1
            && input.run_id == run
            && input.preflight_id == id
            && input.wallet_sha256 == wallet_hash,
        "preflight run binding mismatch"
    );
    ensure!(
        bundle.scenario_sha256 == scenario_hash
            && digest(&bundle.scenario)? == scenario_hash
            && scenario(input)? == bundle.scenario,
        "preflight scenario binding mismatch"
    );
    let (capture, observed, state, raw) = account(input)?;
    let r = &input.request;
    let ids: BTreeSet<_> = input.checks.iter().map(|c| c.id.clone()).collect();
    ensure!(
        ids.len() == input.checks.len() && ids == r.check_ids.iter().cloned().collect(),
        "selected execution evidence mismatch"
    );
    let mut verified = Vec::new();
    for check in &input.checks {
        let proof = execution::replay(
            check.capture.as_bytes(),
            run,
            &check.id,
            wallet_hash,
            &check.capture_sha256,
        )?;
        ensure!(
            sha256(format!("{}\n", proof.value()).as_bytes()) == check.result_sha256,
            "original execution result digest mismatch"
        );
        verified.push(proof);
    }
    let lifecycle: LifecycleScenario =
        serde_json::from_value(bundle.scenario["lifecycle"].clone())?;
    let scope = resolution::current::CurrentEntityScope {
        run,
        wallet_hash,
        source: &r.source,
        mint: &state.mint,
        owner: &state.owner,
        balance: &state.raw_balance,
        scenario_hash,
    };
    let paths = resolution::current::resolve(&scope, &verified)?;
    let mut successor = json!({"status":"NotProvided","issuer_relationship_verified":false,"conversion_ratio":null});
    let mut preparation = "Prepared";
    if let Some(mint) = &r.successor_mint {
        let text = input
            .successor_capture
            .as_ref()
            .context("independent replacement inspection required")?;
        ensure!(
            text.len() <= 10 * 1024 * 1024,
            "replacement capture exceeds budget"
        );
        let c: wallet::Capture = serde_json::from_str(text)?;
        ensure!(
            c.schema_version == 2
                && c.asset.mint == *mint
                && c.selection.as_ref().is_some_and(|s| s.mint == *mint
                    && s.public_owner.is_none()
                    && !s.sample_accounts
                    && s.reference.is_none()),
            "replacement inspection binding mismatch"
        );
        match wallet::evaluate(&c) {
            Ok(value) => {
                if value["inspection"]["status"] == "Completed"
                    && value["mint"]["is_initialized"] == true
                {
                    successor = json!({"status":"MintObserved","message":"Replacement mint exists on Solana","observation":value,"capture_sha256":sha256(text.as_bytes()),"issuer_relationship_verified":false,"conversion_ratio":null});
                } else {
                    preparation = "InvalidScenario";
                    successor = json!({"status":"NotSupportedMint","message":"The replacement address is not a supported initialized token mint","observation":value,"issuer_relationship_verified":false,"conversion_ratio":null});
                }
            }
            Err(_) => {
                preparation = "PreparationIncomplete";
                successor = json!({"status":"Unavailable","message":"Replacement mint inspection could not complete; retry preparation","capture_sha256":sha256(text.as_bytes()),"issuer_relationship_verified":false,"conversion_ratio":null});
            }
        }
    } else {
        ensure!(
            input.successor_capture.is_none(),
            "unexpected replacement capture"
        );
    }
    let mut views = vec![];
    if preparation == "Prepared" {
        let mut times = vec![
            ("BeforeProposedEvent", input.created_at),
            ("ProposedActive", r.effective_at),
        ];
        if let Some(deadline) = r.deadline {
            times.push(("AfterProposedDeadline", deadline));
        }
        let encrypted = state
            .extensions
            .iter()
            .any(|e| e.extension_type.contains("Confidential"));
        for (view, at) in times {
            let status = lifecycle.policy.status_at(at);
            let (impact, reason) = classify_public_exposure(
                state.raw_balance.parse::<u64>()? > 0,
                encrypted,
                status,
                false,
            )?;
            let active = status != LifecycleStatus::Active;
            let mobility = readiness::current::evaluate_selected(
                &paths,
                &lifecycle.policy,
                &scope,
                &at.to_rfc3339(),
                false,
            )?;
            let full = readiness::current::evaluate_selected(
                &paths,
                &lifecycle.policy,
                &scope,
                &at.to_rfc3339(),
                true,
            )?;
            views.push(json!({"id":view,"evaluated_at":at,"lifecycle_status":status,"impact":impact,"reason":reason,
                "balance_raw":state.raw_balance,"account_sha256":digest(&raw)?,"technical_state":"Unchanged",
                "gate_applicability":if active{"Applicable"}else{"PreEvent"},
                "paths":paths.rows().iter().map(|p|json!({"path":p.path_type,"status":p.status,"lifecycle_relevant":active && state.raw_balance!="0" && p.status!=PathStatus::NotApplicable})).collect::<Vec<_>>(),
                "readiness":if active {json!({"mobility":mobility,"full_transition":full})}else{Value::Null}}));
        }
    }
    Ok(
        json!({"schema_version":1,"kind":"current-preflight","preparation_status":preparation,"run_id":run,"preflight_id":id,
        "wallet_capture_sha256":wallet_hash,"scenario_sha256":scenario_hash,"bundle_sha256":bundle_hash,"engine_sha256":input.engine_sha256,
        "entity":{"account":r.source,"owner":state.owner,"mint":state.mint,"balance_raw":state.raw_balance,"balance_decimal":state.ui_balance,"account_sha256":digest(&raw)?},
        "current_state":{"acquisition":observed["acquisition"],"captured_at":capture.completed_at,"account":state,"mint":observed["mint"]},
        "proposed_change":bundle.scenario,"successor_verification":successor,"views":views,"paths":paths.rows(),"selected_assurance":r.assurance,
        "execution_evidence":verified.iter().map(|p|p.value()).collect::<Vec<_>>(),"funds_moved":false,"authorization":false,"population_readiness":null,
        "freshness_policy":"Evidence is pinned to these observed times and this run. No age-based promise or future-state prediction; refresh creates a new unverified run. Scenario re-evaluation performs no live check.",
        "limitations":["UserProposed semantics are hypothetical, not issuer assertions or legal entitlement","Same captured bytes and balance at every policy time; future balances and liquidity are not predicted","Mobility readiness covers the exact selected tested amounts, recipients and routes only, with local signing assumed and possession unknown","Transfer and market sale never prove replacement-token conversion; no conversion ratio is inferred","Selected direct token account only; protocol positions and population readiness are not assessed"]}),
    )
}
pub fn save(bundle: &Bundle, path: &std::path::Path) -> Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(canonical(bundle)?.as_bytes())?;
    file.sync_all()?;
    Ok(())
}
