//! Deterministic conversion-stress fixtures over a synthesized current population.
//!
//! The captured mainnet bytes supply the real mint, the real deployed token and
//! ATA programs, the real Clock and one real holder account. Additional current
//! production states are synthesized from that same mint so a bounded stress run
//! has several distinct state shapes to discover. No network access and no new
//! evidence files.
#![allow(dead_code)]
#[path = "candidate.rs"]
pub mod candidate;

use candidate::{base64_decode, base64_encode, corpus, program, Corpus};
use eplyx_lifecycle_impact::{
    conversion::{demo, *},
    lifecycle::{current::Observation, decode, exposure::sha256},
    stress::{execute, population, select, StressBudget},
};
use serde_json::{json, Value};
use solana_address::Address;
use std::collections::BTreeMap;

pub const SYSTEM: &str = "11111111111111111111111111111111";
pub const ATA: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
pub const RUN: &str = "current-stress-run";
pub const STRESS: &str = "stress-1";

/// Deterministic address with the requested curve property. Bytes 2..32 carry
/// the seed, so different seeds can never collide.
pub fn address(seed: u8, on_curve: bool) -> Address {
    for n in 0..=u16::MAX {
        let mut bytes = [seed; 32];
        bytes[0] = (n & 0xff) as u8;
        bytes[1] = (n >> 8) as u8;
        let a = Address::new_from_array(bytes);
        if a.is_on_curve() == on_curve {
            return a;
        }
    }
    unreachable!("both curve properties occur in this range")
}
fn account(owner: &str, data: &[u8]) -> Value {
    json!({"lamports":2_039_280u64,"owner":owner,"executable":false,
        "rentEpoch":0u64,"space":data.len(),"data":[base64_encode(data),"base64"]})
}
/// A System-owned, empty, non-executable authority: wallet-compatible.
pub fn wallet_authority() -> Value {
    json!({"lamports":1_000_000u64,"owner":SYSTEM,"executable":false,
        "rentEpoch":0u64,"space":0u64,"data":["","base64"]})
}
/// A non-System runtime owner: the controlling program's signing path is not
/// implemented, so this authority must never receive an assumed wallet signer.
pub fn program_authority() -> Value {
    json!({"lamports":1_000_000u64,"owner":"BPFLoaderUpgradeab1e11111111111111111111111",
        "executable":false,"rentEpoch":0u64,"space":8u64,"data":[base64_encode(&[9u8;8]),"base64"]})
}

pub struct Shape {
    pub token_account: Address,
    pub owner: Address,
    pub amount: u64,
    pub authority: Value,
}

