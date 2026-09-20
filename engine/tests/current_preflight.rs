//! Deterministic reconstructed current-run fixtures, not live acquisition acceptance.
use eplyx_lifecycle_impact::{
    expansion::digest,
    lifecycle::{current as wallet, exposure::sha256},
    preflight::{self, *},
    probe::current as execution,
};
use serde_json::{json, Value};
fn root() -> std::path::PathBuf {
    eplyx_lifecycle_impact::repo_root()
}
fn inputs(checks: bool) -> Inputs {
    let c: execution::Capture = serde_json::from_slice(
        &std::fs::read(root().join("reports/milestone4-validation/live-transfer.capture.json"))
            .unwrap(),
    )
    .unwrap();
    let wallet_hash = sha256(c.wallet_capture.as_bytes());
    let mut input = Inputs {
        run_id: "current-test-run".into(),
        preflight_id: "proposal-a".into(),
        created_at: "2030-01-01T00:00:00Z".parse().unwrap(),
        engine_sha256: "test-engine".into(),
        wallet_capture: c.wallet_capture.clone(),
        wallet_sha256: wallet_hash,
        request: Request {
            source: c.request.source,
            successor_mint: None,
            effective_at: "2031-01-01T00:00:00Z".parse().unwrap(),
            deadline: Some("2032-01-01T00:00:00Z".parse().unwrap()),
            post_deadline: Some(PostDeadline::TransitionStillRequired),
            assurance: Assurance::FullTransition,
            check_ids: vec![],
        },
        successor_capture: None,
        checks: vec![],
    };
    if checks {
        for label in ["live-transfer", "live-market"] {
            let mut c: execution::Capture = serde_json::from_slice(
                &std::fs::read(root().join(format!(
                    "reports/milestone4-validation/{label}.capture.json"
                )))
                .unwrap(),
            )
            .unwrap();
            if label == "live-market" {
                let w: wallet::Capture = serde_json::from_str(&input.wallet_capture).unwrap();
                c.observations[1].params[1]["minContextSlot"] =
                    w.observations[3].result.as_ref().unwrap()["context"]["slot"].clone();
            }
            c.run_id = input.run_id.clone();
            c.wallet_capture = input.wallet_capture.clone();
            c.wallet_capture_sha256 = input.wallet_sha256.clone();
            let capture = serde_json::to_string(&c).unwrap();
            let hash = sha256(capture.as_bytes());
            let result = execution::replay(
                capture.as_bytes(),
                &input.run_id,
                &c.check_id,
                &input.wallet_sha256,
                &hash,
            )
            .unwrap();
            input.request.check_ids.push(c.check_id.clone());
            input.checks.push(CheckEvidence {
                id: c.check_id,
                capture_sha256: hash,
                result_sha256: sha256(format!("{}\n", result.value()).as_bytes()),
                engine_sha256: "fixture-test".into(),
                capture,
            });
        }
    }
    input
}
fn evaluate(input: Inputs) -> anyhow::Result<Value> {
    let bundle = preflight::prepare(input)?;
    evaluate_bundle(&bundle)
}
fn evaluate_bundle(b: &Bundle) -> anyhow::Result<Value> {
    let bytes = serde_json::to_vec(b)?;
    preflight::replay(
        &bytes,
        &b.inputs.run_id,
        &b.inputs.preflight_id,
        &b.inputs.wallet_sha256,
        &b.scenario_sha256,
        &sha256(&bytes),
    )
}
fn path(v: &Value, name: &str) -> Value {
    v["paths"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["path_type"] == name)
        .unwrap()["status"]
        .clone()
}
#[test]
fn proposed_provenance_never_becomes_issuer_assertion() {
    let v = evaluate(inputs(false)).unwrap();
    assert_eq!(
        v["proposed_change"]["source"], "UserProposed",
        "user proposal cannot become issuer assertion"
    );
    assert_eq!(v["proposed_change"]["issuer_relationship_verified"], false);
    assert!(v["proposed_change"]["conversion_ratio"].is_null());
    assert_eq!(v["authorization"], false);
}
#[test]
fn unchanged_current_bytes_at_all_proposed_times() {
    let i = inputs(false);
    let original = i.wallet_capture.clone();
    let b = preflight::prepare(i).unwrap();
    let v = evaluate_bundle(&b).unwrap();
    assert_eq!(b.inputs.wallet_capture, original);
    let views = v["views"].as_array().unwrap();
    assert_eq!(views[0]["lifecycle_status"], "Active");
    assert_eq!(views[0]["impact"], "Unaffected");
    assert_eq!(views[0]["gate_applicability"], "PreEvent");
    assert!(views[0]["readiness"].is_null());
    assert_eq!(views[1]["lifecycle_status"], "TransitionRequired");
    assert_eq!(views[1]["impact"], "RequiresTransition");
    assert_eq!(
        views[2]["lifecycle_status"],
        "PostDeadlineTransitionRequired"
    );
    for view in views {
        assert_eq!(
            view["balance_raw"], v["entity"]["balance_raw"],
            "proposed time cannot change current balance"
        );
        assert_eq!(view["account_sha256"], v["entity"]["account_sha256"]);
    }
    assert!(!v.to_string().contains("NoIssuerEntitlement"));
    assert!(v["limitations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s.as_str().unwrap().contains("not predicted")));
}
#[test]
fn saved_phase7_proof_cannot_enter_missing_current_checks() {
    let v = evaluate(inputs(false)).unwrap();
    assert_eq!(
        path(&v, "Transfer"),
        "NotTested",
        "historical proof cannot satisfy current path"
    );
    assert_eq!(path(&v, "SecondaryMarketExit"), "NotTested");
    assert_eq!(path(&v, "OfficialTransition"), "NotTested");
    assert_eq!(path(&v, "Withdrawal"), "NotApplicable");
    assert_eq!(path(&v, "Redemption"), "Unsupported");
    assert_eq!(
        v["views"][1]["readiness"]["mobility"]["status"],
        "Incomplete"
    );
}
#[test]
fn current_mobility_never_proves_official_conversion() {
    let v = evaluate(inputs(true)).unwrap();
    assert_eq!(path(&v, "Transfer"), "Proven");
    assert_eq!(path(&v, "SecondaryMarketExit"), "Proven");
    assert_eq!(
        path(&v, "OfficialTransition"),
        "NotTested",
        "mobility must not prove official conversion"
    );
    assert_eq!(v["views"][1]["readiness"]["mobility"]["status"], "Ready");
    assert_eq!(
        v["views"][1]["readiness"]["full_transition"]["status"],
        "Incomplete"
    );
    for p in v["views"][0]["paths"].as_array().unwrap() {
        assert_eq!(p["lifecycle_relevant"], false);
    }
    for p in v["views"][1]["paths"].as_array().unwrap() {
        assert_eq!(p["status"], path(&v, p["path"].as_str().unwrap()));
    }
}
#[test]
fn refreshed_run_and_other_entity_reject_existing_proof() {
    let i = inputs(true);
    let mut changed = i.clone();
    changed.run_id = "refreshed-run".into();
    assert!(
        evaluate(changed).is_err(),
        "refresh cannot inherit earlier run proof"
    );
    let mut changed = i.clone();
    changed.request.source = "11111111111111111111111111111111".into();
    assert!(
        evaluate(changed).is_err(),
        "other account cannot inherit proof"
    );
    let mut changed = i.clone();
    let mut wallet: wallet::Capture = serde_json::from_str(&changed.wallet_capture).unwrap();
    wallet.selection.as_mut().unwrap().public_owner =
        Some("11111111111111111111111111111111".into());
    changed.wallet_capture = serde_json::to_string(&wallet).unwrap();
    changed.wallet_sha256 = sha256(changed.wallet_capture.as_bytes());
    assert!(
        evaluate(changed).is_err(),
        "other owner cannot inherit proof"
    );
    let mut changed = i;
    changed.wallet_sha256 = "0".repeat(64);
    assert!(evaluate(changed).is_err());
}
#[test]
fn scenario_digest_cannot_reuse_readiness() {
    let b = preflight::prepare(inputs(false)).unwrap();
    let bytes = serde_json::to_vec(&b).unwrap();
    assert!(
        preflight::replay(
            &bytes,
            &b.inputs.run_id,
            &b.inputs.preflight_id,
            &b.inputs.wallet_sha256,
            &"0".repeat(64),
            &sha256(&bytes)
        )
        .is_err(),
        "different scenario digest must reject readiness"
    );
    let mut b = b;
    b.scenario["source"] = "IssuerAsserted".into();
    b.scenario_sha256 = digest(&b.scenario).unwrap();
    assert!(evaluate_bundle(&b).is_err());
}
#[test]
fn entity_ready_never_becomes_population_ready() {
    let v = evaluate(inputs(true)).unwrap();
    assert_eq!(
        v["views"][1]["readiness"]["mobility"]["scope"],
        "SelectedEntityPreflightReadiness"
    );
    assert!(
        v["population_readiness"].is_null(),
        "selected entity must not grant population readiness"
    );
    assert!(v["views"][1]["readiness"]["mobility"]["population_readiness"].is_null());
    assert!(!v.to_string().contains("positive_balance_entities"));
}
#[test]
fn successor_is_independently_inspected_and_never_issuer_verified() {
    let mut i = inputs(false);
    i.request.successor_mint = Some("So11111111111111111111111111111111111111112".into());
    assert!(
        evaluate(i.clone()).is_err(),
        "successor needs separate inspection"
    );
    i.successor_capture = Some(
        std::fs::read_to_string(root().join("reports/milestone2-validation/custom.capture.json"))
            .unwrap(),
    );
    let v = evaluate(i.clone()).unwrap();
    assert_eq!(v["successor_verification"]["status"], "MintObserved");
    assert_eq!(
        v["successor_verification"]["issuer_relationship_verified"],
        false
    );
    assert!(v["successor_verification"]["conversion_ratio"].is_null());
    i.request.successor_mint = Some("11111111111111111111111111111111".into());
    i.successor_capture = Some(
        std::fs::read_to_string(root().join("reports/milestone2-validation/nonmint.capture.json"))
            .unwrap(),
    );
    let v = evaluate(i.clone()).unwrap();
    assert_eq!(v["preparation_status"], "InvalidScenario");
    assert_eq!(v["views"], json!([]));
    let mut c: wallet::Capture =
        serde_json::from_str(i.successor_capture.as_ref().unwrap()).unwrap();
    c.observations[1].result = None;
    c.observations[1].error = Some("bounded acquisition unavailable".into());
    i.successor_capture = Some(serde_json::to_string(&c).unwrap());
    assert_eq!(
        evaluate(i).unwrap()["preparation_status"],
        "PreparationIncomplete"
    );
}
#[test]
fn request_boundaries_and_offline_canonical_replay() {
    let i = inputs(false);
    let b = preflight::prepare(i.clone()).unwrap();
    assert_eq!(evaluate_bundle(&b).unwrap(), evaluate_bundle(&b).unwrap());
    let mut bad = i.clone();
    bad.request.effective_at = bad.created_at;
    assert!(preflight::prepare(bad).is_err());
    let mut bad = i.clone();
    bad.request.deadline = Some(bad.request.effective_at);
    assert!(preflight::prepare(bad).is_err());
    let mut bad = i;
    bad.request.post_deadline = None;
    assert!(preflight::prepare(bad).is_err());
}

#[test]
fn unavailable_current_check_is_indeterminate_not_failure_or_readiness() {
    let mut i = inputs(true);
    i.checks.truncate(1);
    i.request.check_ids = vec![i.checks[0].id.clone()];
    let c = &mut i.checks[0];
    let mut capture: execution::Capture = serde_json::from_str(&c.capture).unwrap();
    capture.observations.clear();
    c.capture = serde_json::to_string(&capture).unwrap();
    c.capture_sha256 = sha256(c.capture.as_bytes());
    let proof = execution::replay(
        c.capture.as_bytes(),
        &i.run_id,
        &c.id,
        &i.wallet_sha256,
        &c.capture_sha256,
    )
    .unwrap();
    c.result_sha256 = sha256(format!("{}\n", proof.value()).as_bytes());
    let v = evaluate(i).unwrap();
    assert_eq!(path(&v, "Transfer"), "Indeterminate");
    assert_eq!(
        v["views"][1]["readiness"]["mobility"]["status"],
        "Incomplete"
    );
}

#[test]
fn another_decoded_account_in_same_wallet_does_not_inherit_proof() {
    let mut i = inputs(true);
    i.checks.truncate(1);
    i.request.check_ids = vec![i.checks[0].id.clone()];
    let second = "123aUGPWa93jiga876U3rLdBP86JNFSoz9tSQWCAskMc";
    let mut w: wallet::Capture = serde_json::from_str(&i.wallet_capture).unwrap();
    let rows = w.observations[3].result.as_mut().unwrap()["value"]
        .as_array_mut()
        .unwrap();
    let mut row = rows[0].clone();
    row["pubkey"] = second.into();
    rows.push(row);
    i.wallet_capture = serde_json::to_string(&w).unwrap();
    i.wallet_sha256 = sha256(i.wallet_capture.as_bytes());
    let c = &mut i.checks[0];
    let mut capture: execution::Capture = serde_json::from_str(&c.capture).unwrap();
    capture.wallet_capture = i.wallet_capture.clone();
    capture.wallet_capture_sha256 = i.wallet_sha256.clone();
    c.capture = serde_json::to_string(&capture).unwrap();
    c.capture_sha256 = sha256(c.capture.as_bytes());
    let proof = execution::replay(
        c.capture.as_bytes(),
        &i.run_id,
        &c.id,
        &i.wallet_sha256,
        &c.capture_sha256,
    )
    .unwrap();
    c.result_sha256 = sha256(format!("{}\n", proof.value()).as_bytes());
    i.request.source = second.into();
    assert!(
        evaluate(i)
            .unwrap_err()
            .to_string()
            .contains("entity binding mismatch"),
        "a second decoded account cannot inherit its peer's proof"
    );
}

#[test]
fn exact_failed_market_is_not_globally_blocked_by_entity_preset() {
    let mut i = inputs(true);
    i.checks.remove(0);
    i.request.check_ids = vec![i.checks[0].id.clone()];
    let c = &mut i.checks[0];
    let mut capture: execution::Capture = serde_json::from_str(&c.capture).unwrap();
    capture.request.minimum_output_decimal = Some("1000000".into());
    c.capture = serde_json::to_string(&capture).unwrap();
    c.capture_sha256 = sha256(c.capture.as_bytes());
    let proof = execution::replay(
        c.capture.as_bytes(),
        &i.run_id,
        &c.id,
        &i.wallet_sha256,
        &c.capture_sha256,
    )
    .unwrap();
    assert_eq!(proof.value()["status"], "Failed");
    c.result_sha256 = sha256(format!("{}\n", proof.value()).as_bytes());
    let v = evaluate(i).unwrap();
    assert_eq!(path(&v, "SecondaryMarketExit"), "Failed");
    assert_eq!(
        v["views"][1]["readiness"]["mobility"]["status"],
        "Incomplete"
    );
    assert_eq!(path(&v, "OfficialTransition"), "NotTested");
}
