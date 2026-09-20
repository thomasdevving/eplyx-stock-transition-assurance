//! Command-line regression harness for Eplyx Lifecycle Impact.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use eplyx_lifecycle_impact::lifecycle::{
    exposure::{self, meteora_dlmm::MeteoraDlmmAdapter},
    policy::LifecycleScenario,
    rpc::HttpSolanaRpc,
    AssetDescriptor, LifecycleSnapshot, LifecycleStateSource, SolanaTokenAssetSource,
};
use eplyx_lifecycle_impact::{
    compare_all, corpus, default_artifact, fixture_program_id, load_versions, report::render_text,
    report::Report, types::Category, ChangeScenario,
};

#[derive(Parser)]
#[command(
    name = "eplyx-lifecycle",
    about = "Eplyx Lifecycle Impact — production snapshots and upgrade regression harness",
    long_about = "Evaluates program upgrades against identical fixture state and transactions, \
                  and external lifecycle policy against frozen production observations offline. \
                  Official lifecycle transition execution remains untested."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate structured prospective fields without RPC.
    ValidateCurrentPreflight {
        #[arg(long)]
        input: PathBuf,
    },
    /// Construct a bound UserProposed scenario offline.
    PrepareCurrentPreflight {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Independent bounded mint capture for a user-proposed replacement.
    CapturePreflightSuccessor {
        #[arg(long)]
        mint: String,
        #[arg(long)]
        out: PathBuf,
    },
    /// Reconstruct current evidence and evaluate the prospective scenario offline.
    ReplayCurrentPreflight {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        run_id: String,
        #[arg(long)]
        preflight_id: String,
        #[arg(long)]
        wallet_sha256: String,
        #[arg(long)]
        scenario_sha256: String,
        #[arg(long)]
        capture_sha256: String,
    },
    /// Fetch bounded current mainnet observations; never constructs execution proof.
    InspectCurrent(CurrentArgs),
    /// Validate a typed current transfer request entirely offline.
    CurrentCheckCapabilities {
        #[arg(long)]
        input: PathBuf,
    },
    ValidateCurrentCheck {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        request: PathBuf,
    },
    /// Capture current transfer dependencies using bounded read-only RPC. Does not execute.
    CaptureCurrentCheck {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        run_id: String,
        #[arg(long)]
        check_id: String,
        #[arg(long)]
        out: PathBuf,
    },
    /// Rebuild and execute a current check in the offline VM, verifying exact bindings.
    ReplayCurrentCheck {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        run_id: String,
        #[arg(long)]
        check_id: String,
        #[arg(long)]
        wallet_sha256: String,
        #[arg(long)]
        capture_sha256: String,
    },
    /// Validate public addresses using the Solana parser, without network access.
    ValidateAddress {
        #[arg(long, required = true)]
        mint: Vec<String>,
    },
    /// Re-decode saved current-run responses offline.
    ReplayCurrent {
        #[arg(long)]
        input: PathBuf,
    },
    /// Assess structured rollout assertions against pinned historical evidence offline.
    EvaluateRollout(RolloutArgs),
    /// Evaluate a candidate and guard an inert temporary local marker.
    GuardRollout(GuardRolloutArgs),
    /// Run the frozen candidate demonstrations and record actual local marker behavior.
    DemoRollout(DemoRolloutArgs),
    /// Compare deterministic lifecycle times over identical pinned production state offline.
    CompareScenarios(CounterfactualArgs),
    /// Normalize one pinned official issuer notice entirely offline.
    IngestNotice(NoticeArgs),
    /// Regenerate and verify an event, then create its lifecycle scenario.
    ScenarioFromEvent(NoticeArgs),
    /// Orchestrate notice-derived impact with compatible frozen path/readiness evidence.
    PreflightFromNotice(NoticeArgs),
    /// Evaluate an explicit assurance policy against existing frozen measurements offline.
    Readiness(ReadinessArgs),
    /// Decode one real LP position from a bounded discovery and finalized capture offline.
    DiscoverPosition(PositionArgs),
    /// Execute one native LP withdrawal against captured deployed programs offline.
    ProbeWithdrawal(WithdrawalArgs),
    /// Reproduce a bounded official-transition investigation from frozen public evidence.
    InvestigateTransition(InvestigateTransitionArgs),
    /// Resolve one entity's five lifecycle paths using verified offline evidence.
    ResolvePaths(ResolvePathsArgs),
    /// Observe concrete venue candidates through read-only standard RPC.
    VenueDiscover(VenueDiscoverArgs),
    /// Save a deterministic bounded expansion plan before new execution.
    CoverageExpand(CoverageExpandArgs),
    /// Capture only the immutable selected groups, read-only finalized RPC.
    CapturePlan(CapturePlanArgs),
    /// Execute the immutable groups locally using supplied capture artifacts.
    ExecutePlan(ExecutePlanArgs),
    /// Fresh-replay evidence and compute the exact before/after assurance delta.
    CoverageUpdate(CoverageUpdateArgs),
    /// Join population impact and replay a matrix of captured execution witnesses offline.
    Coverage(CoverageArgs),
    /// Select representative classes and create an offline amount/path matrix.
    CoveragePlan(CoveragePlanArgs),
    /// Execute one captured secondary-market exit probe entirely offline.
    Probe(ProbeArgs),
    /// Capture real route state and deployed bytecode read-only for one local probe.
    ProbeCapture(ProbeCaptureArgs),
    /// Apply a lifecycle policy offline to two time views of one frozen snapshot.
    Impact(ImpactArgs),
    /// Verify one protocol candidate and attach an evidence-backed exposure graph.
    Exposure(ExposureArgs),
    /// Freeze mint, token accounts and owner authority evidence using standard RPC.
    Snapshot(SnapshotArgs),
    /// Load and verify a snapshot entirely offline, then print its summary.
    SnapshotShow(SnapshotShowArgs),
    /// Compare the synthetic corpus against the current and candidate builds.
    Compare(CompareArgs),
    /// Generate the deterministic test state locally.
    Generate(GenerateArgs),
    /// Reproduce a fixture or economic regression group.
    Reproduce(ReproduceArgs),
    /// List the synthetic test corpus.
    List(ListArgs),
}

#[derive(Parser)]
struct CurrentArgs {
    #[arg(long)]
    asset: Option<PathBuf>,
    #[arg(long, conflicts_with = "asset", requires = "selection")]
    mint: Option<String>,
    #[arg(long, requires = "mint", conflicts_with = "asset")]
    selection: Option<PathBuf>,
    #[arg(long)]
    out: PathBuf,
}

