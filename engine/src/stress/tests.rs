//! Deterministic tests for population discovery, classification, selection and
//! the non-inheritance invariants.
//!
//! Everything here is built from synthetic RPC transcripts, so it never needs a
//! network and never needs a deployed program. The tests that require actual SBF
//! execution live in `engine/tests/conversion_stress.rs`.
use super::{
    classify, execute,
    population::{self, Capture},
    readiness as stress_readiness, select, AuthorityResolution, AuthorityResolutionCompleteness,
    CaseResult, EnumerationCompleteness, SelectionReason, ShapeCoverage, StressBudget,
    SHAPE_PROOF_SCOPE,
};
use crate::{
    conversion::{
        AmountMode, AuthorityModel, CandidateAuthority, ConversionPlan, ConversionTerms,
        MechanismId, PlanProvenance, ReplacementDelivery, ReserveConfig, Rounding,
        SourceConsumption, ADAPTER_ID,
    },
    expansion::Eligibility,
    lifecycle::{current::Observation, decode, exposure::sha256, EntityType},
    resolution::PathStatus,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use solana_address::Address;
use solana_program_pack::Pack;
use spl_token_2022_interface::state::{Account, AccountState, Mint};

const MAINNET: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";
const LEGACY: &str = decode::LEGACY_PROGRAM;
const SYSTEM: &str = "11111111111111111111111111111111";

/// Deterministic address with the requested curve property.
///
/// Only the first two bytes are searched; bytes 2..32 carry the seed, so two
/// different seeds can never collide no matter how far the search runs. A
/// wallet-compatible authority must be on-curve, and mints and token accounts
/// are off-curve, so both are needed.
fn address(seed: u8, want_on_curve: bool) -> Address {
    for n in 0..=u16::MAX {
        let mut bytes = [seed; 32];
        bytes[0] = (n & 0xff) as u8;
        bytes[1] = (n >> 8) as u8;
        let a = Address::new_from_array(bytes);
        if a.is_on_curve() == want_on_curve {
            return a;
        }
    }
    unreachable!("both curve properties occur in this range")
}
fn on_curve(seed: u8) -> Address {
    address(seed, true)
}
fn off_curve(seed: u8) -> Address {
    address(seed, false)
}
fn raw(owner: &str, data: Vec<u8>) -> Value {
    json!({
        "lamports": 2_039_280u64,
        "owner": owner,
        "executable": false,
        "rentEpoch": 0u64,
        "space": data.len(),
        "data": [STANDARD.encode(&data), "base64"],
    })
}
fn mint_account(decimals: u8, supply: u64) -> Value {
    let mut data = vec![0u8; Mint::LEN];
    Mint {
        mint_authority: None.into(),
        supply,
        decimals,
        is_initialized: true,
        freeze_authority: None.into(),
    }
    .pack_into_slice(&mut data);
    raw(LEGACY, data)
}
struct TokenAccount {
    owner: Address,
    amount: u64,
    state: AccountState,
    delegate: Option<Address>,
    delegated: u64,
    close_authority: Option<Address>,
}
impl TokenAccount {
    fn new(owner: Address, amount: u64) -> Self {
        Self {
            owner,
            amount,
            state: AccountState::Initialized,
            delegate: None,
            delegated: 0,
            close_authority: None,
        }
    }
    fn bytes(&self, mint: Address) -> Vec<u8> {
        let mut data = vec![0u8; Account::LEN];
        Account {
            mint,
            owner: self.owner,
            amount: self.amount,
            delegate: self.delegate.into(),
            state: self.state,
            is_native: None.into(),
            delegated_amount: self.delegated,
            close_authority: self.close_authority.into(),
        }
        .pack_into_slice(&mut data);
        data
    }
}
fn wallet_authority() -> Value {
    json!({"lamports": 1_000_000u64, "owner": SYSTEM, "executable": false,
        "rentEpoch": 0u64, "space": 0u64, "data": ["", "base64"]})
}
fn program_authority() -> Value {
    json!({"lamports": 1_000_000u64, "owner": "BPFLoaderUpgradeab1e11111111111111111111111",
        "executable": false, "rentEpoch": 0u64, "space": 8u64,
        "data": [STANDARD.encode([7u8; 8]), "base64"]})
}
fn observation(method: &str, params: Value, result: Option<Value>) -> Observation {
    Observation {
        method: method.into(),
        params,
        started_at: "2026-09-22T00:00:01.000Z".into(),
        completed_at: "2026-09-22T00:00:02.000Z".into(),
        error: result.is_none().then(|| "provider unavailable".to_string()),
        result,
    }
}

struct World {
    mint: Address,
    budget: StressBudget,
    accounts: Vec<(Address, Vec<u8>)>,
    authorities: Vec<(Address, Value)>,
    scan_ok: bool,
    resolve_authorities: bool,
}
impl World {
    /// Four wallet-compatible positive accounts, one program-owned positive
    /// account and one zero-balance account, over a legacy SPL mint.
    fn standard() -> Self {
        let mint = off_curve(3);
        let mut accounts = vec![];
        let mut authorities = vec![];
        for (i, amount) in [(0u8, 1_000u64), (1, 25_000), (2, 400_000), (3, 9_000_000)] {
            let owner = on_curve(10 + i * 7);
            accounts.push((
                off_curve(40 + i),
                TokenAccount::new(owner, amount).bytes(mint),
            ));
            authorities.push((owner, wallet_authority()));
        }
        let program_owner = on_curve(200);
        accounts.push((
            off_curve(60),
            TokenAccount::new(program_owner, 77_000).bytes(mint),
        ));
        authorities.push((program_owner, program_authority()));
        let empty_owner = on_curve(220);
        accounts.push((off_curve(70), TokenAccount::new(empty_owner, 0).bytes(mint)));
        Self {
            mint,
            budget: StressBudget::default(),
            accounts,
            authorities,
            scan_ok: true,
            resolve_authorities: true,
        }
    }
    fn capture(&self) -> Capture {
        let mut rows: Vec<(Address, Vec<u8>)> = self.accounts.clone();
        rows.sort_by_key(|(a, _)| a.to_string());
        let mut observations = vec![observation(
            "getGenesisHash",
            json!([]),
            Some(json!(MAINNET)),
        )];
        let mint_cfg = json!({"encoding":"base64","commitment":"finalized"});
        observations.push(observation(
            "getAccountInfo",
            json!([self.mint.to_string(), mint_cfg]),
            Some(json!({"context":{"slot":1000u64},"value":mint_account(6, 10_000_000)})),
        ));
        let scan_cfg = json!({"encoding":"base64","commitment":"finalized","minContextSlot":1000u64,
            "withContext":true,"filters":[{"memcmp":{"offset":0,"bytes":self.mint.to_string()}}]});
        if !self.scan_ok {
            observations.push(observation(
                "getProgramAccounts",
                json!([LEGACY, scan_cfg]),
                None,
            ));
            return self.finish(observations);
        }
        let value: Vec<Value> = rows
            .iter()
            .map(|(a, d)| json!({"pubkey": a.to_string(), "account": raw(LEGACY, d.clone())}))
            .collect();
        observations.push(observation(
            "getProgramAccounts",
            json!([LEGACY, scan_cfg]),
            Some(json!({"context":{"slot":1001u64},"value":value})),
        ));
        if self.resolve_authorities {
            // The transcript must match the deterministic plan exactly: distinct
            // positive-balance authorities, ascending, chunked by the budget.
            let mut wanted: Vec<String> = vec![];
            for (address, data) in &rows {
                let _ = address;
                let state = decode::decode_token_account(
                    &raw(LEGACY, data.clone()),
                    LEGACY,
                    &self.mint.to_string(),
                    6,
                )
                .unwrap();
                if state.raw_balance != "0" && !wanted.contains(&state.owner) {
                    wanted.push(state.owner);
                }
            }
            wanted.sort();
            for batch in wanted.chunks(self.budget.authority_batch_size) {
                let values: Vec<Value> = batch
                    .iter()
                    .map(|a| {
                        self.authorities
                            .iter()
                            .find(|(k, _)| k.to_string() == *a)
                            .map(|(_, v)| v.clone())
                            .unwrap_or(Value::Null)
                    })
                    .collect();
                observations.push(observation(
                    "getMultipleAccounts",
                    json!([batch, {"encoding":"base64","commitment":"finalized","minContextSlot":1001u64}]),
                    Some(json!({"context":{"slot":1002u64},"value":values})),
                ));
            }
        }
        self.finish(observations)
    }
    fn finish(&self, observations: Vec<Observation>) -> Capture {
        Capture {
            schema_version: 1,
            kind: population::KIND.into(),
            run_id: "run-1".into(),
            stress_id: "stress-1".into(),
            mint: self.mint.to_string(),
            budget: self.budget.clone(),
            rpc_origin: "https://rpc.test".into(),
            started_at: "2026-09-22T00:00:00.000Z".into(),
            completed_at: "2026-09-22T00:00:03.000Z".into(),
            decoder: population::DECODER.into(),
            observations,
        }
    }
}

fn candidate_plan(source_mint: &str) -> ConversionPlan {
    ConversionPlan {
        schema_version: 1,
        id: "11111111-2222-4333-8444-555555555555".into(),
        version: 1,
        provenance: PlanProvenance::OperatorSupplied,
        mechanism: MechanismId::EplyxDemoCandidateConversion,
        adapter_id: ADAPTER_ID.into(),
        mechanism_ref: "Eplyx Demo Candidate Conversion (registered repository candidate)".into(),
        source_mint: source_mint.into(),
        replacement_mint: off_curve(90).to_string(),
        source_account: off_curve(91).to_string(),
        amount_mode: AmountMode::Full,
        amount_decimal: None,
        terms: ConversionTerms {
            ratio_numerator: 1,
            ratio_denominator: 1,
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
const PROGRAM: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const FROZEN_AT: &str = "2026-09-22T00:01:00.000Z";

fn evaluated(world: &World) -> population::PopulationObservation {
    let capture = world.capture();
    let bytes = serde_json::to_vec(&capture).unwrap();
    population::evaluate_bytes(&bytes, &world.budget).unwrap()
}
fn planned(world: &World) -> (population::PopulationObservation, select::StressTestPlan) {
    let observation = evaluated(world);
    let plan = select::build(
        &observation,
        &candidate_plan(&world.mint.to_string()),
        PROGRAM,
        FROZEN_AT,
    )
    .unwrap();
    (observation, plan)
}

// ---------------------------------------------------------------- population

#[test]
fn fresh_population_cannot_be_supplied_by_a_historical_snapshot() {
    // A Phase 2 snapshot is a different, incompatible shape. There is no path
    // that turns saved account inventory into a current stress population.
    let historical = json!({
        "schema_version": 1, "asset": {"name":"x","mint":off_curve(3).to_string(),
        "expected_token_program": null, "expected_genesis_hash": null, "verification": []},
        "captured_at": "2020-01-01T00:00:00Z", "slot": 1, "entities": [], "evidence": []
    });
    let bytes = serde_json::to_vec(&historical).unwrap();
    let error = population::evaluate_bytes(&bytes, &StressBudget::default()).unwrap_err();
    assert!(
        error.to_string().contains("missing field") || error.to_string().contains("unknown field"),
        "a historical snapshot must not deserialize into a stress population: {error}"
    );
}

#[test]
fn complete_enumeration_and_authority_resolution_are_independent_axes() {
    let world = World::standard();
    let complete = evaluated(&world);
    assert_eq!(
        complete.enumeration.completeness,
        EnumerationCompleteness::CompleteForQuery
    );
    assert_eq!(
        complete.authority_resolution.completeness,
        AuthorityResolutionCompleteness::Complete
    );

    // Skipping authority lookups must leave the token-account enumeration itself
    // complete. "We do not know how many accounts exist" is a different finding
    // from "we have every account but not every authority model".
    let mut partial = World::standard();
    partial.resolve_authorities = false;
    let observation = evaluated(&partial);
    assert_eq!(
        observation.enumeration.completeness,
        EnumerationCompleteness::CompleteForQuery,
        "authority budget must never downgrade enumeration completeness"
    );
    assert_eq!(
        observation.authority_resolution.completeness,
        AuthorityResolutionCompleteness::NotPerformed
    );
    assert_eq!(
        observation.enumeration.rows_decoded,
        complete.enumeration.rows_decoded
    );
    assert!(!observation.fully_resolved());
}

#[test]
fn a_failed_scan_is_unavailable_and_never_a_complete_empty_population() {
    let mut world = World::standard();
    world.scan_ok = false;
    let observation = evaluated(&world);
    assert_eq!(
        observation.enumeration.completeness,
        EnumerationCompleteness::Unavailable
    );
    assert_eq!(observation.summary.token_accounts_observed, 0);
    assert!(observation
        .enumeration
        .gaps
        .iter()
        .any(|g| g.contains("Population discovery incomplete")));
    assert!(observation
        .limitations
        .iter()
        .any(|l| l.contains("not the token's holder population")));
}

#[test]
fn reaching_the_decode_budget_is_partial_not_complete() {
    let mut world = World::standard();
    world.budget.max_decoded_accounts = 3;
    world.budget.max_authority_lookups = 3;
    world.resolve_authorities = false;
    let observation = evaluated(&world);
    assert_eq!(
        observation.enumeration.completeness,
        EnumerationCompleteness::Partial
    );
    assert_eq!(observation.enumeration.rows_returned, 6);
    assert_eq!(observation.enumeration.rows_decoded, 3);
}

#[test]
fn zero_balance_is_observed_but_is_never_exposure() {
    let observation = evaluated(&World::standard());
    assert_eq!(observation.summary.token_accounts_observed, 6);
    assert_eq!(observation.summary.positive_balance_accounts_observed, 5);
    assert_eq!(observation.summary.zero_balance_accounts_observed, 1);
    // 1000 + 25000 + 400000 + 9000000 + 77000, with the zero account excluded.
    assert_eq!(observation.summary.observed_public_balance_raw, "9503000");
    assert_eq!(observation.positive_entities().count(), 5);
}

#[test]
fn an_unknown_balance_is_not_treated_as_zero() {
    let dimensions = shape_with(|d| {
        d.account_extension_types = vec!["ConfidentialTransferAccount".into()];
    });
    let (eligibility, reason) = classify::eligibility(&dimensions);
    assert_eq!(eligibility, Eligibility::Unsupported);
    assert!(reason.contains("unknown, not zero"));
}

#[test]
fn rows_for_another_mint_or_program_are_rejected_and_kept_separate() {
    let mut world = World::standard();
    let other_mint = off_curve(150);
    world.accounts.push((
        off_curve(80),
        TokenAccount::new(on_curve(30), 5_000).bytes(other_mint),
    ));
    world.resolve_authorities = false;
    let observation = evaluated(&world);
    assert_eq!(observation.summary.token_accounts_observed, 6);
    assert_eq!(observation.undecoded.len(), 1);
    assert_eq!(observation.summary.undecodable_rows_observed, 1);
    assert_eq!(
        observation.enumeration.completeness,
        EnumerationCompleteness::Partial
    );
    // An undecodable row is retained with its evidence, never counted as zero.
    assert!(observation.undecoded[0].raw_data_sha256.is_some());
    assert_eq!(observation.summary.observed_public_balance_raw, "9503000");
}

#[test]
fn several_accounts_of_one_authority_remain_separate_entities() {
    let mut world = World::standard();
    let shared = on_curve(10);
    world.accounts.push((
        off_curve(85),
        TokenAccount::new(shared, 1_234).bytes(world.mint),
    ));
    let observation = evaluated(&world);
    assert_eq!(observation.summary.token_accounts_observed, 7);
    assert_eq!(observation.summary.positive_balance_accounts_observed, 6);
    let ids: Vec<_> = observation
        .positive_entities()
        .filter(|e| e.authority == shared.to_string())
        .map(|e| e.entity_id.clone())
        .collect();
    assert_eq!(ids.len(), 2, "one authority, two distinct entities");
    assert_ne!(ids[0], ids[1]);
}

// ------------------------------------------------------------- classification

fn shape_with(f: impl FnOnce(&mut classify::ShapeDimensions)) -> classify::ShapeDimensions {
    let mut d = classify::ShapeDimensions {
        authority_model: "WalletCompatible".into(),
        authority_resolved: true,
        authority_on_curve: true,
        authority_account_exists: true,
        authority_runtime_owner: Some(SYSTEM.into()),
        authority_executable: Some(false),
        account_initialized: true,
        account_frozen: false,
        delegate_present: false,
        active_delegation: false,
        close_authority_present: false,
        account_extension_types: vec![],
        mint_token_program: LEGACY.into(),
        mint_paused: false,
        mint_transfer_hook_active: false,
        mint_transfer_fee_configured: false,
        mint_default_account_state: None,
        mint_permanent_delegate: false,
        mint_confidential_transfer: false,
        mint_confidential_mint_burn: false,
        mint_non_transferable: false,
        balance_positive: true,
    };
    f(&mut d);
    d
}

#[test]
fn the_shape_key_is_deterministic_and_carries_no_identity_or_amount() {
    let a = shape_with(|_| {});
    let b = shape_with(|_| {});
    assert_eq!(
        classify::shape_key(&a).unwrap(),
        classify::shape_key(&b).unwrap()
    );
    // Every execution-relevant change must produce a different shape.
    for mutate in [
        |d: &mut classify::ShapeDimensions| d.account_frozen = true,
        |d: &mut classify::ShapeDimensions| d.active_delegation = true,
        |d: &mut classify::ShapeDimensions| d.delegate_present = true,
        |d: &mut classify::ShapeDimensions| d.close_authority_present = true,
        |d: &mut classify::ShapeDimensions| d.mint_transfer_hook_active = true,
        |d: &mut classify::ShapeDimensions| d.authority_model = "ProgramOwnedAuthority".into(),
    ] {
        let changed = shape_with(mutate);
        assert_ne!(
            classify::shape_key(&a).unwrap(),
            classify::shape_key(&changed).unwrap()
        );
    }
}

#[test]
fn balances_and_buckets_never_enter_the_shape_key() {
    let observation = evaluated(&World::standard());
    let mint = observation.mint_config.as_ref().unwrap();
    let wallets: Vec<_> = observation
        .positive_entities()
        .filter(|e| e.authority_model == EntityType::WalletCompatible)
        .collect();
    assert!(wallets.len() >= 2);
    let first = classify::dimensions(wallets[0], mint).unwrap();
    let second = classify::dimensions(wallets[1], mint).unwrap();
    assert_ne!(wallets[0].state.raw_balance, wallets[1].state.raw_balance);
    assert_eq!(
        classify::shape_key(&first).unwrap(),
        classify::shape_key(&second).unwrap(),
        "accounts differing only in balance share one state shape"
    );
}

#[test]
fn only_a_resolved_wallet_authority_is_an_executable_candidate() {
    assert_eq!(
        classify::eligibility(&shape_with(|_| {})).0,
        Eligibility::ExecutableCandidate
    );
    for (model, expected) in [
        ("ProgramOwnedAuthority", Eligibility::Unsupported),
        ("TokenMultisig", Eligibility::Unsupported),
        ("Unknown", Eligibility::Unsupported),
    ] {
        let d = shape_with(|d| d.authority_model = model.into());
        assert_eq!(classify::eligibility(&d).0, expected, "{model}");
    }
    let unresolved = shape_with(|d| {
        d.authority_resolved = false;
        d.authority_model = "Unknown".into();
    });
    assert_eq!(
        classify::eligibility(&unresolved).0,
        Eligibility::CaptureRequired
    );
    assert_eq!(
        classify::eligibility(&shape_with(|d| d.balance_positive = false)).0,
        Eligibility::Invalid
    );
    assert_eq!(
        classify::eligibility(&shape_with(|d| d.account_frozen = true)).0,
        Eligibility::Unsupported
    );
}

#[test]
fn no_authority_model_but_a_resolved_wallet_receives_an_assumed_signer() {
    assert!(classify::assumed_local_signer(
        &EntityType::WalletCompatible,
        AuthorityResolution::Resolved
    ));
    // An unresolved authority is unknown, even if its recorded model defaulted.
    assert!(!classify::assumed_local_signer(
        &EntityType::WalletCompatible,
        AuthorityResolution::NotResolved
    ));
    for model in [
        EntityType::ProgramOwnedAuthority,
        EntityType::TokenMultisig,
        EntityType::Unknown,
    ] {
        assert!(
            !classify::assumed_local_signer(&model, AuthorityResolution::Resolved),
            "{model:?} must never be given a wallet signer"
        );
    }
}

// ------------------------------------------------------------------ selection

#[test]
fn selection_is_deterministic_under_a_reordered_population() {
    let world = World::standard();
    let (_, plan) = planned(&world);
    let mut shuffled = World::standard();
    shuffled.accounts.reverse();
    shuffled.authorities.reverse();
    let (_, other) = planned(&shuffled);
    assert_eq!(plan.classification_sha256, other.classification_sha256);
    assert_eq!(plan.sha256().unwrap(), other.sha256().unwrap());
    assert_eq!(
        plan.selected
            .iter()
            .map(|c| c.token_account.clone())
            .collect::<Vec<_>>(),
        other
            .selected
            .iter()
            .map(|c| c.token_account.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn selection_covers_shapes_first_then_uncovered_balance_buckets() {
    let (_, plan) = planned(&World::standard());
    assert_eq!(
        plan.selected[0].selection_reason,
        SelectionReason::NewStateShape,
        "the first phase must be state-shape coverage"
    );
    let reasons: Vec<_> = plan.selected.iter().map(|c| c.selection_reason).collect();
    assert!(reasons.contains(&SelectionReason::NewBalanceBucket));
    // Every selected case records why it exists.
    assert!(plan.selected.iter().all(|c| !c.selection_detail.is_empty()));
    // Only executable candidates may be selected.
    let executable: Vec<_> = plan
        .state_shapes
        .iter()
        .filter(|s| s.eligibility == Eligibility::ExecutableCandidate)
        .map(|s| s.state_shape_sha256.clone())
        .collect();
    assert!(plan
        .selected
        .iter()
        .all(|c| executable.contains(&c.state_shape_sha256)));
}

#[test]
fn a_program_owned_account_is_never_selected_and_is_reported_as_unsupported() {
    let (_, plan) = planned(&World::standard());
    let unsupported: Vec<_> = plan
        .state_shapes
        .iter()
        .filter(|s| s.eligibility == Eligibility::Unsupported)
        .collect();
    assert_eq!(unsupported.len(), 1);
    assert_eq!(unsupported[0].entities_in_shape, 1);
    assert_eq!(unsupported[0].represented_raw, "77000");
    assert_eq!(unsupported[0].entities_selected, 0);
    assert!(unsupported[0]
        .eligibility_reason
        .contains("non-System runtime program"));
    assert_eq!(*plan.eligibility_counts.get("Unsupported").unwrap(), 1);
}

#[test]
fn the_full_observed_balance_is_bound_and_never_silently_capped() {
    let (observation, plan) = planned(&World::standard());
    for case in &plan.selected {
        let entity = observation
            .positive_entities()
            .find(|e| e.token_account == case.token_account)
            .unwrap();
        assert_eq!(case.selected_amount_raw, entity.state.raw_balance);
        assert_eq!(case.observed_balance_raw, entity.state.raw_balance);
        assert!(!case.amount_capped);
        assert_eq!(
            case.case_plan.amount_mode,
            AmountMode::Custom,
            "the frozen case pins an exact amount rather than whatever a later capture holds"
        );
        assert_eq!(
            crate::probe::current::exact_amount(&case.selected_amount_decimal, 6).unwrap(),
            case.selected_amount_raw.parse::<u64>().unwrap()
        );
    }
}

#[test]
fn every_case_plan_keeps_the_candidate_terms_and_gets_its_own_digest() {
    let base = candidate_plan(&World::standard().mint.to_string());
    let (_, plan) = planned(&World::standard());
    let mut digests = std::collections::BTreeSet::new();
    for case in &plan.selected {
        assert_eq!(case.case_plan.terms, base.terms);
        assert_eq!(case.case_plan.replacement_mint, base.replacement_mint);
        assert_eq!(case.case_plan.mechanism, base.mechanism);
        assert_eq!(case.case_plan.provenance, PlanProvenance::OperatorSupplied);
        assert_eq!(case.case_plan.source_account, case.token_account);
        assert_eq!(case.case_plan_sha256, case.case_plan.sha256().unwrap());
        assert!(digests.insert(case.case_plan_sha256.clone()));
    }
    assert_eq!(plan.candidate_plan_sha256, base.sha256().unwrap());
}

#[test]
fn the_plan_binds_its_population_candidate_plan_and_candidate_program() {
    let world = World::standard();
    let (observation, plan) = planned(&world);
    let base = candidate_plan(&world.mint.to_string());
    plan.validate(&observation, &base, PROGRAM).unwrap();

    // A different candidate program build invalidates the frozen plan.
    assert!(plan.validate(&observation, &base, &"b".repeat(64)).is_err());

    // A different candidate plan invalidates it.
    let mut other = base.clone();
    other.terms.conversion_fee_bps = 250;
    assert!(plan.validate(&observation, &other, PROGRAM).is_err());

    // A different population capture invalidates it: entity ids embed the
    // capture digest, so a refreshed world can never reuse this plan.
    let mut refreshed = World::standard();
    refreshed.accounts[0].1 = TokenAccount::new(on_curve(10), 1_001).bytes(refreshed.mint);
    let new_observation = evaluated(&refreshed);
    assert_ne!(new_observation.capture_sha256, observation.capture_sha256);
    assert!(plan.validate(&new_observation, &base, PROGRAM).is_err());
}

#[test]
fn a_refreshed_population_produces_a_new_world_with_no_inherited_proof() {
    let (first, first_plan) = planned(&World::standard());
    let mut refreshed = World::standard();
    refreshed.accounts[0].1 = TokenAccount::new(on_curve(10), 2_000).bytes(refreshed.mint);
    let (second, second_plan) = planned(&refreshed);
    assert_ne!(first.capture_sha256, second.capture_sha256);
    assert_ne!(first_plan.sha256().unwrap(), second_plan.sha256().unwrap());
    // Entity identity is scoped to its capture, so no case id or entity id from
    // the old world can address anything in the new one.
    let old: std::collections::BTreeSet<_> =
        first_plan.selected.iter().map(|c| &c.entity_id).collect();
    assert!(second_plan
        .selected
        .iter()
        .all(|c| !old.contains(&c.entity_id)));
}

#[test]
fn a_tampered_plan_cannot_survive_revalidation() {
    let world = World::standard();
    let (observation, plan) = planned(&world);
    let base = candidate_plan(&world.mint.to_string());

    // Rewriting an expected classification after the fact.
    let mut edited = plan.clone();
    edited.state_shapes[0].eligibility = Eligibility::Invalid;
    assert!(edited.validate(&observation, &base, PROGRAM).is_err());

    // Dropping a selected case, for instance one that later failed.
    let mut dropped = plan.clone();
    dropped.selected.pop();
    assert!(dropped.validate(&observation, &base, PROGRAM).is_err());

    // Swapping a selected case for a different account.
    let mut swapped = plan.clone();
    swapped.selected[0].token_account = off_curve(70).to_string();
    assert!(swapped.validate(&observation, &base, PROGRAM).is_err());

    // Lowering a selected amount.
    let mut lowered = plan.clone();
    lowered.selected[0].selected_amount_raw = "1".into();
    assert!(lowered.validate(&observation, &base, PROGRAM).is_err());
}

#[test]
fn balance_buckets_are_derived_from_this_capture_and_labelled_as_ordering_only() {
    let (_, plan) = planned(&World::standard());
    assert_eq!(plan.buckets.population, 5);
    assert_eq!(plan.buckets.bucket_count, 4);
    assert!(plan.buckets.note.contains("not a statistical sample"));
    assert!(plan.buckets.method.contains("not economic classes"));
    let total: usize = plan.buckets.boundaries.iter().map(|b| b.entities).sum();
    assert_eq!(total, 5);
}

// -------------------------------------------------- non-inheritance invariants

fn case_result_for(case: &super::SelectedCase, status: PathStatus, executed: bool) -> CaseResult {
    CaseResult {
        case_id: case.case_id.clone(),
        entity_id: case.entity_id.clone(),
        token_account: case.token_account.clone(),
        authority: case.authority.clone(),
        authority_model: case.authority_model.clone(),
        state_shape_sha256: case.state_shape_sha256.clone(),
        shape_label: case.shape_label.clone(),
        balance_bucket: case.balance_bucket,
        selection_reason: case.selection_reason,
        selected_amount_raw: case.selected_amount_raw.clone(),
        selected_amount_decimal: case.selected_amount_decimal.clone(),
        case_plan_sha256: case.case_plan_sha256.clone(),
        candidate_program_sha256: PROGRAM.into(),
        status,
        reason: None,
        execution_performed: executed,
        local_execution_performed: executed,
        signer_assumed_locally: executed,
        signer_possession_known: false,
        candidate_authority_assumed_locally: true,
        issuer_binding_established: false,
        official_transition: PathStatus::NotTested,
        funds_moved: false,
        execution_fixture_sha256: executed.then(|| "f".repeat(64)),
        acquisition: json!({}),
        detail: json!({}),
        result_sha256: "d".repeat(64),
    }
}
fn shape_coverage_for(plan: &select::StressTestPlan, results: &[CaseResult]) -> Vec<ShapeCoverage> {
    plan.state_shapes
        .iter()
        .map(|s| {
            let executed: Vec<String> = results
                .iter()
                .filter(|r| r.state_shape_sha256 == s.state_shape_sha256 && r.execution_performed)
                .map(|r| r.entity_id.clone())
                .collect();
            ShapeCoverage {
                state_shape_sha256: s.state_shape_sha256.clone(),
                shape_label: s.shape_label.clone(),
                eligibility: s.eligibility,
                entities_in_shape: s.entities_in_shape,
                entities_selected: s.entities_selected,
                entities_executed: executed.len(),
                entities_untested: s.entities_in_shape - executed.len(),
                executed_entity_ids: executed,
                represented_raw: s.represented_raw.clone(),
                tested_raw: "0".into(),
                proof_scope: SHAPE_PROOF_SCOPE.into(),
            }
        })
        .collect()
}

#[test]
fn one_tested_entity_can_never_prove_its_peers() {
    let (_, plan) = planned(&World::standard());
    let results: Vec<_> = plan
        .selected
        .iter()
        .map(|c| case_result_for(c, PathStatus::Proven, true))
        .collect();
    let shapes = shape_coverage_for(&plan, &results);
    super::assert_no_proof_inheritance(&plan.selected, &results, &shapes).unwrap();

    // Claiming an untested peer of a tested entity is Proven must fail.
    let mut inflated = results.clone();
    let mut peer = inflated[0].clone();
    peer.entity_id = super::entity_id(&plan.population_capture_sha256, &off_curve(70).to_string());
    inflated.push(peer);
    let error = super::assert_no_proof_inheritance(&plan.selected, &inflated, &shapes).unwrap_err();
    assert!(error.to_string().contains("exactly one result"));
}

#[test]
fn one_tested_shape_can_never_prove_all_of_its_members() {
    let (_, plan) = planned(&World::standard());
    let results: Vec<_> = plan
        .selected
        .iter()
        .map(|c| case_result_for(c, PathStatus::Proven, true))
        .collect();
    let mut shapes = shape_coverage_for(&plan, &results);
    // Marking every member of a shape as executed is rejected.
    let target = shapes
        .iter_mut()
        .find(|s| s.entities_in_shape > s.entities_executed && s.entities_executed > 0);
    if let Some(shape) = target {
        shape.entities_executed = shape.entities_in_shape;
        shape.entities_untested = 0;
        let error =
            super::assert_no_proof_inheritance(&plan.selected, &results, &shapes).unwrap_err();
        assert!(error.to_string().contains("consistent with exact executed"));
    }
    // Attributing evidence to an entity that was never selected is rejected.
    let mut shapes = shape_coverage_for(&plan, &results);
    shapes[0]
        .executed_entity_ids
        .push("current-stress:other".into());
    shapes[0].entities_executed = shapes[0].executed_entity_ids.len();
    shapes[0].entities_in_shape = shapes[0].entities_in_shape.max(shapes[0].entities_executed);
    shapes[0].entities_selected = shapes[0].entities_selected.max(shapes[0].entities_executed);
    shapes[0].entities_untested = shapes[0].entities_in_shape - shapes[0].entities_executed;
    let error = super::assert_no_proof_inheritance(&plan.selected, &results, &shapes).unwrap_err();
    assert!(error
        .to_string()
        .contains("entities that were never selected"));
}

#[test]
fn a_failed_case_cannot_be_removed_replaced_or_re_aimed_after_execution() {
    let (_, plan) = planned(&World::standard());
    let mut results: Vec<_> = plan
        .selected
        .iter()
        .map(|c| case_result_for(c, PathStatus::Proven, true))
        .collect();
    results[1].status = PathStatus::Failed;

    // Removing it.
    let mut removed = results.clone();
    removed.remove(1);
    let shapes = shape_coverage_for(&plan, &removed);
    assert!(super::assert_no_proof_inheritance(&plan.selected, &removed, &shapes).is_err());

    // Re-pointing it at a different account that did succeed.
    let mut repointed = results.clone();
    repointed[1].token_account = plan.selected[0].token_account.clone();
    let shapes = shape_coverage_for(&plan, &repointed);
    let error =
        super::assert_no_proof_inheritance(&plan.selected, &repointed, &shapes).unwrap_err();
    assert!(error.to_string().contains("bound to its exact frozen case"));

    // Quietly lowering its amount.
    let mut lowered = results;
    lowered[1].selected_amount_raw = "1".into();
    let shapes = shape_coverage_for(&plan, &lowered);
    assert!(super::assert_no_proof_inheritance(&plan.selected, &lowered, &shapes).is_err());
}

#[test]
fn proven_requires_an_actual_local_execution() {
    let (_, plan) = planned(&World::standard());
    let mut results: Vec<_> = plan
        .selected
        .iter()
        .map(|c| case_result_for(c, PathStatus::Proven, true))
        .collect();
    results[0].execution_performed = false;
    results[0].local_execution_performed = false;
    let shapes = shape_coverage_for(&plan, &results);
    let error = super::assert_no_proof_inheritance(&plan.selected, &results, &shapes).unwrap_err();
    assert!(error.to_string().contains("actual local execution"));
}

#[test]
fn no_case_may_claim_issuer_binding_possession_or_an_official_transition() {
    let (_, plan) = planned(&World::standard());
    for mutate in [
        |r: &mut CaseResult| r.official_transition = PathStatus::Proven,
        |r: &mut CaseResult| r.issuer_binding_established = true,
        |r: &mut CaseResult| r.signer_possession_known = true,
        |r: &mut CaseResult| r.funds_moved = true,
    ] {
        let mut results: Vec<_> = plan
            .selected
            .iter()
            .map(|c| case_result_for(c, PathStatus::Proven, true))
            .collect();
        mutate(&mut results[0]);
        let shapes = shape_coverage_for(&plan, &results);
        assert!(super::assert_no_proof_inheritance(&plan.selected, &results, &shapes).is_err());
    }
}

#[test]
fn each_entity_balance_is_counted_exactly_once() {
    let mut balances = std::collections::BTreeMap::new();
    balances.insert("a".to_string(), 100u64);
    balances.insert("b".to_string(), 250u64);
    assert_eq!(super::sum_once(&balances), "350");
    // Re-inserting the same entity cannot inflate the total.
    balances.insert("a".to_string(), 100u64);
    assert_eq!(super::sum_once(&balances), "350");
}

// ------------------------------------------------------------------- readiness

fn readiness_for(world: &World, statuses: &[PathStatus]) -> (Value, Value) {
    let (observation, plan) = planned(world);
    let results: Vec<_> = plan
        .selected
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let status = statuses.get(i).copied().unwrap_or(PathStatus::Proven);
            case_result_for(c, status, status != PathStatus::Indeterminate)
        })
        .collect();
    let shapes = shape_coverage_for(&plan, &results);
    stress_readiness::evaluate_stress(
        &observation,
        &plan,
        &results,
        &shapes,
        "2026-09-22T00:05:00.000Z",
    )
    .unwrap()
}

#[test]
fn an_exact_failed_case_blocks_the_stress_policy() {
    let (stress, _) = readiness_for(&World::standard(), &[PathStatus::Failed]);
    assert_eq!(stress["status"], json!("Blocked"));
    assert_eq!(stress["scope"], json!("ConversionStressReadiness"));
    let finding = stress["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["requirement_id"] == "selected-case-outcomes")
        .unwrap();
    assert_eq!(finding["effect"], json!("Blocking"));
}

#[test]
fn an_indeterminate_case_leaves_evidence_incomplete_rather_than_blocked() {
    let (stress, _) = readiness_for(&World::standard(), &[PathStatus::Indeterminate]);
    assert_eq!(stress["status"], json!("Incomplete"));
    let finding = stress["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["requirement_id"] == "selected-case-outcomes")
        .unwrap();
    assert_eq!(finding["effect"], json!("IncompleteEvidence"));
}

#[test]
fn unsupported_population_states_keep_the_stress_policy_incomplete() {
    // Every selected case passes, but a program-controlled account remains.
    let (stress, _) = readiness_for(&World::standard(), &[]);
    assert_eq!(stress["status"], json!("Incomplete"));
    let finding = stress["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["requirement_id"] == "population-acquisition")
        .unwrap();
    assert_eq!(finding["effect"], json!("IncompleteEvidence"));
    assert!(finding["observed"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o.as_str().unwrap().contains("independent")));
}

#[test]
fn population_readiness_is_never_satisfied_by_a_bounded_sample() {
    let (stress, population) = readiness_for(&World::standard(), &[]);
    assert_eq!(population["scope"], json!("PopulationRolloutReadiness"));
    assert_eq!(population["status"], json!("Incomplete"));
    assert_eq!(
        population["rollout_readiness"]["exhaustive_execution"],
        json!(false)
    );
    // The two scopes are separate values and neither overwrites the other.
    assert_eq!(stress["scope"], json!("ConversionStressReadiness"));
    assert_eq!(stress["population_readiness"], Value::Null);
    assert_eq!(stress["official_transition_established"], json!(false));
    assert!(stress["not_asset_safety"]
        .as_str()
        .unwrap()
        .contains("not a safety"));
}

#[test]
fn incomplete_enumeration_keeps_the_acquisition_requirement_unsatisfied() {
    let mut world = World::standard();
    world.budget.max_decoded_accounts = 3;
    world.budget.max_authority_lookups = 3;
    world.resolve_authorities = false;
    let (observation, plan) = planned(&world);
    assert_eq!(
        plan.enumeration_completeness,
        EnumerationCompleteness::Partial
    );
    assert_eq!(
        plan.authority_resolution_completeness,
        AuthorityResolutionCompleteness::NotPerformed
    );
    let results: Vec<_> = plan
        .selected
        .iter()
        .map(|c| case_result_for(c, PathStatus::Proven, true))
        .collect();
    let shapes = shape_coverage_for(&plan, &results);
    let (stress, _) = stress_readiness::evaluate_stress(
        &observation,
        &plan,
        &results,
        &shapes,
        "2026-09-22T00:05:00.000Z",
    )
    .unwrap();
    assert_eq!(stress["status"], json!("Incomplete"));
}

// ------------------------------------------------------------------- structure

#[test]
fn the_stress_modules_carry_no_asset_issuer_or_venue_literals() {
    // The stress pipeline must be generic: a second asset gets the same code.
    for (name, source) in [
        ("mod.rs", include_str!("mod.rs")),
        ("population.rs", include_str!("population.rs")),
        ("classify.rs", include_str!("classify.rs")),
        ("select.rs", include_str!("select.rs")),
        ("execute.rs", include_str!("execute.rs")),
        ("readiness.rs", include_str!("readiness.rs")),
    ] {
        for needle in [
            "SPACEX",
            "spacex",
            "SPCXx",
            "OPENAI",
            "openai",
            "PreStocks",
            "prestocks",
            "PreANxu",
            "Prewe",
            "USDC",
            "EPjFWdd5",
        ] {
            assert!(
                !source.contains(needle),
                "{name} must not contain the asset or issuer literal {needle}"
            );
        }
    }
}

#[test]
fn the_stress_pipeline_never_submits_a_transaction() {
    for source in [
        include_str!("mod.rs"),
        include_str!("population.rs"),
        include_str!("classify.rs"),
        include_str!("select.rs"),
        include_str!("execute.rs"),
        include_str!("readiness.rs"),
    ] {
        for needle in [
            "sendTransaction",
            "simulateTransaction",
            "requestAirdrop",
            "signTransaction",
            "sendAndConfirm",
        ] {
            assert!(
                !source.contains(needle),
                "no mainnet submission path: {needle}"
            );
        }
    }
}

#[test]
fn the_budget_is_bounded_serial_and_never_browser_supplied() {
    let budget = StressBudget::default();
    budget.validate().unwrap();
    assert_eq!(budget.max_concurrent_vm_executions, 1);
    assert_eq!(budget.max_concurrent_rpc_requests, 1);
    assert_eq!(budget.executions_per_case, 1);
    assert_eq!(budget.rpc_requests_per_case, 5);
    // Large enough for known current populations, and still explicitly bounded.
    assert!(budget.max_decoded_accounts >= 100_000);
    let mut invalid = budget.clone();
    invalid.max_selected_cases = 0;
    assert!(invalid.validate().is_err());
    let mut concurrent = budget;
    concurrent.max_concurrent_vm_executions = 4;
    assert!(concurrent.validate().is_err());
}

#[test]
fn a_capture_bundle_is_bound_to_its_frozen_plan() {
    let (_, plan) = planned(&World::standard());
    let digest = plan.sha256().unwrap();
    assert!(execute::capture_cases(&plan, &"0".repeat(64), LEGACY, &FailingRpc).is_err());
    // With the right digest the capture proceeds and records every case, even
    // though this provider answers nothing.
    let bundle = execute::capture_cases(&plan, &digest, LEGACY, &FailingRpc).unwrap();
    assert_eq!(bundle.cases.len(), plan.selected.len());
    assert_eq!(bundle.stress_plan_sha256, digest);
    for (case, captured) in plan.selected.iter().zip(&bundle.cases) {
        assert_eq!(captured.case_id, case.case_id);
        assert_eq!(captured.token_account, case.token_account);
        assert!(captured.observations.len() < 5);
    }
}
struct FailingRpc;
impl crate::lifecycle::rpc::SolanaRpc for FailingRpc {
    fn origin(&self) -> String {
        "https://rpc.test".into()
    }
    fn call(&self, _method: &str, _params: Value) -> anyhow::Result<Value> {
        anyhow::bail!("provider unavailable")
    }
}

#[test]
fn population_capture_digests_bind_entity_identity() {
    let observation = evaluated(&World::standard());
    let capture = World::standard().capture();
    let bytes = serde_json::to_vec(&capture).unwrap();
    assert_eq!(observation.capture_sha256, sha256(&bytes));
    for entity in observation.positive_entities() {
        assert!(entity
            .entity_id
            .starts_with(&format!("current-stress:{}:", observation.capture_sha256)));
    }
}