/// One synthesized current population over the captured source mint.
pub struct Population {
    pub corpus: &'static Corpus,
    pub budget: StressBudget,
    /// Extra current states beyond the one real captured holder account.
    pub extra: Vec<Shape>,
    pub include_real_holder: bool,
    pub raw: BTreeMap<String, Value>,
}
impl Population {
    pub fn standard() -> Self {
        let c = corpus();
        let mint_bytes = base64_decode(c.raw[&c.source_mint]["data"][0].as_str().unwrap());
        let mint: Address = c.source_mint.parse().unwrap();
        let mut extra = vec![];
        let mut raw = BTreeMap::new();
        // Three further wallet-compatible accounts spread across balances, and
        // one program-controlled authority the executor cannot exercise.
        for (index, amount) in [(0u8, 4_000u64), (1, 90_000), (2, 5_000_000)] {
            let owner = address(30 + index * 9, true);
            let token_account = address(120 + index, false);
            raw.insert(
                token_account.to_string(),
                account(
                    c.raw[&c.source_mint]["owner"].as_str().unwrap(),
                    &demo::proposed_token_account(&mint_bytes, &mint, &owner, amount).unwrap(),
                ),
            );
            raw.insert(owner.to_string(), wallet_authority());
            extra.push(Shape {
                token_account,
                owner,
                amount,
                authority: wallet_authority(),
            });
        }
        // A zero-balance account: observed, never exposure, never selected.
        let empty_owner = address(240, true);
        let empty_account = address(180, false);
        raw.insert(
            empty_account.to_string(),
            account(
                c.raw[&c.source_mint]["owner"].as_str().unwrap(),
                &demo::proposed_token_account(&mint_bytes, &mint, &empty_owner, 0).unwrap(),
            ),
        );
        raw.insert(empty_owner.to_string(), wallet_authority());
        extra.push(Shape {
            token_account: empty_account,
            owner: empty_owner,
            amount: 0,
            authority: wallet_authority(),
        });
        let program_owner = address(210, true);
        let program_account = address(150, false);
        raw.insert(
            program_account.to_string(),
            account(
                c.raw[&c.source_mint]["owner"].as_str().unwrap(),
                &demo::proposed_token_account(&mint_bytes, &mint, &program_owner, 777_000).unwrap(),
            ),
        );
        raw.insert(program_owner.to_string(), program_authority());
        extra.push(Shape {
            token_account: program_account,
            owner: program_owner,
            amount: 777_000,
            authority: program_authority(),
        });
        // Deliberately fewer cases than there are executable candidates, so the
        // population always keeps untested peers. A fixture that tests every
        // candidate cannot tell proof-inheritance faults from correct behaviour.
        let budget = StressBudget {
            max_selected_cases: 3,
            ..StressBudget::default()
        };
        Self {
            corpus: c,
            budget,
            extra,
            include_real_holder: true,
            raw,
        }
    }
    fn source_program(&self) -> String {
        self.corpus.raw[&self.corpus.source_mint]["owner"]
            .as_str()
            .unwrap()
            .to_string()
    }
    /// Every account either capture stage may ask for.
    pub fn lookup(&self, address: &str) -> Option<Value> {
        self.raw
            .get(address)
            .cloned()
            .or_else(|| self.corpus.raw.get(address).cloned())
    }
    fn rows(&self) -> Vec<(String, Value)> {
        let mut rows: Vec<(String, Value)> = self
            .extra
            .iter()
            .map(|s| {
                (
                    s.token_account.to_string(),
                    self.raw[&s.token_account.to_string()].clone(),
                )
            })
            .collect();
        if self.include_real_holder {
            rows.push((
                self.corpus.source_account.clone(),
                self.corpus.raw[&self.corpus.source_account].clone(),
            ));
        }
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows
    }
    pub fn capture(&self) -> population::Capture {
        let c = self.corpus;
        let mint_slot = c.source_mint_result["context"]["slot"].as_u64().unwrap();
        let enumeration_slot = mint_slot;
        let observe = |index: u32, method: &str, params: Value, result: Value| Observation {
            method: method.into(),
            params,
            started_at: format!("2026-09-22T00:00:{:02}.000Z", index * 2),
            completed_at: format!("2026-09-22T00:00:{:02}.000Z", index * 2 + 1),
            result: Some(result),
            error: None,
        };
        let mut observations = vec![
            observe(0, "getGenesisHash", json!([]), json!(c.genesis)),
            observe(
                1,
                "getAccountInfo",
                json!([c.source_mint, {"encoding":"base64","commitment":"finalized"}]),
                c.source_mint_result.clone(),
            ),
        ];
        let rows = self.rows();
        let value: Vec<Value> = rows
            .iter()
            .map(|(a, v)| json!({"pubkey":a,"account":v}))
            .collect();
        let scan_config = json!({"encoding":"base64","commitment":"finalized",
            "minContextSlot":mint_slot,"withContext":true,
            "filters":[{"memcmp":{"offset":0,"bytes":c.source_mint}}]});
        observations.push(observe(
            2,
            "getProgramAccounts",
            json!([self.source_program(), scan_config]),
            json!({"context":{"slot":enumeration_slot},"value":value}),
        ));
        // Authority resolution over the distinct positive-balance authorities.
        let mut wanted: Vec<String> = vec![];
        let decimals = c.source_decimals;
        for (_, raw) in &rows {
            let state =
                decode::decode_token_account(raw, &self.source_program(), &c.source_mint, decimals)
                    .unwrap();
            if state.raw_balance != "0" && !wanted.contains(&state.owner) {
                wanted.push(state.owner);
            }
        }
        wanted.sort();
        for (index, batch) in wanted.chunks(self.budget.authority_batch_size).enumerate() {
            let values: Vec<Value> = batch
                .iter()
                .map(|a| self.lookup(a).unwrap_or(Value::Null))
                .collect();
            observations.push(observe(
                3 + index as u32,
                "getMultipleAccounts",
                json!([batch, {"encoding":"base64","commitment":"finalized","minContextSlot":enumeration_slot}]),
                json!({"context":{"slot":enumeration_slot},"value":values}),
            ));
        }
        population::Capture {
            schema_version: 1,
            kind: "conversion-stress-population".into(),
            run_id: RUN.into(),
            stress_id: STRESS.into(),
            mint: c.source_mint.clone(),
            budget: self.budget.clone(),
            rpc_origin: "https://api.mainnet-beta.solana.com".into(),
            started_at: "2026-09-22T00:00:00.000Z".into(),
            completed_at: "2026-09-22T00:05:00.000Z".into(),
            decoder: population::DECODER.into(),
            observations,
        }
    }
}