#[derive(Parser)]
struct RolloutArgs {
    #[arg(long)]
    plan: PathBuf,
    #[arg(long, default_value = "probes/phase14-evidence-binding.json")]
    binding: PathBuf,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    #[arg(long)]
    out: Option<PathBuf>,
    /// Select an existing pinned lifecycle view without changing historical proof.
    #[arg(long, value_enum)]
    target_view: Option<RolloutTargetView>,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum RolloutTargetView {
    #[value(name = "before_transition")]
    BeforeTransition,
    #[value(name = "after_transition")]
    AfterTransition,
    #[value(name = "after_deadline")]
    AfterDeadline,
}

impl RolloutTargetView {
    fn id(self) -> &'static str {
        match self {
            Self::BeforeTransition => "before_transition",
            Self::AfterTransition => "after_transition",
            Self::AfterDeadline => "after_deadline",
        }
    }
}

#[derive(Parser)]
struct GuardRolloutArgs {
    #[command(flatten)]
    evaluation: RolloutArgs,
    #[arg(long)]
    marker: PathBuf,
}

#[derive(Parser)]
struct DemoRolloutArgs {
    #[arg(long, default_value = "probes/phase14-demo-cases.json")]
    cases: PathBuf,
    #[arg(long, default_value = "probes/phase14-evidence-binding.json")]
    binding: PathBuf,
    #[arg(long)]
    marker_directory: PathBuf,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Parser)]
struct CounterfactualArgs {
    #[arg(long, default_value = "snapshots/spacex-exposure.json")]
    snapshot: PathBuf,
    #[arg(long, default_value = "scenarios/spacex-transition.json")]
    scenario: PathBuf,
    #[arg(long, default_value = "policies/stocklana-spacex-preflight-v1.json")]
    readiness_policy: PathBuf,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Parser)]
struct NoticeArgs {
    #[arg(long)]
    workflow: PathBuf,
    #[arg(long)]
    event: Option<PathBuf>,
    #[arg(long)]
    scenario: Option<PathBuf>,
    #[arg(long)]
    binding: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    out_binding: Option<PathBuf>,
    #[arg(long)]
    out_impact: Option<PathBuf>,
    #[arg(long)]
    out_resolution: Option<PathBuf>,
    #[arg(long)]
    out_readiness: Option<PathBuf>,
}

#[derive(Parser)]
struct ReadinessArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    scenario: PathBuf,
    #[arg(long)]
    policy: PathBuf,
    #[arg(long)]
    direct_resolution: PathBuf,
    #[arg(long)]
    position_resolution: PathBuf,
    #[arg(long)]
    coverage: PathBuf,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Parser)]
struct PositionArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    scenario: PathBuf,
    #[arg(long)]
    discovery: PathBuf,
    #[arg(long)]
    fixture: PathBuf,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    #[arg(long)]
    out: Option<PathBuf>,
}
#[derive(Parser)]
struct WithdrawalArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    scenario: PathBuf,
    #[arg(long)]
    discovery: PathBuf,
    #[arg(long)]
    position: PathBuf,
    #[arg(long)]
    fixture: PathBuf,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Parser)]
struct InvestigateTransitionArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    scenario: PathBuf,
    #[arg(long)]
    entity: String,
    #[arg(long)]
    research: PathBuf,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    out_resolution: Option<PathBuf>,
    #[arg(long)]
    out_discovery: Option<PathBuf>,
}

#[derive(Parser)]
struct ResolvePathsArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    scenario: PathBuf,
    #[arg(long)]
    entity: String,
    #[arg(long)]
    coverage: PathBuf,
    #[arg(long)]
    discovery: PathBuf,
    #[arg(long)]
    evidence_bundle: PathBuf,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Parser)]
struct VenueDiscoverArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    coverage: PathBuf,
    #[arg(long)]
    rpc: String,
    #[arg(long)]
    out: PathBuf,
}
#[derive(Parser)]
struct ExpansionInputs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    impact: PathBuf,
    #[arg(long)]
    coverage: PathBuf,
    #[arg(long)]
    baseline_plan: PathBuf,
    #[arg(long)]
    inventory: PathBuf,
}
#[derive(Parser)]
struct CoverageExpandArgs {
    #[command(flatten)]
    inputs: ExpansionInputs,
    #[arg(long)]
    selector_config: PathBuf,
    #[arg(long)]
    out_plan: PathBuf,
}
#[derive(Parser)]
struct CapturePlanArgs {
    #[command(flatten)]
    inputs: ExpansionInputs,
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    rpc: String,
    #[arg(long)]
    out_dir: PathBuf,
}
#[derive(Parser)]
struct ExecutePlanArgs {
    #[command(flatten)]
    inputs: ExpansionInputs,
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    captures: PathBuf,
    #[arg(long)]
    out_dir: PathBuf,
}
#[derive(Parser)]
struct CoverageUpdateArgs {
    #[command(flatten)]
    inputs: ExpansionInputs,
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    captures: PathBuf,
    #[arg(long)]
    results: PathBuf,
    #[arg(long)]
    out: PathBuf,
}
#[derive(Parser)]
struct CoverageArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    impact: PathBuf,
    #[arg(long)]
    plan: PathBuf,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    #[arg(long)]
    out: Option<PathBuf>,
}
#[derive(Parser)]
struct CoveragePlanArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    impact: PathBuf,
    /// Repeat for each captured executable venue/holder route.
    #[arg(long)]
    probe: Vec<PathBuf>,
    /// Independent exact raw input amounts; every case resets captured state.
    #[arg(long, value_delimiter = ',', default_value = "1000,10000,30000")]
    amounts_raw: Vec<u64>,
    #[arg(long)]
    out: PathBuf,
}

#[derive(Parser)]
struct ProbeArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    scenario: PathBuf,
    #[arg(long)]
    probe: PathBuf,
    #[arg(long)]
    at: Option<chrono::DateTime<chrono::Utc>>,
    #[arg(long)]
    out: Option<PathBuf>,
}
#[derive(Parser)]
struct ProbeCaptureArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    scenario: PathBuf,
    #[arg(long)]
    rpc: String,
    #[arg(long)]
    entity_id: Option<String>,
    #[arg(long, default_value_t = 10000)]
    amount_raw: u64,
    #[arg(long)]
    fixture_out: PathBuf,
    #[arg(long)]
    probe_out: PathBuf,
}

#[derive(Parser)]
struct ImpactArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long)]
    scenario: PathBuf,
    /// Hypothetical lifecycle evaluation time (RFC3339); never a new chain capture.
    #[arg(long)]
    at: chrono::DateTime<chrono::Utc>,
    /// Baseline semantic time. Defaults to one nanosecond before effective_at.
    #[arg(long)]
    before: Option<chrono::DateTime<chrono::Utc>>,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    /// New deterministic JSON report path; existing artifacts are never overwritten.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Adapter {
    #[value(alias = "meteora")]
    MeteoraDlmm,
}

