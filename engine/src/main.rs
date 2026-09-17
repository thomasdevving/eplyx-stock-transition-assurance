//! Command-line regression harness for Eplyx Lifecycle Impact.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use eplyx_lifecycle_impact::{
    compare_all, corpus, default_artifact, fixture_program_id, load_versions, report::render_text,
    report::Report, types::Category,
};

#[derive(Parser)]
#[command(
    name = "eplyx-lifecycle",
    about = "Eplyx Lifecycle Impact — Phase 1 regression harness",
    long_about = "Evaluates program upgrades against identical fixture state and transactions. \
                  LifecycleChange is a placeholder; lifecycle execution is not yet implemented."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compare the synthetic corpus against the current and candidate builds.
    Compare(CompareArgs),
    /// Generate the deterministic test state locally.
    Generate(GenerateArgs),
    /// Reproduce a fixture or economic regression group.
    Reproduce(ReproduceArgs),
    /// List the synthetic test corpus.
    List(ListArgs),
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
        Command::Compare(args) => compare(args),
        Command::Generate(args) => generate(args),
        Command::Reproduce(args) => reproduce(args),
        Command::List(args) => list(args),
    }
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
        println!("{}", eplyx_lifecycle_impact::report::render_reproduction(&diff));
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

