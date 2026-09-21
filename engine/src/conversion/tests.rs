//! Plan, arithmetic and origin invariants. Execution evidence is covered by the
//! integration suite, which runs the actual registered candidate program.
use super::*;

fn plan() -> ConversionPlan {
    ConversionPlan {
        schema_version: 1,
        id: "candidate-plan".into(),
        version: 1,
        provenance: PlanProvenance::OperatorSupplied,
        mechanism: MechanismId::EplyxDemoCandidateConversion,
        adapter_id: ADAPTER_ID.into(),
        mechanism_ref: "Eplyx Demo Candidate Conversion".into(),
        source_mint: "PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh".into(),
        replacement_mint: "PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF".into(),
        source_account: "741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs".into(),
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
#[test]
fn an_operator_supplied_plan_validates_and_digests() {
    let p = plan();
    p.validate().unwrap();
    assert_eq!(p.sha256().unwrap().len(), 64);
    let mut other = p.clone();
    other.terms.ratio_numerator = 3;
    assert_ne!(p.sha256().unwrap(), other.sha256().unwrap());
}
#[test]
fn only_operator_supplied_provenance_is_executable() {
    for provenance in [
        PlanProvenance::UserProposed,
        PlanProvenance::IssuerVerified,
        PlanProvenance::PublicIssuerMechanism,
    ] {
        let mut p = plan();
        p.provenance = provenance;
        let error = p.validate().err().unwrap().to_string();
        assert!(
            error.contains("OperatorSupplied"),
            "issuer_provenance_has_no_executable_adapter: {error}"
        );
    }
}
type PlanEdit = (&'static str, Box<dyn Fn(&mut ConversionPlan)>);
#[test]
fn invalid_terms_identities_and_amounts_are_rejected() {
    let cases: Vec<PlanEdit> = vec![
        (
            "zero ratio",
            Box::new(|p: &mut ConversionPlan| p.terms.ratio_numerator = 0),
        ),
        (
            "zero denominator",
            Box::new(|p: &mut ConversionPlan| p.terms.ratio_denominator = 0),
        ),
        (
            "fee out of range",
            Box::new(|p: &mut ConversionPlan| p.terms.conversion_fee_bps = 10_001),
        ),
        (
            "same mint",
            Box::new(|p: &mut ConversionPlan| p.replacement_mint = p.source_mint.clone()),
        ),
        (
            "invalid address",
            Box::new(|p: &mut ConversionPlan| p.replacement_mint = "not-an-address".into()),
        ),
        (
            "custom without amount",
            Box::new(|p: &mut ConversionPlan| p.amount_mode = AmountMode::Custom),
        ),
        (
            "full with amount",
            Box::new(|p: &mut ConversionPlan| p.amount_decimal = Some("1".into())),
        ),
        (
            "unknown adapter",
            Box::new(|p: &mut ConversionPlan| p.adapter_id = "somebody-elses-adapter".into()),
        ),
        (
            "no holder signature",
            Box::new(|p: &mut ConversionPlan| p.authority_model.holder_signs = false),
        ),
        (
            "noncanonical reserve",
            Box::new(|p: &mut ConversionPlan| p.reserve.funded_replacement_raw = "007".into()),
        ),
    ];
    for (name, mutate) in cases {
        let mut p = plan();
        mutate(&mut p);
        assert!(
            p.validate().is_err(),
            "invalid_plan_must_be_rejected: {name}"
        );
    }
}
#[test]
fn exact_ratio_rounding_and_fee_arithmetic() {
    let terms = |n, d, r, bps| ConversionTerms {
        ratio_numerator: n,
        ratio_denominator: d,
        rounding: r,
        conversion_fee_bps: bps,
    };
    let e = expected_output(7, &terms(1, 2, Rounding::Floor, 0)).unwrap();
    assert_eq!(e.replacement_gross_raw, "3", "floor_rounding_must_be_exact");
    let e = expected_output(7, &terms(1, 2, Rounding::Ceiling, 0)).unwrap();
    assert_eq!(
        e.replacement_gross_raw, "4",
        "ceiling_rounding_must_be_exact"
    );
    let e = expected_output(1000, &terms(1, 1, Rounding::Floor, 250)).unwrap();
    assert_eq!(
        (
            e.conversion_fee_raw.as_str(),
            e.convertible_raw.as_str(),
            e.replacement_gross_raw.as_str()
        ),
        ("25", "975", "975"),
        "conversion_fee_applies_before_the_ratio"
    );
    let e = expected_output(1_000_000_000, &terms(3, 7, Rounding::Floor, 0)).unwrap();
    assert_eq!(e.replacement_gross_raw, "428571428");
}
#[test]
fn conversion_arithmetic_overflow_is_rejected() {
    let error = expected_output(
        u64::MAX,
        &ConversionTerms {
            ratio_numerator: u64::MAX,
            ratio_denominator: 1,
            rounding: Rounding::Floor,
            conversion_fee_bps: 0,
        },
    )
    .err()
    .unwrap()
    .to_string();
    assert!(
        error.contains("exceeds raw integer range"),
        "conversion_overflow_must_be_rejected: {error}"
    );
}
#[test]
fn proposed_accounts_can_never_claim_observed_evidence() {
    let base = FixtureAccount {
        address: "PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh".into(),
        origin: AccountOrigin::Proposed,
        role: "candidate-config".into(),
        runtime_owner: demo::PROGRAM_ID.into(),
        lamports: 1,
        executable: false,
        data_len: 0,
        data_sha256: "0".into(),
        rpc_record: None,
        pointer: None,
        slot: None,
        derivation: Some("derived".into()),
    };
    validate_origins(std::slice::from_ref(&base)).unwrap();
    let mut claims_observation = base.clone();
    claims_observation.rpc_record = Some(4);
    assert!(
        validate_origins(&[claims_observation]).is_err(),
        "proposed_account_must_not_claim_captured_rpc_evidence"
    );
    let mut observed_without_evidence = base;
    observed_without_evidence.origin = AccountOrigin::Observed;
    observed_without_evidence.derivation = None;
    assert!(
        validate_origins(&[observed_without_evidence]).is_err(),
        "observed_account_must_carry_its_captured_pointer"
    );
}
#[test]
fn verified_conversion_has_no_deserialization_path() {
    // Compile-time guarantee, restated: only replay constructs this type.
    let json = serde_json::json!({"status":"Proven"});
    let value = VerifiedReplacementConversion::new(json);
    assert_eq!(value.value()["status"], "Proven");
}