#[derive(Parser)]
struct ExposureArgs {
    #[arg(long)]
    snapshot: PathBuf,
    #[arg(long, value_enum)]
    adapter: Adapter,
    /// Candidate address only: protocol/mint/vault relationships are verified on-chain.
    #[arg(long)]
    pool: String,
    #[arg(long)]
    rpc: String,
    /// New schema-2 snapshot path; existing snapshots are never overwritten.
    #[arg(long)]
    out: PathBuf,
}

#[derive(Parser)]
struct SnapshotArgs {
    #[arg(long)]
    mint: String,
    /// Standard HTTP(S) Solana JSON-RPC URL. Never saved in full in the snapshot.
    #[arg(long)]
    rpc: String,
    /// New snapshot path. Existing frozen snapshots are not overwritten.
    #[arg(long)]
    out: PathBuf,
    /// Optional asset identity / expected program / chain configuration JSON.
    #[arg(long)]
    asset_config: Option<PathBuf>,
}

#[derive(Parser)]
struct SnapshotShowArgs {
    #[arg(long)]
    input: PathBuf,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Parser)]
struct CompareArgs {
    #[arg(long, alias = "current")]
    v1: Option<PathBuf>,
    #[arg(long, alias = "candidate")]
    v2: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    #[arg(long)]
    fixture: Option<String>,
    #[arg(long)]
    category: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    fail_on_critical: bool,
    #[arg(long)]
    no_minimize: bool,
}

