//! Shared deterministic candidate-conversion fixtures, assembled from captured
//! mainnet bytes. No network access and no new evidence files.
#![allow(dead_code)]
use eplyx_lifecycle_impact::{
    conversion::{current as conversion, demo, *},
    lifecycle::{current as wallet, exposure::sha256},
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::OnceLock};

/// Token-2022 replacement (another PreStocks-style mint) and a legacy SPL replacement.
pub const OPENAI_MINT: &str = "PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF";
pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

pub struct Corpus {
    pub wallet: String,
    pub wallet_sha256: String,
    pub genesis: String,
    pub source_mint_result: Value,
    pub raw: BTreeMap<String, Value>,
    pub clock_slot: u64,
    pub discovery_slot: u64,
    pub source_account: String,
    pub owner: String,
    pub source_mint: String,
    pub source_decimals: u8,
}
pub fn corpus() -> &'static Corpus {
    static C: OnceLock<Corpus> = OnceLock::new();
    C.get_or_init(|| build_corpus("live-transfer.capture.json"))
}
/// A second freshly observed source asset through the same generic adapter.
pub fn second_asset() -> &'static Corpus {
    static C: OnceLock<Corpus> = OnceLock::new();
    C.get_or_init(|| build_corpus("live-second-transfer.capture.json"))
}
pub fn build_corpus(check: &str) -> Corpus {
    {
        let root = eplyx_lifecycle_impact::repo_root().join("reports/milestone4-validation");
        let load = |name: &str| -> Value {
            serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap()
        };
        let transfer = load(check);
        let market = load("live-market.capture.json");
        let openai = load("openai.capture.json");
        let wallet_capture = transfer["wallet_capture"].as_str().unwrap().to_string();
        let parsed: wallet::Capture = serde_json::from_str(&wallet_capture).unwrap();
        let observed = wallet::evaluate(&parsed).unwrap();
        let source_account = transfer["request"]["source"].as_str().unwrap().to_string();
        let mut raw = BTreeMap::new();
        let mut absorb = |observation: &Value| {
            let addresses = observation["params"][0].as_array().unwrap();
            let values = observation["result"]["value"].as_array().unwrap();
            for (address, value) in addresses.iter().zip(values) {
                if !value.is_null() {
                    raw.entry(address.as_str().unwrap().to_string())
                        .or_insert_with(|| value.clone());
                }
            }
        };
        absorb(&transfer["observations"][3]);
        absorb(&market["observations"][6]);
        raw.insert(
            OPENAI_MINT.into(),
            openai["observations"][1]["result"]["value"].clone(),
        );
        let row = observed["wallet_observation"]["token_accounts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["address"] == source_account)
            .unwrap()
            .clone();
        Corpus {
            genesis: observed["acquisition"]["genesis_hash"]
                .as_str()
                .unwrap()
                .into(),
            source_mint_result: transfer["observations"][1]["result"].clone(),
            clock_slot: transfer["observations"][3]["result"]["context"]["slot"]
                .as_u64()
                .unwrap(),
            discovery_slot: row["slot"].as_u64().unwrap(),
            owner: parsed
                .selection
                .as_ref()
                .unwrap()
                .public_owner
                .clone()
                .unwrap(),
            source_mint: parsed.asset.mint.clone(),
            wallet_sha256: sha256(wallet_capture.as_bytes()),
            wallet: wallet_capture,
            source_decimals: serde_json::from_value::<
                eplyx_lifecycle_impact::lifecycle::decode::MintConfig,
            >(observed["mint"].clone())
            .unwrap()
            .decimals,
            source_account,
            raw,
        }
    }
}
pub fn program() -> Vec<u8> {
    demo::program_bytes().expect("run ./scripts/build-programs.sh")
}
pub fn plan(replacement: &str) -> ConversionPlan {
    plan_for(corpus(), replacement)
}
pub fn plan_for(c: &Corpus, replacement: &str) -> ConversionPlan {
    ConversionPlan {
        schema_version: 1,
        id: "operator-candidate-1".into(),
        version: 1,
        provenance: PlanProvenance::OperatorSupplied,
        mechanism: MechanismId::EplyxDemoCandidateConversion,
        adapter_id: eplyx_lifecycle_impact::conversion::ADAPTER_ID.into(),
        mechanism_ref: "Eplyx Demo Candidate Conversion".into(),
        source_mint: c.source_mint.clone(),
        replacement_mint: replacement.into(),
        source_account: c.source_account.clone(),
        amount_mode: AmountMode::Custom,
        amount_decimal: Some("0.000001".into()),
        terms: ConversionTerms {
            ratio_numerator: 1,
            ratio_denominator: 2,
            rounding: Rounding::Floor,
            conversion_fee_bps: 0,
        },
        authority_model: AuthorityModel {
            holder_signs: true,
            candidate_authority: CandidateAuthority::ProgramDerived,
        },
        source_consumption: SourceConsumption::Burn,
        replacement_delivery: ReplacementDelivery::ProposedReserveRelease,
        reserve: ReserveConfig {
            funded_replacement_raw: "1000000000000".into(),
        },
        effective_at: None,
        deadline: None,
    }
}
pub fn config(slot: u64) -> Value {
    json!({"encoding":"base64","commitment":"finalized","minContextSlot":slot})
}
pub fn record(index: u32, method: &str, params: Value, result: Value) -> Value {
    json!({"method":method,"params":params,
        "started_at":format!("2026-09-21T00:00:{:02}Z", index * 2),
        "completed_at":format!("2026-09-21T00:00:{:02}Z", index * 2 + 1),
        "result":result,"error":Value::Null})
}
/// Assemble a deterministic candidate-conversion capture from captured mainnet bytes.
pub fn capture(plan: &ConversionPlan, run: &str, check: &str) -> conversion::Capture {
    capture_for(corpus(), plan, run, check)
}
pub fn capture_for(
    c: &Corpus,
    plan: &ConversionPlan,
    run: &str,
    check: &str,
) -> conversion::Capture {
    let digest = plan.sha256().unwrap();
    let replacement_raw = c.raw.get(&plan.replacement_mint).unwrap().clone();
    let replacement_program = replacement_raw["owner"].as_str().unwrap().to_string();
    let source_program = c.raw[&c.source_mint]["owner"].as_str().unwrap().to_string();
    let mut programs = vec![
        source_program.clone(),
        replacement_program.clone(),
        "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".to_string(),
    ];
    programs.sort();
    programs.dedup();
    let headers: Vec<Value> = programs
        .iter()
        .map(|p| c.raw.get(p).cloned().unwrap())
        .collect();
    let programdata = demo::programdata_addresses(&headers).unwrap();
    let overlay = demo::derive(
        &digest,
        &c.owner,
        &plan.replacement_mint,
        &replacement_program,
    )
    .unwrap();
    let addresses = demo::address_plan(
        plan,
        &overlay,
        &c.owner,
        &source_program,
        &replacement_program,
        &programdata,
    );
    let values: Vec<Value> = addresses
        .iter()
        .map(|a| c.raw.get(a).cloned().unwrap_or(Value::Null))
        .collect();
    let source_slot = c.source_mint_result["context"]["slot"].as_u64().unwrap();
    let observations = vec![
        record(0, "getGenesisHash", json!([]), json!(c.genesis)),
        record(
            1,
            "getAccountInfo",
            json!([plan.source_mint, config(c.discovery_slot)]),
            c.source_mint_result.clone(),
        ),
        record(
            2,
            "getAccountInfo",
            json!([plan.replacement_mint, config(source_slot)]),
            json!({"context":{"slot":source_slot},"value":replacement_raw}),
        ),
        record(
            3,
            "getMultipleAccounts",
            json!([programs, config(source_slot)]),
            json!({"context":{"slot":c.clock_slot},"value":headers}),
        ),
        record(
            4,
            "getMultipleAccounts",
            json!([addresses, config(c.clock_slot)]),
            json!({"context":{"slot":c.clock_slot},"value":values}),
        ),
    ];
    conversion::Capture {
        schema_version: 1,
        run_id: run.into(),
        check_id: check.into(),
        wallet_capture: c.wallet.clone(),
        wallet_capture_sha256: c.wallet_sha256.clone(),
        plan: plan.clone(),
        plan_sha256: digest,
        started_at: "2026-09-21T00:00:00Z".into(),
        completed_at: "2026-09-21T00:01:00Z".into(),
        rpc_origin: "https://api.mainnet-beta.solana.com".into(),
        observations: serde_json::from_value(serde_json::to_value(observations).unwrap()).unwrap(),
    }
}
/// Rebuild and execute one candidate conversion, exposing the pieces reconciliation
/// consumes so that tampering with an actual result can be tested directly.
pub fn build_execution(
    plan: &ConversionPlan,
) -> (
    demo::BuiltConversion,
    eplyx_lifecycle_impact::executor::ProbeTransactionExecution,
    u64,
) {
    let c = corpus();
    let capture = capture_for(c, plan, "reconciliation-run", "reconciliation-check");
    let evidence: Vec<eplyx_lifecycle_impact::lifecycle::RpcEvidence> = capture
        .observations
        .iter()
        .enumerate()
        .map(|(id, r)| eplyx_lifecycle_impact::lifecycle::RpcEvidence {
            id,
            method: r.method.clone(),
            params: r.params.clone(),
            result: r.result.clone().unwrap(),
        })
        .collect();
    let amount = 1000;
    let context = demo::ConversionContext {
        genesis_hash: c.genesis.clone(),
        minimum_slot: c.discovery_slot,
        owner: c.owner.clone(),
        source_program: c.raw[&c.source_mint]["owner"].as_str().unwrap().into(),
        source_decimals: c.source_decimals,
        amount,
    };
    let program = program();
    let built = demo::build(plan, &capture.plan_sha256, &context, &evidence, &program).unwrap();
    let execution = eplyx_lifecycle_impact::executor::execute_probe_message(
        &built.plan.accounts,
        &built.plan.watch,
        built.plan.clock.clone(),
        &built.plan.programs,
        built.plan.message.clone(),
    )
    .unwrap();
    (built, execution, amount)
}
pub const RUN: &str = "current-conversion-run";
pub const CHECK: &str = "candidate-check-1";
pub fn run_capture(capture: &conversion::Capture) -> anyhow::Result<Value> {
    let bytes = serde_json::to_vec(capture)?;
    let program = program();
    let verified = conversion::replay(
        &bytes,
        &capture.run_id,
        &capture.check_id,
        &capture.wallet_capture_sha256,
        &sha256(&bytes),
        &capture.plan_sha256,
        &program,
        &sha256(&program),
    )?;
    Ok(verified.value().clone())
}
pub fn proven(replacement: &str) -> Value {
    let result = run_capture(&capture(&plan(replacement), RUN, CHECK)).unwrap();
    assert_eq!(
        result["status"], "Proven",
        "candidate conversion must execute and reconcile: {}",
        result["reason"]
    );
    result
}
pub fn base64_decode(text: &str) -> Vec<u8> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD
        .decode(text)
        .unwrap()
}
pub fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
