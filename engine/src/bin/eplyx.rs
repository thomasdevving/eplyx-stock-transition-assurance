//! Local developer interface over the canonical transition package engine.
use anyhow::{bail, ensure, Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use clap::{Parser, Subcommand};
use eplyx_lifecycle_impact::{
    conversion::{
        demo, package,
        package_gate::{Outcome, Policy},
        package_preflight, search,
    },
    expansion::canonical,
    lifecycle::{
        exposure::sha256,
        rpc::{HttpSolanaRpc, SolanaRpc},
    },
    local_store::{
        counterexample_id, replay_inputs, reproduction_id, safe_id, Metadata, Reproduction,
        ReproductionOutcome, RunSource, SavedCounterexample, METADATA_VERSION,
        REPRODUCTION_VERSION,
    },
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

const MAINNET_GENESIS: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";

#[derive(Parser)]
#[command(
    name = "eplyx",
    about = "Local Solana transition preflight and counterexample search"
)]
struct Cli {
    #[arg(long, global = true, default_value = "eplyx.toml")]
    config: PathBuf,
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    Init {
        #[arg(long)]
        force: bool,
        #[arg(long)]
        minimal: bool,
    },
    Doctor,
    Preflight {
        #[arg(long, value_enum)]
        gate: Option<Policy>,
        #[arg(long)]
        verbose: bool,
    },
    Search {
        #[arg(long)]
        run: Option<String>,
        #[arg(long)]
        offline: bool,
    },
    Reproduce {
        id: String,
    },
    Runs {
        #[arg(long)]
        json: bool,
    },
    Show {
        id: String,
    },
    /// Serve a read-only local dashboard over `.eplyx/` on 127.0.0.1.
    Dashboard {
        /// Port on 127.0.0.1; defaults to the first free port from 4173.
        #[arg(long)]
        port: Option<u16>,
        /// Print the URL without opening a browser.
        #[arg(long)]
        no_open: bool,
    },
    #[command(hide = true)]
    FinishPackagePreflight {
        package: PathBuf,
        #[arg(long)]
        result: PathBuf,
    },
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectConfig {
    project: Project,
    transition: Transition,
    program: Program,
    terms: TermConfig,
    execution: Execution,
    #[serde(default)]
    invariants: Vec<package::InvariantDefinition>,
    gate: Gate,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Project {
    name: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Transition {
    source_mint: String,
    replacement_mint: String,
    adapter: String,
    effective_at: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Program {
    path: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TermConfig {
    numerator: String,
    denominator: String,
    rounding: String,
    fee_bps: u16,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Execution {
    public_owner: String,
    source_account: String,
    #[serde(default)]
    amount_decimal: Option<String>,
    reserve_funded_replacement_raw: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Gate {
    policy: Policy,
}

fn main() -> ExitCode {
    match execute(Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn root_and_config(input: &Path) -> Result<(PathBuf, PathBuf)> {
    let absolute = if input.is_absolute() {
        input.to_path_buf()
    } else {
        std::env::current_dir()?.join(input)
    };
    let root = absolute
        .parent()
        .context("config has no parent")?
        .canonicalize()?;
    ensure!(absolute.file_name().is_some(), "invalid config path");
    Ok((root, absolute))
}

fn read_config(path: &Path) -> Result<ProjectConfig> {
    ensure!(!path.is_symlink(), "config symlink is not supported");
    let size = fs::metadata(path)
        .with_context(|| format!("missing {}; run `eplyx init`", path.display()))?
        .len();
    ensure!(size <= 32 * 1024, "eplyx.toml exceeds 32 KiB");
    let bytes =
        fs::read(path).with_context(|| format!("missing {}; run `eplyx init`", path.display()))?;
    ensure!(bytes.len() <= 32 * 1024, "eplyx.toml exceeds 32 KiB");
    let body = std::str::from_utf8(&bytes)?;
    let document: toml_edit::DocumentMut = body.parse().context("invalid eplyx.toml")?;
    let parsed: ProjectConfig = serde_json::from_value(table_json(document.as_table())?)
        .context("invalid eplyx.toml fields")?;
    ensure!(
        !parsed.project.name.trim().is_empty() && parsed.project.name.len() <= 80,
        "set a project name of 1–80 characters"
    );
    ensure!(
        parsed.invariants.len() <= package::MAX_INVARIANTS,
        "too many invariants"
    );
    Ok(parsed)
}

fn table_json(table: &toml_edit::Table) -> Result<Value> {
    let mut map = serde_json::Map::new();
    for (key, item) in table.iter() {
        let value = if let Some(nested) = item.as_table() {
            table_json(nested)?
        } else if let Some(array) = item.as_array_of_tables() {
            Value::Array(array.iter().map(table_json).collect::<Result<Vec<_>>>()?)
        } else if let Some(value) = item.as_value() {
            if let Some(text) = value.as_str() {
                Value::String(text.into())
            } else if let Some(number) = value.as_integer() {
                json!(number)
            } else if let Some(boolean) = value.as_bool() {
                json!(boolean)
            } else {
                bail!("unsupported eplyx.toml value for {key}");
            }
        } else {
            bail!("unsupported eplyx.toml item for {key}");
        };
        map.insert(key.into(), value);
    }
    Ok(Value::Object(map))
}

fn program_path(root: &Path, config: &ProjectConfig) -> Result<PathBuf> {
    let relative = Path::new(&config.program.path);
    ensure!(
        !relative.is_absolute()
            && relative.components().all(|c| matches!(
                c,
                std::path::Component::CurDir | std::path::Component::Normal(_)
            )),
        "program path must stay relative to project root"
    );
    let full = root
        .join(relative)
        .canonicalize()
        .context("candidate program missing; build SBF and set [program].path")?;
    ensure!(
        full.starts_with(root) && full.is_file(),
        "candidate program path escapes project root or is not a file"
    );
    Ok(full)
}

fn candidate_bytes(root: &Path, config: &ProjectConfig) -> Result<Vec<u8>> {
    let path = program_path(root, config)?;
    ensure!(
        fs::metadata(&path)?.len() <= package::MAX_PROGRAM_BYTES,
        "candidate program exceeds package bound"
    );
    let bytes = fs::read(path)?;
    ensure!(
        bytes.len() as u64 <= package::MAX_PROGRAM_BYTES,
        "candidate program exceeds package bound"
    );
    Ok(bytes)
}

fn store(root: &Path, name: &str) -> Result<PathBuf> {
    let base = root.join(".eplyx");
    if base.exists() {
        ensure!(
            base.canonicalize()? == base && base.is_dir(),
            ".eplyx must be a real project directory"
        );
    } else {
        fs::create_dir(&base)?;
    }
    for child in ["runs", "counterexamples", "cache"] {
        let path = base.join(child);
        if path.exists() {
            ensure!(
                path.canonicalize()? == path && path.is_dir(),
                "invalid .eplyx store child"
            );
        } else {
            fs::create_dir(&path)?;
        }
    }
    let project_file = base.join("project.json");
    if !project_file.exists() {
        let id = format!(
            "project_{}",
            &sha256(root.to_string_lossy().as_bytes())[..20]
        );
        write_new(
            &project_file,
            serde_json::to_vec_pretty(&json!({"schema_version":1,"id":id,"name":name}))?.as_slice(),
        )?;
    }
    Ok(base)
}

fn existing_store(root: &Path) -> Result<PathBuf> {
    let base = root.join(".eplyx");
    ensure!(
        base.is_dir() && base.canonicalize()? == base,
        "local run store missing or invalid; run `eplyx init`"
    );
    for child in ["runs", "counterexamples"] {
        let member = base.join(child);
        ensure!(
            member.is_dir() && member.canonicalize()? == member,
            "local run store is invalid"
        );
    }
    Ok(base)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn package_in(
    root: &Path,
    config: &ProjectConfig,
    destination: &Path,
) -> Result<package::ValidatedPackage> {
    let bytes = candidate_bytes(root, config)?;
    let effective_at = DateTime::parse_from_rfc3339(&config.transition.effective_at)
        .context("set [transition].effective_at to an RFC 3339 UTC timestamp")?
        .with_timezone(&Utc);
    let execution = package::Config {
        public_owner: config.execution.public_owner.clone(),
        source_account: config.execution.source_account.clone(),
        amount_decimal: config.execution.amount_decimal.clone(),
        reserve_funded_replacement_raw: config.execution.reserve_funded_replacement_raw.clone(),
    };
    let config_bytes = serde_json::to_vec(&execution)?;
    let invariant_version =
        (!config.invariants.is_empty()).then_some(package::INVARIANT_SCHEMA_VERSION);
    let manifest = package::Manifest {
        schema_version: if invariant_version.is_some() {
            package::INVARIANT_PACKAGE_VERSION
        } else {
            package::VERSION
        },
        source_mint: config.transition.source_mint.clone(),
        replacement_mint: config.transition.replacement_mint.clone(),
        adapter: config.transition.adapter.clone(),
        candidate_program: package::CandidateProgram {
            program_id: demo::PROGRAM_ID.into(),
            artifact: "program.so".into(),
            sha256: sha256(&bytes),
        },
        terms: package::Terms {
            numerator: config.terms.numerator.clone(),
            denominator: config.terms.denominator.clone(),
            rounding: config.terms.rounding.clone(),
            fee_bps: config.terms.fee_bps,
        },
        effective_at,
        config: "config.json".into(),
        config_sha256: sha256(&config_bytes),
        invariant_schema_version: invariant_version,
        invariants: invariant_version.map(|_| config.invariants.clone()),
    };
    fs::create_dir(destination)?;
    write_new(
        &destination.join("eplyx.json"),
        canonical(&manifest)?.as_bytes(),
    )?;
    write_new(&destination.join("config.json"), &config_bytes)?;
    write_new(&destination.join("program.so"), &bytes)?;
    let validated = package::load(destination)?;
    validated.conversion_plan()?;
    Ok(validated)
}

fn validate_ephemeral(
    root: &Path,
    config: &ProjectConfig,
    base: &Path,
) -> Result<package::ValidatedPackage> {
    let path = base.join("cache").join(format!(
        "validate_{}_{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    let result = package_in(root, config, &path);
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    result
}

fn rpc_url() -> Result<String> {
    let url = std::env::var("SOLANA_RPC_URL")
        .context("set SOLANA_RPC_URL to a Solana mainnet RPC provider")?;
    ensure!(
        !url.trim().is_empty(),
        "set SOLANA_RPC_URL to a Solana mainnet RPC provider"
    );
    Ok(url)
}

fn check_rpc() -> Result<()> {
    let rpc = HttpSolanaRpc::bounded(&rpc_url()?)?;
    check_genesis(&rpc)
}

fn check_genesis(rpc: &impl SolanaRpc) -> Result<()> {
    let genesis = rpc.call("getGenesisHash", json!([]))?;
    ensure!(
        genesis.as_str() == Some(MAINNET_GENESIS),
        "RPC genesis is not Solana mainnet; set SOLANA_RPC_URL to a mainnet provider"
    );
    Ok(())
}

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn metadata(
    root: &Path,
    id: &str,
    package: &package::ValidatedPackage,
    policy: Policy,
    outcome: &str,
) -> Result<Metadata> {
    let status = git(root, &["status", "--porcelain"]);
    Ok(Metadata {
        schema_version: METADATA_VERSION,
        run_id: id.into(),
        timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        eplyx_version: env!("CARGO_PKG_VERSION").into(),
        engine_binary_sha256: sha256(&fs::read(std::env::current_exe()?)?),
        git_commit: git(root, &["rev-parse", "HEAD"]),
        git_branch: git(root, &["branch", "--show-current"]),
        git_dirty: status.map(|s| !s.is_empty()),
        candidate_program_sha256: package.program_sha256.clone(),
        transition_package_sha256: package.transition_package_sha256.clone(),
        gate_policy: policy.name().into(),
        gate_outcome: outcome.into(),
        run_source: Some(RunSource::detect()),
    })
}

fn run_dir(base: &Path, id: &str) -> Result<PathBuf> {
    safe_id(id, "run_")?;
    let path = base.join("runs").join(id);
    ensure!(
        path.canonicalize()? == path,
        "run path must not be a symlink"
    );
    for member in ["package", "result"] {
        let child = path.join(member);
        ensure!(
            child.canonicalize()? == child,
            "run member must not be a symlink"
        );
    }
    Ok(path)
}

fn print_report(
    program: &str,
    package: &package::ValidatedPackage,
    report: &Value,
    path: &Path,
    verbose: bool,
) -> Result<()> {
    let cases = report["stress_results"].as_array().map_or(0, Vec::len);
    let proven = report["stress_results"].as_array().map_or(0, |xs| {
        xs.iter().filter(|x| x["status"] == "Proven").count()
    });
    let population = &report["population_summary"];
    let outcome: Outcome = serde_json::from_value(report["deployment_gate"]["outcome"].clone())?;
    let invariant_lines = report["invariants"].as_array().map_or_else(
        || "  None configured".to_string(),
        |findings| {
            findings
                .iter()
                .map(|finding| {
                    let marker = match finding["status"].as_str() {
                        Some("Satisfied") => "✓",
                        Some("Violated") => "✗",
                        Some("Indeterminate") => "?",
                        _ => "-",
                    };
                    format!(
                        "  {marker} {}",
                        finding["invariant_type"]
                            .as_str()
                            .unwrap_or("unknown")
                            .replace('_', " ")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        },
    );
    println!("Eplyx Pre-flight\n\nCandidate\n  {program}\n  {}\n\nProduction\n  {} token accounts observed\n  {} positive balances\n  Population readiness: {}\n\nCandidate conversion\n  {}\n  Official transition: {}\n\nStress\n  {proven} / {cases} exact states passed\n\nInvariants\n{invariant_lines}\n\nDeployment\n  {}\n\nRun\n  {}\n\nNo mainnet funds moved.",
        &package.program_sha256[..12],
        population["counts"]["token_accounts_observed"].as_u64().unwrap_or(0),
        population["counts"]["positive_balance_accounts_observed"].as_u64().unwrap_or(0),
        report["population_rollout_readiness"]["status"].as_str().unwrap_or("Unknown"),
        report["conversion_result"]["status"].as_str().unwrap_or("Unknown"),
        report["official_transition"].as_str().unwrap_or("Unknown"),
        outcome.label(), path.display());
    if verbose {
        println!("\n{}", fs::read_to_string(path.join("result/report.md"))?);
    }
    Ok(())
}

fn preflight(
    root: &Path,
    config: &ProjectConfig,
    base: &Path,
    gate: Policy,
    verbose: bool,
) -> Result<(String, u8)> {
    let _ = rpc_url()?;
    let hash = sha256(&candidate_bytes(root, config)?);
    let id = format!(
        "run_{}_{}",
        Utc::now().format("%Y%m%d%H%M%S%3f"),
        &hash[..12]
    );
    let path = base.join("runs").join(&id);
    fs::create_dir(&path)?;
    let package = package_in(root, config, &path.join("package"))?;
    let report = package_preflight::run(&path.join("package"), &path.join("result"), gate)?;
    let exit = package_preflight::exit_code(&report)?;
    let outcome = report["gate_outcome"].as_str().unwrap_or("Unknown");
    let meta = metadata(root, &id, &package, gate, outcome)?;
    write_new(
        &path.join("metadata.json"),
        serde_json::to_vec_pretty(&meta)?.as_slice(),
    )?;
    print_report(&config.program.path, &package, &report, &path, verbose)?;
    Ok((id, exit))
}

fn verify_run(
    root: &Path,
    config: &ProjectConfig,
    base: &Path,
    id: &str,
) -> Result<(PathBuf, Value)> {
    let path = run_dir(base, id)?;
    let meta: Metadata = serde_json::from_slice(&fs::read(path.join("metadata.json"))?)?;
    ensure!(meta.run_id == id, "run metadata ID mismatch");
    let expected = validate_ephemeral(root, config, base)?;
    let saved = package::load(&path.join("package"))?;
    ensure!(
        expected.transition_package_sha256 == saved.transition_package_sha256
            && expected.program_sha256 == saved.program_sha256
            && meta.transition_package_sha256 == saved.transition_package_sha256
            && meta.candidate_program_sha256 == saved.program_sha256,
        "run is incompatible with current config or candidate binary"
    );
    let report = package_preflight::replay(&path.join("package"), &path.join("result"))?;
    Ok((path, report))
}

fn counterexample_count(base: &Path, run: &str) -> Result<usize> {
    let mut count = 0;
    for entry in fs::read_dir(base.join("counterexamples"))? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let bytes = fs::read(entry.path())?;
        if bytes.len() > 1024 * 1024 {
            continue;
        }
        if let Ok(saved) = serde_json::from_slice::<SavedCounterexample>(&bytes) {
            if saved.parent_run == run {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn local_runs(base: &Path) -> Result<Vec<Value>> {
    let mut rows = Vec::new();
    for entry in fs::read_dir(base.join("runs"))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        if let Ok(bytes) = fs::read(entry.path().join("metadata.json")) {
            let meta: Metadata = serde_json::from_slice(&bytes)?;
            rows.push(json!({"run":meta.run_id,"commit":meta.git_commit,"result":meta.gate_outcome,"counterexamples":counterexample_count(base, &meta.run_id)?}));
        }
    }
    rows.sort_by(|a, b| a["run"].as_str().cmp(&b["run"].as_str()));
    Ok(rows)
}

fn search_run(
    root: &Path,
    config: &ProjectConfig,
    base: &Path,
    selected: Option<String>,
    offline: bool,
) -> Result<()> {
    let id = match selected {
        Some(id) => id,
        None => preflight(root, config, base, config.gate.policy, false)?.0,
    };
    let (path, _) = verify_run(root, config, base, &id)?;
    if !offline {
        let _ = rpc_url()?;
    }
    let result = search::run(
        &path.join("package"),
        &path.join("result"),
        &path.join("search"),
        !offline,
    )?;
    let search_sha256 = sha256(&fs::read(path.join("search/counterexamples.json"))?);
    let observed = result
        .counterexamples
        .iter()
        .filter(|c| matches!(c, search::Counterexample::Observed { .. }))
        .count();
    let derived = result.counterexamples.len() - observed;
    println!("\nEplyx Counterexample Search\n\nObserved executions  {}\nObserved failures    {observed}\nDerived failures     {derived}\nBoundary VM probes   {}\n\n{}",
        result.budget.observed_executions, result.budget.boundary_executions, result.conclusion);
    let example = result
        .counterexamples
        .iter()
        .find(|c| {
            matches!(
                c,
                search::Counterexample::Derived {
                    search_dimension: search::Dimension::ProposedReserve,
                    ..
                }
            )
        })
        .or_else(|| {
            result
                .counterexamples
                .iter()
                .find(|c| matches!(c, search::Counterexample::Derived { .. }))
        })
        .or_else(|| result.counterexamples.first());
    if let Some(example) = example {
        match example {
            search::Counterexample::Derived {
                search_dimension,
                derived_value_raw,
                first_passing_value_raw,
                last_passing_value_raw,
                failure_signature,
                ..
            } => {
                println!("\nExample {}\n  Dimension  {:?}\n  Failing value  {}\n  Passing boundary  {}\n  Failure  {}\n  Rollback verified  {}",
                    counterexample_id(example)?, search_dimension, derived_value_raw,
                    first_passing_value_raw.as_deref().or(last_passing_value_raw.as_deref()).unwrap_or("unknown"),
                    failure_signature.instruction_error, failure_signature.rollback_verified);
            }
            search::Counterexample::Observed {
                failure_signature, ..
            } => {
                println!(
                    "\nExample {}\n  Observed exact failure  {}\n  Rollback verified  {}",
                    counterexample_id(example)?,
                    failure_signature.instruction_error,
                    failure_signature.rollback_verified
                );
            }
        }
    }
    let count = result.counterexamples.len();
    for counterexample in result.counterexamples {
        let cx_id = counterexample_id(&counterexample)?;
        let saved = SavedCounterexample {
            schema_version: 1,
            id: cx_id.clone(),
            parent_run: id.clone(),
            search_sha256: search_sha256.clone(),
            replay_inputs: Some(replay_inputs(&id)),
            counterexample,
        };
        let dest = base.join("counterexamples").join(format!("{cx_id}.json"));
        write_new(&dest, serde_json::to_vec_pretty(&saved)?.as_slice())?;
    }
    println!(
        "\nSaved {count} counterexamples under {}\nRun {}\n\nNo mainnet funds moved.",
        base.join("counterexamples").display(),
        path.display()
    );
    Ok(())
}

fn reproduce(base: &Path, id: &str) -> Result<()> {
    safe_id(id, "cx_")?;
    std::env::remove_var("SOLANA_RPC_URL");
    let mut parent = None;
    let result = verify_reproduction(base, id, &mut parent);
    // History is recorded for successes and failures alike; a recording
    // problem is reported but never changes the verification outcome.
    match record_reproduction(base, id, parent, &result) {
        Ok(path) => {
            if result.is_ok() {
                println!("Reproduced {id} successfully. Failure signature and rollback verified by offline VM replay.\nNo network used.\nRecorded {path}");
            }
        }
        Err(error) => eprintln!("warning: reproduction history not recorded: {error:#}"),
    }
    result
}

fn verify_reproduction(
    base: &Path,
    id: &str,
    parent: &mut Option<SavedCounterexample>,
) -> Result<()> {
    let saved: SavedCounterexample = serde_json::from_slice(&fs::read(
        base.join("counterexamples").join(format!("{id}.json")),
    )?)?;
    *parent = Some(saved.clone());
    ensure!(
        saved.schema_version == 1
            && saved.id == id
            && saved
                .replay_inputs
                .as_ref()
                .is_none_or(|inputs| inputs == &replay_inputs(&saved.parent_run))
            && counterexample_id(&saved.counterexample)? == id,
        "counterexample identity mismatch"
    );
    let path = run_dir(base, &saved.parent_run)?;
    ensure!(
        path.join("search").canonicalize()? == path.join("search"),
        "search path must not be a symlink"
    );
    ensure!(
        sha256(&fs::read(path.join("search/counterexamples.json"))?) == saved.search_sha256,
        "search artifact digest mismatch"
    );
    let _ = package_preflight::replay(&path.join("package"), &path.join("result"))?;
    let verified = search::replay(
        &path.join("package"),
        &path.join("result"),
        &path.join("search"),
    )?;
    ensure!(
        verified.counterexamples.contains(&saved.counterexample),
        "counterexample missing from offline replay"
    );
    Ok(())
}

fn record_reproduction(
    base: &Path,
    id: &str,
    saved: Option<SavedCounterexample>,
    result: &Result<()>,
) -> Result<String> {
    let directory = base.join("reproductions");
    if directory.exists() {
        ensure!(
            directory.canonicalize()? == directory && directory.is_dir(),
            "invalid .eplyx/reproductions"
        );
    } else {
        fs::create_dir(&directory)?;
    }
    let now = Utc::now();
    let record = Reproduction {
        schema_version: REPRODUCTION_VERSION,
        id: reproduction_id(&now.format("%Y%m%d%H%M%S%3f").to_string(), id),
        counterexample_id: id.into(),
        parent_run: saved.as_ref().map(|s| s.parent_run.clone()),
        search_sha256: saved.as_ref().map(|s| s.search_sha256.clone()),
        timestamp: now.to_rfc3339_opts(SecondsFormat::Millis, true),
        outcome: if result.is_ok() {
            ReproductionOutcome::Reproduced
        } else {
            ReproductionOutcome::Failed
        },
        failure_signature_matched: result.is_ok(),
        error: result.as_ref().err().map(|error| {
            format!("{error:#}").replace(&base.to_string_lossy().into_owned(), ".eplyx")
        }),
        eplyx_version: env!("CARGO_PKG_VERSION").into(),
        engine_binary_sha256: sha256(&fs::read(std::env::current_exe()?)?),
        no_rpc: std::env::var_os("SOLANA_RPC_URL").is_none(),
    };
    let name = format!("{}.json", record.id);
    write_new(
        &directory.join(&name),
        serde_json::to_vec_pretty(&record)?.as_slice(),
    )?;
    Ok(format!(".eplyx/reproductions/{name}"))
}

/// Project facts for the dashboard that live outside `.eplyx/`: the current
/// config and candidate, and Git state. Read once, locally; never an RPC URL.
fn dashboard_context(root: &Path, path: &Path) -> Value {
    let hide_root = |text: String| text.replace(&root.to_string_lossy().into_owned(), "<project>");
    let config_path = path
        .strip_prefix(root)
        .map(eplyx_lifecycle_impact::artifact_path)
        .unwrap_or_else(|_| "eplyx.toml".into());
    let (config, candidate) = if !path.exists() {
        (
            json!({"path": config_path, "state": "Missing"}),
            json!({"error": "eplyx.toml is missing"}),
        )
    } else {
        match read_config(path) {
            Ok(config) => (
                json!({
                    "path": config_path,
                    "state": "Valid",
                    "name": config.project.name,
                    "program_path": config.program.path,
                    "adapter": config.transition.adapter,
                    "gate_policy": config.gate.policy.name(),
                    "source_mint": config.transition.source_mint,
                    "replacement_mint": config.transition.replacement_mint,
                }),
                match candidate_bytes(root, &config) {
                    Ok(bytes) => json!({"path": config.program.path, "sha256": sha256(&bytes)}),
                    Err(error) => {
                        json!({"path": config.program.path, "error": hide_root(format!("{error:#}"))})
                    }
                },
            ),
            Err(error) => (
                json!({"path": config_path, "state": "Invalid", "error": hide_root(format!("{error:#}"))}),
                json!({"error": "eplyx.toml is invalid"}),
            ),
        }
    };
    json!({
        "config": config,
        "candidate": candidate,
        "git": {
            "branch": git(root, &["branch", "--show-current"]).filter(|b| !b.is_empty()),
            "commit": git(root, &["rev-parse", "HEAD"]),
            "dirty": git(root, &["status", "--porcelain"]).map(|s| !s.is_empty()),
        },
        "version": env!("CARGO_PKG_VERSION"),
    })
}

fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", ""]);
        command
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = Command::new("xdg-open");
    let _ = command
        .arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

fn dashboard(root: &Path, path: &Path, port: Option<u16>, no_open: bool) -> Result<()> {
    use std::io::IsTerminal;
    // The dashboard reads local artifacts only; it never needs a provider.
    std::env::remove_var("SOLANA_RPC_URL");
    let server = eplyx_lifecycle_impact::dashboard::Dashboard::bind(
        root,
        port,
        dashboard_context(root, path),
    )?;
    let (name, runs, counterexamples) = server.banner()?;
    let url = server.url();
    println!("Eplyx dashboard\nProject: {name}\nRuns: {runs}\nCounterexamples: {counterexamples}\n\n{url}\n\nRead-only view of .eplyx/ on 127.0.0.1. Nothing is uploaded. Press Ctrl+C to stop.");
    if !no_open && std::io::stdout().is_terminal() && std::env::var_os("CI").is_none() {
        open_browser(&url);
    }
    server.serve()
}

fn init(root: &Path, path: &Path, force: bool, minimal: bool) -> Result<()> {
    let ignore = root.join(".gitignore");
    if ignore.exists() {
        ensure!(!ignore.is_symlink(), "refusing to edit .gitignore symlink");
    }
    if path.exists() {
        ensure!(
            force,
            "{} already exists; use `eplyx init --force` to replace it",
            path.display()
        );
        ensure!(!path.is_symlink(), "refusing to replace config symlink");
    }
    let name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("my-transition");
    let deploy = root.join("target/deploy");
    let mut candidates = if deploy.is_dir() {
        fs::read_dir(&deploy)?
            .filter_map(|item| item.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "so"))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    candidates.sort();
    let program = if candidates.len() == 1 {
        format!(
            "./target/deploy/{}",
            candidates[0]
                .file_name()
                .context("invalid deploy filename")?
                .to_string_lossy()
        )
    } else {
        "./target/deploy/migration.so".into()
    };
    let mut template = format!("# Eplyx local transition package. Fill the blank public addresses before doctor.\n[project]\nname = {name:?}\n\n[transition]\nsource_mint = \"\"\nreplacement_mint = \"\"\nadapter = \"fixed_ratio_conversion_v1\"\neffective_at = \"2026-10-01T00:00:00Z\"\n\n[program]\npath = {program:?}\n\n[terms]\nnumerator = \"1\"\ndenominator = \"2\"\nrounding = \"floor\"\nfee_bps = 0\n\n[execution]\npublic_owner = \"\"\nsource_account = \"\"\n# amount_decimal = \"0.000001\"  # omit for the full public balance\nreserve_funded_replacement_raw = \"0\"\n\n[gate]\npolicy = \"block-only\"\n");
    if !minimal {
        template.push_str("\n[[invariants]]\ntype = \"conversion_output_matches\"\nseverity = \"blocking\"\n\n[[invariants]]\ntype = \"no_selected_case_failed\"\nseverity = \"blocking\"\n");
    }
    if force {
        fs::write(path, template)?;
    } else {
        write_new(path, template.as_bytes())?;
    }
    let base = store(root, name)?;
    let existing = fs::read_to_string(&ignore).unwrap_or_default();
    if !existing.lines().any(|line| line.trim() == ".eplyx/") {
        let mut file = OpenOptions::new().append(true).create(true).open(&ignore)?;
        if !existing.is_empty() && !existing.ends_with('\n') {
            writeln!(file)?;
        }
        writeln!(file, ".eplyx/")?;
    }
    println!(
        "Created {} and {}. Edit the mint, account, and reserve fields, then run `eplyx doctor`.",
        path.display(),
        base.display()
    );
    Ok(())
}

fn execute(cli: Cli) -> Result<u8> {
    let (root, path) = root_and_config(&cli.config)?;
    if let Action::FinishPackagePreflight { package, result } = &cli.command {
        package_preflight::finish(package, result)?;
        return Ok(0);
    }
    if let Action::Init { force, minimal } = cli.command {
        init(&root, &path, force, minimal)?;
        return Ok(0);
    }
    let config = if matches!(
        &cli.command,
        Action::Reproduce { .. }
            | Action::Runs { .. }
            | Action::Show { .. }
            | Action::Dashboard { .. }
    ) {
        None
    } else {
        Some(read_config(&path)?)
    };
    let base = match config.as_ref() {
        Some(config) => store(&root, &config.project.name)?,
        None => existing_store(&root)?,
    };
    match cli.command {
        Action::Doctor => {
            let config = config.as_ref().context("config required for doctor")?;
            let package = validate_ephemeral(&root, config, &base)
                .context("fix eplyx.toml or rebuild the candidate SBF program")?;
            check_rpc()?;
            println!("Eplyx doctor\n\n✓ Config              {}\n✓ Candidate program  {}\n✓ Program hash       {}\n✓ Adapter            {}\n✓ RPC                Solana mainnet\n✓ Local run store    {}\n\nReady to preflight.",
                path.display(), config.program.path, &package.program_sha256[..12], config.transition.adapter, base.display());
            Ok(0)
        }
        Action::Preflight { gate, verbose } => {
            let config = config.as_ref().context("config required for preflight")?;
            if !verbose {
                std::env::set_var("EPLYX_CLI_QUIET", "1");
            }
            Ok(preflight(
                &root,
                config,
                &base,
                gate.unwrap_or(config.gate.policy),
                verbose,
            )?
            .1)
        }
        Action::Search { run, offline } => {
            let config = config.as_ref().context("config required for search")?;
            std::env::set_var("EPLYX_CLI_QUIET", "1");
            search_run(&root, config, &base, run, offline)?;
            Ok(0)
        }
        Action::Reproduce { id } => {
            std::env::set_var("EPLYX_CLI_QUIET", "1");
            reproduce(&base, &id)?;
            Ok(0)
        }
        Action::Runs { json } => {
            let rows = local_runs(&base)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else {
                println!("RUN                              COMMIT       RESULT   COUNTEREXAMPLES");
                for row in rows {
                    println!(
                        "{:<32} {:<12} {:<8} {}",
                        row["run"].as_str().unwrap_or(""),
                        row["commit"]
                            .as_str()
                            .unwrap_or("-")
                            .chars()
                            .take(10)
                            .collect::<String>(),
                        row["result"].as_str().unwrap_or(""),
                        row["counterexamples"]
                    );
                }
            }
            Ok(0)
        }
        Action::Show { id } => {
            let path = run_dir(&base, &id)?;
            let meta: Metadata = serde_json::from_slice(&fs::read(path.join("metadata.json"))?)?;
            let report: Value =
                serde_json::from_slice(&fs::read(path.join("result/report.json"))?)?;
            println!("Run {}\nPackage {}\nProgram {}\nCommit {}\nBranch {}\nPopulation: {} observed accounts; {} positive balances\nConversion {}\nStress {}\nInvariant findings {}\nGate {}\nCounterexamples {}\nArtifacts {}", id, meta.transition_package_sha256, meta.candidate_program_sha256, meta.git_commit.as_deref().unwrap_or("unknown"), meta.git_branch.as_deref().unwrap_or("unknown"), report["population_summary"]["counts"]["token_accounts_observed"], report["population_summary"]["counts"]["positive_balance_accounts_observed"], report["conversion_result"]["status"], report["conversion_stress_readiness"]["status"], report["invariants"].as_array().map_or(0, Vec::len), meta.gate_outcome, counterexample_count(&base, &id)?, path.display());
            Ok(0)
        }
        Action::Dashboard { port, no_open } => {
            dashboard(&root, &path, port, no_open)?;
            Ok(0)
        }
        Action::Init { .. } | Action::FinishPackagePreflight { .. } => {
            bail!("internal command dispatch error")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct GenesisRpc(&'static str);
    impl SolanaRpc for GenesisRpc {
        fn call(&self, method: &str, _params: Value) -> Result<Value> {
            ensure!(method == "getGenesisHash", "unexpected RPC method");
            Ok(json!(self.0))
        }
        fn origin(&self) -> String {
            "local-test".into()
        }
    }

    fn temp_project() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "eplyx-cli-{}-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path.canonicalize().unwrap()
    }

    fn configured_project() -> (PathBuf, ProjectConfig) {
        let root = temp_project();
        let source =
            eplyx_lifecycle_impact::repo_root().join("examples/transitions/demo-fixed-ratio");
        fs::create_dir_all(root.join("target/deploy")).unwrap();
        fs::copy(
            source.join("program.so"),
            root.join("target/deploy/migration.so"),
        )
        .unwrap();
        let manifest: package::Manifest =
            serde_json::from_slice(&fs::read(source.join("eplyx.json")).unwrap()).unwrap();
        let execution: package::Config =
            serde_json::from_slice(&fs::read(source.join("config.json")).unwrap()).unwrap();
        let body = format!("[project]\nname = \"fixture\"\n[transition]\nsource_mint = {:?}\nreplacement_mint = {:?}\nadapter = \"fixed_ratio_conversion_v1\"\neffective_at = \"2026-10-01T00:00:00Z\"\n[program]\npath = \"./target/deploy/migration.so\"\n[terms]\nnumerator = \"1\"\ndenominator = \"2\"\nrounding = \"floor\"\nfee_bps = 0\n[execution]\npublic_owner = {:?}\nsource_account = {:?}\namount_decimal = \"0.000001\"\nreserve_funded_replacement_raw = \"1000000000000\"\n[gate]\npolicy = \"block-only\"\n[[invariants]]\ntype = \"conversion_output_matches\"\nseverity = \"blocking\"\n", manifest.source_mint, manifest.replacement_mint, execution.public_owner, execution.source_account);
        fs::write(root.join("eplyx.toml"), body).unwrap();
        let config = read_config(&root.join("eplyx.toml")).unwrap();
        (root, config)
    }

    #[test]
    fn init_parses_and_refuses_overwrite() {
        let root = temp_project();
        let path = root.join("eplyx.toml");
        init(&root, &path, false, false).unwrap();
        assert!(
            read_config(&path).is_ok(),
            "init must create parseable TOML"
        );
        assert!(init(&root, &path, false, false).is_err());
        assert!(root.join(".eplyx/project.json").is_file());
        assert!(fs::read_to_string(root.join(".gitignore"))
            .unwrap()
            .contains(".eplyx/"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn package_identity_and_relative_path_are_exact() {
        let (root, mut config) = configured_project();
        let original = package_in(&root, &config, &root.join("one")).unwrap();
        assert_eq!(original.manifest.config, "config.json");
        assert_eq!(original.manifest.candidate_program.artifact, "program.so");
        assert_eq!(
            replay_inputs("run_example").package,
            "runs/run_example/package"
        );
        let repeat = package_in(&root, &config, &root.join("two")).unwrap();
        assert_eq!(
            original.transition_package_sha256,
            repeat.transition_package_sha256
        );
        config
            .invariants
            .push(package::InvariantDefinition::NoSelectedCaseFailed {
                severity: eplyx_lifecycle_impact::conversion::invariants::Severity::Blocking,
            });
        let changed = package_in(&root, &config, &root.join("three")).unwrap();
        assert_ne!(
            original.transition_package_sha256,
            changed.transition_package_sha256
        );
        let program = root.join("target/deploy/migration.so");
        let mut bytes = fs::read(&program).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        fs::write(&program, bytes).unwrap();
        let changed_program = package_in(&root, &config, &root.join("four")).unwrap();
        assert_ne!(
            changed.transition_package_sha256,
            changed_program.transition_package_sha256
        );
        assert!(program_path(&root, &config).unwrap().starts_with(&root));
        config.program.path = "../escape.so".into();
        assert!(program_path(&root, &config).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_program_and_local_ids_fail_closed() {
        let (root, mut config) = configured_project();
        config.program.path = "./target/deploy/missing.so".into();
        assert!(program_path(&root, &config).is_err());
        assert!(safe_id("../run_abc", "run_").is_err());
        assert!(safe_id("cx_abc", "cx_").is_ok());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn program_symlink_cannot_escape_project() {
        let (root, mut config) = configured_project();
        let outside = temp_project();
        fs::copy(
            root.join("target/deploy/migration.so"),
            outside.join("program.so"),
        )
        .unwrap();
        std::os::unix::fs::symlink(
            outside.join("program.so"),
            root.join("target/deploy/link.so"),
        )
        .unwrap();
        config.program.path = "./target/deploy/link.so".into();
        assert!(program_path(&root, &config).is_err());
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn config_rejects_rpc_and_claimed_proof_fields() {
        let (root, _) = configured_project();
        let path = root.join("eplyx.toml");
        let mut body = fs::read_to_string(&path).unwrap();
        body.push_str("\n[rpc]\nurl = \"https://provider.example/secret\"\n");
        fs::write(&path, body).unwrap();
        assert!(read_config(&path).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn oversized_config_and_candidate_are_rejected_before_loading() {
        let (root, config) = configured_project();
        fs::write(
            root.join("target/deploy/migration.so"),
            vec![0; package::MAX_PROGRAM_BYTES as usize + 1],
        )
        .unwrap();
        assert!(candidate_bytes(&root, &config).is_err());
        fs::write(root.join("eplyx.toml"), vec![b'x'; 32 * 1024 + 1]).unwrap();
        assert!(read_config(&root.join("eplyx.toml")).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn doctor_rejects_wrong_genesis() {
        assert!(check_genesis(&GenesisRpc("wrong-cluster")).is_err());
        assert!(check_genesis(&GenesisRpc(MAINNET_GENESIS)).is_ok());
    }

    #[test]
    fn run_reuse_rejects_changed_candidate_before_replay() {
        let (root, config) = configured_project();
        let base = store(&root, &config.project.name).unwrap();
        let id = "run_exactbinding";
        let directory = base.join("runs").join(id);
        fs::create_dir(&directory).unwrap();
        let package = package_in(&root, &config, &directory.join("package")).unwrap();
        fs::create_dir(directory.join("result")).unwrap();
        let meta = metadata(&root, id, &package, Policy::BlockOnly, "Warn").unwrap();
        write_new(
            &directory.join("metadata.json"),
            &serde_json::to_vec(&meta).unwrap(),
        )
        .unwrap();
        let program = root.join("target/deploy/migration.so");
        let mut bytes = fs::read(&program).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        fs::write(program, bytes).unwrap();
        let error = verify_run(&root, &config, &base, id)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("incompatible"), "{error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn local_counterexample_id_is_stable_and_tamper_detected() {
        let root = temp_project();
        let base = store(&root, "fixture").unwrap();
        let counterexample = search::Counterexample::Observed {
            id: "observed-case".into(),
            parent_run: "run_parent".into(),
            transition_package_sha256: "a".repeat(64),
            candidate_program_sha256: "b".repeat(64),
            observed_source_account: "account".into(),
            observed_state_digest: "c".repeat(64),
            observed_amount_raw: "10".into(),
            execution_plan_sha256: "d".repeat(64),
            execution_fixture_sha256: "e".repeat(64),
            failure_signature: search::FailureSignature {
                stage: "instruction".into(),
                program: "candidate".into(),
                instruction_error: "Custom(13)".into(),
                relevant_log: None,
                rollback_verified: true,
            },
            provenance: "Observed".into(),
            limitations: "Exact only".into(),
        };
        let id = counterexample_id(&counterexample).unwrap();
        assert_eq!(id, counterexample_id(&counterexample).unwrap());
        let saved = SavedCounterexample {
            schema_version: 1,
            id: id.clone(),
            parent_run: "run_parent".into(),
            search_sha256: "f".repeat(64),
            replay_inputs: Some(replay_inputs("run_parent")),
            counterexample,
        };
        let path = base.join("counterexamples").join(format!("{id}.json"));
        write_new(&path, &serde_json::to_vec(&saved).unwrap()).unwrap();
        assert_eq!(counterexample_count(&base, "run_parent").unwrap(), 1);
        let mut tampered: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        tampered["counterexample"]["observed_amount_raw"] = json!("11");
        fs::write(&path, serde_json::to_vec(&tampered).unwrap()).unwrap();
        assert!(reproduce(&base, &id).is_err());
        let records: Vec<_> = fs::read_dir(base.join("reproductions"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(records.len(), 1, "failed attempts are recorded too");
        let record: Reproduction = serde_json::from_slice(&fs::read(&records[0]).unwrap()).unwrap();
        assert_eq!(record.outcome, ReproductionOutcome::Failed);
        assert!(!record.failure_signature_matched);
        assert!(record.no_rpc);
        assert_eq!(record.parent_run.as_deref(), Some("run_parent"));
        assert!(record.error.unwrap().contains("identity mismatch"));
        assert!(eplyx_lifecycle_impact::local_store::is_safe_id(
            &record.id, "repro_"
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_contains_git_context_without_rpc_secret() {
        let (root, config) = configured_project();
        let package = package_in(&root, &config, &root.join("package")).unwrap();
        let meta = metadata(
            &eplyx_lifecycle_impact::repo_root(),
            "run_test",
            &package,
            Policy::BlockOnly,
            "Warn",
        )
        .unwrap();
        assert_eq!(
            meta.git_commit,
            git(&eplyx_lifecycle_impact::repo_root(), &["rev-parse", "HEAD"])
        );
        assert!(meta.git_branch.is_some());
        let bytes = serde_json::to_string(&meta).unwrap();
        assert!(!bytes.contains("SOLANA_RPC_URL"));
        assert_eq!(meta.schema_version, METADATA_VERSION);
        assert!(meta.run_source.is_some());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn local_runs_are_sorted_by_id() {
        let (root, config) = configured_project();
        let base = store(&root, &config.project.name).unwrap();
        let package = package_in(&root, &config, &root.join("package")).unwrap();
        for id in ["run_z", "run_a"] {
            let directory = base.join("runs").join(id);
            fs::create_dir(&directory).unwrap();
            let meta = metadata(&root, id, &package, Policy::BlockOnly, "Warn").unwrap();
            write_new(
                &directory.join("metadata.json"),
                &serde_json::to_vec(&meta).unwrap(),
            )
            .unwrap();
        }
        let rows = local_runs(&base).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["run"], "run_a");
        assert_eq!(rows[1]["run"], "run_z");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn saved_runs_are_listable_without_current_config() {
        let root = temp_project();
        let path = root.join("eplyx.toml");
        init(&root, &path, false, true).unwrap();
        fs::remove_file(&path).unwrap();
        let code = execute(Cli {
            config: path,
            command: Action::Runs { json: true },
        })
        .unwrap();
        assert_eq!(code, 0);
        fs::remove_dir_all(root).unwrap();
    }
}