#[derive(Parser)]
struct GenerateArgs {
    /// Directory to write fixture JSON into.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Parser)]
struct ReproduceArgs {
    /// Fixture ID (e.g. boundary-position-017) or regression group ID
    /// (e.g. newly-liquidatable, or regression-group:newly-liquidatable).
    target: String,
    #[arg(long, alias = "current")]
    v1: Option<PathBuf>,
    #[arg(long, alias = "candidate")]
    v2: Option<PathBuf>,
    /// Skip minimization when reproducing a regression group.
    #[arg(long)]
    no_minimize: bool,
}

#[derive(Parser)]
struct ListArgs {
    #[arg(long)]
    category: Option<String>,
}

fn parse_category(name: &str) -> Result<Category> {
    Category::ALL
        .into_iter()
        .find(|c| c.as_str() == name)
        .ok_or_else(|| {
            anyhow!(
                "unknown category {name:?}; valid categories: {}",
                Category::ALL
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode> {
    match Cli::parse().command {
        Command::ValidateCurrentPreflight { input } => {
            let input = serde_json::from_slice(&std::fs::read(input)?)?;
            eplyx_lifecycle_impact::preflight::validate(&input)?;
            println!("{{}}");
            Ok(ExitCode::SUCCESS)
        }
        Command::PrepareCurrentPreflight { input, out } => {
            let input = serde_json::from_slice(&std::fs::read(input)?)?;
            let bundle = eplyx_lifecycle_impact::preflight::prepare(input)?;
            eplyx_lifecycle_impact::preflight::save(&bundle, &out)?;
            println!(
                "{}",
                serde_json::json!({"scenario_sha256":bundle.scenario_sha256,"scenario":bundle.scenario})
            );
            Ok(ExitCode::SUCCESS)
        }
        Command::CapturePreflightSuccessor { mint, out } => {
            use eplyx_lifecycle_impact::lifecycle::current;
            let url = std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".into());
            let capture = current::capture_selected(
                current::InspectionSelection {
                    cluster: "solana-mainnet".into(),
                    mint,
                    reference: None,
                    sample_accounts: false,
                    public_owner: None,
                },
                &HttpSolanaRpc::bounded(&url)?,
            )?;
            current::save(&capture, &out)?;
            println!("{{}}");
            Ok(ExitCode::SUCCESS)
        }
        Command::ReplayCurrentPreflight {
            input,
            run_id,
            preflight_id,
            wallet_sha256,
            scenario_sha256,
            capture_sha256,
        } => {
            let result = eplyx_lifecycle_impact::preflight::replay(
                &std::fs::read(input)?,
                &run_id,
                &preflight_id,
                &wallet_sha256,
                &scenario_sha256,
                &capture_sha256,
            )?;
            println!("{}", result);
            Ok(ExitCode::SUCCESS)
        }
        Command::CurrentCheckCapabilities { input } => {
            println!(
                "{}",
                eplyx_lifecycle_impact::probe::current::capabilities(&std::fs::read_to_string(
                    input
                )?)?
            );
            Ok(ExitCode::SUCCESS)
        }
        Command::ValidateCurrentCheck { input, request } => {
            let wallet = std::fs::read_to_string(input)?;
            let request = serde_json::from_slice(&std::fs::read(request)?)?;
            println!(
                "{}",
                eplyx_lifecycle_impact::probe::current::validate(&wallet, &request)?
            );
            Ok(ExitCode::SUCCESS)
        }
        Command::CaptureCurrentCheck {
            input,
            request,
            run_id,
            check_id,
            out,
        } => {
            use eplyx_lifecycle_impact::probe::current;
            let wallet = std::fs::read_to_string(input)?;
            let request = serde_json::from_slice(&std::fs::read(request)?)?;
            let url = std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".into());
            let capture = current::capture(
                wallet,
                request,
                run_id,
                check_id,
                &HttpSolanaRpc::bounded_execution(&url)?,
            )?;
            current::save(&capture, &out)?;
            println!("{{}} ");
            Ok(ExitCode::SUCCESS)
        }
        Command::ReplayCurrentCheck {
            input,
            run_id,
            check_id,
            wallet_sha256,
            capture_sha256,
        } => {
            let verified = eplyx_lifecycle_impact::probe::current::replay(
                &std::fs::read(input)?,
                &run_id,
                &check_id,
                &wallet_sha256,
                &capture_sha256,
            )?;
            println!("{}", verified.value());
            Ok(ExitCode::SUCCESS)
        }
        Command::ValidateAddress { mint } => {
            anyhow::ensure!(mint.len() <= 100, "address count exceeds budget");
            for value in &mint {
                let _: solana_address::Address = value
                    .parse()
                    .context("Enter a valid Solana token address")?;
            }
            println!("{}", serde_json::to_string(&mint)?);
            Ok(ExitCode::SUCCESS)
        }
        Command::InspectCurrent(args) => {
            use eplyx_lifecycle_impact::lifecycle::current;
            let url = std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".into());
            let rpc = HttpSolanaRpc::bounded(&url)?;
            let capture = if let Some(mint) = args.mint {
                let selection: current::InspectionSelection = serde_json::from_slice(
                    &std::fs::read(args.selection.context("selection required")?)?,
                )?;
                anyhow::ensure!(selection.mint == mint, "selection mint mismatch");
                current::capture_selected(selection, &rpc)?
            } else {
                let asset = serde_json::from_slice(&std::fs::read(
                    args.asset.context("mint or asset required")?,
                )?)?;
                current::capture(asset, &rpc)?
            };
            current::save(&capture, &args.out)?;
            eprintln!("CURRENT_STAGE:Decoding and preparing findings");
            println!("{}", serde_json::to_string(&current::evaluate(&capture)?)?);
            Ok(ExitCode::SUCCESS)
        }
        Command::ReplayCurrent { input } => {
            println!(
                "{}",
                serde_json::to_string(&eplyx_lifecycle_impact::lifecycle::current::replay(
                    &input
                )?)?
            );
            Ok(ExitCode::SUCCESS)
        }
        Command::EvaluateRollout(args) => evaluate_rollout(args, None),
        Command::GuardRollout(args) => evaluate_rollout(args.evaluation, Some(args.marker)),
        Command::DemoRollout(args) => demo_rollout(args),
        Command::CompareScenarios(args) => compare_scenarios(args),
        Command::IngestNotice(args) => notice(args, 0),
        Command::ScenarioFromEvent(args) => notice(args, 1),
        Command::PreflightFromNotice(args) => notice(args, 2),
        Command::Readiness(args) => readiness(args),
        Command::DiscoverPosition(args) => discover_position(args),
        Command::ProbeWithdrawal(args) => probe_withdrawal(args),
        Command::InvestigateTransition(args) => investigate_transition(args),
        Command::ResolvePaths(args) => resolve_paths(args),
        Command::VenueDiscover(args) => venue_discover(args),
        Command::CoverageExpand(args) => coverage_expand(args),
        Command::CapturePlan(args) => capture_plan(args),
        Command::ExecutePlan(args) => execute_plan(args),
        Command::CoverageUpdate(args) => coverage_update(args),
        Command::Coverage(args) => lifecycle_coverage(args),
        Command::CoveragePlan(args) => lifecycle_coverage_plan(args),
        Command::Probe(args) => execution_probe(args),
        Command::ProbeCapture(args) => capture_execution_probe(args),
        Command::Impact(args) => lifecycle_impact(args),
        Command::Exposure(args) => exposure(args),
        Command::Snapshot(args) => snapshot(args),
        Command::SnapshotShow(args) => {
            let snapshot = LifecycleSnapshot::load(&args.input)?;
            println!(
                "{}",
                match &snapshot.exposures {
                    Some(graph) => graph.render_summary(&snapshot),
                    None => snapshot.render_summary(),
                }
            );
            Ok(ExitCode::SUCCESS)
        }
        Command::Compare(args) => compare(args),
        Command::Generate(args) => generate(args),
        Command::Reproduce(args) => reproduce(args),
        Command::List(args) => list(args),
    }
}

fn notice(args: NoticeArgs, stage: u8) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::{expansion, notice::workflow::NoticeWorkflow};
    let outputs = [
        &args.out,
        &args.out_binding,
        &args.out_impact,
        &args.out_resolution,
        &args.out_readiness,
    ];
    let mut seen = std::collections::BTreeSet::new();
    for p in outputs.into_iter().flatten() {
        anyhow::ensure!(!p.exists(), "notice output already exists: {}", p.display());
        let parent = p
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(std::path::Path::new("."));
        let absolute = parent
            .canonicalize()?
            .join(p.file_name().context("output filename missing")?);
        anyhow::ensure!(seen.insert(absolute), "duplicate notice output");
    }
    anyhow::ensure!(
        stage == 2
            || (args.out_impact.is_none()
                && args.out_resolution.is_none()
                && args.out_readiness.is_none()),
        "pipeline outputs require preflight-from-notice"
    );
    anyhow::ensure!(
        stage != 0
            || (args.event.is_none()
                && args.scenario.is_none()
                && args.binding.is_none()
                && args.out_binding.is_none()),
        "ingest-notice accepts workflow and event output only"
    );
    anyhow::ensure!(
        stage != 1 || (args.scenario.is_none() && args.binding.is_none()),
        "scenario-from-event regenerates scenario/binding"
    );
    let workflow = NoticeWorkflow::load(&args.workflow)?;
    let base = args.workflow.parent().unwrap_or(std::path::Path::new("."));
    let event = if let Some(p) = &args.event {
        expansion::load(p)?
    } else {
        workflow.normalize(base)?.event().clone()
    };
    if stage == 0 {
        if let Some(p) = &args.out {
            expansion::save(&event, p)?;
        }
        match args.format {
            Format::Json => print!("{}", event.to_json()?),
            Format::Text => print!("{}", event.render_text()),
        }
        return Ok(ExitCode::SUCCESS);
    }
    let (expected, expected_binding) = workflow.generate(base, &event)?;
    let scenario = if let Some(p) = &args.scenario {
        expansion::load(p)?
    } else {
        expected
    };
    let binding = if let Some(p) = &args.binding {
        expansion::load(p)?
    } else {
        expected_binding
    };
    if stage == 1 {
        workflow.verify_scenario(base, &event, &scenario, &binding)?;
        if let Some(p) = &args.out {
            expansion::save(&scenario, p)?;
        }
        if let Some(p) = &args.out_binding {
            expansion::save(&binding, p)?;
        }
        match args.format {Format::Json=>print!("{}",scenario.to_json()?),Format::Text=>println!("Generated lifecycle scenario {}\nOfficialTransition: NotTested; evaluation boundary: DemoConfigured",scenario.id)}
        return Ok(ExitCode::SUCCESS);
    }
    let (report, impact) = workflow.preflight(base, &event, &scenario, &binding)?;
    if let Some(p) = &args.out {
        expansion::save(&report, p)?;
    }
    if let Some(p) = &args.out_binding {
        expansion::save(&binding, p)?;
    }
    if let Some(p) = &args.out_impact {
        expansion::save(&impact, p)?;
    }
    if let Some(p) = &args.out_resolution {
        expansion::save(&report.resolution, p)?;
    }
    if let Some(p) = &args.out_readiness {
        expansion::save(&report.readiness, p)?;
    }
    match args.format {
        Format::Json => print!("{}", report.to_json()?),
        Format::Text => print!("{}", report.render_text()),
    }
    Ok(ExitCode::from(report.readiness.overall_status.exit_code()))
}

fn readiness(args: ReadinessArgs) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::{
        expansion,
        readiness::{self, evidence::ReadinessEvidenceManifest, LifecycleReadinessPolicy},
    };
    anyhow::ensure!(
        !args.out.as_ref().is_some_and(|p| p.exists()),
        "readiness output already exists"
    );
    let policy: LifecycleReadinessPolicy = expansion::load(&args.policy)?;
    let normalized = policy.normalized()?;
    let policy_base = args.policy.parent().unwrap_or(std::path::Path::new("."));
    let manifest: ReadinessEvidenceManifest =
        serde_json::from_slice(&normalized.evidence_manifest.read(policy_base)?)?;
    let manifest_path = policy_base.join(&normalized.evidence_manifest.file);
    let evidence = manifest.verify(
        manifest_path.parent().unwrap_or(std::path::Path::new(".")),
        &args.snapshot,
        &args.scenario,
        &args.direct_resolution,
        &args.position_resolution,
        &args.coverage,
    )?;
    let report = readiness::evaluate(&normalized, &evidence)?;
    if let Some(out) = &args.out {
        expansion::save(&report, out)?;
    }
    match args.format {
        Format::Json => print!("{}", report.to_json()?),
        Format::Text => print!("{}", report.render_text()),
    }
    Ok(ExitCode::from(report.overall_status.exit_code()))
}

fn expansion_inputs(
    args: &ExpansionInputs,
) -> Result<(
    LifecycleSnapshot,
    eplyx_lifecycle_impact::lifecycle::consequence::LifecycleImpactReport,
    eplyx_lifecycle_impact::coverage::CoverageReport,
    eplyx_lifecycle_impact::coverage::CoveragePlan,
    eplyx_lifecycle_impact::expansion::discovery::VenueInventory,
)> {
    use eplyx_lifecycle_impact::expansion as ex;
    let (s, i) = load_coverage_population(&args.snapshot, &args.impact)?;
    let b: eplyx_lifecycle_impact::coverage::CoverageReport = ex::load(&args.coverage)?;
    let bp: eplyx_lifecycle_impact::coverage::CoveragePlan = ex::load(&args.baseline_plan)?;
    let inventory = ex::load(&args.inventory)?;
    anyhow::ensure!(
        ex::digest(&b)?
            == eplyx_lifecycle_impact::lifecycle::exposure::sha256(&std::fs::read(&args.coverage)?)
            && ex::digest(&bp)? == b.plan_sha256
            && ex::digest(&i)? == b.impact_sha256
            && i.after.snapshot_sha256 == b.snapshot_sha256,
        "baseline file/input hashes differ"
    );
    Ok((s, i, b, bp, inventory))
}
fn discover_position(args: PositionArgs) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::{expansion, position};
    anyhow::ensure!(
        !args.out.as_ref().is_some_and(|p| p.exists()),
        "position output already exists"
    );
    let snapshot = LifecycleSnapshot::load(&args.snapshot)?;
    let scenario = LifecycleScenario::load(&args.scenario)?;
    let position = position::meteora_dlmm::discover(
        &snapshot,
        &scenario,
        &std::fs::read(&args.discovery)?,
        &std::fs::read(&args.fixture)?,
    )?;
    if let Some(out) = &args.out {
        expansion::save(&position, out)?;
    }
    match args.format {
        Format::Json => print!("{}", expansion::canonical(&position)?),
        Format::Text => println!(
            "Position {}\nPool {}\nAuthority {} ({:?})\nPrincipal raw {:?}",
            position.position_id,
            position.pool,
            position.authority,
            position.authority_model,
            position.principal_exposure_raw
        ),
    }
    Ok(ExitCode::SUCCESS)
}
fn probe_withdrawal(args: WithdrawalArgs) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::{expansion, position};
    anyhow::ensure!(
        !args.out.as_ref().is_some_and(|p| p.exists()),
        "withdrawal output already exists"
    );
    let snapshot = LifecycleSnapshot::load(&args.snapshot)?;
    let scenario = LifecycleScenario::load(&args.scenario)?;
    let selected = expansion::load(&args.position)?;
    let probe = position::WithdrawalProbe::full(&selected);
    let report = position::meteora_dlmm::execute(
        &selected,
        &probe,
        &snapshot,
        &scenario,
        &std::fs::read(&args.discovery)?,
        &std::fs::read(&args.fixture)?,
    )?;
    if let Some(out) = &args.out {
        expansion::save(&report, out)?;
    }
    match args.format {
        Format::Json => print!("{}", report.to_json()?),
        Format::Text => print!("{}", report.render_text()),
    }
    Ok(ExitCode::SUCCESS)
}

fn investigate_transition(args: InvestigateTransitionArgs) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::{expansion, transition};
    let paths: Vec<_> = [&args.out, &args.out_resolution, &args.out_discovery]
        .into_iter()
        .flatten()
        .collect();
    anyhow::ensure!(
        paths.iter().all(|p| !p.exists()),
        "transition output already exists"
    );
    anyhow::ensure!(
        paths
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == paths.len(),
        "transition output paths must be distinct"
    );
    let snapshot = LifecycleSnapshot::load(&args.snapshot)?;
    let scenario = LifecycleScenario::load(&args.scenario)?;
    let report =
        transition::research::investigate(&args.research, &snapshot, &scenario, &args.entity)?;
    if let Some(path) = &args.out {
        expansion::save(&report, path)?;
    }
    if let Some(path) = &args.out_resolution {
        expansion::save(&report.updated_resolution, path)?;
    }
    if let Some(path) = &args.out_discovery {
        expansion::save(&report.updated_discovery, path)?;
    }
    match args.format {
        Format::Json => print!("{}", report.to_json()?),
        Format::Text => print!("{}", report.render_text()),
    }
    Ok(ExitCode::SUCCESS)
}
fn resolve_paths(args: ResolvePathsArgs) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::{expansion, resolution};
    if args.out.as_ref().is_some_and(|p| p.exists()) {
        return Err(anyhow!("path resolution output already exists"));
    }
    let s = LifecycleSnapshot::load(&args.snapshot)?;
    let scenario = LifecycleScenario::load(&args.scenario)?;
    let report = resolution::phase7::resolve(
        &args.evidence_bundle,
        &s,
        &scenario,
        &args.entity,
        &args.coverage,
        &args.discovery,
    )?;
    if let Some(path) = &args.out {
        expansion::save(&report, path)?;
    }
    match args.format {
        Format::Json => print!("{}", report.to_json()?),
        Format::Text => print!("{}", report.render_text()),
    }
    Ok(ExitCode::SUCCESS)
}
fn venue_discover(args: VenueDiscoverArgs) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::expansion as ex;
    if args.out.exists() {
        return Err(anyhow!("inventory output already exists"));
    }
    let s = LifecycleSnapshot::load(&args.snapshot)?;
    let b = ex::load(&args.coverage)?;
    let inventory = ex::discovery::discover(&s, &b, &HttpSolanaRpc::new(&args.rpc)?)?;
    ex::save(&inventory, &args.out)?;
    print!("{}", ex::canonical(&inventory)?);
    Ok(ExitCode::SUCCESS)
}
fn coverage_expand(args: CoverageExpandArgs) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::expansion as ex;
    if args.out_plan.exists() {
        return Err(anyhow!("expansion plan output already exists"));
    }
    let (s, i, b, bp, inventory) = expansion_inputs(&args.inputs)?;
    b.validate(
        &s,
        &i,
        &bp,
        args.inputs
            .baseline_plan
            .parent()
            .unwrap_or_else(|| std::path::Path::new(".")),
    )?;
    let cfg = ex::load(&args.selector_config)?;
    let plan = ex::expand(&s, &b, &inventory, &cfg)?;
    ex::save(&plan, &args.out_plan)?;
    print!("{}", ex::canonical(&plan)?);
    Ok(ExitCode::SUCCESS)
}
fn capture_plan(args: CapturePlanArgs) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::expansion as ex;
    if args.out_dir.exists() {
        return Err(anyhow!("capture output directory already exists"));
    }
    let (s, i, b, _, inventory) = expansion_inputs(&args.inputs)?;
    let plan = ex::load(&args.plan)?;
    let manifest = ex::pipeline::capture_plan(
        &s,
        &i,
        &b,
        &plan,
        &inventory,
        &HttpSolanaRpc::new(&args.rpc)?,
        &args.out_dir,
    )?;
    print!("{}", ex::canonical(&manifest)?);
    Ok(ExitCode::SUCCESS)
}
fn execute_plan(args: ExecutePlanArgs) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::expansion as ex;
    if args.out_dir.exists() {
        return Err(anyhow!("execution output directory already exists"));
    }
    let (s, _, b, _, inventory) = expansion_inputs(&args.inputs)?;
    let plan = ex::load(&args.plan)?;
    let manifest = ex::load(&args.captures)?;
    let results = ex::pipeline::execute_plan(
        &s,
        &b,
        &plan,
        &inventory,
        &manifest,
        args.captures
            .parent()
            .unwrap_or_else(|| std::path::Path::new(".")),
        &args.out_dir,
    )?;
    print!("{}", ex::canonical(&results)?);
    Ok(ExitCode::SUCCESS)
}
fn coverage_update(args: CoverageUpdateArgs) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::expansion as ex;
    if args.out.exists() {
        return Err(anyhow!("coverage delta output already exists"));
    }
    let (s, _, b, _, inventory) = expansion_inputs(&args.inputs)?;
    let plan = ex::load(&args.plan)?;
    let manifest = ex::load(&args.captures)?;
    let results = ex::load(&args.results)?;
    let delta = ex::pipeline::update(
        &s,
        &b,
        &plan,
        &inventory,
        &manifest,
        (
            args.captures
                .parent()
                .unwrap_or_else(|| std::path::Path::new(".")),
            args.results
                .parent()
                .unwrap_or_else(|| std::path::Path::new(".")),
        ),
        &results,
    )?;
    ex::save(&delta, &args.out)?;
    print!("{}", ex::canonical(&delta)?);
    Ok(ExitCode::SUCCESS)
}

