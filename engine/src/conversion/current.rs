//! Current-run candidate conversion: bounded read-only capture, then offline
//! execution of the registered candidate mechanism. Evidence is created only by
//! rebuilding and running the actual program; no serialized status is accepted.
use super::{coherence, demo, AmountMode, ConversionPlan, VerifiedReplacementConversion};
use crate::{
    executor,
    expansion::digest,
    lifecycle::{current as observation, decode, exposure::sha256, rpc::SolanaRpc, RpcEvidence},
    probe::{current::exact_amount, ProbeClock, ProbeMessage},
    resolution::PathStatus,
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_address::Address;
use std::{cell::RefCell, io::Write, path::Path};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capture {
    pub schema_version: u32,
    pub run_id: String,
    pub check_id: String,
    pub wallet_capture: String,
    pub wallet_capture_sha256: String,
    pub plan: ConversionPlan,
    pub plan_sha256: String,
    pub started_at: String,
    pub completed_at: String,
    pub rpc_origin: String,
    pub observations: Vec<observation::Observation>,
}
fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}
struct Scope {
    context: demo::ConversionContext,
    plan_sha256: String,
    amount: u64,
    decimals: u8,
    source_raw: Value,
    unsupported: Option<String>,
}
/// Revalidate the supplied plan against the exact current wallet run.
fn scope(wallet: &str, plan: &ConversionPlan) -> Result<Scope> {
    ensure!(
        wallet.len() <= 10 * 1024 * 1024,
        "wallet capture exceeds budget"
    );
    plan.validate()?;
    let capture: observation::Capture = serde_json::from_str(wallet)?;
    ensure!(
        capture.schema_version == 3,
        "a current wallet observation is required"
    );
    let observed = observation::evaluate(&capture)?;
    ensure!(
        plan.source_mint == capture.asset.mint,
        "the plan's source mint is not the mint of this wallet run"
    );
    let row = observed["wallet_observation"]["token_accounts"]
        .as_array()
        .context("wallet accounts unavailable")?
        .iter()
        .find(|a| a["address"] == plan.source_account)
        .context("the plan's source account was not discovered in this wallet run")?;
    let state: decode::TokenAccountState = serde_json::from_value(row["state"].clone())?;
    let mint: decode::MintConfig = serde_json::from_value(observed["mint"].clone())?;
    let owner = capture
        .selection
        .as_ref()
        .and_then(|s| s.public_owner.as_ref())
        .context("missing public owner")?;
    ensure!(
        state.owner == *owner && state.mint == capture.asset.mint,
        "selected source account identity mismatch"
    );
    let balance = state.raw_balance.parse::<u64>()?;
    let amount = match plan.amount_mode {
        AmountMode::Full => balance,
        AmountMode::Custom => exact_amount(
            plan.amount_decimal
                .as_deref()
                .context("custom amount required")?,
            mint.decimals,
        )?,
    };
    ensure!(
        amount > 0 && amount <= balance,
        "the candidate amount must be positive and no greater than the observed public balance"
    );
    let source_raw = capture.observations[3]
        .result
        .as_ref()
        .context("wallet lookup missing")?["value"]
        .as_array()
        .context("missing accounts")?
        .iter()
        .find(|a| a["pubkey"] == plan.source_account)
        .context("source raw bytes missing")?["account"]
        .clone();
    let mut boundary = None;
    if !state.is_initialized || state.is_frozen {
        boundary = Some("Uninitialized or frozen source token account".into());
    }
    if state
        .extensions
        .iter()
        .any(|e| e.extension_type.contains("Confidential"))
    {
        boundary = Some("Confidential source accounts are unsupported".into());
    }
    for extension in &mint.extensions {
        if (extension.extension_type == "Pausable" && extension.config["paused"] == true)
            || extension.extension_type == "ConfidentialMintBurn"
        {
            boundary = Some(format!(
                "Unsupported source configuration for candidate burn: {}",
                extension.extension_type
            ));
        }
    }
    let authority: Address = owner.parse()?;
    let owner_raw = &capture.observations[2]
        .result
        .as_ref()
        .context("owner inspection missing")?["value"];
    if !authority.is_on_curve()
        || owner_raw.is_null()
        || owner_raw["owner"] != demo::SYSTEM_PROGRAM
        || owner_raw["executable"] != false
        || !decode::raw_account_bytes(owner_raw)?.is_empty()
    {
        boundary = Some("Recorded authority is not a supported directly signing wallet".into());
    }
    Ok(Scope {
        context: demo::ConversionContext {
            genesis_hash: observed["acquisition"]["genesis_hash"]
                .as_str()
                .context("missing genesis")?
                .into(),
            minimum_slot: row["slot"].as_u64().context("missing discovery slot")?,
            owner: owner.clone(),
            source_program: mint.token_program.clone(),
            source_decimals: mint.decimals,
            amount,
        },
        plan_sha256: plan.sha256()?,
        amount,
        decimals: mint.decimals,
        source_raw,
        unsupported: boundary,
    })
}
/// Offline plan validation against the current run. Never execution evidence.
pub fn validate(wallet: &str, plan: &ConversionPlan) -> Result<Value> {
    let s = scope(wallet, plan)?;
    let expectation = super::expected_output(s.amount, &plan.terms)?;
    Ok(json!({
        "plan_sha256": s.plan_sha256,
        "amount_raw": s.amount.to_string(),
        "amount_decimal": decode::decimal_amount(s.amount, s.decimals),
        "expected": expectation,
        "provenance": plan.provenance,
        "status": if s.unsupported.is_some() { PathStatus::Unsupported } else { PathStatus::NotTested },
        "reason": s.unsupported,
        "execution_performed": false,
        "official_transition": PathStatus::NotTested,
    }))
}
struct Recorder<'a, R> {
    rpc: &'a R,
    observations: RefCell<Vec<observation::Observation>>,
}
impl<R: SolanaRpc> SolanaRpc for Recorder<'_, R> {
    fn origin(&self) -> String {
        self.rpc.origin()
    }
    fn call(&self, method: &str, params: Value) -> Result<Value> {
        let start = now();
        let value = self.rpc.call(method, params.clone());
        self.observations
            .borrow_mut()
            .push(observation::Observation {
                method: method.into(),
                params,
                started_at: start,
                completed_at: now(),
                result: value.as_ref().ok().cloned(),
                error: value.as_ref().err().map(ToString::to_string),
            });
        value
    }
}
fn config(slot: u64) -> Value {
    json!({"encoding":"base64","commitment":"finalized","minContextSlot":slot})
}
/// Four discovery requests, then at most three serial final-bank attempts.
/// The replacement mint is inspected independently here; existence never
/// establishes an issuer relationship.
fn acquire(s: &Scope, plan: &ConversionPlan, rpc: &impl SolanaRpc) -> Result<()> {
    rpc.call("getGenesisHash", json!([]))?;
    let source_mint = rpc.call(
        "getAccountInfo",
        json!([plan.source_mint, config(s.context.minimum_slot)]),
    )?;
    let source_slot = crate::probe::meteora_dlmm::slot(&source_mint)?;
    let replacement = rpc.call(
        "getAccountInfo",
        json!([plan.replacement_mint, config(source_slot)]),
    )?;
    let replacement_slot = crate::probe::meteora_dlmm::slot(&replacement)?;
    let replacement_program = replacement["value"]["owner"]
        .as_str()
        .context("the replacement mint account does not exist")?
        .to_string();
    let mut programs = vec![
        s.context.source_program.clone(),
        replacement_program.clone(),
        crate::probe::meteora_dlmm::ATA_PROGRAM.into(),
    ];
    programs.sort();
    programs.dedup();
    let headers = rpc.call(
        "getMultipleAccounts",
        json!([programs, config(replacement_slot)]),
    )?;
    let programdata = demo::programdata_addresses(
        headers["value"]
            .as_array()
            .context("missing program headers")?,
    )?;
    let overlay = demo::derive(
        &s.plan_sha256,
        &s.context.owner,
        &plan.replacement_mint,
        &replacement_program,
    )?;
    let addresses = demo::address_plan(
        plan,
        &overlay,
        &s.context.owner,
        &s.context.source_program,
        &replacement_program,
        &programdata,
    );
    coherence::capture_final(
        &addresses,
        crate::probe::meteora_dlmm::slot(&headers)?,
        |params| rpc.call("getMultipleAccounts", params).ok(),
    );
    Ok(())
}
pub fn capture(
    wallet: String,
    plan: ConversionPlan,
    run_id: String,
    check_id: String,
    rpc: &impl SolanaRpc,
) -> Result<Capture> {
    let s = scope(&wallet, &plan)?;
    let recorder = Recorder {
        rpc,
        observations: RefCell::new(vec![]),
    };
    let started_at = now();
    if s.unsupported.is_none() {
        eprintln!("CURRENT_STAGE:Revalidating current state");
        // An incomplete acquisition stays a non-executed check; it never becomes Failed.
        let _ = acquire(&s, &plan, &recorder);
    }
    Ok(Capture {
        schema_version: 3,
        run_id,
        check_id,
        wallet_capture_sha256: sha256(wallet.as_bytes()),
        wallet_capture: wallet,
        plan_sha256: s.plan_sha256,
        plan,
        started_at,
        completed_at: now(),
        rpc_origin: rpc.origin(),
        observations: recorder.observations.into_inner(),
    })
}
pub fn save(capture: &Capture, path: &Path) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(&serde_json::to_vec(capture)?)?;
    file.sync_all()?;
    Ok(())
}
/// Rebuild the observed bank and the proposed overlay, run the actual registered
/// candidate program and reconcile exactly. This is also the offline replay path.
/// Every binding is a separate argument on purpose: each one must be checked.
#[allow(clippy::too_many_arguments)]
pub fn replay(
    bytes: &[u8],
    run_id: &str,
    check_id: &str,
    wallet_hash: &str,
    capture_hash: &str,
    plan_hash: &str,
    program: &[u8],
    program_hash: &str,
) -> Result<VerifiedReplacementConversion> {
    ensure!(
        bytes.len() <= 64 * 1024 * 1024 && sha256(bytes) == capture_hash,
        "conversion capture digest mismatch"
    );
    let c: Capture = serde_json::from_slice(bytes)?;
    ensure!(
        (c.schema_version == 1 || c.schema_version == 2 || c.schema_version == 3)
            && c.run_id == run_id
            && c.check_id == check_id
            && c.wallet_capture_sha256 == wallet_hash
            && sha256(c.wallet_capture.as_bytes()) == wallet_hash,
        "conversion run or wallet binding mismatch"
    );
    ensure!(
        sha256(program) == program_hash,
        "candidate program digest mismatch"
    );
    let s = scope(&c.wallet_capture, &c.plan)?;
    ensure!(
        c.plan_sha256 == s.plan_sha256 && c.plan_sha256 == plan_hash,
        "conversion plan digest mismatch"
    );
    let start = chrono::DateTime::parse_from_rfc3339(&c.started_at)?;
    let end = chrono::DateTime::parse_from_rfc3339(&c.completed_at)?;
    ensure!(
        start <= end && c.observations.len() <= 4 + coherence::MAX_FINAL_ATTEMPTS,
        "invalid conversion acquisition interval or budget"
    );
    let mut previous = start;
    for r in &c.observations {
        let a = chrono::DateTime::parse_from_rfc3339(&r.started_at)?;
        let b = chrono::DateTime::parse_from_rfc3339(&r.completed_at)?;
        ensure!(
            previous <= a && a <= b && b <= end && r.result.is_some() != r.error.is_some(),
            "invalid acquisition record"
        );
        previous = b;
    }
    let mut result = json!({
        "schema_version": 1, "kind": "current-conversion", "run_id": run_id, "check_id": check_id,
        "wallet_capture_sha256": wallet_hash, "execution_capture_sha256": capture_hash,
        "plan": c.plan, "plan_sha256": c.plan_sha256, "provenance": c.plan.provenance,
        "candidate_mechanism": {
            "id": demo::PROGRAM_ID, "name": "Eplyx Demo Candidate Conversion",
            "adapter_id": super::ADAPTER_ID, "revision": demo::REVISION,
            "artifact": demo::ARTIFACT, "program_sha256": program_hash, "loader": demo::LOADER,
            "origin": "Proposed", "registered_by": "This repository",
            "deployed_on_mainnet": false, "issuer_mechanism": false,
            "program_id_preimage": demo::PROGRAM_PREIMAGE,
            "note": "A candidate mechanism the operator intends to deploy. It is not deployed on any cluster and is not a PreStocks, SPACEX or issuer mechanism."
        },
        "mint": c.plan.source_mint, "source": c.plan.source_account, "owner": s.context.owner,
        "replacement_mint": c.plan.replacement_mint,
        "amount_raw": s.amount.to_string(),
        "amount_decimal": decode::decimal_amount(s.amount, s.decimals),
        "status": PathStatus::Indeterminate,
        "execution_performed": false, "local_execution_performed": false, "funds_moved": false,
        "signer_assumed_locally": false, "signer_possession_known": false,
        "candidate_authority_assumed_locally": true, "candidate_authority_possession_known": false,
        "issuer_binding_established": false, "official_transition": PathStatus::NotTested,
        "lifecycle_event": Value::Null, "readiness": Value::Null, "authorization": false,
        "acquisition": {"started_at": c.started_at, "completed_at": c.completed_at,
            "rpc_origin": c.rpc_origin, "commitment": "finalized",
            "discovery_slot": s.context.minimum_slot,
            "consistency": "Wallet discovery and this candidate check are separate observations. The final captured batch is authoritative for the conversion result."},
        "scope": "Only this exact source account, source mint, amount, replacement mint, destination, candidate plan version, candidate program build, proposed overlay, captured bank and assumed local signing. No other account, amount, plan or mechanism inherits this evidence.",
    });
    if c.schema_version >= 2 {
        result["execution_context"] = coherence::diagnostics(&c.observations);
    }
    let fail = |mut result: Value,
                status: PathStatus,
                reason: String|
     -> Result<VerifiedReplacementConversion> {
        result["status"] = serde_json::to_value(status)?;
        result["reason"] = reason.into();
        Ok(VerifiedReplacementConversion::new(result))
    };
    if let Some(reason) = s.unsupported {
        return fail(result, PathStatus::Unsupported, reason);
    }
    if c.observations.len() < 5 || c.observations.iter().any(|r| r.error.is_some()) {
        let final_capture_started = c.schema_version >= 2
            && c.observations
                .get(3)
                .is_some_and(|record| record.result.is_some());
        return fail(
            result,
            PathStatus::Indeterminate,
            if final_capture_started {
                format!(
                    "{}: the final account batch was unavailable within the fixed capture budget.",
                    coherence::COHERENCE_FAILURE
                )
            } else {
                "Required public state for this candidate conversion could not be captured within the bounded request budget.".into()
            },
        );
    }
    let evidence: Vec<RpcEvidence> = c
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
    if c.schema_version == 3 {
        let verified_final = (|| -> Result<coherence::ExecutionContext> {
            let replacement_program = evidence[2].result["value"]["owner"]
                .as_str()
                .context("missing replacement token program")?;
            let headers = evidence[3].result["value"]
                .as_array()
                .context("missing program headers")?;
            let programdata = demo::programdata_addresses(headers)?;
            let overlay = demo::derive(
                &c.plan_sha256,
                &s.context.owner,
                &c.plan.replacement_mint,
                replacement_program,
            )?;
            let addresses = demo::address_plan(
                &c.plan,
                &overlay,
                &s.context.owner,
                &s.context.source_program,
                replacement_program,
                &programdata,
            );
            coherence::verify(
                &evidence,
                &addresses,
                crate::probe::meteora_dlmm::slot(&evidence[3].result)?,
                &c.plan.source_account,
            )
        })();
        match verified_final {
            Ok(context) => result["execution_context"] = serde_json::to_value(context)?,
            Err(error) => {
                return fail(
                    result,
                    PathStatus::Indeterminate,
                    format!("{}: {error:#}", coherence::COHERENCE_FAILURE),
                )
            }
        }
    }
    // The selected account must not have moved between discovery and this check.
    let final_batch = evidence.last().context("missing final account batch")?;
    let index = final_batch.params[0]
        .as_array()
        .context("missing conversion account plan")?
        .iter()
        .position(|a| a == &Value::String(c.plan.source_account.clone()))
        .context("source account missing from the conversion plan")?;
    let final_source = &final_batch.result["value"][index];
    let mut execution_amount = s.amount;
    if c.schema_version == 3 {
        if final_source.is_null() {
            return fail(
                result,
                PathStatus::Indeterminate,
                "IdentityChanged: selected source account is absent at final capture.".into(),
            );
        }
        let final_state = match decode::decode_token_account(
            final_source,
            &s.context.source_program,
            &c.plan.source_mint,
            s.context.source_decimals,
        ) {
            Ok(state) => state,
            Err(error) => {
                return fail(
                    result,
                    PathStatus::Indeterminate,
                    format!(
                    "IdentityChanged: final selected source token account is invalid: {error:#}"
                ),
                )
            }
        };
        if final_state.owner != s.context.owner
            || !final_state.is_initialized
            || final_state.is_frozen
            || final_state
                .extensions
                .iter()
                .any(|extension| extension.extension_type.contains("Confidential"))
        {
            return fail(
                result,
                PathStatus::Indeterminate,
                "NoLongerExecutable: final source authority or supported token state changed."
                    .into(),
            );
        }
        let final_balance: u64 = final_state.raw_balance.parse()?;
        if c.plan.amount_mode == AmountMode::Full {
            execution_amount = final_balance;
        }
        if execution_amount == 0 || execution_amount > final_balance {
            return fail(
                result,
                PathStatus::Indeterminate,
                "NoLongerExecutable: the exact requested amount is unavailable at final capture."
                    .into(),
            );
        }
        result["discovery_amount_raw"] = s.amount.to_string().into();
        result["amount_raw"] = execution_amount.to_string().into();
        result["amount_decimal"] = decode::decimal_amount(execution_amount, s.decimals).into();
        result["source_revalidated"] = json!({
            "unchanged_since_discovery": final_source == &s.source_raw,
            "discovery_data_sha256": sha256(&decode::raw_account_bytes(&s.source_raw)?),
            "final_data_sha256": sha256(&decode::raw_account_bytes(final_source)?),
            "discovery_amount_raw": s.amount.to_string(),
            "final_amount_raw": final_balance.to_string(),
            "execution_amount_raw": execution_amount.to_string(),
            "discovery_lamports": s.source_raw["lamports"],
            "final_lamports": final_source["lamports"],
            "execution_fixture_is_authoritative": true,
        });
    } else {
        if ["data", "owner", "executable", "lamports"]
            .iter()
            .any(|field| final_source[field] != s.source_raw[field])
        {
            return fail(result, PathStatus::Indeterminate, "SourceStateChanged: the selected account changed while this candidate conversion was being prepared. Refresh current state and reconfirm the plan.".into());
        }
        result["source_revalidated"] =
            json!({"unchanged_since_discovery": true, "execution_fixture_is_authoritative": true});
    }
    let execution_context = demo::ConversionContext {
        genesis_hash: s.context.genesis_hash.clone(),
        minimum_slot: s.context.minimum_slot,
        owner: s.context.owner.clone(),
        source_program: s.context.source_program.clone(),
        source_decimals: s.context.source_decimals,
        amount: execution_amount,
    };
    let built = match if c.schema_version == 3 {
        demo::build_coherent_rebound(
            &c.plan,
            &c.plan_sha256,
            &execution_context,
            &evidence,
            program,
        )
    } else if c.schema_version == 2 {
        demo::build_coherent(
            &c.plan,
            &c.plan_sha256,
            &execution_context,
            &evidence,
            program,
        )
    } else {
        demo::build(
            &c.plan,
            &c.plan_sha256,
            &execution_context,
            &evidence,
            program,
        )
    } {
        Ok(built) => built,
        Err(error) => {
            let text = format!("{error:#}");
            return fail(
                result,
                if text.contains("Unsupported: ") {
                    PathStatus::Unsupported
                } else {
                    PathStatus::Indeterminate
                },
                format!("Candidate conversion precondition could not be established: {text}"),
            );
        }
    };
    let p = &built.plan;
    if c.schema_version == 3 {
        let final_values = final_batch.result["value"]
            .as_array()
            .context("missing final bank")?;
        let addresses = final_batch.params[0]
            .as_array()
            .context("missing final addresses")?;
        let final_mint = |address: &str| -> Result<&Value> {
            let index = addresses
                .iter()
                .position(|value| value == address)
                .context("mint omitted from final bank")?;
            Ok(&final_values[index])
        };
        result["mint_revalidated"] = json!({
            "source_discovery_data_sha256": sha256(&decode::raw_account_bytes(&evidence[1].result["value"])?),
            "source_final_data_sha256": sha256(&decode::raw_account_bytes(final_mint(&c.plan.source_mint)?)?),
            "replacement_discovery_data_sha256": sha256(&decode::raw_account_bytes(&evidence[2].result["value"])?),
            "replacement_final_data_sha256": sha256(&decode::raw_account_bytes(final_mint(&c.plan.replacement_mint)?)?),
            "final_bank_is_authoritative": true,
        });
    }
    if c.schema_version >= 2 {
        result["execution_context"] = serde_json::to_value(&built.execution_context)?;
    }
    result["destination"] = built.overlay.destination.clone().into();
    result["proposed_overlay"] = json!({
        "origin": "Proposed",
        "addresses": built.overlay,
        "reserve_funded_replacement_raw": c.plan.reserve.funded_replacement_raw,
        "note": "Deterministically derived from the candidate program, the plan digest and the observed replacement identity. None of it is observed mainnet state and none of it is issuer controlled."
    });
    result["observed_state"] = json!({
        "origin": "Observed",
        "source_balance_before_raw": built.source_before_raw,
        "source_decimals": built.source_decimals,
        "replacement_decimals": built.replacement_decimals,
        "replacement_token_program": built.replacement_program,
        "replacement_mint_inspection": "Independently captured in this check. Existence and layout only; no issuer relationship, ratio or economic equivalence is established.",
        "holder_replacement_account_existed": built.destination_existed,
    });
    result["fixture_accounts"] = serde_json::to_value(&built.accounts)?;
    result["expected"] = serde_json::to_value(&built.expectation)?;
    result["clock"] = serde_json::to_value(ProbeClock::from(&p.clock))?;
    result["message"] = serde_json::to_value(ProbeMessage::from(&p.message))?;
    result["account_evidence"] = serde_json::to_value(&p.account_evidence)?;
    result["account_plan"] = final_batch.params[0].clone();
    result["deployed_programs"] = json!(p
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
    result["assumptions"] = serde_json::to_value(&p.assumptions)?;
    result["preconditions"] = serde_json::to_value(&p.preconditions)?;
    result["runtime_profile"] = json!({"backend":"LiteSVM 0.16","features":"pinned mainnet/default profile","signature_verification":false,"recent_blockhash_verification":false,"synthetic_fee_payer":crate::probe::meteora_dlmm::payer().to_string(),"validator_bank_reproduction":false,"network_access":false});
    result["execution_fixture_sha256"] = digest(&json!({
        "plan_sha256": c.plan_sha256, "candidate_program_sha256": built.candidate_program_sha256,
        "accounts": built.accounts, "clock": ProbeClock::from(&p.clock),
        "message": ProbeMessage::from(&p.message), "account_plan": final_batch.params[0],
    }))?
    .into();
    demo::assert_candidate_program_identity(&p.programs, program, program_hash)?;
    eprintln!("CURRENT_STAGE:Running candidate conversion locally");
    let execution = executor::execute_probe_message(
        &p.accounts,
        &p.watch,
        p.clock.clone(),
        &p.programs,
        p.message.clone(),
    )?;
    eprintln!("CURRENT_STAGE:Reconciling conversion");
    result["execution_performed"] = true.into();
    result["local_execution_performed"] = true.into();
    result["signer_assumed_locally"] = true.into();
    result["execution"] = serde_json::to_value(&execution)?;
    let deltas = match demo::reconcile(&c.plan, &built, execution_amount, &execution) {
        Ok(deltas) => deltas,
        Err(error) => {
            return fail(
                result,
                PathStatus::Indeterminate,
                format!("The candidate instruction ran, but exact reconciliation could not be established: {error:#}"),
            )
        }
    };
    result["status"] = serde_json::to_value(if deltas.reconciled {
        if execution.success {
            PathStatus::Proven
        } else {
            PathStatus::Failed
        }
    } else {
        PathStatus::Indeterminate
    })?;
    if !deltas.reconciled {
        result["reason"] = "The candidate instruction ran, but exact source, supply, ratio, rounding, fee or replacement deltas did not reconcile. No conversion proof was granted.".into();
    } else if !execution.success {
        result["reason"] = format!(
            "The candidate conversion failed in local simulation: {}. No mainnet funds moved and the watched accounts rolled back.",
            execution.error.clone().unwrap_or_else(|| "instruction error".into())
        )
        .into();
    }
    result["reconciliation"] = serde_json::to_value(deltas)?;
    result["limitations"] = json!([
        "This proves the supplied candidate plan against this exact captured state. It does not establish an issuer-defined official transition.",
        "The candidate reserve, configuration and authority are proposed rollout state assumed locally; the candidate authority cannot establish issuer identity or control.",
        "Holder signing is assumed locally; key possession and authorization remain unknown.",
        "No mainnet transaction is constructed, signed or submitted, and no funds moved.",
    ]);
    eprintln!("CURRENT_STAGE:Preparing evidence");
    Ok(VerifiedReplacementConversion::new(result))
}
