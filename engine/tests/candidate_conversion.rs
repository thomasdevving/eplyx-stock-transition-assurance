//! Operator-supplied candidate conversion, executed against reconstructed current
//! production bytes. A proven candidate plan is never an official issuer transition.
#[path = "common/candidate.rs"]
mod harness;
use eplyx_lifecycle_impact::{
    conversion::{current as conversion, demo, *},
    lifecycle::exposure::sha256,
    probe::current as execution,
};
use harness::*;
use serde_json::{json, Value};

fn set_final_clock_slot(capture: &mut conversion::Capture, record: usize, slot: u64) {
    let addresses = capture.observations[record].params[0].as_array().unwrap();
    let index = addresses
        .iter()
        .position(|address| address == "SysvarC1ock11111111111111111111111111111111")
        .unwrap();
    let response = capture.observations[record].result.as_mut().unwrap();
    let raw = &mut response["value"][index];
    let mut bytes = base64_decode(raw["data"][0].as_str().unwrap());
    bytes[..8].copy_from_slice(&slot.to_le_bytes());
    raw["data"][0] = json!(base64_encode(&bytes));
}

#[test]
fn rebound_full_candidate_uses_final_source_amount_and_final_mint_supply() {
    let mut p = plan(OPENAI_MINT);
    p.amount_mode = AmountMode::Full;
    p.amount_decimal = None;
    let mut c = capture(&p, RUN, CHECK);
    c.schema_version = 3;
    let addresses = c.observations[4].params[0].as_array().unwrap().clone();
    let final_values = c.observations[4].result.as_mut().unwrap()["value"]
        .as_array_mut()
        .unwrap();
    let source_index = addresses
        .iter()
        .position(|address| address == &p.source_account)
        .unwrap();
    let mut source = base64_decode(final_values[source_index]["data"][0].as_str().unwrap());
    let discovery = u64::from_le_bytes(source[64..72].try_into().unwrap());
    source[64..72].copy_from_slice(&900u64.to_le_bytes());
    final_values[source_index]["data"][0] = json!(base64_encode(&source));
    let mint_index = addresses
        .iter()
        .position(|address| address == &p.replacement_mint)
        .unwrap();
    let mut replacement = base64_decode(final_values[mint_index]["data"][0].as_str().unwrap());
    let supply = u64::from_le_bytes(replacement[36..44].try_into().unwrap());
    replacement[36..44].copy_from_slice(&(supply + 1).to_le_bytes());
    final_values[mint_index]["data"][0] = json!(base64_encode(&replacement));
    let v = run_capture(&c).unwrap();
    assert_eq!(
        v["status"], "Proven",
        "final_current_state_rebinds_candidate"
    );
    assert_eq!(v["discovery_amount_raw"], discovery.to_string());
    assert_eq!(v["amount_raw"], "900");
    assert_eq!(v["reconciliation"]["source_burned_raw"], "900");
    assert_ne!(
        v["mint_revalidated"]["replacement_discovery_data_sha256"],
        v["mint_revalidated"]["replacement_final_data_sha256"]
    );
}

#[test]
fn coherent_retry_replays_the_exact_final_bank_and_not_discovery_bytes() {
    let mut capture = capture(&plan(OPENAI_MINT), RUN, CHECK);
    capture.schema_version = 2;
    let slot = capture.observations[4].result.as_ref().unwrap()["context"]["slot"]
        .as_u64()
        .unwrap();
    let mut final_record = capture.observations[4].clone();
    final_record.params[1]["minContextSlot"] = json!(slot + 3);
    final_record.result.as_mut().unwrap()["context"]["slot"] = json!(slot + 3);
    final_record.started_at = "2026-09-21T00:00:10Z".into();
    final_record.completed_at = "2026-09-21T00:00:11Z".into();
    capture.observations.push(final_record);
    set_final_clock_slot(&mut capture, 4, slot + 2);
    set_final_clock_slot(&mut capture, 5, slot + 3);
    let result = run_capture(&capture).unwrap();
    assert_eq!(result["status"], "Proven", "final_recapture_bytes_used");
    assert_eq!(result["execution_context"]["final_context_slot"], slot + 3);
    assert_eq!(
        result["execution_context"]["attempts"][1]["min_context_slot"],
        slot + 3
    );
    assert_eq!(result["execution_context"]["atomic_single_slot"], false);
    assert_eq!(result["execution_context"]["clock_slot"], slot + 3);
    assert_eq!(
        result["execution_context"]["discovery_contexts"][3]["response_context_slot"],
        capture.observations[3].result.as_ref().unwrap()["context"]["slot"]
    );
}