fn load_coverage_population(
    snapshot: &std::path::Path,
    impact: &std::path::Path,
) -> Result<(
    LifecycleSnapshot,
    eplyx_lifecycle_impact::lifecycle::consequence::LifecycleImpactReport,
)> {
    Ok((
        LifecycleSnapshot::load(snapshot)?,
        serde_json::from_slice(&std::fs::read(impact)?)?,
    ))
}
fn lifecycle_coverage_plan(args: CoveragePlanArgs) -> Result<ExitCode> {
    if args.out.exists() {
        return Err(anyhow!("coverage plan output already exists"));
    }
    let (snapshot, impact) = load_coverage_population(&args.snapshot, &args.impact)?;
    let parent = args
        .out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    std::fs::create_dir_all(parent)?;
    let parent = std::fs::canonicalize(parent)?;
    let mut seeds = Vec::new();
    for path in &args.probe {
        let (mut spec, _) = eplyx_lifecycle_impact::probe::load_probe(path)?;
        let fixture = std::fs::canonicalize(
            path.parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join(&spec.fixture),
        )?;
        // Retain portability for sibling fixture references, use a resolved path
        // when seeds come from another directory. No new path dependency.
        spec.fixture = fixture
            .strip_prefix(&parent)
            .unwrap_or(&fixture)
            .to_string_lossy()
            .into_owned();
        seeds.push(spec);
    }
    let plan = eplyx_lifecycle_impact::coverage::build_plan(
        &snapshot,
        &impact,
        &seeds,
        &args.amounts_raw,
    )?;
    eplyx_lifecycle_impact::probe::save_json(&plan, &args.out)?;
    print!("{}", serde_json::to_string_pretty(&plan)? + "\n");
    Ok(ExitCode::SUCCESS)
}
fn lifecycle_coverage(args: CoverageArgs) -> Result<ExitCode> {
    if args.out.as_ref().is_some_and(|p| p.exists()) {
        return Err(anyhow!("coverage output already exists"));
    }
    let (snapshot, impact) = load_coverage_population(&args.snapshot, &args.impact)?;
    let plan: eplyx_lifecycle_impact::coverage::CoveragePlan =
        serde_json::from_slice(&std::fs::read(&args.plan)?)?;
    let report = eplyx_lifecycle_impact::coverage::evaluate(
        &snapshot,
        &impact,
        &plan,
        args.plan
            .parent()
            .unwrap_or_else(|| std::path::Path::new(".")),
    )?;
    if let Some(path) = &args.out {
        eplyx_lifecycle_impact::probe::save_json(&report, path)?;
    }
    print!(
        "{}",
        match args.format {
            Format::Text => report.render_text(),
            Format::Json => report.to_json()?,
        }
    );
    Ok(ExitCode::SUCCESS)
}

