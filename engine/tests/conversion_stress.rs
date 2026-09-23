//! Bounded production-state conversion stress testing, executed for real.
//!
//! Each selected case runs the registered candidate program in the existing VM
//! against its own captured bank. Sampled evidence never becomes state-shape or
//! population proof, and stress readiness never becomes population readiness.
#[path = "common/stress.rs"]
mod harness;
use base64::{engine::general_purpose::STANDARD, Engine};
use eplyx_lifecycle_impact::{
    expansion::Eligibility,
    lifecycle::exposure::sha256,
    stress::{execute, population, select},
};
use harness::candidate::{OPENAI_MINT, USDC_MINT};
use harness::*;
use serde_json::{json, Value};
use solana_program_pack::Pack;
use spl_token_2022_interface::state::{Account, AccountState};

fn standard() -> Value {
    let p = Population::standard();
    let prepared = prepare(&p, OPENAI_MINT);
    run(&prepared).unwrap()
}

/// A new-schema capture deliberately arrives in reverse RPC row order. Its
/// selector remains address ordered, while every evidence pointer must still
/// target the original contextual response's account object.
fn prepare_rebound() -> Prepared {
    prepare_rebound_with_reserve("1000000000000")
}

fn prepare_rebound_with_reserve(reserve: &str) -> Prepared {
    let p = Population::standard();
    let program = candidate::program();
    let program_sha256 = sha256(&program);
    let mut capture = p.capture();
    capture.schema_version = 2;
    capture.decoder = population::DECODER_V2.into();
    capture.observations[2].result.as_mut().unwrap()["value"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let population_bytes = serde_json::to_vec(&capture).unwrap();
    let population_sha256 = sha256(&population_bytes);
    let observation = population::evaluate_bytes(&population_bytes, &p.budget).unwrap();
    let mut candidate = candidate_plan(&p.corpus.source_mint, OPENAI_MINT);
    candidate.reserve.funded_replacement_raw = reserve.into();
    let plan = select::build(&observation, &candidate, &program_sha256, FROZEN_AT).unwrap();
    assert_eq!(plan.schema_version, 2);
    let plan_bytes = eplyx_lifecycle_impact::expansion::canonical(&plan)
        .unwrap()
        .into_bytes();
    let plan_sha256 = plan.sha256().unwrap();
    let mut cases = bundle(&p, &plan, &plan_sha256);
    cases.schema_version = 3;
    let bundle_bytes = serde_json::to_vec(&cases).unwrap();
    Prepared {
        population_bytes,
        population_sha256,
        plan,
        plan_bytes,
        plan_sha256,
        bundle_sha256: sha256(&bundle_bytes),
        bundle_bytes,
        program,
        program_sha256,
        budget: p.budget,
    }
}

#[test]
fn final_bucket_drift_is_disclosed_without_rewriting_discovery_selection() {
    let mut prepared = prepare_rebound();
    let case_index = prepared
        .plan
        .selected
        .iter()
        .position(|case| case.balance_bucket != 0)
        .unwrap();
    let selected_bucket = prepared.plan.selected[case_index].balance_bucket;
    alter_final_source_at(&mut prepared, case_index, |raw| {
        alter_token_base(raw, |account| account.amount = 2)
    });
    let result = run(&prepared).unwrap();
    let case = &result["results"][case_index];
    assert_eq!(
        case["status"], "Proven",
        "bucket_drift_does_not_erase_exact_final_execution"
    );
    assert_eq!(
        case["balance_bucket"], selected_bucket,
        "discovery_bucket_stays_frozen"
    );
    assert_eq!(
        case["detail"]["revalidation"]["final_bucket_at_discovery_thresholds"],
        0
    );
    assert_eq!(
        case["detail"]["revalidation"]["selection_bucket_preserved"], false,
        "bucket_drift_cannot_silently_satisfy_original_coverage"
    );
    assert_eq!(case["detail"]["execution_plan"]["final_amount_raw"], "2");
}

#[test]
fn executable_shape_drift_proves_only_final_state_and_qualifies_shape_coverage() {
    let mut prepared = prepare_rebound();
    alter_final_source(&mut prepared, |raw| {
        alter_token_base(raw, |account| {
            account.delegate = Some(solana_address::Address::new_from_array([77; 32])).into();
            account.delegated_amount = 0;
        })
    });
    let result = run(&prepared).unwrap();
    let case = &result["results"][0];
    assert_eq!(
        case["status"], "Proven",
        "execution_safe_shape_drift_can_rebind"
    );
    assert_eq!(
        case["detail"]["revalidation"]["classification"],
        "SelectionStateChangedButExecutable"
    );
    assert_eq!(
        case["detail"]["revalidation"]["selection_shape_preserved"],
        false
    );
    let discovery_shape = case["state_shape_sha256"].as_str().unwrap();
    let coverage = result["shape_coverage"]
        .as_array()
        .unwrap()
        .iter()
        .find(|shape| shape["state_shape_sha256"] == discovery_shape)
        .unwrap();
    assert!(
        !coverage["executed_entity_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == &case["entity_id"]),
        "drifted_final_shape_cannot_cover_discovery_shape"
    );
}

#[test]
fn underfunded_final_amount_fails_in_vm_with_exact_rollback() {
    let mut prepared = prepare_rebound_with_reserve("1");
    alter_final_source(&mut prepared, |raw| {
        alter_token_base(raw, |account| account.amount -= 1)
    });
    let result = run(&prepared).unwrap();
    let case = &result["results"][0];
    assert_eq!(
        case["status"], "Failed",
        "underfunded_reserve_must_genuinely_fail"
    );
    assert_eq!(case["execution_performed"], true);
    assert_eq!(
        case["detail"]["rollback_verified"], true,
        "failed_vm_execution_must_roll_back"
    );
    assert_eq!(
        case["detail"]["execution_plan"]["final_amount_raw"],
        case["detail"]["revalidation"]["final_amount_raw"]
    );
}

fn alter_final_source(prepared: &mut Prepared, modify: impl FnOnce(&mut Value)) {
    alter_final_source_at(prepared, 0, modify);
}

fn alter_final_source_at(
    prepared: &mut Prepared,
    case_index: usize,
    modify: impl FnOnce(&mut Value),
) {
    let mut bundle: execute::CaptureBundle =
        serde_json::from_slice(&prepared.bundle_bytes).unwrap();
    let selected = bundle.cases[case_index].token_account.clone();
    let final_record = bundle.cases[case_index].observations.last_mut().unwrap();
    let index = final_record.params[0]
        .as_array()
        .unwrap()
        .iter()
        .position(|address| address == &selected)
        .unwrap();
    modify(&mut final_record.result.as_mut().unwrap()["value"][index]);
    prepared.bundle_bytes = serde_json::to_vec(&bundle).unwrap();
    prepared.bundle_sha256 = sha256(&prepared.bundle_bytes);
}

fn alter_token_base(raw: &mut Value, modify: impl FnOnce(&mut Account)) {
    let mut bytes = STANDARD.decode(raw["data"][0].as_str().unwrap()).unwrap();
    let mut account = Account::unpack_unchecked(&bytes[..Account::LEN]).unwrap();
    modify(&mut account);
    account.pack_into_slice(&mut bytes[..Account::LEN]);
    raw["data"][0] = STANDARD.encode(bytes).into();
}

#[test]
fn changed_final_amount_after_capture_digest_freeze_is_rejected_before_execution() {
    let mut prepared = prepare_rebound();
    let original_digest = prepared.bundle_sha256.clone();
    alter_final_source(&mut prepared, |raw| {
        alter_token_base(raw, |account| account.amount -= 1)
    });
    prepared.bundle_sha256 = original_digest;
    let error = run(&prepared).unwrap_err();
    assert!(
        error.to_string().contains("stress input digest mismatch"),
        "final_amount_change_after_freeze_rejected"
    );
}

#[test]
fn peer_capture_cannot_replace_a_frozen_selected_identity() {
    let mut prepared = prepare_rebound();
    let mut bundle: execute::CaptureBundle =
        serde_json::from_slice(&prepared.bundle_bytes).unwrap();
    bundle.cases[0].token_account = bundle.cases[1].token_account.clone();
    prepared.bundle_bytes = serde_json::to_vec(&bundle).unwrap();
    prepared.bundle_sha256 = sha256(&prepared.bundle_bytes);
    let error = match run(&prepared) {
        Ok(_) => panic!("peer_replacement_rejected"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("selected cases may not be reordered or replaced"),
        "peer_replacement_rejected"
    );
}

#[test]
fn unverified_final_context_cannot_claim_state_rebinding_or_execute() {
    let mut prepared = prepare_rebound();
    let mut bundle: execute::CaptureBundle =
        serde_json::from_slice(&prepared.bundle_bytes).unwrap();
    let first = &mut bundle.cases[0].observations[4];
    let clock_index = first.params[0]
        .as_array()
        .unwrap()
        .iter()
        .position(|address| address == "SysvarC1ock11111111111111111111111111111111")
        .unwrap();
    let raw = &mut first.result.as_mut().unwrap()["value"][clock_index];
    let mut bytes = STANDARD.decode(raw["data"][0].as_str().unwrap()).unwrap();
    bytes[..8].copy_from_slice(&999_999u64.to_le_bytes());
    raw["data"][0] = STANDARD.encode(bytes).into();
    prepared.bundle_bytes = serde_json::to_vec(&bundle).unwrap();
    prepared.bundle_sha256 = sha256(&prepared.bundle_bytes);
    let result = run(&prepared).unwrap();
    let case = &result["results"][0];
    assert_eq!(case["status"], "Indeterminate");
    assert_eq!(case["execution_performed"], false);
    assert!(
        case["reason"]
            .as_str()
            .unwrap()
            .contains("CouldNotEstablishCoherentExecutionContext"),
        "coherence_precedes_rebinding"
    );
    assert!(case["detail"].get("revalidation").is_none());
}

#[test]
fn new_population_pointer_targets_the_selected_raw_account_after_provider_reordering() {
    let prepared = prepare_rebound();
    let capture: population::Capture = serde_json::from_slice(&prepared.population_bytes).unwrap();
    let observation =
        population::evaluate_bytes(&prepared.population_bytes, &prepared.budget).unwrap();
    for entity in observation
        .entities
        .iter()
        .filter(|e| e.state.raw_balance != "0")
    {
        let pointer = &entity.token_account_evidence.pointer;
        let row = capture.observations[2]
            .result
            .as_ref()
            .unwrap()
            .pointer(pointer.strip_suffix("/account").unwrap())
            .unwrap();
        assert_eq!(
            row["pubkey"], entity.token_account,
            "provider_order_cannot_repoint_selection_evidence"
        );
        assert_eq!(
            &row["account"],
            capture.observations[2]
                .result
                .as_ref()
                .unwrap()
                .pointer(pointer)
                .unwrap()
        );
    }
    let result = run(&prepared).unwrap();
    for case in result["results"].as_array().unwrap() {
        assert_eq!(case["status"], "Proven", "unchanged_final_state_executes");
        assert_eq!(case["detail"]["revalidation"]["changed_fields"], json!([]));
        assert!(case["detail"]["execution_plan_sha256"].as_str().is_some());
    }
}

#[test]
fn lamports_only_change_uses_final_bank_and_does_not_invalidate_conversion() {
    let mut prepared = prepare_rebound();
    alter_final_source(&mut prepared, |raw| {
        raw["lamports"] = json!(raw["lamports"].as_u64().unwrap() + 1)
    });
    let result = run(&prepared).unwrap();
    let case = &result["results"][0];
    assert_eq!(
        case["status"], "Proven",
        "irrelevant_discovery_lamports_must_not_block_final_bank"
    );
    assert_eq!(
        case["detail"]["revalidation"]["changed_fields"],
        json!(["lamports"])
    );
    assert_eq!(
        case["detail"]["revalidation"]["selection_shape_preserved"],
        true
    );
}

#[test]
fn full_at_final_capture_resolves_drifted_amount_before_vm_without_replacing_identity() {
    let mut prepared = prepare_rebound();
    let selected = prepared.plan.selected[0].token_account.clone();
    let discovery_amount: u64 = prepared.plan.selected[0]
        .selected_amount_raw
        .parse()
        .unwrap();
    alter_final_source(&mut prepared, |raw| {
        alter_token_base(raw, |account| account.amount -= 1)
    });
    let result = run(&prepared).unwrap();
    let case = &result["results"][0];
    assert_eq!(
        case["token_account"], selected,
        "drifted_selected_identity_never_replaced"
    );
    assert_eq!(
        case["status"], "Proven",
        "final_current_amount_must_execute"
    );
    assert_eq!(case["selected_amount_raw"], discovery_amount.to_string());
    assert_eq!(
        case["detail"]["execution_plan"]["final_amount_raw"],
        (discovery_amount - 1).to_string()
    );
    assert_eq!(
        case["detail"]["reconciliation"]["source_burned_raw"],
        (discovery_amount - 1).to_string()
    );
    assert_ne!(
        case["detail"]["resolved_case_plan_sha256"],
        case["case_plan_sha256"]
    );
    assert_eq!(
        case["detail"]["execution_plan"]["final_source_data_sha256"],
        case["detail"]["revalidation"]["final_data_sha256"],
        "execution_plan_uses_final_source_bytes"
    );
    assert_ne!(
        case["detail"]["execution_plan"]["final_source_data_sha256"],
        case["detail"]["revalidation"]["discovery_data_sha256"],
        "stale_population_bytes_cannot_supply_final_proof"
    );
    assert!(
        case["detail"]["scope"]
            .as_str()
            .unwrap()
            .contains("final coherent state"),
        "proof_scope_binds_final_not_stale_discovery_state"
    );
    assert_eq!(
        case["detail"]["execution_plan_sha256"],
        eplyx_lifecycle_impact::expansion::digest(&case["detail"]["execution_plan"]).unwrap(),
        "vm_result_cannot_rewrite_frozen_execution_plan"
    );
}

#[test]
fn zero_frozen_authority_and_token_program_drift_never_execute_a_peer() {
    for (name, mutate) in [
        ("zero", 0u8),
        ("frozen", 1),
        ("authority", 2),
        ("runtime_owner", 3),
        ("mint", 4),
        ("extension", 5),
    ] {
        let mut prepared = prepare_rebound();
        let selected = prepared.plan.selected[0].token_account.clone();
        alter_final_source(&mut prepared, |raw| match mutate {
            0 => alter_token_base(raw, |account| account.amount = 0),
            1 => alter_token_base(raw, |account| account.state = AccountState::Frozen),
            2 => alter_token_base(raw, |account| {
                account.owner = solana_address::Address::new_from_array([77; 32])
            }),
            3 => raw["owner"] = "11111111111111111111111111111111".into(),
            4 => alter_token_base(raw, |account| {
                account.mint = solana_address::Address::new_from_array([88; 32])
            }),
            _ => {
                let mut bytes = STANDARD.decode(raw["data"][0].as_str().unwrap()).unwrap();
                bytes[166] = 255;
                raw["data"][0] = STANDARD.encode(bytes).into();
            }
        });
        let result = run(&prepared).unwrap();
        let case = &result["results"][0];
        assert_eq!(
            case["token_account"], selected,
            "{name}: selected_identity_never_replaced"
        );
        assert_eq!(
            case["status"], "Indeterminate",
            "{name}: unsupported_final_state_never_executes"
        );
        assert_eq!(case["execution_performed"], false);
        if name == "authority" {
            assert_eq!(
                case["detail"]["revalidation"]["classification"], "NoLongerExecutable",
                "unsupported_authority_drift_never_executes"
            );
        }
    }
}

#[test]
fn changed_selected_source_stays_selected_and_indeterminate() {
    let mut prepared = prepare(&Population::standard(), OPENAI_MINT);
    let mut bundle: execute::CaptureBundle =
        serde_json::from_slice(&prepared.bundle_bytes).unwrap();
    bundle.schema_version = 2;
    let selected = bundle.cases[0].token_account.clone();
    let last = bundle.cases[0].observations.last_mut().unwrap();
    let index = last.params[0]
        .as_array()
        .unwrap()
        .iter()
        .position(|address| address == &selected)
        .unwrap();
    let source = &mut last.result.as_mut().unwrap()["value"][index];
    source["lamports"] = json!(source["lamports"].as_u64().unwrap() + 1);
    prepared.bundle_bytes = serde_json::to_vec(&bundle).unwrap();
    prepared.bundle_sha256 = sha256(&prepared.bundle_bytes);
    let result = run(&prepared).unwrap_or_else(|_| panic!("frozen_selected_case_never_replaced"));
    assert_eq!(
        result["results"][0]["token_account"], selected,
        "frozen_selected_case_never_replaced"
    );
    assert_eq!(result["results"][0]["status"], "Indeterminate");
    assert_eq!(result["results"][0]["execution_performed"], false);
    assert!(result["results"][0]["reason"]
        .as_str()
        .unwrap()
        .contains("SourceStateChanged"));
    assert_eq!(
        result["selected_cases"].as_array().unwrap().len(),
        bundle.cases.len()
    );
}

#[test]
fn every_selected_case_really_executes_and_reconciles_at_its_own_full_balance() {
    let v = standard();
    assert_eq!(v["kind"], "current-conversion-stress");
    let results = v["results"].as_array().unwrap();
    assert!(!results.is_empty(), "the selector must choose cases");
    let cases = v["selected_cases"].as_array().unwrap();
    assert_eq!(results.len(), cases.len());
    for (case, result) in cases.iter().zip(results) {
        assert_eq!(result["status"], "Proven", "{}", result["reason"]);
        assert_eq!(result["execution_performed"], true);
        assert_eq!(result["local_execution_performed"], true);
        assert_eq!(result["signer_assumed_locally"], true);
        assert_eq!(result["signer_possession_known"], false);
        assert_eq!(result["funds_moved"], false);
        assert_eq!(result["official_transition"], "NotTested");
        assert_eq!(result["issuer_binding_established"], false);
        // The exact frozen amount is the account's whole observed balance.
        assert_eq!(result["selected_amount_raw"], case["selected_amount_raw"]);
        assert_eq!(case["selected_amount_raw"], case["observed_balance_raw"]);
        assert_eq!(case["amount_capped"], false);
        // The candidate program actually ran and both CPIs actually happened.
        let d = &result["detail"];
        assert_eq!(d["execution"]["success"], true);
        assert!(d["execution"]["compute_units"].as_u64().unwrap() > 0);
        let r = &d["reconciliation"];
        assert_eq!(r["reconciled"], true);
        assert_eq!(r["candidate_program_invoked"], true);
        assert_eq!(r["burn_cpi_observed"], true);
        assert_eq!(r["release_cpi_observed"], true);
        // Exact per-case arithmetic: 1/2 floor, no conversion fee.
        let amount: u128 = case["selected_amount_raw"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(r["source_debited_raw"], amount.to_string());
        assert_eq!(r["source_burned_raw"], amount.to_string());
        assert_eq!(
            r["expected"]["replacement_gross_raw"],
            (amount / 2).to_string()
        );
        assert_eq!(r["replacement_released_raw"], (amount / 2).to_string());
    }
}

#[test]
fn distinct_amounts_prove_that_cases_run_independently_and_are_never_summed() {
    let v = standard();
    let amounts: Vec<String> = v["selected_cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["selected_amount_raw"].as_str().unwrap().to_string())
        .collect();
    assert!(
        amounts
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            > 1,
        "the population must offer distinct balances to exercise"
    );
    // No field anywhere sums independent conversion outputs into capacity.
    let text = serde_json::to_string(&v).unwrap();
    for forbidden in [
        "available_liquidity",
        "rollout_capacity",
        "simultaneous",
        "total_convertible",
    ] {
        assert!(
            !text.contains(forbidden),
            "{forbidden} must not be reported"
        );
    }
    // Tested balance is the sum of the exact tested entities, counted once each.
    let c = &v["coverage_summary"];
    let tested: u128 = c["tested_public_balance_raw"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let expected: u128 = amounts.iter().map(|a| a.parse::<u128>().unwrap()).sum();
    assert_eq!(
        tested, expected,
        "independent_outputs_must_not_be_summed_as_capacity"
    );
    let population: u128 = c["population_public_balance_raw"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        tested < population,
        "independent_outputs_must_not_be_summed_as_capacity"
    );
    // Proven balance is exactly the balance of the entities that were proven.
    let proven_expected: u128 = v["results"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["status"] == "Proven")
        .map(|r| {
            r["selected_amount_raw"]
                .as_str()
                .unwrap()
                .parse::<u128>()
                .unwrap()
        })
        .sum();
    let proven: u128 = c["proven_public_balance_raw"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(
        proven, proven_expected,
        "one_tested_entity_must_not_prove_its_peers"
    );
    assert!(
        c["exact_accounts_proven"].as_u64().unwrap()
            < c["positive_balance_accounts_observed"].as_u64().unwrap(),
        "the fixture must leave untested peers for this assertion to mean anything"
    );
    // A zero-balance account is observed but is never exposure and never tested.
    assert!(
        c["zero_balance_accounts_observed"].as_u64().unwrap() >= 1,
        "the population must contain a zero-balance account to test this"
    );
    assert_eq!(
        c["positive_balance_accounts_observed"].as_u64().unwrap()
            + c["zero_balance_accounts_observed"].as_u64().unwrap(),
        c["accounts_observed"].as_u64().unwrap(),
        "zero_balance_must_not_count_as_exposure"
    );
    for case in v["selected_cases"].as_array().unwrap() {
        assert_ne!(
            case["observed_balance_raw"], "0",
            "zero_balance_must_not_count_as_exposure"
        );
    }
}

#[test]
fn a_program_controlled_authority_is_surfaced_and_never_given_a_wallet_signer() {
    let v = standard();
    let unsupported = v["unsupported_summary"].as_array().unwrap();
    assert!(
        !unsupported.is_empty(),
        "unsupported_authorities_must_stay_visible"
    );
    let row = unsupported
        .iter()
        .find(|u| {
            u["shape_label"]
                .as_str()
                .unwrap()
                .contains("Program-controlled")
        })
        .expect("unsupported_authorities_must_stay_visible");
    assert_eq!(row["positive_balance_accounts"], 1);
    assert_eq!(row["represented_raw"], "777000");
    assert!(row["boundary"]
        .as_str()
        .unwrap()
        .contains("not proof that these accounts cannot convert"));
    // It is never selected and never executed.
    for case in v["selected_cases"].as_array().unwrap() {
        assert_ne!(case["authority_model"], "ProgramOwnedAuthority");
    }
    assert!(
        v["coverage_summary"]["unsupported_positive_balance_accounts"]
            .as_u64()
            .unwrap()
            >= 1
    );
}

#[test]
fn shape_coverage_never_becomes_entity_coverage() {
    let v = standard();
    let c = &v["coverage_summary"];
    let proven = c["exact_accounts_proven"].as_u64().unwrap();
    let positive = c["positive_balance_accounts_observed"].as_u64().unwrap();
    assert!(
        proven < positive,
        "a bounded run cannot prove every account"
    );
    for shape in v["shape_coverage"].as_array().unwrap() {
        let executed = shape["entities_executed"].as_u64().unwrap();
        let members = shape["entities_in_shape"].as_u64().unwrap();
        assert_eq!(
            shape["executed_entity_ids"].as_array().unwrap().len() as u64,
            executed
        );
        assert_eq!(
            shape["entities_untested"].as_u64().unwrap(),
            members - executed,
            "one_tested_shape_must_not_prove_its_members"
        );
        assert!(
            executed <= shape["entities_selected"].as_u64().unwrap(),
            "one_tested_shape_must_not_prove_its_members"
        );
        assert!(
            executed <= members,
            "one_tested_shape_must_not_prove_its_members"
        );
        assert!(shape["proof_scope"]
            .as_str()
            .unwrap()
            .contains("Every other member of this shape is untested"));
        // A shape row carries no status of its own.
        assert!(shape.get("status").is_none());
    }
    // Shape coverage and entity coverage are separate denominators.
    assert!(
        c["state_shapes_with_executed_case"].as_u64().unwrap()
            <= c["state_shapes_discovered"].as_u64().unwrap()
    );
    assert_ne!(
        c["state_shapes_with_executed_case"], c["exact_accounts_proven"],
        "the two coverage kinds must not be reported as one number"
    );
}

#[test]
fn stress_readiness_and_population_readiness_stay_separate_and_neither_is_ready() {
    let v = standard();
    assert_eq!(v["readiness"]["scope"], "ConversionStressReadiness");
    assert_eq!(v["readiness"]["population_readiness"], Value::Null);
    assert_eq!(v["readiness"]["official_transition_established"], false);
    // Unsupported production state remains, so the declared policy is not met.
    assert_eq!(v["readiness"]["status"], "Incomplete");
    assert_eq!(
        v["population_rollout_readiness"]["scope"],
        "PopulationRolloutReadiness"
    );
    assert_eq!(
        v["population_rollout_readiness"]["status"], "Incomplete",
        "population_readiness_must_not_follow_from_a_sample"
    );
    assert_eq!(
        v["population_rollout_readiness"]["rollout_readiness"]["exhaustive_execution"],
        false
    );
    assert_eq!(v["official_transition"], "NotTested");
    assert_eq!(v["authorization"], false);
    assert_eq!(v["funds_moved"], false);
    assert!(v["readiness"]["not_asset_safety"]
        .as_str()
        .unwrap()
        .contains("not a safety"));
}

#[test]
fn the_plan_is_frozen_before_execution_and_cannot_be_rewritten_afterwards() {
    let p = Population::standard();
    let mut prepared = prepare(&p, OPENAI_MINT);
    run(&prepared).unwrap();

    // Dropping a case after the fact, for instance one that failed.
    let mut edited = prepared.plan.clone();
    edited.selected.pop();
    prepared.plan_bytes = eplyx_lifecycle_impact::expansion::canonical(&edited)
        .unwrap()
        .into_bytes();
    prepared.plan_sha256 = edited.sha256().unwrap();
    let error = run(&prepared).unwrap_err().to_string();
    assert!(
        error.contains("bundle binding mismatch") || error.contains("deterministic pre-execution"),
        "a_failed_case_must_not_be_replaced_after_execution: {error}"
    );

    // Lowering a selected amount after the fact.
    let mut prepared = prepare(&p, OPENAI_MINT);
    let mut edited = prepared.plan.clone();
    edited.selected[0].selected_amount_raw = "1".into();
    prepared.plan_bytes = eplyx_lifecycle_impact::expansion::canonical(&edited)
        .unwrap()
        .into_bytes();
    prepared.plan_sha256 = edited.sha256().unwrap();
    let outcome = run(&prepared);
    assert!(
        outcome.is_err(),
        "selection_must_not_be_rewritten_after_execution"
    );
    let error = outcome.unwrap_err().to_string();
    assert!(
        error.contains("deterministic pre-execution"),
        "selection_must_not_be_rewritten_after_execution: {error}"
    );

    // Swapping in a different candidate program build.
    let mut prepared = prepare(&p, OPENAI_MINT);
    prepared.program_sha256 = "b".repeat(64);
    assert!(run(&prepared).is_err());
}

#[test]
fn a_refreshed_population_starts_untested_and_inherits_no_proof() {
    let first = Population::standard();
    let before = run(&prepare(&first, OPENAI_MINT)).unwrap();

    // A new capture of a changed world: a different population digest.
    let mut second = Population::standard();
    second.extra[0].amount = 4_500;
    let c = second.corpus;
    let mint_bytes =
        harness::candidate::base64_decode(c.raw[&c.source_mint]["data"][0].as_str().unwrap());
    let key = second.extra[0].token_account.to_string();
    let owner = second.extra[0].owner;
    second.raw.insert(
        key.clone(),
        serde_json::json!({"lamports":2_039_280u64,
            "owner":c.raw[&c.source_mint]["owner"],"executable":false,"rentEpoch":0u64,
            "space":0,"data":[harness::candidate::base64_encode(
                &eplyx_lifecycle_impact::conversion::demo::proposed_token_account(
                    &mint_bytes, &c.source_mint.parse().unwrap(), &owner, 4_500).unwrap()),"base64"]}),
    );
    // Restore the byte-accurate space field the decoder checks.
    let data = second.raw[&key]["data"][0].as_str().unwrap().to_string();
    let length = harness::candidate::base64_decode(&data).len();
    second.raw.get_mut(&key).unwrap()["space"] = serde_json::json!(length);

    let refreshed = prepare(&second, OPENAI_MINT);
    let after = run(&refreshed).unwrap();
    assert_ne!(
        before["population_capture"]["capture_sha256"],
        after["population_capture"]["capture_sha256"]
    );
    assert_ne!(
        before["selection_plan"]["stress_plan_sha256"],
        after["selection_plan"]["stress_plan_sha256"]
    );
    // Entity identity is scoped to its capture, so nothing addresses across.
    let old: std::collections::BTreeSet<&str> = before["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["entity_id"].as_str().unwrap())
        .collect();
    for r in after["results"].as_array().unwrap() {
        assert!(!old.contains(r["entity_id"].as_str().unwrap()));
    }
    // The old evidence cannot be replayed against the new world. The swapped
    // inputs are internally consistent, so only the plan-to-population binding
    // can reject them.
    let old_world = prepare(&first, OPENAI_MINT);
    let mut mixed = prepare(&second, OPENAI_MINT);
    mixed.population_bytes = old_world.population_bytes.clone();
    mixed.population_sha256 = old_world.population_sha256.clone();
    assert!(
        run(&mixed).is_err(),
        "refreshed_world_must_not_inherit_stress_proof"
    );
}

#[test]
fn offline_replay_reruns_every_selected_conversion_and_reproduces_the_result() {
    let p = Population::standard();
    let prepared = prepare(&p, OPENAI_MINT);
    let first = run(&prepared).unwrap();
    let second = run(&prepared).unwrap();
    // Everything but the evaluation timestamp is reproduced exactly.
    let strip = |mut v: Value| {
        v["readiness"]["policy"] = Value::Null;
        v["population_rollout_readiness"]["policy"] = Value::Null;
        v
    };
    assert_eq!(
        serde_json::to_string(&strip(first.clone())).unwrap().len(),
        serde_json::to_string(&strip(second.clone())).unwrap().len()
    );
    for (a, b) in first["results"]
        .as_array()
        .unwrap()
        .iter()
        .zip(second["results"].as_array().unwrap())
    {
        assert_eq!(a["result_sha256"], b["result_sha256"]);
        assert_eq!(a["execution_fixture_sha256"], b["execution_fixture_sha256"]);
        assert_eq!(a["status"], b["status"]);
    }
    assert_eq!(first["coverage_summary"], second["coverage_summary"]);
    assert_eq!(first["shape_coverage"], second["shape_coverage"]);
}

#[test]
fn a_second_replacement_asset_runs_through_the_same_generic_pipeline() {
    let p = Population::standard();
    // A legacy SPL replacement instead of a Token-2022 one.
    let v = run(&prepare(&p, USDC_MINT)).unwrap();
    for r in v["results"].as_array().unwrap() {
        assert_eq!(r["status"], "Proven", "{}", r["reason"]);
    }
    assert_eq!(v["readiness"]["scope"], "ConversionStressReadiness");
    assert_eq!(v["official_transition"], "NotTested");
}

#[test]
fn an_unavailable_enumeration_is_reported_rather_than_replaced_by_saved_data() {
    let mut p = Population::standard();
    let mut capture = p.capture();
    capture.observations.truncate(3);
    capture.observations[2].result = None;
    capture.observations[2].error =
        Some("RPC getProgramAccounts failed with code Some(-32601)".into());
    let bytes = serde_json::to_vec(&capture).unwrap();
    let observation = population::evaluate_bytes(&bytes, &p.budget).unwrap();
    assert_eq!(
        observation.enumeration.completeness,
        eplyx_lifecycle_impact::stress::EnumerationCompleteness::Unsupported
    );
    assert_eq!(observation.summary.token_accounts_observed, 0);
    // With no decoded population there is nothing to select and nothing to claim.
    p.include_real_holder = false;
    let candidate = candidate_plan(&p.corpus.source_mint, OPENAI_MINT);
    let plan = select::build(&observation, &candidate, &"a".repeat(64), FROZEN_AT).unwrap();
    assert!(plan.selected.is_empty());
    assert_eq!(plan.positive_balance_entities, 0);
}

#[test]
fn a_bounded_budget_limits_the_number_of_exact_cases() {
    let mut p = Population::standard();
    p.budget.max_selected_cases = 2;
    let prepared = prepare(&p, OPENAI_MINT);
    assert_eq!(prepared.plan.selected.len(), 2);
    let v = run(&prepared).unwrap();
    assert_eq!(v["coverage_summary"]["exact_accounts_selected"], 2);
    assert_eq!(v["selection_plan"]["budget"]["max_selected_cases"], 2);
    // The untested remainder is still reported honestly.
    assert!(
        v["coverage_summary"]["positive_balance_accounts_observed"]
            .as_u64()
            .unwrap()
            > 2
    );
}

#[test]
fn selection_covers_distinct_state_shapes_before_balance_buckets() {
    let p = Population::standard();
    let prepared = prepare(&p, OPENAI_MINT);
    let reasons: Vec<&str> = prepared
        .plan
        .selected
        .iter()
        .map(|c| match c.selection_reason {
            eplyx_lifecycle_impact::stress::SelectionReason::NewStateShape => "shape",
            eplyx_lifecycle_impact::stress::SelectionReason::NewBalanceBucket => "bucket",
            eplyx_lifecycle_impact::stress::SelectionReason::HighestRemainingBalance => "balance",
        })
        .collect();
    assert_eq!(reasons[0], "shape", "state-shape coverage runs first");
    assert!(
        reasons.contains(&"bucket"),
        "uncovered balance buckets are sought next: {reasons:?}"
    );
    // Only executable candidates are ever selected.
    let executable: Vec<&str> = prepared
        .plan
        .state_shapes
        .iter()
        .filter(|s| s.eligibility == Eligibility::ExecutableCandidate)
        .map(|s| s.state_shape_sha256.as_str())
        .collect();
    for case in &prepared.plan.selected {
        assert!(executable.contains(&case.state_shape_sha256.as_str()));
    }
}

#[test]
fn a_case_whose_state_could_not_be_captured_stays_indeterminate_not_failed() {
    let p = Population::standard();
    let mut prepared = prepare(&p, OPENAI_MINT);
    let mut bundle: execute::CaptureBundle =
        serde_json::from_slice(&prepared.bundle_bytes).unwrap();
    bundle.cases[0].observations.truncate(2);
    prepared.bundle_bytes = serde_json::to_vec(&bundle).unwrap();
    prepared.bundle_sha256 =
        eplyx_lifecycle_impact::lifecycle::exposure::sha256(&prepared.bundle_bytes);
    let v = run(&prepared).unwrap();
    let first = &v["results"][0];
    assert_eq!(first["status"], "Indeterminate");
    assert_eq!(first["execution_performed"], false);
    assert!(first["reason"]
        .as_str()
        .unwrap()
        .contains("no outcome is claimed"));
    // Missing evidence is never collapsed into failure, and the case survives.
    assert_eq!(v["failures"].as_array().unwrap().len(), 0);
    assert_eq!(
        v["results"].as_array().unwrap().len(),
        v["selected_cases"].as_array().unwrap().len()
    );
    assert_eq!(v["readiness"]["status"], "Incomplete");
}