#[test]
fn exhausted_coherence_capture_is_indeterminate_and_never_failed_execution() {
    let mut capture = capture(&plan(OPENAI_MINT), RUN, CHECK);
    capture.schema_version = 2;
    let slot = capture.observations[4].result.as_ref().unwrap()["context"]["slot"]
        .as_u64()
        .unwrap();
    set_final_clock_slot(&mut capture, 4, slot + 2);
    let result = run_capture(&capture).unwrap();
    assert_eq!(
        result["status"], "Indeterminate",
        "stabilization_never_becomes_failed_or_proven"
    );
    assert_eq!(result["execution_performed"], false);
    assert!(result["reason"]
        .as_str()
        .unwrap()
        .contains("CouldNotEstablishCoherentExecutionContext"));
    assert_eq!(
        result["execution_context"]["coherence_status"],
        "Unverified"
    );
}

#[test]
fn proposed_candidate_config_keeps_the_exact_operator_fee() {
    let mut candidate = plan(OPENAI_MINT);
    candidate.terms.conversion_fee_bps = 37;
    let c = corpus();
    let replacement_program = c.raw[OPENAI_MINT]["owner"].as_str().unwrap();
    let overlay = demo::derive(
        &candidate.sha256().unwrap(),
        &c.owner,
        OPENAI_MINT,
        replacement_program,
    )
    .unwrap();
    let bytes = demo::config_data(&candidate, &overlay, c.source_decimals, 6).unwrap();
    assert_eq!(
        &bytes[146..148],
        &37u16.to_le_bytes(),
        "candidate_config_fee_bound"
    );
}
#[test]
fn candidate_conversion_executes_the_actual_program_and_reconciles_exactly() {
    let v = proven(OPENAI_MINT);
    assert_eq!(
        v["execution"]["success"], true,
        "proven_requires_an_actual_successful_vm_transaction"
    );
    assert_eq!(v["local_execution_performed"], true);
    assert_eq!(v["funds_moved"], false);
    assert!(
        v["execution"]["compute_units"].as_u64().unwrap() > 0,
        "real VM execution must meter compute"
    );
    assert!(
        v["execution"]["logs"].as_array().unwrap().iter().any(|l| l
            .as_str()
            .unwrap()
            .contains(&format!("Program {} invoke [1]", demo::PROGRAM_ID))),
        "the registered candidate program itself must run"
    );
    let r = &v["reconciliation"];
    assert_eq!(r["reconciled"], true);
    assert_eq!(r["candidate_program_invoked"], true);
    assert_eq!(
        r["burn_cpi_observed"], true,
        "source must actually be burned by the deployed token program"
    );
    assert_eq!(
        r["release_cpi_observed"], true,
        "replacement must actually be released"
    );
    assert_eq!(v["amount_raw"], "1000");
    assert_eq!(r["source_debited_raw"], "1000");
    assert_eq!(r["source_burned_raw"], "1000");
    assert_eq!(r["source_supply_change_raw"], "-1000");
    assert_eq!(
        r["expected"]["replacement_gross_raw"], "500",
        "exact ratio must hold"
    );
    assert_eq!(r["replacement_released_raw"], "500");
}
#[test]
fn ratio_rounding_and_conversion_fee_are_enforced_by_the_program() {
    for (numerator, denominator, rounding, bps, expected) in [
        (1u64, 3u64, Rounding::Floor, 0u16, "333"),
        (1, 3, Rounding::Ceiling, 0, "334"),
        (2, 1, Rounding::Floor, 0, "2000"),
        (1, 1, Rounding::Floor, 250, "975"),
        (1, 3, Rounding::Ceiling, 250, "325"),
    ] {
        let mut p = plan(OPENAI_MINT);
        p.terms = ConversionTerms {
            ratio_numerator: numerator,
            ratio_denominator: denominator,
            rounding,
            conversion_fee_bps: bps,
        };
        let v = run_capture(&capture(&p, RUN, CHECK)).unwrap();
        assert_eq!(
            v["status"], "Proven",
            "{numerator}/{denominator} {rounding:?} {bps}: {}",
            v["reason"]
        );
        assert_eq!(
            v["reconciliation"]["replacement_released_raw"], expected,
            "program_enforced_ratio_rounding_and_fee_must_be_exact"
        );
        assert!(v["reconciliation"]["program_reported"]
            .as_str()
            .unwrap()
            .contains(&format!("released={expected}")));
    }
}
#[test]
fn transfer_fees_stay_separate_from_the_conversion_fee() {
    let mut p = plan(OPENAI_MINT);
    p.terms.conversion_fee_bps = 100;
    p.terms.ratio_numerator = 1;
    p.terms.ratio_denominator = 1;
    let v = run_capture(&capture(&p, RUN, CHECK)).unwrap();
    assert_eq!(v["status"], "Proven", "{}", v["reason"]);
    let r = &v["reconciliation"];
    assert_eq!(r["conversion_fee_raw"], "10");
    assert_eq!(
        r["source_token_2022_transfer_fee_raw"], "0",
        "burning the source cannot incur a source transfer fee"
    );
    let released: u64 = r["replacement_released_raw"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let credited: u64 = r["replacement_public_credit_raw"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let withheld: u64 = r["replacement_token_2022_transfer_fee_raw"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(released, 990);
    assert_eq!(
        credited + withheld,
        released,
        "replacement credit plus its withheld fee must equal the release"
    );
    assert_eq!(
        r["destination"]["withheld_fee_change_raw"],
        withheld.to_string()
    );
}
#[test]
fn a_legacy_replacement_asset_uses_the_same_generic_adapter() {
    let v = proven(USDC_MINT);
    assert_eq!(
        v["observed_state"]["replacement_token_program"],
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    );
    assert_eq!(
        v["reconciliation"]["replacement_token_2022_transfer_fee_raw"],
        "0"
    );
    assert_eq!(
        v["reconciliation"]["replacement_public_credit_raw"],
        v["reconciliation"]["replacement_released_raw"]
    );
}
#[test]
fn proposed_overlay_is_never_labelled_observed() {
    let v = proven(OPENAI_MINT);
    let accounts: Vec<Value> = serde_json::from_value(v["fixture_accounts"].clone()).unwrap();
    let proposed: Vec<&Value> = accounts
        .iter()
        .filter(|a| a["origin"] == "Proposed")
        .collect();
    assert!(
        proposed.len() >= 4,
        "config, authority, reserve and candidate program are proposed"
    );
    for account in &proposed {
        assert!(
            account["rpc_record"].is_null() && account["slot"].is_null(),
            "proposed_account_must_never_carry_captured_evidence: {account}"
        );
        assert!(!account["derivation"].is_null());
    }
    let overlay = &v["proposed_overlay"]["addresses"];
    for key in ["config", "vault_authority", "reserve_vault"] {
        let address = overlay[key].as_str().unwrap();
        assert!(
            accounts
                .iter()
                .any(|a| a["address"] == address && a["origin"] == "Proposed"),
            "{key} must be proposed"
        );
    }
    for observed in accounts.iter().filter(|a| a["origin"] == "Observed") {
        assert!(
            !observed["rpc_record"].is_null(),
            "observed accounts carry their pointer"
        );
    }
    assert_eq!(v["proposed_overlay"]["origin"], "Proposed");
}
#[test]
fn the_candidate_program_is_never_described_as_deployed() {
    let v = proven(OPENAI_MINT);
    assert_eq!(v["candidate_mechanism"]["deployed_on_mainnet"], false);
    assert_eq!(v["candidate_mechanism"]["issuer_mechanism"], false);
    assert_eq!(v["candidate_mechanism"]["origin"], "Proposed");
    for program in v["deployed_programs"].as_array().unwrap() {
        let candidate = program["program"] == demo::PROGRAM_ID;
        assert_eq!(program["deployed_on_mainnet"], !candidate);
        assert_eq!(
            program["origin"],
            if candidate { "Proposed" } else { "Observed" }
        );
    }
}
#[test]
fn a_proven_candidate_plan_never_establishes_an_official_transition() {
    let v = proven(OPENAI_MINT);
    assert_eq!(
        v["official_transition"], "NotTested",
        "candidate_proof_must_not_promote_official_transition"
    );
    assert_eq!(v["issuer_binding_established"], false);
    assert_eq!(v["provenance"], "OperatorSupplied");
    assert_eq!(v["authorization"], false);
    assert_eq!(v["readiness"], Value::Null);
}
#[test]
fn authority_assumptions_are_explicit_and_bounded() {
    let v = proven(OPENAI_MINT);
    assert_eq!(v["signer_assumed_locally"], true);
    assert_eq!(
        v["signer_possession_known"], false,
        "holder key possession stays unknown"
    );
    assert_eq!(v["candidate_authority_assumed_locally"], true);
    assert_eq!(v["candidate_authority_possession_known"], false);
    let assumptions = v["assumptions"].as_array().unwrap();
    assert!(assumptions
        .iter()
        .any(|a| a.as_str().unwrap().contains("not an issuer key")));
}
#[test]
fn wrong_bindings_are_all_rejected() {
    let p = plan(OPENAI_MINT);
    let c = capture(&p, RUN, CHECK);
    let bytes = serde_json::to_vec(&c).unwrap();
    let hash = sha256(&bytes);
    let program = program();
    let program_hash = sha256(&program);
    let call = |run: &str,
                check: &str,
                wallet: &str,
                capture: &str,
                plan: &str,
                program: &[u8],
                phash: &str| {
        conversion::replay(&bytes, run, check, wallet, capture, plan, program, phash)
    };
    assert!(
        call(
            "other-run",
            CHECK,
            &c.wallet_capture_sha256,
            &hash,
            &c.plan_sha256,
            &program,
            &program_hash
        )
        .is_err(),
        "wrong parent run must be rejected"
    );
    assert!(
        call(
            RUN,
            "other-check",
            &c.wallet_capture_sha256,
            &hash,
            &c.plan_sha256,
            &program,
            &program_hash
        )
        .is_err(),
        "wrong check id must be rejected"
    );
    assert!(
        call(
            RUN,
            CHECK,
            &"0".repeat(64),
            &hash,
            &c.plan_sha256,
            &program,
            &program_hash
        )
        .is_err(),
        "wrong wallet digest must be rejected"
    );
    assert!(
        call(
            RUN,
            CHECK,
            &c.wallet_capture_sha256,
            &"0".repeat(64),
            &c.plan_sha256,
            &program,
            &program_hash
        )
        .is_err(),
        "wrong capture digest must be rejected"
    );
    assert!(
        call(
            RUN,
            CHECK,
            &c.wallet_capture_sha256,
            &hash,
            &"0".repeat(64),
            &program,
            &program_hash
        )
        .is_err(),
        "wrong plan digest must be rejected"
    );
    assert!(
        call(
            RUN,
            CHECK,
            &c.wallet_capture_sha256,
            &hash,
            &c.plan_sha256,
            &program,
            &"0".repeat(64)
        )
        .is_err(),
        "wrong candidate binary digest must be rejected"
    );
    let mut tampered = program.clone();
    *tampered.last_mut().unwrap() ^= 0xff;
    assert!(
        call(
            RUN,
            CHECK,
            &c.wallet_capture_sha256,
            &hash,
            &c.plan_sha256,
            &tampered,
            &program_hash
        )
        .is_err(),
        "tampered candidate binary must be rejected"
    );
}
type PlanEdit = (&'static str, Box<dyn Fn(&mut ConversionPlan)>);
#[test]
fn wrong_identities_and_amounts_are_rejected() {
    let c = corpus();
    let cases: Vec<PlanEdit> = vec![
        (
            "wrong source mint",
            Box::new(|p: &mut ConversionPlan| p.source_mint = USDC_MINT.into()),
        ),
        (
            "wrong source account",
            Box::new(|p: &mut ConversionPlan| {
                p.source_account = "123aUGPWa93jiga876U3rLdBP86JNFSoz9tSQWCAskMc".into()
            }),
        ),
        (
            "amount above the fresh balance",
            Box::new(|p: &mut ConversionPlan| p.amount_decimal = Some("100000000000".into())),
        ),
    ];
    for (name, mutate) in cases {
        let mut p = plan(OPENAI_MINT);
        mutate(&mut p);
        let capture = capture(&p, RUN, CHECK);
        assert!(
            run_capture(&capture).is_err(),
            "mismatched_scope_must_be_rejected: {name}"
        );
    }
    // A replacement mint that is not a mint at all cannot become a conversion target.
    let mut p = plan(OPENAI_MINT);
    p.replacement_mint = c.source_account.clone();
    let result = run_capture(&capture(&p, RUN, CHECK));
    assert!(
        result.is_err() || result.unwrap()["status"] != "Proven",
        "wrong replacement mint must be rejected"
    );
}
#[test]
fn an_impossible_candidate_reserve_fails_in_local_simulation() {
    let mut p = plan(OPENAI_MINT);
    p.reserve.funded_replacement_raw = "1".into();
    let v = run_capture(&capture(&p, RUN, CHECK)).unwrap();
    assert_eq!(
        v["status"], "Failed",
        "an underfunded proposed reserve must actually fail: {}",
        v["reason"]
    );
    assert_eq!(v["execution"]["success"], false);
    assert_eq!(v["execution_performed"], true);
    assert_eq!(v["funds_moved"], false);
    let r = &v["reconciliation"];
    assert_eq!(
        r["reconciled"], true,
        "a failure must still reconcile as a verified rollback"
    );
    assert_eq!(r["source_debited_raw"], "0");
    assert_eq!(r["replacement_released_raw"], "0");
    assert!(v["reason"]
        .as_str()
        .unwrap()
        .contains("No mainnet funds moved"));
}
#[test]
fn calculation_alone_never_creates_proof() {
    let p = plan(OPENAI_MINT);
    let mut c = capture(&p, RUN, CHECK);
    c.observations.truncate(3);
    let v = run_capture(&c).unwrap();
    assert_eq!(
        v["status"], "Indeterminate",
        "missing_required_state_must_be_indeterminate_not_proven"
    );
    assert_eq!(v["execution_performed"], false);
    assert_eq!(v["local_execution_performed"], false);
    // The expected arithmetic is available offline, and still proves nothing.
    let expected = expected_output(1000, &p.terms).unwrap();
    assert_eq!(expected.replacement_gross_raw, "500");
}
#[test]
fn a_failed_acquisition_is_indeterminate_not_failed() {
    let p = plan(OPENAI_MINT);
    let mut c = capture(&p, RUN, CHECK);
    c.observations[4].result = None;
    c.observations[4].error = Some("provider unavailable".into());
    let v = run_capture(&c).unwrap();
    assert_eq!(v["status"], "Indeterminate");
    assert!(v["reason"]
        .as_str()
        .unwrap()
        .contains("bounded request budget"));
}
#[test]
fn a_changed_source_account_requires_reconfirmation() {
    let p = plan(OPENAI_MINT);
    let mut c = capture(&p, RUN, CHECK);
    let index = c.observations[4].params[0]
        .as_array()
        .unwrap()
        .iter()
        .position(|a| a == &json!(p.source_account))
        .unwrap();
    c.observations[4].result.as_mut().unwrap()["value"][index]["lamports"] = json!(1);
    let v = run_capture(&c).unwrap();
    assert_eq!(v["status"], "Indeterminate");
    assert!(
        v["reason"]
            .as_str()
            .unwrap()
            .contains("Refresh current state"),
        "a changed selected account must require reconfirmation"
    );
}
#[test]
fn an_unsupported_configuration_is_not_a_failure() {
    let p = plan(OPENAI_MINT);
    let mut c = capture(&p, RUN, CHECK);
    // A frozen holder account is an executor boundary, not a failed conversion.
    let mut parsed: Value = serde_json::from_str(&c.wallet_capture).unwrap();
    let accounts = parsed["observations"][3]["result"]["value"]
        .as_array_mut()
        .unwrap();
    for entry in accounts.iter_mut() {
        if entry["pubkey"] == json!(p.source_account) {
            let data = entry["account"]["data"][0].as_str().unwrap().to_string();
            let mut bytes = base64_decode(&data);
            bytes[108] = 2; // AccountState::Frozen
            entry["account"]["data"][0] = json!(base64_encode(&bytes));
        }
    }
    c.wallet_capture = serde_json::to_string(&parsed).unwrap();
    c.wallet_capture_sha256 = sha256(c.wallet_capture.as_bytes());
    let bytes = serde_json::to_vec(&c).unwrap();
    let program = program();
    let v = conversion::replay(
        &bytes,
        RUN,
        CHECK,
        &c.wallet_capture_sha256,
        &sha256(&bytes),
        &c.plan_sha256,
        &program,
        &sha256(&program),
    )
    .unwrap();
    assert_eq!(v.value()["status"], "Unsupported");
    assert_eq!(v.value()["execution_performed"], false);
}
#[test]
fn a_transfer_check_can_never_satisfy_a_conversion() {
    // Transfer evidence is a different artifact kind with a different path; it
    // carries no conversion reconciliation and cannot be read as one.
    let root = eplyx_lifecycle_impact::repo_root().join("reports/milestone4-validation");
    let bytes = std::fs::read(root.join("live-transfer.capture.json")).unwrap();
    let c: execution::Capture = serde_json::from_slice(&bytes).unwrap();
    let hash = sha256(&bytes);
    let verified = execution::replay(
        &bytes,
        &c.run_id,
        &c.check_id,
        &c.wallet_capture_sha256,
        &hash,
    )
    .unwrap();
    assert_eq!(verified.value()["kind"], "current-execution");
    assert_eq!(verified.value()["path"], "Transfer");
    assert!(verified.value()["reconciliation"]["replacement_released_raw"].is_null());
    let program = program();
    assert!(
        conversion::replay(
            &bytes,
            &c.run_id,
            &c.check_id,
            &c.wallet_capture_sha256,
            &hash,
            &"0".repeat(64),
            &program,
            &sha256(&program)
        )
        .is_err(),
        "transfer_evidence_must_not_parse_as_conversion_evidence"
    );
}
#[test]
fn offline_replay_reruns_the_actual_program_and_reproduces_the_result() {
    let p = plan(OPENAI_MINT);
    let c = capture(&p, RUN, CHECK);
    let first = run_capture(&c).unwrap();
    let second = run_capture(&c).unwrap();
    assert_eq!(
        first, second,
        "offline replay must reproduce the canonical result"
    );
    assert_eq!(
        first["execution"]["compute_units"],
        second["execution"]["compute_units"]
    );
    assert_eq!(
        first["execution_fixture_sha256"], second["execution_fixture_sha256"],
        "the execution fixture must be deterministic"
    );
}
#[test]
fn offline_validation_reports_terms_without_execution() {
    let c = corpus();
    let p = plan(OPENAI_MINT);
    let v = conversion::validate(&c.wallet, &p).unwrap();
    assert_eq!(v["status"], "NotTested");
    assert_eq!(v["execution_performed"], false);
    assert_eq!(v["official_transition"], "NotTested");
    assert_eq!(v["expected"]["replacement_gross_raw"], "500");
    assert_eq!(v["plan_sha256"], p.sha256().unwrap());
}

#[test]
fn a_second_freshly_observed_source_asset_uses_the_same_generic_adapter() {
    // OPENAI, a different current wallet run, converted to a hypothetical
    // replacement under the same mechanism. No SPACEX identity is involved.
    let c = second_asset();
    let spacex = "PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh";
    assert_ne!(c.source_mint, spacex, "the second asset must not be SPACEX");
    let mut p = plan_for(c, spacex);
    p.id = "operator-candidate-openai".into();
    p.terms.ratio_numerator = 3;
    p.terms.ratio_denominator = 4;
    let v = run_capture(&capture_for(
        c,
        &p,
        "second-asset-run",
        "second-asset-check",
    ))
    .unwrap();
    assert_eq!(v["status"], "Proven", "{}", v["reason"]);
    assert_eq!(v["mint"], c.source_mint);
    assert_eq!(v["replacement_mint"], spacex);
    assert_eq!(v["reconciliation"]["source_debited_raw"], "1000");
    assert_eq!(
        v["reconciliation"]["replacement_released_raw"], "750",
        "the same generic ratio arithmetic applies to any asset"
    );
    assert_eq!(v["official_transition"], "NotTested");
    assert_eq!(
        v["issuer_binding_established"], false,
        "no issuer truth is claimed for the second asset either"
    );
}
#[test]
fn the_conversion_engine_contains_no_asset_specific_branch() {
    // The mechanism, adapter and plan model are generic: asset identities arrive
    // only as plan data and captured bytes. Comments are stripped so that the
    // check is about code, and the one user-facing disclaimer is explicit.
    let root = eplyx_lifecycle_impact::repo_root().join("engine/src/conversion");
    for file in ["mod.rs", "demo.rs", "current.rs"] {
        let text = std::fs::read_to_string(root.join(file)).unwrap();
        let code: String = text
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for mint in [
            "PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh",
            "Xs3oZwbHvqis4NYcf4YKWmEia2eC84wSiVrcYcTqpH8",
            "PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF",
        ] {
            assert!(
                !code.contains(mint),
                "conversion/{file} must not hardcode the mint {mint}"
            );
        }
        for line in code.lines().filter(|l| {
            ["SPACEX", "SPCXx", "PreStocks"]
                .iter()
                .any(|n| l.contains(n))
        }) {
            assert!(
                line.contains("not a PreStocks, SPACEX or issuer mechanism"),
                "conversion/{file} names an issuer outside its disclaimer: {line}"
            );
        }
    }
}
fn set_amount(account: &mut eplyx_lifecycle_impact::types::AccountSnapshot, delta: i64) {
    let current = u64::from_le_bytes(account.data[64..72].try_into().unwrap());
    let updated = (current as i64 + delta) as u64;
    account.data[64..72].copy_from_slice(&updated.to_le_bytes());
}
#[test]
fn a_short_replacement_credit_fails_exact_reconciliation() {
    let plan = plan(OPENAI_MINT);
    let (built, mut execution, amount) = build_execution(&plan);
    assert!(
        demo::reconcile(&plan, &built, amount, &execution)
            .unwrap()
            .reconciled
    );
    // The holder receives one raw unit less than the release and fee imply.
    set_amount(
        execution
            .post_accounts
            .get_mut(&built.overlay.destination)
            .unwrap(),
        -1,
    );
    assert!(
        !demo::reconcile(&plan, &built, amount, &execution)
            .unwrap()
            .reconciled,
        "short_replacement_credit_must_reject_proof"
    );
}
#[test]
fn a_release_that_ignores_the_terms_fails_reconciliation() {
    // A legacy replacement has no transfer fee, so reserve and credit can be
    // moved together: only the declared ratio and rounding are violated.
    let plan = plan(USDC_MINT);
    let (built, mut execution, amount) = build_execution(&plan);
    assert!(
        demo::reconcile(&plan, &built, amount, &execution)
            .unwrap()
            .reconciled
    );
    set_amount(
        execution
            .post_accounts
            .get_mut(&built.overlay.reserve_vault)
            .unwrap(),
        -1,
    );
    set_amount(
        execution
            .post_accounts
            .get_mut(&built.overlay.destination)
            .unwrap(),
        1,
    );
    assert!(
        !demo::reconcile(&plan, &built, amount, &execution)
            .unwrap()
            .reconciled,
        "release_outside_the_declared_terms_must_reject_proof"
    );
}
#[test]
fn an_authority_that_cannot_sign_directly_is_unsupported() {
    // A recorded authority that is not a plain wallet is an executor boundary,
    // not a failed conversion and not proof that conversion is impossible.
    let p = plan(OPENAI_MINT);
    let mut c = capture(&p, RUN, CHECK);
    let mut parsed: Value = serde_json::from_str(&c.wallet_capture).unwrap();
    parsed["observations"][2]["result"]["value"]["data"][0] = json!("AAAAAAAA");
    parsed["observations"][2]["result"]["value"]["space"] = json!(6);
    c.wallet_capture = serde_json::to_string(&parsed).unwrap();
    c.wallet_capture_sha256 = sha256(c.wallet_capture.as_bytes());
    let bytes = serde_json::to_vec(&c).unwrap();
    let program = program();
    let v = conversion::replay(
        &bytes,
        RUN,
        CHECK,
        &c.wallet_capture_sha256,
        &sha256(&bytes),
        &c.plan_sha256,
        &program,
        &sha256(&program),
    )
    .unwrap();
    assert_eq!(
        v.value()["status"],
        "Unsupported",
        "unsupported_authority_must_be_a_boundary_not_a_failure"
    );
    assert_eq!(v.value()["execution_performed"], false);
    assert!(v.value()["reason"]
        .as_str()
        .unwrap()
        .contains("directly signing wallet"));
}