fn capture_execution_probe(args: ProbeCaptureArgs) -> Result<ExitCode> {
    if args.fixture_out.exists() || args.probe_out.exists() {
        return Err(anyhow!("probe and fixture outputs must not exist"));
    }
    let snapshot = LifecycleSnapshot::load(&args.snapshot)?;
    let scenario = LifecycleScenario::load(&args.scenario)?;
    let fixture_path = if args.fixture_out.is_absolute() {
        args.fixture_out.clone()
    } else {
        std::env::current_dir()?.join(&args.fixture_out)
    };
    let probe_path = if args.probe_out.is_absolute() {
        args.probe_out.clone()
    } else {
        std::env::current_dir()?.join(&args.probe_out)
    };
    let fixture_reference = fixture_path
        .strip_prefix(probe_path.parent().context("probe path has no parent")?)
        .context("keep the fixture beneath the probe directory for portable replay")?
        .to_string_lossy()
        .into();
    let (spec, fixture) = eplyx_lifecycle_impact::probe::capture::capture(
        &snapshot,
        &scenario,
        args.entity_id.as_deref(),
        args.amount_raw,
        fixture_reference,
        &HttpSolanaRpc::new(&args.rpc)?,
    )?;
    eplyx_lifecycle_impact::probe::save_json(&fixture, &args.fixture_out)?;
    eplyx_lifecycle_impact::probe::save_json(&spec, &args.probe_out)?;
    println!(
        "Captured {} execution accounts at finalized slot {}; no transaction submitted",
        fixture.evidence[3].result["value"]
            .as_array()
            .context("missing captured batch")?
            .len(),
        fixture.evidence[3].result["context"]["slot"]
    );
    Ok(ExitCode::SUCCESS)
}
fn execution_probe(args: ProbeArgs) -> Result<ExitCode> {
    if let Some(path) = &args.out {
        if path.exists() {
            return Err(anyhow!("probe output must not exist"));
        }
    }
    let snapshot = LifecycleSnapshot::load(&args.snapshot)?;
    let scenario = LifecycleScenario::load(&args.scenario)?;
    let (spec, fixture) = eplyx_lifecycle_impact::probe::load_probe(&args.probe)?;
    let report = eplyx_lifecycle_impact::probe::run(
        &snapshot,
        &scenario,
        &spec,
        &fixture,
        args.at.unwrap_or(scenario.policy.effective_at),
    )?;
    if let Some(path) = &args.out {
        eplyx_lifecycle_impact::probe::save_json(&report, path)?;
    }
    print!("{}", report.to_json()?);
    Ok(ExitCode::SUCCESS)
}

