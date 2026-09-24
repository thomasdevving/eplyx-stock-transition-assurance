//! Bounded counterexample search over a verified package run. Search findings are
//! separate from analytical readiness and never confer issuer or population proof.
use super::{
    demo, expected_output, package, package_preflight, AccountOrigin, AmountMode, ConversionPlan,
};
use crate::{
    executor::{self, ProbeTransactionExecution},
    expansion::{canonical, digest, Eligibility},
    lifecycle::{decode, exposure::sha256, rpc::HttpSolanaRpc, RpcEvidence},
    probe::{ProbeClock, ProbeMessage},
    resolution::PathStatus,
    stress::{
        classify, execute, population,
        select::{self, StressTestPlan},
        CaseResult, SelectedCase, SelectionReason, FULL_AT_FINAL_POLICY,
    },
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use spl_token_2022_interface::{
    extension::{
        transfer_fee::TransferFeeAmount, BaseStateWithExtensions, StateWithExtensions,
        StateWithExtensionsMut,
    },
    state::{Account, Mint},
};
use std::{collections::BTreeSet, fs, path::Path};

pub const VERSION: &str = "eplyx-counterexample-search/v1";
pub const MAX_OBSERVED: usize = 25;
pub const MAX_BOUNDARY: usize = 20;
pub const MAX_MINIMIZATION: usize = 20;
pub const NO_FINDING: &str = "No counterexample found within this search domain and budget.";
const MAX_ARTIFACT: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dimension {
    SourceAmount,
    ProposedReserve,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Method {
    ExactObservedSearch,
    BinarySearch,
    OrderedProbe,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureSignature {
    pub stage: String,
    pub program: String,
    pub instruction_error: String,
    pub relevant_log: Option<String>,
    pub rollback_verified: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    pub dimension: Dimension,
    pub value_raw: String,
    pub method: Method,
    pub status: PathStatus,
    pub execution_plan_sha256: String,
    pub execution_fixture_sha256: String,
    pub failure_signature: Option<FailureSignature>,
    pub origin: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Counterexample {
    Observed {
        id: String,
        parent_run: String,
        transition_package_sha256: String,
        candidate_program_sha256: String,
        observed_source_account: String,
        observed_state_digest: String,
        observed_amount_raw: String,
        execution_plan_sha256: String,
        execution_fixture_sha256: String,
        failure_signature: FailureSignature,
        provenance: String,
        limitations: String,
    },
    Derived {
        id: String,
        parent_run: String,
        transition_package_sha256: String,
        candidate_program_sha256: String,
        observed_source_account: String,
        observed_state_digest: String,
        observed_amount_raw: String,
        search_dimension: Dimension,
        original_value_raw: String,
        derived_value_raw: String,
        execution_plan_sha256: String,
        execution_fixture_sha256: String,
        failure_signature: FailureSignature,
        original_failure_signature: Option<Box<FailureSignature>>,
        signature_preserved: Option<bool>,
        minimized: bool,
        last_passing_value_raw: Option<String>,
        first_passing_value_raw: Option<String>,
        minimization_trace: Vec<Probe>,
        provenance: String,
        limitations: String,
    },
}

impl Counterexample {
    pub fn claim(&self) -> &'static str {
        match self {
            Self::Observed { .. } => "Observed production state",
            Self::Derived { .. } => "Derived from observed production state",
        }
    }
}

pub fn render_human(result: &SearchResult) -> String {
    if result.counterexamples.is_empty() {
        return format!("{}\nSearch domain: {}\nObserved selections: {} of {}\nBoundary VM probes: {} of {}\nNo mainnet funds moved.\n",
            result.conclusion, result.search_domain, result.budget.observed_cases_selected,
            result.budget.max_observed_executions, result.budget.boundary_executions,
            result.budget.max_boundary_executions);
    }
    let mut lines = vec!["COUNTEREXAMPLE FOUND".to_string()];
    for counterexample in &result.counterexamples {
        match counterexample {
            Counterexample::Observed {
                observed_source_account,
                observed_amount_raw,
                failure_signature,
                ..
            } => {
                lines.push(format!(
                    "Kind: {}\nAccount: {}\nCaptured amount: {}\nFailure: {}",
                    counterexample.claim(),
                    observed_source_account,
                    observed_amount_raw,
                    failure_signature.instruction_error
                ));
            }
            Counterexample::Derived {
                observed_source_account,
                search_dimension,
                original_value_raw,
                derived_value_raw,
                last_passing_value_raw,
                first_passing_value_raw,
                failure_signature,
                signature_preserved,
                minimized,
                ..
            } => {
                lines.push(format!("Kind: {}\nObserved source: {}\nSearch dimension: {:?}\nOriginal value: {}\nDerived failing value: {}\nLast passing amount: {}\nFirst passing reserve: {}\nFailure: {}\nMinimized: {}\nFailure signature preserved: {}",
                    counterexample.claim(), observed_source_account, search_dimension,
                    original_value_raw, derived_value_raw,
                    last_passing_value_raw.as_deref().unwrap_or("unknown"),
                    first_passing_value_raw.as_deref().unwrap_or("unknown"),
                    failure_signature.instruction_error, minimized,
                    signature_preserved.map_or("no original failure", |v| if v {"yes"} else {"no"})));
            }
        }
    }
    lines.push("No mainnet funds moved. Derived variants are local states only.".into());
    lines.join("\n\n")
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub max_observed_executions: usize,
    pub max_boundary_executions: usize,
    pub max_minimization_executions: usize,
    pub observed_cases_selected: usize,
    pub observed_executions: usize,
    pub boundary_executions: usize,
    pub minimization_executions: usize,
}
impl Budget {
    fn new() -> Self {
        Self {
            max_observed_executions: MAX_OBSERVED,
            max_boundary_executions: MAX_BOUNDARY,
            max_minimization_executions: MAX_MINIMIZATION,
            observed_cases_selected: 0,
            observed_executions: 0,
            boundary_executions: 0,
            minimization_executions: 0,
        }
    }
    fn charge_boundary(&mut self, minimizing: bool) -> Result<()> {
        ensure!(
            self.boundary_executions < MAX_BOUNDARY,
            "boundary search budget exhausted"
        );
        if minimizing {
            ensure!(
                self.minimization_executions < MAX_MINIMIZATION,
                "minimization budget exhausted"
            );
            self.minimization_executions += 1;
        }
        self.boundary_executions += 1;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchResult {
    pub version: String,
    pub parent_run: String,
    pub transition_package_sha256: String,
    pub candidate_program_sha256: String,
    pub population_capture_sha256: String,
    pub stress_plan_sha256: String,
    pub case_capture_sha256: String,
    pub budget: Budget,
    pub search_domain: String,
    pub derived_domain: Option<DerivedDomain>,
    pub observed_wave: Vec<String>,
    pub additional_waves: Vec<WaveRecord>,
    pub trace: Vec<Probe>,
    pub counterexamples: Vec<Counterexample>,
    pub conclusion: String,
    pub official_transition: String,
    pub funds_moved: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedDomain {
    pub observed_source_account: String,
    pub seed_selection_reason: String,
    pub source_amount_min_raw: String,
    pub source_amount_max_raw: String,
    pub source_amount_mutation_available: bool,
    pub proposed_reserve_min_raw: String,
    pub proposed_reserve_max_raw: String,
    pub replacement_mint_captured_supply_raw: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaveRecord {
    pub wave: usize,
    pub freeze_sha256: String,
    pub frozen_plan_sha256: String,
    pub case_capture_sha256: String,
    pub selected_exact_accounts: Vec<String>,
    pub selection_reasons: Vec<String>,
    pub result_sha256: Vec<String>,
    pub outcomes: Vec<PathStatus>,
    pub failure_signatures: Vec<Option<FailureSignature>>,
    pub next_search_decision: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WaveFreeze {
    search_version: String,
    wave: usize,
    transition_package_sha256: String,
    candidate_program_sha256: String,
    parent_population_sha256: String,
    parent_stress_plan_sha256: String,
    frozen_plan_sha256: String,
    amount_policy: String,
    selected_exact_accounts: Vec<String>,
    selection_reasons: Vec<String>,
    frozen_at: String,
}

fn freeze(
    wave: usize,
    plan: &StressTestPlan,
    package_sha256: &str,
    parent_plan_sha256: &str,
) -> Result<WaveFreeze> {
    Ok(WaveFreeze {
        search_version: VERSION.into(),
        wave,
        transition_package_sha256: package_sha256.into(),
        candidate_program_sha256: plan.candidate_program_sha256.clone(),
        parent_population_sha256: plan.population_capture_sha256.clone(),
        parent_stress_plan_sha256: parent_plan_sha256.into(),
        frozen_plan_sha256: plan.sha256()?,
        amount_policy: FULL_AT_FINAL_POLICY.into(),
        selected_exact_accounts: plan
            .selected
            .iter()
            .map(|c| c.token_account.clone())
            .collect(),
        selection_reasons: plan
            .selected
            .iter()
            .map(|c| c.selection_detail.clone())
            .collect(),
        frozen_at: plan.frozen_at.clone(),
    })
}

fn read(root: &Path, name: &str, max: usize) -> Result<Vec<u8>> {
    let bytes = fs::read(root.join(name))?;
    ensure!(bytes.len() <= max, "search input exceeds artifact bound");
    Ok(bytes)
}

fn signature(
    execution: &ProbeTransactionExecution,
    rollback_verified: bool,
) -> Option<FailureSignature> {
    if execution.success || !rollback_verified {
        return None;
    }
    let error = execution.error.clone()?;
    let log = execution
        .logs
        .iter()
        .rev()
        .find(|line| line.contains("failed: ") || line.contains("EPLYX_CANDIDATE_CONVERSION"))
        .cloned();
    Some(FailureSignature {
        stage: "ExecutedInstruction".into(),
        program: demo::PROGRAM_ID.into(),
        instruction_error: error,
        relevant_log: log,
        rollback_verified,
    })
}

fn observed_signature(result: &CaseResult) -> Result<Option<FailureSignature>> {
    if result.status != PathStatus::Failed || !result.execution_performed {
        return Ok(None);
    }
    let execution: ProbeTransactionExecution =
        serde_json::from_value(result.detail["execution"].clone())?;
    Ok(signature(
        &execution,
        result.detail["rollback_verified"] == true,
    ))
}

fn evidence(capture: &execute::CaseCapture) -> Result<Vec<RpcEvidence>> {
    capture
        .observations
        .iter()
        .enumerate()
        .map(|(id, record)| {
            Ok(RpcEvidence {
                id,
                method: record.method.clone(),
                params: record.params.clone(),
                result: record
                    .result
                    .clone()
                    .context("incomplete final case capture")?,
            })
        })
        .collect()
}

fn observed_source(
    case: &crate::stress::SelectedCase,
    capture: &execute::CaseCapture,
    mint: &decode::MintConfig,
) -> Result<(decode::TokenAccountState, u64)> {
    let final_record = capture
        .observations
        .last()
        .context("missing final case capture")?;
    let address = final_record.params[0]
        .as_array()
        .context("missing final account plan")?;
    let index = address
        .iter()
        .position(|a| a == &case.token_account)
        .context("source absent from final bank")?;
    let raw = &final_record
        .result
        .as_ref()
        .context("missing final account batch")?["value"][index];
    let state = decode::decode_token_account(
        raw,
        &mint.token_program,
        &case.case_plan.source_mint,
        mint.decimals,
    )?;
    let withheld = if mint.token_program == decode::TOKEN_2022_PROGRAM {
        let bytes = decode::account_bytes(raw, &mint.token_program)?;
        let typed = StateWithExtensions::<Account>::unpack(&bytes)?;
        typed
            .get_extension::<TransferFeeAmount>()
            .map(|fee| u64::from(fee.withheld_amount))
            .unwrap_or(0)
    } else {
        0
    };
    let observed: u64 = state.raw_balance.parse()?;
    ensure!(
        withheld <= observed,
        "captured withheld amount exceeds source balance"
    );
    Ok((state, withheld.max(1)))
}

fn replacement_supply(case: &SelectedCase, capture: &execute::CaseCapture) -> Result<u64> {
    let final_record = capture
        .observations
        .last()
        .context("missing final case capture")?;
    let addresses = final_record.params[0]
        .as_array()
        .context("missing final account plan")?;
    let index = addresses
        .iter()
        .position(|a| a == &case.case_plan.replacement_mint)
        .context("replacement mint absent from final bank")?;
    let raw = &final_record
        .result
        .as_ref()
        .context("missing final account batch")?["value"][index];
    Ok(decode::decode_mint(raw)?.raw_supply.parse()?)
}

fn typed_amount_mutation(
    built: &mut demo::BuiltConversion,
    plan: &ConversionPlan,
    observed: u64,
    derived: u64,
) -> Result<()> {
    mutate_token_state(&mut built.plan.accounts, plan, observed, derived)?;
    let delta = observed - derived;
    for fixture in &mut built.accounts {
        if fixture.address == plan.source_account || fixture.address == plan.source_mint {
            let account = built
                .plan
                .accounts
                .iter()
                .find(|a| a.address == fixture.address)
                .context("derived account missing")?;
            fixture.origin = AccountOrigin::DerivedForSearch;
            fixture.derivation = Some(format!("Typed token-state reconstruction: source amount {observed} -> {derived}; mint supply decreases by {delta}"));
            fixture.data_sha256 = sha256(&account.account.data);
        }
    }
    super::validate_origins(&built.accounts)?;
    built.source_before_raw = derived.to_string();
    Ok(())
}

fn mutate_token_state(
    accounts: &mut [crate::types::NamedAccount],
    plan: &ConversionPlan,
    observed: u64,
    derived: u64,
) -> Result<()> {
    ensure!(
        derived > 0 && derived <= observed,
        "derived amount outside observed-source domain"
    );
    let delta = observed - derived;
    let mut source_found = false;
    let mut mint_found = false;
    for account in accounts {
        if account.address == plan.source_account {
            source_found = true;
            let mut state = StateWithExtensionsMut::<Account>::unpack(&mut account.account.data)?;
            ensure!(
                state.base.amount == observed,
                "source amount is not the observed parent"
            );
            let withheld = state
                .get_extension::<TransferFeeAmount>()
                .map(|fee| u64::from(fee.withheld_amount))
                .unwrap_or(0);
            ensure!(
                derived >= withheld,
                "derived source amount is below captured withheld fees"
            );
            state.base.amount = derived;
            state.pack_base();
        } else if account.address == plan.source_mint {
            mint_found = true;
            let mut state = StateWithExtensionsMut::<Mint>::unpack(&mut account.account.data)?;
            ensure!(
                state.base.supply >= delta,
                "derived supply would be negative"
            );
            state.base.supply -= delta;
            state.pack_base();
        }
    }
    ensure!(
        source_found && mint_found,
        "typed mutation requires source and mint accounts"
    );
    Ok(())
}

fn probe(
    case: &crate::stress::SelectedCase,
    capture: &execute::CaseCapture,
    population: &population::PopulationObservation,
    program: &[u8],
    dimension: Dimension,
    value: u64,
    method: Method,
) -> Result<Probe> {
    let mint = population
        .mint_config
        .as_ref()
        .context("missing source mint")?;
    let (source, minimum) = observed_source(case, capture, mint)?;
    let original: u64 = source.raw_balance.parse()?;
    let mut plan = case.case_plan.clone();
    let amount = if dimension == Dimension::SourceAmount {
        value
    } else {
        original
    };
    ensure!(
        amount >= minimum && amount <= original,
        "derived amount outside observed-source domain"
    );
    if dimension == Dimension::SourceAmount {
        ensure!(
            amount < original,
            "unchanged source amount is not a derived variant"
        );
    }
    plan.amount_decimal = Some(decode::decimal_amount(amount, mint.decimals));
    if dimension == Dimension::ProposedReserve {
        let declared: u64 = case.case_plan.reserve.funded_replacement_raw.parse()?;
        let expected: u64 = expected_output(original, &case.case_plan.terms)?
            .replacement_gross_raw
            .parse()?;
        ensure!(
            value >= declared
                && value <= declared.max(expected)
                && value <= replacement_supply(case, capture)?,
            "derived reserve outside operator-declared upward search domain"
        );
        plan.reserve.funded_replacement_raw = value.to_string();
    }
    plan.validate()?;
    let context = demo::ConversionContext {
        genesis_hash: population.acquisition.genesis_hash.clone(),
        minimum_slot: case.discovery_slot,
        owner: case.authority.clone(),
        source_program: mint.token_program.clone(),
        source_decimals: mint.decimals,
        amount,
    };
    let mut built = demo::build_coherent_rebound(
        &plan,
        &case.case_plan_sha256,
        &context,
        &evidence(capture)?,
        program,
    )?;
    if dimension == Dimension::SourceAmount {
        typed_amount_mutation(&mut built, &plan, original, amount)?;
    } else {
        for fixture in &mut built.accounts {
            if fixture.address == built.overlay.reserve_vault {
                fixture.origin = AccountOrigin::DerivedProposedForSearch;
                fixture.derivation = Some(format!("Operator proposed reserve {} -> {value} through registered token-account fixture constructor", case.case_plan.reserve.funded_replacement_raw));
            }
        }
        super::validate_origins(&built.accounts)?;
    }
    demo::assert_candidate_program_identity(&built.plan.programs, program, &sha256(program))?;
    let fixture = digest(&(
        &built.plan.accounts,
        &built.plan.watch,
        ProbeClock::from(&built.plan.clock),
        ProbeMessage::from(&built.plan.message),
        &built.accounts,
    ))?;
    let execution_plan = digest(&(
        VERSION,
        case.case_plan_sha256.as_str(),
        sha256(program),
        dimension,
        value,
        &fixture,
    ))?;
    // A timed-out VM produces no finding and aborts the entire search. The
    // worker owns its bank and cannot run a second search probe concurrently.
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("eplyx-derived-search-vm".into())
        .spawn(move || {
            let result = (|| -> Result<Probe> {
                let execution = executor::execute_probe_message(
                    &built.plan.accounts,
                    &built.plan.watch,
                    built.plan.clock.clone(),
                    &built.plan.programs,
                    built.plan.message.clone(),
                )?;
                let reconciliation = demo::reconcile(&plan, &built, amount, &execution)?;
                let status = if !reconciliation.reconciled {
                    PathStatus::Indeterminate
                } else if execution.success {
                    PathStatus::Proven
                } else {
                    PathStatus::Failed
                };
                let failure_signature =
                    signature(&execution, !execution.success && reconciliation.reconciled);
                Ok(Probe {
                    dimension,
                    value_raw: value.to_string(),
                    method,
                    status,
                    execution_plan_sha256: execution_plan,
                    execution_fixture_sha256: fixture,
                    failure_signature,
                    origin: match dimension {
                        Dimension::SourceAmount => "DerivedForSearch from observed account and mint, reconstructed through typed Token-2022 state".into(),
                        Dimension::ProposedReserve => "DerivedForSearch from ProposedByOperator reserve, constructed through registered token fixture".into(),
                    },
                })
            })();
            let _ = sender.send(result);
        })?;
    receiver
        .recv_timeout(std::time::Duration::from_secs(120))
        .context("derived search VM timed out or stopped")?
}

fn minimum_insufficient_amount(
    min: u64,
    max: u64,
    reserve: u64,
    plan: &ConversionPlan,
) -> Result<Option<(u64, Method)>> {
    plan.validate()?;
    let output = |amount| -> Result<u64> {
        Ok(expected_output(amount, &plan.terms)?
            .replacement_gross_raw
            .parse()?)
    };
    // The registered fixed-ratio adapter applies a nondecreasing integer fee and
    // ratio, then floor/ceiling rounding. No other mechanism may assert this.
    let monotonic = plan.mechanism == super::MechanismId::EplyxDemoCandidateConversion;
    search_integer_boundary(min, max, reserve, output, monotonic)
}

fn search_integer_boundary(
    min: u64,
    max: u64,
    reserve: u64,
    output: impl Fn(u64) -> Result<u64>,
    monotonic_established: bool,
) -> Result<Option<(u64, Method)>> {
    ensure!(min > 0 && min <= max, "empty source search domain");
    if !monotonic_established {
        for amount in min..=max.min(min.saturating_add(MAX_BOUNDARY as u64 - 1)) {
            if output(amount)? > reserve {
                return Ok(Some((amount, Method::OrderedProbe)));
            }
        }
        return Ok(None);
    }
    if output(max)? <= reserve {
        return Ok(None);
    }
    let mut low = min;
    let mut high = max;
    while low < high {
        let mid = low + (high - low) / 2;
        if output(mid)? > reserve {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    Ok(Some((low, Method::BinarySearch)))
}

fn amount_minimal_verified(
    minimum: u64,
    boundary: u64,
    witnesses: &[Probe],
    parent_signature: Option<&FailureSignature>,
) -> bool {
    let Some(failing) = witnesses.last() else {
        return false;
    };
    if failing.dimension != Dimension::SourceAmount
        || failing.value_raw != boundary.to_string()
        || failing.status != PathStatus::Failed
    {
        return false;
    }
    let Some(signature) = failing.failure_signature.as_ref() else {
        return false;
    };
    if parent_signature.is_some_and(|original| original != signature) {
        return false;
    }
    boundary == minimum
        || witnesses.iter().any(|p| {
            p.dimension == Dimension::SourceAmount
                && p.value_raw == (boundary - 1).to_string()
                && p.status == PathStatus::Proven
        })
}

fn reserve_boundary_verified(output: u64, witnesses: &[Probe]) -> bool {
    output > 0
        && witnesses.iter().any(|p| {
            p.dimension == Dimension::ProposedReserve
                && p.value_raw == (output - 1).to_string()
                && p.status == PathStatus::Failed
                && p.failure_signature.is_some()
        })
        && witnesses.iter().any(|p| {
            p.dimension == Dimension::ProposedReserve
                && p.value_raw == output.to_string()
                && p.status == PathStatus::Proven
        })
}

fn wave_files(wave: usize) -> (String, String) {
    (
        format!("wave-{wave}.plan.json"),
        format!("wave-{wave}.cases.json"),
    )
}

fn freeze_file(wave: usize) -> String {
    format!("wave-{wave}.freeze.json")
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn assert_frozen_case(
    case_id: &str,
    entity_id: &str,
    token_account: &str,
    case_plan_sha256: &str,
    capture: &execute::CaseCapture,
) -> Result<()> {
    ensure!(
        capture.case_id == case_id
            && capture.entity_id == entity_id
            && capture.token_account == token_account
            && capture.case_plan_sha256 == case_plan_sha256,
        "search wave case replaced within frozen wave"
    );
    Ok(())
}

pub(crate) fn select_wave(
    base: &StressTestPlan,
    population: &population::PopulationObservation,
    previous: &[CaseResult],
    wave: usize,
    frozen_at: &str,
) -> Result<StressTestPlan> {
    let mint = population
        .mint_config
        .as_ref()
        .context("missing mint for search selection")?;
    let (_, buckets) = select::buckets(population)?;
    let excluded: BTreeSet<&str> = previous.iter().map(|r| r.token_account.as_str()).collect();
    let mut covered: BTreeSet<String> = previous
        .iter()
        .filter(|r| r.execution_performed)
        .map(|r| r.state_shape_sha256.clone())
        .collect();
    let failures: Vec<u64> = previous
        .iter()
        .filter(|r| r.status == PathStatus::Failed)
        .filter_map(|r| {
            r.detail["revalidation"]["final_amount_raw"]
                .as_str()?
                .parse()
                .ok()
        })
        .collect();
    let mut candidates = Vec::new();
    for entity in population.positive_entities() {
        if excluded.contains(entity.token_account.as_str()) {
            continue;
        }
        let dimensions = classify::dimensions(entity, mint)?;
        if classify::eligibility(&dimensions).0 != Eligibility::ExecutableCandidate {
            continue;
        }
        candidates.push((
            entity,
            entity.balance()?,
            classify::shape_key(&dimensions)?,
            classify::shape_label(&dimensions),
        ));
    }
    let mut selected = Vec::new();
    let cap = if wave == 3 { 5 } else { 10 };
    while selected.len() < cap && !candidates.is_empty() {
        candidates.sort_by(|a, b| {
            let key = |x: &(&crate::stress::StressEntity, u64, String, String)| {
                (
                    if covered.contains(&x.2) { 1 } else { 0 },
                    failures
                        .iter()
                        .map(|f| f.abs_diff(x.1))
                        .min()
                        .unwrap_or(u64::MAX),
                    std::cmp::Reverse(x.1),
                    x.0.token_account.clone(),
                )
            };
            key(a).cmp(&key(b))
        });
        let (entity, balance, shape, label) = candidates.remove(0);
        let new_shape = covered.insert(shape.clone());
        let reason = if new_shape {
            SelectionReason::NewStateShape
        } else {
            SelectionReason::HighestRemainingBalance
        };
        let amount_decimal = decode::decimal_amount(balance, mint.decimals);
        let mut case_plan = base.candidate_plan.clone();
        case_plan.source_account = entity.token_account.clone();
        case_plan.amount_mode = AmountMode::Custom;
        case_plan.amount_decimal = Some(amount_decimal.clone());
        case_plan.validate()?;
        let order = selected.len();
        selected.push(SelectedCase {
            case_id: format!("search-w{wave}-{order:02}"),
            selection_order: order,
            selection_reason: reason,
            selection_detail: match reason {
                SelectionReason::NewStateShape => "Untested exact account adds a discovered executable state shape.".into(),
                SelectionReason::HighestRemainingBalance if !failures.is_empty() => "Untested exact account is a nearest balance neighbor to a prior-wave executed failure.".into(),
                _ => "Untested exact account has an extreme balance in the supported class.".into(),
            },
            entity_id: entity.entity_id.clone(),
            token_account: entity.token_account.clone(),
            authority: entity.authority.clone(),
            authority_model: entity.authority_model.clone(),
            state_shape_sha256: shape,
            shape_label: label,
            balance_bucket: *buckets.get(&entity.token_account).context("missing frozen bucket")?,
            observed_balance_raw: balance.to_string(),
            discovery_slot: entity.token_account_evidence.slot,
            selected_amount_raw: balance.to_string(),
            selected_amount_decimal: amount_decimal,
            amount_policy: FULL_AT_FINAL_POLICY.into(),
            amount_capped: false,
            case_plan_sha256: case_plan.sha256()?,
            case_plan,
        });
    }
    let mut plan = base.clone();
    plan.selected = selected;
    plan.selector_version = VERSION.into();
    plan.ordering_rule = "Prior-wave outcomes only; new executable shape, nearest earlier failed balance, then highest balance and account address. Every selected identity is frozen before capture.".into();
    plan.selection_strategy = vec![plan.ordering_rule.clone()];
    plan.frozen_at = frozen_at.into();
    Ok(plan)
}

#[allow(clippy::too_many_arguments)]
fn replay_wave(
    directory: &Path,
    wave: usize,
    base: &StressTestPlan,
    population: &population::PopulationObservation,
    original_capture: &population::Capture,
    previous: &[CaseResult],
    program: &[u8],
    program_sha256: &str,
    package_sha256: &str,
    parent_plan_sha256: &str,
) -> Result<(WaveRecord, Vec<CaseResult>)> {
    let (plan_file, capture_file) = wave_files(wave);
    let plan_bytes = read(directory, &plan_file, 16 * 1024 * 1024)?;
    let saved_plan: StressTestPlan = serde_json::from_slice(&plan_bytes)?;
    let expected = select_wave(base, population, previous, wave, &saved_plan.frozen_at)?;
    ensure!(
        saved_plan == expected
            && !saved_plan.selected.is_empty()
            && saved_plan.selected.len() <= MAX_OBSERVED,
        "search wave selection was rewritten after freezing"
    );
    ensure!(
        canonical(&saved_plan)?.as_bytes() == plan_bytes,
        "search wave plan is not canonical"
    );
    let freeze_bytes = read(directory, &freeze_file(wave), 16 * 1024)?;
    let expected_freeze = freeze(wave, &saved_plan, package_sha256, parent_plan_sha256)?;
    ensure!(
        freeze_bytes == canonical(&expected_freeze)?.into_bytes(),
        "search wave package, selection or search version changed after freeze"
    );
    let case_bytes = read(directory, &capture_file, 192 * 1024 * 1024)?;
    let bundle: execute::CaptureBundle = serde_json::from_slice(&case_bytes)?;
    ensure!(
        bundle.schema_version == 3
            && bundle.kind == execute::BUNDLE_KIND
            && bundle.run_id == saved_plan.run_id
            && bundle.stress_id == saved_plan.stress_id
            && bundle.population_capture_sha256 == saved_plan.population_capture_sha256
            && bundle.stress_plan_sha256 == sha256(&plan_bytes)
            && bundle.cases.len() == saved_plan.selected.len(),
        "search wave capture differs from frozen plan"
    );
    let freeze_time = chrono::DateTime::parse_from_rfc3339(&saved_plan.frozen_at)?;
    let bundle_start = chrono::DateTime::parse_from_rfc3339(&bundle.started_at)?;
    let bundle_end = chrono::DateTime::parse_from_rfc3339(&bundle.completed_at)?;
    ensure!(
        freeze_time <= bundle_start && bundle_start <= bundle_end,
        "search wave capture predates its frozen plan"
    );
    let mut results = Vec::new();
    for (case, capture) in saved_plan.selected.iter().zip(&bundle.cases) {
        assert_frozen_case(
            &case.case_id,
            &case.entity_id,
            &case.token_account,
            &case.case_plan_sha256,
            capture,
        )?;
        ensure!(
            capture.observations.len()
                < saved_plan.budget.rpc_requests_per_case + super::coherence::MAX_FINAL_ATTEMPTS,
            "search wave case exceeded capture budget"
        );
        let case_start = chrono::DateTime::parse_from_rfc3339(&capture.started_at)?;
        let case_end = chrono::DateTime::parse_from_rfc3339(&capture.completed_at)?;
        ensure!(
            bundle_start <= case_start && case_start <= case_end && case_end <= bundle_end,
            "search wave case interval is outside frozen capture"
        );
        let mut prior = case_start;
        for record in &capture.observations {
            let start = chrono::DateTime::parse_from_rfc3339(&record.started_at)?;
            let end = chrono::DateTime::parse_from_rfc3339(&record.completed_at)?;
            ensure!(
                prior <= start
                    && start <= end
                    && end <= case_end
                    && record.result.is_some() != record.error.is_some(),
                "invalid search wave RPC observation interval or outcome"
            );
            prior = end;
        }
        results.push(execute::case_result(
            case,
            capture,
            population,
            Some(original_capture),
            &saved_plan,
            program,
            program_sha256,
        )?);
    }
    let record = WaveRecord {
        wave,
        freeze_sha256: sha256(&freeze_bytes),
        frozen_plan_sha256: sha256(&plan_bytes),
        case_capture_sha256: sha256(&case_bytes),
        selected_exact_accounts: saved_plan
            .selected
            .iter()
            .map(|c| c.token_account.clone())
            .collect(),
        selection_reasons: saved_plan
            .selected
            .iter()
            .map(|c| c.selection_detail.clone())
            .collect(),
        result_sha256: results.iter().map(|r| r.result_sha256.clone()).collect(),
        outcomes: results.iter().map(|r| r.status).collect(),
        failure_signatures: results
            .iter()
            .map(observed_signature)
            .collect::<Result<Vec<_>>>()?,
        next_search_decision: if wave == 3 {
            "Observed execution budget reached; move to typed derived boundary search.".into()
        } else {
            "Recompute the next frozen selection from all prior-wave outcomes and remaining observed accounts.".into()
        },
    };
    Ok((record, results))
}

fn capture_live_waves(
    package: &package::ValidatedPackage,
    package_directory: &Path,
    parent: &Path,
    output: &Path,
    rpc_url: &str,
) -> Result<()> {
    package_preflight::replay(package_directory, parent)?;
    let population_bytes = read(parent, "population.capture.json", 192 * 1024 * 1024)?;
    let plan_bytes = read(parent, "stress.plan.json", 192 * 1024 * 1024)?;
    let case_bytes = read(parent, "stress.cases.json", 192 * 1024 * 1024)?;
    let base: StressTestPlan = serde_json::from_slice(&plan_bytes)?;
    ensure!(
        base.schema_version == 2,
        "live search requires coherent rebinding"
    );
    let population = population::evaluate_bytes(&population_bytes, &base.budget)?;
    let original_capture: population::Capture = serde_json::from_slice(&population_bytes)?;
    let bundle: execute::CaptureBundle = serde_json::from_slice(&case_bytes)?;
    let mut previous = Vec::new();
    for (case, capture) in base.selected.iter().zip(&bundle.cases) {
        previous.push(execute::case_result(
            case,
            capture,
            &population,
            Some(&original_capture),
            &base,
            &package.program,
            &package.program_sha256,
        )?);
    }
    let rpc = HttpSolanaRpc::bounded_population(
        rpc_url,
        base.budget.max_response_bytes,
        base.budget.case_timeout_seconds,
    )?;
    for wave in 1..=3 {
        let frozen_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let plan = select_wave(&base, &population, &previous, wave, &frozen_at)?;
        if plan.selected.is_empty() {
            break;
        }
        let (plan_file, case_file) = wave_files(wave);
        let plan_path = output.join(plan_file);
        plan.save(&plan_path)?; // create_new and sync: freeze before first RPC.
        let frozen = freeze(
            wave,
            &plan,
            &package.transition_package_sha256,
            &sha256(&plan_bytes),
        )?;
        write_new(
            &output.join(freeze_file(wave)),
            canonical(&frozen)?.as_bytes(),
        )?;
        let capture = execute::capture_cases(
            &plan,
            &plan.sha256()?,
            &population
                .mint_config
                .as_ref()
                .context("missing mint")?
                .token_program,
            &rpc,
        )?;
        execute::save(&capture, &output.join(case_file))?;
        let (_, results) = replay_wave(
            output,
            wave,
            &base,
            &population,
            &original_capture,
            &previous,
            &package.program,
            &package.program_sha256,
            &package.transition_package_sha256,
            &sha256(&plan_bytes),
        )?;
        previous.extend(results);
    }
    Ok(())
}

fn record_observed(
    package: &package::ValidatedPackage,
    run_id: &str,
    case: &SelectedCase,
    capture: &execute::CaseCapture,
    result: &CaseResult,
) -> Result<Option<Counterexample>> {
    let Some(failure_signature) = observed_signature(result)? else {
        return Ok(None);
    };
    Ok(Some(Counterexample::Observed {
        id: format!("observed-{}", case.case_id),
        parent_run: run_id.into(),
        transition_package_sha256: package.transition_package_sha256.clone(),
        candidate_program_sha256: package.program_sha256.clone(),
        observed_source_account: case.token_account.clone(),
        observed_state_digest: digest(capture.observations.last().context("missing final bank")?)?,
        observed_amount_raw: result.detail["revalidation"]["final_amount_raw"].as_str().context("missing final amount")?.into(),
        execution_plan_sha256: result.detail["execution_plan_sha256"].as_str().context("missing execution plan")?.into(),
        execution_fixture_sha256: result.execution_fixture_sha256.clone().context("missing fixture digest")?,
        failure_signature,
        provenance: "ObservedOnChain final coherent account plus actual local VM failure".into(),
        limitations: "Exact captured account and local assumed signer only; no peer, population or official transition proof.".into(),
    }))
}

fn compute(
    package: &package::ValidatedPackage,
    package_directory: &Path,
    result: &Path,
    search_directory: Option<&Path>,
) -> Result<SearchResult> {
    // The original report is independently replayed from captures and the exact
    // packaged SBF. Saved statuses below are never accepted as proof.
    let verified_report = package_preflight::replay(package_directory, result)?;
    let population_bytes = read(result, "population.capture.json", 192 * 1024 * 1024)?;
    let plan_bytes = read(result, "stress.plan.json", 192 * 1024 * 1024)?;
    let case_bytes = read(result, "stress.cases.json", 192 * 1024 * 1024)?;
    let plan: StressTestPlan = serde_json::from_slice(&plan_bytes)?;
    let population = population::evaluate_bytes(&population_bytes, &plan.budget)?;
    let original_capture: population::Capture = serde_json::from_slice(&population_bytes)?;
    let bundle: execute::CaptureBundle = serde_json::from_slice(&case_bytes)?;
    ensure!(
        plan.schema_version == 2 && bundle.schema_version == 3,
        "counterexample search requires coherent final-state rebinding"
    );
    ensure!(
        plan.population_capture_sha256 == sha256(&population_bytes)
            && bundle.stress_plan_sha256 == sha256(&plan_bytes),
        "search parent binding mismatch"
    );
    let mut budget = Budget::new();
    let mut trace = Vec::new();
    let mut counterexamples = Vec::new();
    let mut seeds: Vec<(SelectedCase, execute::CaseCapture, CaseResult)> = Vec::new();
    let mut previous = Vec::new();
    for (case, capture) in plan.selected.iter().zip(&bundle.cases) {
        let result = execute::case_result(
            case,
            capture,
            &population,
            Some(&original_capture),
            &plan,
            &package.program,
            &package.program_sha256,
        )?;
        let reported = verified_report["stress_results"]
            .as_array()
            .context("missing verified results")?
            .iter()
            .find(|r| r["case_id"] == case.case_id)
            .context("verified case missing")?;
        ensure!(
            reported["result_sha256"] == result.result_sha256,
            "search seed differs from package replay"
        );
        if let Some(finding) = record_observed(package, &plan.run_id, case, capture, &result)? {
            counterexamples.push(finding);
        }
        if matches!(result.status, PathStatus::Proven | PathStatus::Failed) {
            seeds.push((case.clone(), capture.clone(), result.clone()));
        }
        previous.push(result);
    }
    let observed_wave = plan
        .selected
        .iter()
        .map(|c| c.token_account.clone())
        .collect();
    let mut additional_waves = Vec::new();
    if let Some(directory) = search_directory {
        for wave in 1..=3 {
            let (plan_file, case_file) = wave_files(wave);
            let has_plan = directory.join(&plan_file).exists();
            let has_capture = directory.join(&case_file).exists();
            let has_freeze = directory.join(freeze_file(wave)).exists();
            ensure!(
                has_plan == has_capture && has_plan == has_freeze,
                "incomplete frozen search wave"
            );
            if !has_plan {
                for later in (wave + 1)..=4 {
                    let (next_plan, next_capture) = wave_files(later);
                    ensure!(
                        !directory.join(next_plan).exists()
                            && !directory.join(next_capture).exists()
                            && !directory.join(freeze_file(later)).exists(),
                        "search wave gap or execution beyond the observed budget"
                    );
                }
                break;
            }
            let (record, results) = replay_wave(
                directory,
                wave,
                &plan,
                &population,
                &original_capture,
                &previous,
                &package.program,
                &package.program_sha256,
                &package.transition_package_sha256,
                &sha256(&plan_bytes),
            )?;
            budget.observed_cases_selected += record.selected_exact_accounts.len();
            budget.observed_executions += results.iter().filter(|r| r.execution_performed).count();
            ensure!(
                budget.observed_cases_selected <= MAX_OBSERVED
                    && budget.observed_executions <= MAX_OBSERVED,
                "observed search budget exceeded"
            );
            let wave_plan: StressTestPlan =
                serde_json::from_slice(&read(directory, &plan_file, 16 * 1024 * 1024)?)?;
            let wave_capture: execute::CaptureBundle =
                serde_json::from_slice(&read(directory, &case_file, 192 * 1024 * 1024)?)?;
            for ((case, capture), result) in wave_plan
                .selected
                .iter()
                .zip(&wave_capture.cases)
                .zip(&results)
            {
                if let Some(finding) =
                    record_observed(package, &plan.run_id, case, capture, result)?
                {
                    counterexamples.push(finding);
                }
                if matches!(result.status, PathStatus::Proven | PathStatus::Failed) {
                    seeds.push((case.clone(), capture.clone(), result.clone()));
                }
            }
            previous.extend(results);
            additional_waves.push(record);
        }
    }
    // Select one exact executed seed deterministically: a failed case first,
    // then highest final amount and address. No outcome changes Wave 0 identities.
    seeds.sort_by(|a, b| {
        let rank = |r: &CaseResult| if r.status == PathStatus::Failed { 0 } else { 1 };
        rank(&a.2)
            .cmp(&rank(&b.2))
            .then_with(|| {
                b.2.detail["revalidation"]["final_amount_raw"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<u64>()
                    .unwrap_or(0)
                    .cmp(
                        &a.2.detail["revalidation"]["final_amount_raw"]
                            .as_str()
                            .unwrap_or("0")
                            .parse::<u64>()
                            .unwrap_or(0),
                    )
            })
            .then_with(|| a.0.token_account.cmp(&b.0.token_account))
    });
    let mut derived_domain = None;
    if let Some((case, capture, seed)) = seeds.first() {
        let mint = population
            .mint_config
            .as_ref()
            .context("missing source mint")?;
        let (source, minimum) = observed_source(case, capture, mint)?;
        let original: u64 = seed.detail["revalidation"]["final_amount_raw"]
            .as_str()
            .context("missing seed amount")?
            .parse()?;
        ensure!(
            source.raw_balance == original.to_string(),
            "seed source amount differs from final capture"
        );
        let reserve: u64 = case.case_plan.reserve.funded_replacement_raw.parse()?;
        let supply = replacement_supply(case, capture)?;
        let output: u64 = expected_output(original, &case.case_plan.terms)?
            .replacement_gross_raw
            .parse()?;
        derived_domain = Some(DerivedDomain {
            observed_source_account: case.token_account.clone(),
            seed_selection_reason: "Executed failure first; then highest final captured amount; then token-account address. Deterministic across all prior observed waves.".into(),
            source_amount_min_raw: minimum.to_string(),
            source_amount_max_raw: if original > minimum { original - 1 } else { original }.to_string(),
            source_amount_mutation_available: original > minimum,
            proposed_reserve_min_raw: reserve.to_string(),
            proposed_reserve_max_raw: if output > reserve && reserve <= supply {
                output.min(supply)
            } else {
                reserve
            }
            .to_string(),
            replacement_mint_captured_supply_raw: supply.to_string(),
        });
        let parent_signature = observed_signature(seed)?;
        let amount_boundary = if original > minimum {
            minimum_insufficient_amount(minimum, original - 1, reserve, &case.case_plan)?
        } else {
            None
        };
        if let Some((boundary, method)) = amount_boundary {
            // Verify both sides with the actual candidate; host arithmetic alone
            // can locate a proposed boundary but cannot grant a counterexample.
            let mut witnesses = Vec::new();
            if boundary > minimum {
                budget.charge_boundary(true)?;
                witnesses.push(probe(
                    case,
                    capture,
                    &population,
                    &package.program,
                    Dimension::SourceAmount,
                    boundary - 1,
                    method,
                )?);
            }
            budget.charge_boundary(true)?;
            witnesses.push(probe(
                case,
                capture,
                &population,
                &package.program,
                Dimension::SourceAmount,
                boundary,
                method,
            )?);
            let failing = witnesses.last().context("missing boundary witness")?;
            let last_passing = witnesses
                .iter()
                .rev()
                .skip(1)
                .find(|p| p.status == PathStatus::Proven)
                .map(|p| p.value_raw.clone());
            if let Some(failure_signature) = failing.failure_signature.clone() {
                let preserved = parent_signature.as_ref().map(|s| s == &failure_signature);
                counterexamples.push(Counterexample::Derived {
                    id: format!("derived-amount-{}", case.case_id), parent_run: plan.run_id.clone(),
                    transition_package_sha256: package.transition_package_sha256.clone(),
                    candidate_program_sha256: package.program_sha256.clone(),
                    observed_source_account: case.token_account.clone(),
                    observed_state_digest: digest(capture.observations.last().context("missing final bank")?)?,
                    observed_amount_raw: original.to_string(), search_dimension: Dimension::SourceAmount,
                    original_value_raw: original.to_string(), derived_value_raw: boundary.to_string(),
                    execution_plan_sha256: failing.execution_plan_sha256.clone(),
                    execution_fixture_sha256: failing.execution_fixture_sha256.clone(),
                    failure_signature, original_failure_signature: parent_signature.clone().map(Box::new),
                    signature_preserved: preserved,
                    minimized: amount_minimal_verified(minimum, boundary, &witnesses, parent_signature.as_ref()),
                    last_passing_value_raw: last_passing, first_passing_value_raw: None,
                    minimization_trace: witnesses.clone(),
                    provenance: "DerivedForSearch from ObservedOnChain source account and mint".into(),
                    limitations: "Typed local state variant; this exact amount and adjusted mint supply are not claimed to exist on mainnet.".into(),
                });
            }
            trace.extend(witnesses);
        }
        if output > reserve && output <= supply {
            let mut witnesses = Vec::new();
            budget.charge_boundary(false)?;
            witnesses.push(probe(
                case,
                capture,
                &population,
                &package.program,
                Dimension::ProposedReserve,
                output - 1,
                Method::OrderedProbe,
            )?);
            budget.charge_boundary(false)?;
            witnesses.push(probe(
                case,
                capture,
                &population,
                &package.program,
                Dimension::ProposedReserve,
                output,
                Method::OrderedProbe,
            )?);
            if let Some(failed) = witnesses
                .iter()
                .find(|p| p.failure_signature.is_some() && p.value_raw != reserve.to_string())
            {
                let failure_signature = failed
                    .failure_signature
                    .clone()
                    .context("failure disappeared")?;
                counterexamples.push(Counterexample::Derived {
                    id: format!("derived-reserve-{}", case.case_id), parent_run: plan.run_id.clone(),
                    transition_package_sha256: package.transition_package_sha256.clone(),
                    candidate_program_sha256: package.program_sha256.clone(),
                    observed_source_account: case.token_account.clone(),
                    observed_state_digest: digest(capture.observations.last().context("missing final bank")?)?,
                    observed_amount_raw: original.to_string(), search_dimension: Dimension::ProposedReserve,
                    original_value_raw: reserve.to_string(), derived_value_raw: failed.value_raw.clone(),
                    execution_plan_sha256: failed.execution_plan_sha256.clone(),
                    execution_fixture_sha256: failed.execution_fixture_sha256.clone(),
                    failure_signature: failure_signature.clone(), original_failure_signature: parent_signature.clone().map(Box::new),
                    signature_preserved: parent_signature.as_ref().map(|s| s == &failure_signature),
                    minimized: reserve_boundary_verified(output, &witnesses)
                        && parent_signature.as_ref().is_none_or(|s| s == &failure_signature),
                    last_passing_value_raw: None, first_passing_value_raw: witnesses.last().filter(|p| p.status == PathStatus::Proven).map(|p| p.value_raw.clone()),
                    minimization_trace: witnesses.clone(),
                    provenance: "DerivedForSearch from ProposedByOperator reserve".into(),
                    limitations: "Locally proposed reserve variant; no derived reserve is represented as captured mainnet state.".into(),
                });
            }
            trace.extend(witnesses);
        }
    }
    let conclusion = if counterexamples.is_empty() {
        NO_FINDING.into()
    } else {
        format!("COUNTEREXAMPLE FOUND: {} exact observed or derived local failures within the recorded search domain.", counterexamples.len())
    };
    Ok(SearchResult {
        version: VERSION.into(), parent_run: plan.run_id,
        transition_package_sha256: package.transition_package_sha256.clone(),
        candidate_program_sha256: package.program_sha256.clone(),
        population_capture_sha256: sha256(&population_bytes), stress_plan_sha256: sha256(&plan_bytes),
        case_capture_sha256: sha256(&case_bytes), budget,
        search_domain: format!("Wave 0 exact selected coherent final states plus {} frozen additional observed waves (at most 25 account selections); derived source amount and proposed reserve are limited by derived_domain, including captured replacement mint supply. One deterministic seed and at most two exact VM boundary witnesses per dimension.", additional_waves.len()),
        derived_domain,
        observed_wave, additional_waves, trace, counterexamples, conclusion,
        official_transition: "NotTested".into(), funds_moved: false,
    })
}

pub fn run(
    package_directory: &Path,
    parent_result: &Path,
    output_directory: &Path,
    live_observed: bool,
) -> Result<SearchResult> {
    let package = package::load(package_directory)?;
    let rpc_url = if live_observed {
        Some(
            std::env::var("SOLANA_RPC_URL")
                .context("live observed search requires server-controlled SOLANA_RPC_URL")?,
        )
    } else {
        None
    };
    // The local VM and offline replay receive no RPC credential environment.
    std::env::remove_var("SOLANA_RPC_URL");
    fs::create_dir(output_directory).context("search output directory must not already exist")?;
    if let Some(rpc_url) = rpc_url.as_deref() {
        capture_live_waves(
            &package,
            package_directory,
            parent_result,
            output_directory,
            rpc_url,
        )?;
    }
    let result = compute(
        &package,
        package_directory,
        parent_result,
        Some(output_directory),
    )?;
    let bytes = canonical(&result)?;
    ensure!(
        bytes.len() <= MAX_ARTIFACT,
        "search artifact exceeds size bound"
    );
    fs::write(output_directory.join("counterexamples.json"), bytes)?;
    Ok(result)
}

pub fn replay(
    package_directory: &Path,
    parent_result: &Path,
    search_directory: &Path,
) -> Result<SearchResult> {
    let package = package::load(package_directory)?;
    std::env::remove_var("SOLANA_RPC_URL");
    let recomputed = compute(
        &package,
        package_directory,
        parent_result,
        Some(search_directory),
    )?;
    let saved = read(search_directory, "counterexamples.json", MAX_ARTIFACT)?;
    verify_recomputed_artifact(&saved, &recomputed)?;
    Ok(recomputed)
}

fn verify_recomputed_artifact(saved: &[u8], recomputed: &SearchResult) -> Result<()> {
    ensure!(
        saved == canonical(recomputed)?.as_bytes(),
        "counterexample search trace differs from offline re-execution"
    );
    Ok(())
}

/// Verify the saved search before its separate finding may affect deployment.
pub fn gate(
    package_directory: &Path,
    parent_result: &Path,
    search_directory: &Path,
    policy: super::package_gate::Policy,
) -> Result<super::package_gate::DeploymentGate> {
    let verified = replay(package_directory, parent_result, search_directory)?;
    let report = package_preflight::replay(package_directory, parent_result)?;
    super::package_gate::evaluate_with_counterexamples(&report, policy, &verified)
}

pub fn gate_finding(result: &SearchResult) -> Value {
    serde_json::json!({
        "kind":"CounterexampleFinding",
        "status": if result.counterexamples.is_empty() { "NoFindingWithinBudget" } else { "Blocking" },
        "reason": if result.counterexamples.is_empty() { result.conclusion.clone() } else { "An exact observed state or a valid derived state in the declared search domain failed actual candidate execution.".into() },
        "transition_package_sha256":result.transition_package_sha256,
        "parent_run":result.parent_run,
        "observed_count":result.counterexamples.iter().filter(|c| matches!(c, Counterexample::Observed { .. })).count(),
        "derived_count":result.counterexamples.iter().filter(|c| matches!(c, Counterexample::Derived { .. })).count(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AccountSnapshot, NamedAccount};
    use solana_address::Address;
    use solana_program_pack::Pack;
    use spl_token_2022_interface::extension::{BaseStateWithExtensionsMut, ExtensionType};
    use spl_token_2022_interface::state::AccountState;

    fn example_plan() -> ConversionPlan {
        package::load(&crate::repo_root().join("examples/transitions/demo-underfunded"))
            .unwrap()
            .conversion_plan()
            .unwrap()
    }
    fn fake_probe(
        dimension: Dimension,
        value: u64,
        status: PathStatus,
        signature: Option<FailureSignature>,
    ) -> Probe {
        Probe {
            dimension,
            value_raw: value.to_string(),
            method: Method::BinarySearch,
            status,
            execution_plan_sha256: "p".repeat(64),
            execution_fixture_sha256: "f".repeat(64),
            failure_signature: signature,
            origin: "DerivedForSearch".into(),
        }
    }
    fn failure(code: u32) -> FailureSignature {
        FailureSignature {
            stage: "ExecutedInstruction".into(),
            program: demo::PROGRAM_ID.into(),
            instruction_error: format!("InstructionError(0, Custom({code}))"),
            relevant_log: Some(format!("failed: custom program error: 0x{code:x}")),
            rollback_verified: true,
        }
    }
    #[test]
    fn budget_is_server_fixed_and_enforced() {
        assert_eq!(MAX_OBSERVED, 25, "observed_budget_is_server_fixed");
        assert_eq!(MAX_BOUNDARY, 20, "boundary_budget_is_server_fixed");
        assert_eq!(MAX_MINIMIZATION, 20, "minimization_budget_is_server_fixed");
        let mut budget = Budget::new();
        for _ in 0..MAX_BOUNDARY {
            budget.charge_boundary(false).unwrap();
        }
        assert!(
            budget.charge_boundary(false).is_err(),
            "budget_excess_must_fail"
        );
    }
    #[test]
    fn provenance_variants_cannot_deserialize_as_each_other() {
        let derived = Counterexample::Derived {
            id: "derived-1".into(),
            parent_run: "run-1".into(),
            transition_package_sha256: "p".repeat(64),
            candidate_program_sha256: "c".repeat(64),
            observed_source_account: "account".into(),
            observed_state_digest: "s".repeat(64),
            observed_amount_raw: "100".into(),
            search_dimension: Dimension::SourceAmount,
            original_value_raw: "100".into(),
            derived_value_raw: "51".into(),
            execution_plan_sha256: "e".repeat(64),
            execution_fixture_sha256: "f".repeat(64),
            failure_signature: failure(13),
            original_failure_signature: Some(Box::new(failure(13))),
            signature_preserved: Some(true),
            minimized: true,
            last_passing_value_raw: Some("50".into()),
            first_passing_value_raw: None,
            minimization_trace: vec![],
            provenance: "DerivedForSearch".into(),
            limitations: "Not current chain state".into(),
        };
        assert_eq!(
            derived.claim(),
            "Derived from observed production state",
            "derived_must_not_be_labeled_observed"
        );
        let mut v = serde_json::to_value(&derived).unwrap();
        v["kind"] = "Observed".into();
        assert!(
            serde_json::from_value::<Counterexample>(v).is_err(),
            "derived_cannot_be_relabelled_observed"
        );
        assert_eq!(
            NO_FINDING, "No counterexample found within this search domain and budget.",
            "bounded_no_finding_wording_is_exact"
        );
        assert!(
            !NO_FINDING.contains("exists"),
            "bounded_no_finding_is_not_absence"
        );
    }
    #[test]
    fn frozen_wave_binding_rejects_replacement_and_replay_rejects_tampering() {
        let capture = execute::CaseCapture {
            case_id: "case-1".into(),
            entity_id: "entity-1".into(),
            token_account: "account-1".into(),
            case_plan_sha256: "p".repeat(64),
            started_at: "2026-01-01T00:00:00Z".into(),
            completed_at: "2026-01-01T00:00:01Z".into(),
            observations: vec![],
        };
        assert!(
            assert_frozen_case("case-1", "entity-1", "account-1", &"p".repeat(64), &capture)
                .is_ok()
        );
        assert!(
            assert_frozen_case(
                "case-1",
                "entity-1",
                "replacement",
                &"p".repeat(64),
                &capture
            )
            .is_err(),
            "failed_case_cannot_be_replaced_in_frozen_wave"
        );
        let result = SearchResult {
            version: VERSION.into(),
            parent_run: "run-1".into(),
            transition_package_sha256: "p".repeat(64),
            candidate_program_sha256: "c".repeat(64),
            population_capture_sha256: "a".repeat(64),
            stress_plan_sha256: "b".repeat(64),
            case_capture_sha256: "d".repeat(64),
            budget: Budget::new(),
            search_domain: "bounded".into(),
            derived_domain: None,
            observed_wave: vec![],
            additional_waves: vec![],
            trace: vec![],
            counterexamples: vec![],
            conclusion: NO_FINDING.into(),
            official_transition: "NotTested".into(),
            funds_moved: false,
        };
        let saved = canonical(&result).unwrap();
        verify_recomputed_artifact(saved.as_bytes(), &result).unwrap();
        let changed = saved.replace("run-1", "run-2");
        assert!(
            verify_recomputed_artifact(changed.as_bytes(), &result).is_err(),
            "serialized_search_outcome_is_not_replay_evidence"
        );
    }

    #[test]
    fn monotonic_boundary_is_integer_exact_and_otherwise_uses_ordered_probe() {
        let plan = example_plan();
        let mut plan = plan;
        plan.terms.ratio_numerator = 1;
        plan.terms.ratio_denominator = 1;
        plan.terms.conversion_fee_bps = 0;
        assert_eq!(
            minimum_insufficient_amount(1, 100, 50, &plan).unwrap(),
            Some((51, Method::BinarySearch)),
            "minimum_failing_amount_is_exact"
        );
        assert_eq!(
            minimum_insufficient_amount(60, 100, 50, &plan).unwrap(),
            Some((60, Method::BinarySearch)),
            "minimum_respects_the_typed_source_domain"
        );
        assert_eq!(
            search_integer_boundary(1, 100, 50, |v| Ok(if v == 3 { 51 } else { 0 }), false)
                .unwrap(),
            Some((3, Method::OrderedProbe)),
            "unproven_monotonicity_forces_ordered_probe"
        );
        assert_eq!(
            search_integer_boundary(1, 100, 50, Ok, true).unwrap(),
            Some((51, Method::BinarySearch))
        );
    }
    #[test]
    fn minimum_claim_requires_adjacent_pass_and_same_failure_signature() {
        let original = failure(13);
        let adjacent = [
            fake_probe(Dimension::SourceAmount, 50, PathStatus::Proven, None),
            fake_probe(
                Dimension::SourceAmount,
                51,
                PathStatus::Failed,
                Some(original.clone()),
            ),
        ];
        assert!(amount_minimal_verified(1, 51, &adjacent, Some(&original)));
        assert!(
            !amount_minimal_verified(1, 52, &adjacent, Some(&original)),
            "failing_value_must_match_claim"
        );
        let nonadjacent = [
            fake_probe(Dimension::SourceAmount, 49, PathStatus::Proven, None),
            adjacent[1].clone(),
        ];
        assert!(
            !amount_minimal_verified(1, 51, &nonadjacent, Some(&original)),
            "nonminimal_claim_must_fail"
        );
        assert!(
            !amount_minimal_verified(1, 51, &adjacent, Some(&failure(14))),
            "changed_failure_signature_must_be_reported"
        );
        assert!(
            !amount_minimal_verified(1, 51, &adjacent[1..], Some(&original)),
            "last_passing_amount_required"
        );
        assert!(
            amount_minimal_verified(51, 51, &adjacent[1..], Some(&original)),
            "a_failure_at_the_domain_floor_is_minimal_within_that_domain"
        );
        let reserve = [
            fake_probe(
                Dimension::ProposedReserve,
                50,
                PathStatus::Failed,
                Some(original),
            ),
            fake_probe(Dimension::ProposedReserve, 51, PathStatus::Proven, None),
        ];
        assert!(reserve_boundary_verified(51, &reserve));
        assert!(!reserve_boundary_verified(52, &reserve));
    }
    #[test]
    fn source_mutation_reconstructs_typed_token_and_mint_state() {
        let plan = example_plan();
        let mint: Address = plan.source_mint.parse().unwrap();
        let owner = Address::new_from_array([9; 32]);
        let mut account_bytes = vec![0; Account::LEN];
        Account::pack(
            Account {
                mint,
                owner,
                amount: 100,
                state: AccountState::Initialized,
                ..Account::default()
            },
            &mut account_bytes,
        )
        .unwrap();
        let mut mint_bytes = vec![0; Mint::LEN];
        Mint::pack(
            Mint {
                supply: 1_000,
                decimals: 6,
                is_initialized: true,
                ..Mint::default()
            },
            &mut mint_bytes,
        )
        .unwrap();
        let named = |address: String, data: Vec<u8>| NamedAccount {
            label: "captured-parent".into(),
            address,
            account: AccountSnapshot {
                lamports: 1,
                owner: decode::TOKEN_2022_PROGRAM.into(),
                data,
                executable: false,
                rent_epoch: 0,
            },
        };
        let mut accounts = vec![
            named(plan.source_account.clone(), account_bytes),
            named(plan.source_mint.clone(), mint_bytes),
        ];
        let source_parent = Account::unpack(&accounts[0].account.data).unwrap();
        let mint_parent = Mint::unpack(&accounts[1].account.data).unwrap();
        assert!(
            mutate_token_state(&mut accounts, &plan, 100, 101).is_err(),
            "derived_amount_outside_observed_domain"
        );
        mutate_token_state(&mut accounts, &plan, 100, 51).unwrap();
        assert_eq!(
            StateWithExtensionsMut::<Account>::unpack(&mut accounts[0].account.data)
                .unwrap()
                .base
                .amount,
            51
        );
        assert_eq!(
            StateWithExtensionsMut::<Mint>::unpack(&mut accounts[1].account.data)
                .unwrap()
                .base
                .supply,
            951
        );
        let mut expected_source = source_parent;
        expected_source.amount = 51;
        let mut expected_mint = mint_parent;
        expected_mint.supply = 951;
        assert_eq!(
            Account::unpack(&accounts[0].account.data).unwrap(),
            expected_source,
            "all_non_amount_source_semantics_are_preserved"
        );
        assert_eq!(
            Mint::unpack(&accounts[1].account.data).unwrap(),
            expected_mint,
            "mint_identity_and_non_supply_semantics_are_preserved"
        );
    }

    #[test]
    fn source_mutation_never_drops_below_captured_withheld_fees() {
        let plan = example_plan();
        let mint: Address = plan.source_mint.parse().unwrap();
        let mut bytes = vec![
            0;
            ExtensionType::try_calculate_account_len::<Account>(&[
                ExtensionType::TransferFeeAmount,
            ])
            .unwrap()
        ];
        let mut state =
            StateWithExtensionsMut::<Account>::unpack_uninitialized(&mut bytes).unwrap();
        state
            .init_extension::<TransferFeeAmount>(false)
            .unwrap()
            .withheld_amount = 50u64.into();
        state.base = Account {
            mint,
            owner: Address::new_from_array([9; 32]),
            amount: 100,
            state: AccountState::Initialized,
            ..Account::default()
        };
        state.pack_base();
        state.init_account_type().unwrap();
        let mut mint_bytes = vec![0; Mint::LEN];
        Mint::pack(
            Mint {
                supply: 1_000,
                decimals: 6,
                is_initialized: true,
                ..Mint::default()
            },
            &mut mint_bytes,
        )
        .unwrap();
        let named = |address: String, data: Vec<u8>| NamedAccount {
            label: "captured-parent".into(),
            address,
            account: AccountSnapshot {
                lamports: 1,
                owner: decode::TOKEN_2022_PROGRAM.into(),
                data,
                executable: false,
                rent_epoch: 0,
            },
        };
        let mut accounts = vec![
            named(plan.source_account.clone(), bytes),
            named(plan.source_mint.clone(), mint_bytes),
        ];
        assert!(
            mutate_token_state(&mut accounts, &plan, 100, 49).is_err(),
            "withheld_fees_define_the_valid_amount_floor"
        );
        mutate_token_state(&mut accounts, &plan, 100, 50).unwrap();
        let updated = StateWithExtensions::<Account>::unpack(&accounts[0].account.data).unwrap();
        assert_eq!(updated.base.amount, 50);
        assert_eq!(
            u64::from(
                updated
                    .get_extension::<TransferFeeAmount>()
                    .unwrap()
                    .withheld_amount
            ),
            50
        );
        assert_eq!(Mint::unpack(&accounts[1].account.data).unwrap().supply, 950);
    }

    #[test]
    fn only_explicit_counterexample_finding_blocks_gate_without_readiness_change() {
        let report = serde_json::json!({
            "transition_package_sha256":"p".repeat(64),
            "candidate_program_sha256":"c".repeat(64),
            "run_id":"run-1",
            "candidate_plan_readiness":"Ready",
            "conversion_stress_readiness":{"status":"Ready"},
            "population_rollout_readiness":{"status":"Ready"},
            "declared_preflight_status":"Ready",
            "official_transition":"NotTested",
            "funds_moved":false,
        });
        let mut search = SearchResult {
            version: VERSION.into(),
            parent_run: "run-1".into(),
            transition_package_sha256: "p".repeat(64),
            candidate_program_sha256: "c".repeat(64),
            population_capture_sha256: "a".repeat(64),
            stress_plan_sha256: "b".repeat(64),
            case_capture_sha256: "d".repeat(64),
            budget: Budget::new(),
            search_domain: "bounded".into(),
            derived_domain: None,
            observed_wave: vec![],
            additional_waves: vec![],
            trace: vec![],
            counterexamples: vec![],
            conclusion: "No counterexample found within this search domain and budget.".into(),
            official_transition: "NotTested".into(),
            funds_moved: false,
        };
        let policy = crate::conversion::package_gate::Policy::BlockOnly;
        assert_eq!(
            crate::conversion::package_gate::evaluate_with_counterexamples(
                &report, policy, &search
            )
            .unwrap()
            .outcome,
            crate::conversion::package_gate::Outcome::Pass
        );
        search.counterexamples.push(Counterexample::Observed {
            id: "observed-1".into(),
            parent_run: "run-1".into(),
            transition_package_sha256: "p".repeat(64),
            candidate_program_sha256: "c".repeat(64),
            observed_source_account: "account".into(),
            observed_state_digest: "s".repeat(64),
            observed_amount_raw: "51".into(),
            execution_plan_sha256: "e".repeat(64),
            execution_fixture_sha256: "f".repeat(64),
            failure_signature: failure(13),
            provenance: "ObservedOnChain".into(),
            limitations: "exact account only".into(),
        });
        assert_eq!(
            crate::conversion::package_gate::evaluate_with_counterexamples(
                &report, policy, &search
            )
            .unwrap()
            .outcome,
            crate::conversion::package_gate::Outcome::Block,
            "counterexample_blocks_only_through_explicit_finding"
        );
        assert_eq!(
            report["candidate_plan_readiness"], "Ready",
            "analytical_evidence_must_remain_unchanged"
        );
        search.parent_run = "other-run".into();
        assert!(
            crate::conversion::package_gate::evaluate_with_counterexamples(
                &report, policy, &search
            )
            .is_err(),
            "finding_scope_must_match_exact_parent"
        );
    }
}
