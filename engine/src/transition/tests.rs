//! Controlled classifier/receipt cases; none claims a production official execution.
use super::*;
fn mechanism() -> OfficialTransitionMechanism {
    OfficialTransitionMechanism {
        id: "mechanism-a".into(),
        source_asset: "asset-a".into(),
        destination_asset: "asset-b".into(),
        mechanism_type: MechanismType::TransferClaim,
        identity: MechanismIdentity {
            issuer_semantics_bound: true,
            observed_official_transaction: true,
            exact_account_plan_verified: true,
            source_destination_pair_observed: true,
        },
        programs: vec!["program-a".into()],
        required_accounts: vec!["source-a".into(), "destination-a".into()],
        required_signers: vec![RequiredSigner {
            address: "owner-a".into(),
            role: AuthorityRole::Holder,
            possession_known: false,
            assumed_locally: true,
        }],
        eligibility_inputs: vec![],
        onchain_evidence: vec!["controlled-observation".into()],
        external_policy_evidence: vec!["controlled-semantics".into()],
        blocking_requirements: vec![],
    }
}
fn scope() -> TransitionScope {
    TransitionScope {
        entity_id: "entity-a".into(),
        mechanism_id: "mechanism-a".into(),
        context_id: "bank-a".into(),
        exact_input_raw: "100".into(),
        source_asset: "asset-a".into(),
        destination_asset: "asset-b".into(),
    }
}
fn deltas() -> TransitionTokenDeltas {
    TransitionTokenDeltas {
        source_before_raw: "100".into(),
        source_after_raw: "0".into(),
        source_effective_raw: "99".into(),
        source_fee_raw: "1".into(),
        successor_before_raw: "5".into(),
        successor_after_raw: "55".into(),
        documented_input_raw: "100".into(),
        documented_successor_credit_raw: "50".into(),
    }
}
fn receipt() -> VerifiedTransitionExecution {
    VerifiedTransitionExecution {
        scope: scope(),
        path_type: ExitPathType::OfficialTransition,
        vm_success: true,
        issuer_authority_fabricated: false,
        deltas: Some(deltas()),
        rollback_verified: false,
    }
}
#[test]
fn successor_mint_reference_alone_cannot_prove_transition() {
    let mut m = mechanism();
    m.identity.issuer_semantics_bound = false;
    m.identity.observed_official_transaction = false;
    m.identity.exact_account_plan_verified = false;
    assert_eq!(assess(&m, &scope(), None).status, PathStatus::NotTested);
    assert!(!assess(&m, &scope(), None).independent_execution_supported);
}
#[test]
fn successor_only_transfer_cannot_prove_source_exchange() {
    let mut m = mechanism();
    m.identity.observed_official_transaction = false;
    m.identity.source_destination_pair_observed = false;
    assert_eq!(
        assess(&m, &scope(), Some(&receipt())).status,
        PathStatus::NotTested
    );
}
#[test]
fn arbitrary_source_successor_pair_is_not_official_identity() {
    let mut m = mechanism();
    m.identity.issuer_semantics_bound = false;
    m.identity.observed_official_transaction = false;
    m.identity.exact_account_plan_verified = false;
    assert!(m.identity.source_destination_pair_observed);
    assert!(!official_identity_established(&m));
    assert_eq!(
        assess(&m, &scope(), Some(&receipt())).status,
        PathStatus::NotTested
    );
}
#[test]
fn generic_burn_mint_pattern_needs_official_binding() {
    let mut m = mechanism();
    m.mechanism_type = MechanismType::BurnMint;
    m.identity.issuer_semantics_bound = false;
    assert_eq!(
        assess(&m, &scope(), Some(&receipt())).status,
        PathStatus::NotTested
    );
}
#[test]
fn issuer_assumed_private_signing_is_unsupported() {
    let mut m = mechanism();
    m.required_signers.push(RequiredSigner {
        address: "issuer-key".into(),
        role: AuthorityRole::Issuer,
        possession_known: false,
        assumed_locally: true,
    });
    let a = assess(&m, &scope(), Some(&receipt()));
    assert_eq!(a.status, PathStatus::Unsupported);
    assert!(!a.independent_execution_supported);
}
#[test]
fn kyc_backend_dependency_is_unsupported() {
    let mut m = mechanism();
    m.eligibility_inputs.push(EligibilityInput::KycBackend);
    assert_eq!(
        assess(&m, &scope(), Some(&receipt())).status,
        PathStatus::Unsupported
    );
    for input in [
        EligibilityInput::PrivateEntitlement,
        EligibilityInput::BackendSignature,
    ] {
        m.eligibility_inputs = vec![input];
        assert_eq!(
            assess(&m, &scope(), Some(&receipt())).status,
            PathStatus::Unsupported
        );
    }
}
#[test]
fn holder_assumption_does_not_grant_issuer_authorization() {
    let m = mechanism();
    assert!(!m.required_signers[0].possession_known);
    assert!(m.required_signers[0].assumed_locally);
    let mut e = receipt();
    e.issuer_authority_fabricated = true;
    assert_eq!(
        assess(&m, &scope(), Some(&e)).status,
        PathStatus::Unsupported
    );
}
#[test]
fn dex_execution_cannot_be_relabelled_official() {
    let mut e = receipt();
    e.path_type = ExitPathType::SecondaryMarketExit;
    assert_eq!(
        assess(&mechanism(), &scope(), Some(&e)).status,
        PathStatus::NotTested
    );
}
#[test]
fn transition_proof_does_not_inherit_to_another_holder() {
    let mut other = scope();
    other.entity_id = "entity-b".into();
    assert_eq!(
        assess(&mechanism(), &other, Some(&receipt())).status,
        PathStatus::NotTested
    );
}
#[test]
fn exact_mechanism_amount_assets_and_bank_are_isolated() {
    for n in 0..5 {
        let mut s = scope();
        match n {
            0 => s.mechanism_id = "different".into(),
            1 => s.exact_input_raw = "99".into(),
            2 => s.context_id = "bank-b".into(),
            3 => s.source_asset = "asset-c".into(),
            _ => s.destination_asset = "asset-c".into(),
        };
        assert_eq!(
            assess(&mechanism(), &s, Some(&receipt())).status,
            PathStatus::NotTested
        );
    }
}
#[test]
fn successful_receipt_requires_exact_source_successor_and_fee_reconciliation() {
    let m = mechanism();
    assert_eq!(
        assess(&m, &scope(), Some(&receipt())).status,
        PathStatus::Proven
    );
    for n in 0..6 {
        let mut e = receipt();
        let d = e.deltas.as_mut().unwrap();
        match n {
            0 => d.source_after_raw = "1".into(),
            1 => d.successor_after_raw = "54".into(),
            2 => d.source_fee_raw = "2".into(),
            3 => d.documented_successor_credit_raw = "51".into(),
            4 => d.documented_input_raw = "99".into(),
            _ => d.source_before_raw = "-100".into(),
        };
        assert_eq!(
            assess(&m, &scope(), Some(&e)).status,
            PathStatus::Indeterminate
        );
    }
    let mut e = receipt();
    e.deltas = None;
    assert_eq!(
        assess(&m, &scope(), Some(&e)).status,
        PathStatus::Indeterminate
    );
}
#[test]
fn failed_transition_execution_never_becomes_proven() {
    let mut e = receipt();
    e.vm_success = false;
    e.rollback_verified = true;
    assert_eq!(
        assess(&mechanism(), &scope(), Some(&e)).status,
        PathStatus::Failed
    );
    e.rollback_verified = false;
    assert_eq!(
        assess(&mechanism(), &scope(), Some(&e)).status,
        PathStatus::Indeterminate
    );
}
#[test]
fn no_mechanism_is_not_nonexistence() {
    let mut m = mechanism();
    m.identity.observed_official_transaction = false;
    let a = assess(&m, &scope(), None);
    assert_eq!(a.transition_exists_established, None);
    assert_eq!(a.status, PathStatus::NotTested);
    assert!(a.reason.contains("bounded investigation"));
    assert!(!a.execution_attempted);
}
#[test]
fn public_state_and_unknown_eligibility_are_distinct() {
    let mut m = mechanism();
    m.eligibility_inputs = vec![EligibilityInput::PublicStateIncomplete];
    assert_eq!(assess(&m, &scope(), None).status, PathStatus::Indeterminate);
    m.eligibility_inputs = vec![EligibilityInput::Unknown];
    assert_eq!(
        assess(&m, &scope(), Some(&receipt())).status,
        PathStatus::NotTested
    );
}
#[test]
fn generic_transition_code_has_no_asset_issuer_literals() {
    for text in [include_str!("mod.rs"), include_str!("research.rs")] {
        for literal in ["SPACEX", "PreStocks", "prestocks.com", "741ZXY", "Xs3oZwb"] {
            assert!(!text.contains(literal), "{literal}");
        }
    }
    let a = assess(&mechanism(), &scope(), None);
    let json = serde_json::to_string(&a).unwrap();
    let b: TransitionAssessment = serde_json::from_str(&json).unwrap();
    assert_eq!(a, b);
}