fn lifecycle_impact(args: ImpactArgs) -> Result<ExitCode> {
    if let Some(path) = &args.out {
        if path.exists() {
            return Err(anyhow!("impact output already exists: {}", path.display()));
        }
    }
    let snapshot = LifecycleSnapshot::load(&args.snapshot)?;
    let scenario = LifecycleScenario::load(&args.scenario)?;
    let before = match args.before {
        Some(before) => before,
        None => std::cmp::min(
            args.at,
            scenario
                .policy
                .effective_at
                .checked_sub_signed(chrono::Duration::nanoseconds(1))
                .context("effective_at has no preceding baseline time")?,
        ),
    };
    let report = ChangeScenario::LifecycleChange(scenario.change.clone())
        .compare_lifecycle(&snapshot, &scenario, before, args.at)?;
    if let Some(path) = &args.out {
        report.save(path)?;
        eprintln!("wrote {}", path.display());
    }
    println!(
        "{}",
        match args.format {
            Format::Text => report.render_text(),
            Format::Json => report.to_json()?,
        }
    );
    Ok(ExitCode::SUCCESS)
}

fn exposure(args: ExposureArgs) -> Result<ExitCode> {
    if args.out.exists() {
        return Err(anyhow!(
            "snapshot output already exists: {}",
            args.out.display()
        ));
    }
    let base = LifecycleSnapshot::load(&args.snapshot)?;
    let rpc = HttpSolanaRpc::new(&args.rpc)?;
    let snapshot = match args.adapter {
        Adapter::MeteoraDlmm => exposure::discover(&base, &MeteoraDlmmAdapter, &args.pool, &rpc)?,
    };
    snapshot.save(&args.out)?;
    println!(
        "{}",
        snapshot
            .exposures
            .as_ref()
            .context("exposure graph missing")?
            .render_summary(&snapshot)
    );
    eprintln!("wrote {}", args.out.display());
    Ok(ExitCode::SUCCESS)
}

fn snapshot(args: SnapshotArgs) -> Result<ExitCode> {
    if args.out.exists() {
        return Err(anyhow!(
            "snapshot output already exists: {}",
            args.out.display()
        ));
    }
    let asset = match args.asset_config {
        Some(path) => {
            let asset: AssetDescriptor = serde_json::from_slice(&std::fs::read(path)?)?;
            if asset.mint != args.mint {
                return Err(anyhow!("--mint disagrees with --asset-config"));
            }
            asset
        }
        None => AssetDescriptor {
            name: args.mint.clone(),
            mint: args.mint,
            expected_token_program: None,
            expected_genesis_hash: None,
            verification: vec![],
        },
    };
    let source = SolanaTokenAssetSource {
        rpc: HttpSolanaRpc::new(&args.rpc)?,
    };
    let snapshot = source.capture(asset)?;
    snapshot.save(&args.out)?;
    println!("{}", snapshot.render_summary());
    eprintln!("wrote {}", args.out.display());
    Ok(ExitCode::SUCCESS)
}

fn compare(args: CompareArgs) -> Result<ExitCode> {
    let program_id = fixture_program_id();
    let mut fixtures = corpus::generate(&program_id);

    if let Some(id) = &args.fixture {
        fixtures.retain(|f| &f.id == id);
        if fixtures.is_empty() {
            return Err(anyhow!("no fixture with id {id:?}"));
        }
    }
    if let Some(name) = &args.category {
        let category = parse_category(name)?;
        fixtures.retain(|f| f.category == category);
    }

    let v1_path = args.v1.unwrap_or_else(|| default_artifact("v1"));
    let v2_path = args.v2.unwrap_or_else(|| default_artifact("v2"));
    let (v1, v2) = load_versions(&v1_path, &v2_path)?;

    let diffs = compare_all(&fixtures, &program_id, &v1, &v2)?;
    let mut report = Report::new(
        program_id.to_string(),
        v1_path.display().to_string(),
        v2_path.display().to_string(),
        &fixtures,
        diffs,
    );
    if !args.no_minimize {
        eplyx_lifecycle_impact::minimize_clusters(
            &mut report,
            &fixtures,
            &program_id,
            &v1,
            &v2,
            eplyx_lifecycle_impact::shrink::ShrinkConfig::default(),
        )?;
    }

    let rendered = match args.format {
        Format::Text => render_text(&report),
        Format::Json => report.to_json()?,
    };

    match &args.out {
        Some(path) => {
            std::fs::write(path, &rendered)
                .with_context(|| format!("writing report to {}", path.display()))?;
            eprintln!("wrote {}", path.display());
        }
        None => println!("{rendered}"),
    }

    if args.fail_on_critical && report.summary.critical > 0 {
        return Ok(ExitCode::from(1));
    }
    Ok(ExitCode::SUCCESS)
}

fn generate(args: GenerateArgs) -> Result<ExitCode> {
    let program_id = fixture_program_id();
    let fixtures = corpus::generate(&program_id);
    let out = args
        .out
        .unwrap_or_else(|| eplyx_lifecycle_impact::repo_root().join("fixtures/states"));
    std::fs::create_dir_all(&out).with_context(|| format!("creating {}", out.display()))?;

    for fixture in &fixtures {
        let path = out.join(format!("{}.json", fixture.id));
        std::fs::write(&path, serde_json::to_string_pretty(fixture)? + "\n")
            .with_context(|| format!("writing {}", path.display()))?;
    }

    let index: Vec<_> = fixtures
        .iter()
        .map(|f| {
            serde_json::json!({
                "id": f.id,
                "category": f.category.as_str(),
                "scenario": f.scenario,
            })
        })
        .collect();
    let index_path = out.join("index.json");
    std::fs::write(&index_path, serde_json::to_string_pretty(&index)? + "\n")?;

    println!("wrote {} fixtures to {}", fixtures.len(), out.display());
    Ok(ExitCode::SUCCESS)
}