pub fn candidate_plan(source_mint: &str, replacement: &str) -> ConversionPlan {
    ConversionPlan {
        schema_version: 1,
        id: "operator-candidate-stress".into(),
        version: 1,
        provenance: PlanProvenance::OperatorSupplied,
        mechanism: MechanismId::EplyxDemoCandidateConversion,
        adapter_id: eplyx_lifecycle_impact::conversion::ADAPTER_ID.into(),
        mechanism_ref: "Eplyx Demo Candidate Conversion".into(),
        source_mint: source_mint.into(),
        replacement_mint: replacement.into(),
        source_account: corpus().source_account.clone(),
        amount_mode: AmountMode::Full,
        amount_decimal: None,
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

pub const FROZEN_AT: &str = "2026-09-22T00:06:00.000Z";

pub struct Prepared {
    pub population_bytes: Vec<u8>,
    pub population_sha256: String,
    pub plan: select::StressTestPlan,
    pub plan_bytes: Vec<u8>,
    pub plan_sha256: String,
    pub bundle_bytes: Vec<u8>,
    pub bundle_sha256: String,
    pub program: Vec<u8>,
    pub program_sha256: String,
    pub budget: StressBudget,
}

/// Freeze a plan over the synthesized population and assemble the per-case
/// transcripts the offline replay will execute.
pub fn prepare(p: &Population, replacement: &str) -> Prepared {
    let program = program();
    let program_sha256 = sha256(&program);
    let capture = p.capture();
    let population_bytes = serde_json::to_vec(&capture).unwrap();
    let population_sha256 = sha256(&population_bytes);
    let observation = population::evaluate_bytes(&population_bytes, &p.budget).unwrap();
    let candidate = candidate_plan(&p.corpus.source_mint, replacement);
    let plan = select::build(&observation, &candidate, &program_sha256, FROZEN_AT).unwrap();
    // The frozen plan is stored canonically, so the file digest and the
    // structural digest are the same value.
    let plan_bytes = eplyx_lifecycle_impact::expansion::canonical(&plan)
        .unwrap()
        .into_bytes();
    let plan_sha256 = plan.sha256().unwrap();
    assert_eq!(sha256(&plan_bytes), plan_sha256);
    let bundle = bundle(p, &plan, &plan_sha256);
    let bundle_bytes = serde_json::to_vec(&bundle).unwrap();
    Prepared {
        bundle_sha256: sha256(&bundle_bytes),
        population_bytes,
        population_sha256,
        plan,
        plan_bytes,
        plan_sha256,
        bundle_bytes,
        program,
        program_sha256,
        budget: p.budget.clone(),
    }
}

/// The five bounded requests per case, in the shape the adapter validates.
pub fn bundle(
    p: &Population,
    plan: &select::StressTestPlan,
    plan_sha256: &str,
) -> execute::CaptureBundle {
    let c = p.corpus;
    let source_program = c.raw[&c.source_mint]["owner"].as_str().unwrap().to_string();
    let cases = plan
        .selected
        .iter()
        .map(|case| {
            let replacement_raw = p.lookup(&case.case_plan.replacement_mint).unwrap();
            let replacement_program = replacement_raw["owner"].as_str().unwrap().to_string();
            let mut programs = vec![
                source_program.clone(),
                replacement_program.clone(),
                ATA.to_string(),
            ];
            programs.sort();
            programs.dedup();
            let headers: Vec<Value> = programs.iter().map(|a| p.lookup(a).unwrap()).collect();
            let programdata = demo::programdata_addresses(&headers).unwrap();
            let overlay = demo::derive(
                &case.case_plan_sha256,
                &case.authority,
                &case.case_plan.replacement_mint,
                &replacement_program,
            )
            .unwrap();
            let addresses = demo::address_plan(
                &case.case_plan,
                &overlay,
                &case.authority,
                &source_program,
                &replacement_program,
                &programdata,
            );
            let values: Vec<Value> = addresses
                .iter()
                .map(|a| p.lookup(a).unwrap_or(Value::Null))
                .collect();
            let source_slot = c.source_mint_result["context"]["slot"].as_u64().unwrap();
            let observe = |index: u32, method: &str, params: Value, result: Value| Observation {
                method: method.into(),
                params,
                started_at: format!("2026-09-22T00:10:{:02}.000Z", index * 2),
                completed_at: format!("2026-09-22T00:10:{:02}.000Z", index * 2 + 1),
                result: Some(result),
                error: None,
            };
            let cfg = |slot: u64| {
                json!({"encoding":"base64","commitment":"finalized","minContextSlot":slot})
            };
            execute::CaseCapture {
                case_id: case.case_id.clone(),
                entity_id: case.entity_id.clone(),
                token_account: case.token_account.clone(),
                case_plan_sha256: case.case_plan_sha256.clone(),
                started_at: "2026-09-22T00:10:00.000Z".into(),
                completed_at: "2026-09-22T00:10:30.000Z".into(),
                observations: vec![
                    observe(0, "getGenesisHash", json!([]), json!(c.genesis)),
                    observe(
                        1,
                        "getAccountInfo",
                        json!([case.case_plan.source_mint, cfg(case.discovery_slot)]),
                        c.source_mint_result.clone(),
                    ),
                    observe(
                        2,
                        "getAccountInfo",
                        json!([case.case_plan.replacement_mint, cfg(source_slot)]),
                        json!({"context":{"slot":source_slot},"value":replacement_raw}),
                    ),
                    observe(
                        3,
                        "getMultipleAccounts",
                        json!([programs, cfg(source_slot)]),
                        json!({"context":{"slot":c.clock_slot},"value":headers}),
                    ),
                    observe(
                        4,
                        "getMultipleAccounts",
                        json!([addresses, cfg(c.clock_slot)]),
                        json!({"context":{"slot":c.clock_slot},"value":values}),
                    ),
                ],
            }
        })
        .collect();
    execute::CaptureBundle {
        schema_version: 1,
        kind: "conversion-stress-cases".into(),
        stress_id: STRESS.into(),
        run_id: RUN.into(),
        stress_plan_sha256: plan_sha256.into(),
        population_capture_sha256: plan.population_capture_sha256.clone(),
        rpc_origin: "https://api.mainnet-beta.solana.com".into(),
        started_at: "2026-09-22T00:09:00.000Z".into(),
        completed_at: "2026-09-22T00:15:00.000Z".into(),
        cases,
    }
}

pub fn run(p: &Prepared) -> anyhow::Result<Value> {
    let verified = execute::replay(
        &p.population_bytes,
        &p.plan_bytes,
        &p.bundle_bytes,
        STRESS,
        RUN,
        &p.population_sha256,
        &p.plan_sha256,
        &p.bundle_sha256,
        &p.program,
        &p.program_sha256,
        &p.budget,
        "2026-09-22T00:20:00.000Z",
    )?;
    Ok(serde_json::to_value(verified.value())?)
}