fn reproduce(args: ReproduceArgs) -> Result<ExitCode> {
    let program_id = fixture_program_id();
    let fixtures = corpus::generate(&program_id);
    let v1_path = args.v1.unwrap_or_else(|| default_artifact("v1"));
    let v2_path = args.v2.unwrap_or_else(|| default_artifact("v2"));
    let (v1, v2) = load_versions(&v1_path, &v2_path)?;

    let target = args
        .target
        .strip_prefix("regression-group:")
        .unwrap_or(&args.target)
        .to_string();

    // A single fixture needs two executions; a regression group needs the whole
    // corpus, so try the cheap resolution first.
    if let Some(fixture) = fixtures.iter().find(|f| f.id == target) {
        let diff = eplyx_lifecycle_impact::compare_fixture(fixture, &program_id, &v1, &v2)?;
        println!(
            "{}",
            eplyx_lifecycle_impact::report::render_reproduction(&diff)
        );
        return Ok(ExitCode::SUCCESS);
    }

    let diffs = compare_all(&fixtures, &program_id, &v1, &v2)?;
    let mut report = Report::new(
        program_id.to_string(),
        v1_path.display().to_string(),
        v2_path.display().to_string(),
        &fixtures,
        diffs,
    );
    if !args.no_minimize {
        eplyx_lifecycle_impact::minimize_clusters(
            &mut report,
            &fixtures,
            &program_id,
            &v1,
            &v2,
            eplyx_lifecycle_impact::shrink::ShrinkConfig::default(),
        )?;
    }

    match report.cluster(&target) {
        Some(cluster) => {
            println!(
                "{}",
                eplyx_lifecycle_impact::report::render_cluster_reproduction(&report, cluster)
            );
            Ok(ExitCode::SUCCESS)
        }
        None => Err(anyhow!(
            "no fixture or regression group {target:?}\n\navailable regression groups:\n{}",
            report
                .clusters
                .iter()
                .map(|c| format!(
                    "  {:<34} {} fixtures{}",
                    c.id,
                    c.fixture_count(),
                    if c.critical { "  [CRITICAL]" } else { "" }
                ))
                .collect::<Vec<_>>()
                .join("\n")
        )),
    }
}

fn list(args: ListArgs) -> Result<ExitCode> {
    let program_id = fixture_program_id();
    let mut fixtures = corpus::generate(&program_id);
    if let Some(name) = &args.category {
        let category = parse_category(name)?;
        fixtures.retain(|f| f.category == category);
    }
    for fixture in &fixtures {
        println!(
            "{:<28} {:<22} {}",
            fixture.id,
            fixture.category.as_str(),
            fixture.scenario
        );
    }
    println!("\n{} fixtures", fixtures.len());
    Ok(ExitCode::SUCCESS)
}

fn compare_scenarios(args: CounterfactualArgs) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::{counterfactual::FrozenCounterfactualWorld, expansion};
    anyhow::ensure!(
        !args.out.as_ref().is_some_and(|p| p.exists()),
        "counterfactual output already exists"
    );
    let world =
        FrozenCounterfactualWorld::load(&args.snapshot, &args.scenario, &args.readiness_policy)?;
    let report = world.evaluate(&world.scenarios()?)?;
    if let Some(out) = &args.out {
        expansion::save(&report, out)?;
    }
    match args.format {
        Format::Json => print!("{}", report.to_json()?),
        Format::Text => print!("{}", report.render_text()),
    }
    Ok(ExitCode::SUCCESS)
}

fn evaluate_rollout(args: RolloutArgs, marker: Option<PathBuf>) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::{
        expansion,
        rollout::{
            run_guarded_stub, CandidateRolloutPlan, GateCommandCompletion, GuardRun,
            RolloutValidator,
        },
    };
    anyhow::ensure!(
        !args.out.as_ref().is_some_and(|p| p.exists()),
        "rollout output already exists"
    );
    anyhow::ensure!(
        !marker.as_ref().is_some_and(|p| p.exists()),
        "guard marker already exists"
    );
    let plan: CandidateRolloutPlan = expansion::load(&args.plan)?;
    let mut plan = plan.normalized()?;
    let validator = RolloutValidator::load(&args.binding)?;
    if let Some(target) = args.target_view {
        plan.target_view = validator
            .counterfactual()
            .scenarios
            .iter()
            .find(|view| view.scenario.id == target.id())
            .context("requested lifecycle view is unavailable")?
            .scenario
            .clone();
    }
    let evaluated = validator.evaluate(
        &plan,
        args.plan.parent().unwrap_or(std::path::Path::new(".")),
    )?;
    let code = evaluated.command_exit_code();
    if let Some(marker) = marker {
        let observation = run_guarded_stub(
            &evaluated,
            GateCommandCompletion::AssuranceEvaluation {
                exit_code: evaluated.report().readiness_exit_code,
            },
            &marker,
        )?;
        let result = GuardRun {
            assessment: evaluated.report().clone(),
            observation,
        };
        if let Some(out) = &args.out {
            expansion::save(&result, out)?;
        }
        match args.format {
            Format::Json => print!("{}", expansion::canonical(&result)?),
            Format::Text => println!(
                "{}Marker created: {}",
                evaluated.render_text(),
                result.observation.marker_created
            ),
        }
    } else {
        if let Some(out) = &args.out {
            expansion::save(evaluated.report(), out)?;
        }
        match args.format {
            Format::Json => print!("{}", evaluated.to_json()?),
            Format::Text => print!("{}", evaluated.render_text()),
        }
    }
    Ok(ExitCode::from(code))
}
fn demo_rollout(args: DemoRolloutArgs) -> Result<ExitCode> {
    use eplyx_lifecycle_impact::{
        expansion,
        rollout::{run_demo, RolloutDemoCases, RolloutValidator},
    };
    anyhow::ensure!(
        !args.out.as_ref().is_some_and(|p| p.exists()),
        "rollout demo output already exists"
    );
    let cases: RolloutDemoCases = expansion::load(&args.cases)?;
    let validator = RolloutValidator::load(&args.binding)?;
    let report = run_demo(
        &validator,
        &cases,
        args.cases.parent().unwrap_or(std::path::Path::new(".")),
        &args.marker_directory,
    )?;
    if let Some(out) = &args.out {
        expansion::save(&report, out)?;
    }
    match args.format {
        Format::Json => print!("{}", expansion::canonical(&report)?),
        Format::Text => {
            println!("ROLLOUT ASSUMPTION DEMONSTRATIONS (local-only)");
            for case in &report.cases {
                println!("{}: candidate {:?}; readiness {:?} ({:?}, exit {}); workflow {:?}; marker created {}",case.assessment.candidate.id,case.assessment.candidate_acceptance,case.assessment.readiness.overall_status,case.assessment.readiness.evaluated_scope,case.assessment.readiness_exit_code,case.observation.disposition,case.observation.marker_created);
            }
            println!("Demo command exit 0 is analysis completion, not population readiness or rollout authorization.");
        }
    }
    Ok(ExitCode::SUCCESS)
}
