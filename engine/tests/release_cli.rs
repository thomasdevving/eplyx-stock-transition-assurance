//! Milestone 17: the installed CLI surface — version identity, install-aware
//! doctor output and compatibility with existing `.eplyx/` stores.
use eplyx_lifecycle_impact::repo_root;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn temp(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "eplyx-release-{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

/// Run the CLI with no RPC and proxies pointing at a closed port, so any
/// accidental network use fails loudly instead of silently succeeding.
fn eplyx(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_eplyx"))
        .args(args)
        .current_dir(dir)
        .env_remove("SOLANA_RPC_URL")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .output()
        .unwrap()
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn version_needs_no_project_config_store_or_network() {
    let dir = temp("version");
    let long = eplyx(&dir, &["--version"]);
    assert!(long.status.success());
    let long = text(&long);
    let lines: Vec<&str> = long.lines().collect();
    assert_eq!(lines[0], format!("eplyx {}", env!("CARGO_PKG_VERSION")));
    assert!(lines[1].starts_with("commit "), "{long}");
    assert!(lines[2].starts_with("target "), "{long}");
    assert!(
        lines[3].contains("eplyx-counterexample-search/v1"),
        "{long}"
    );
    let short = text(&eplyx(&dir, &["-V"]));
    assert_eq!(short.trim(), format!("eplyx {}", env!("CARGO_PKG_VERSION")));
    let json = eplyx(&dir, &["version", "--json"]);
    assert!(json.status.success());
    let body = String::from_utf8_lossy(&json.stdout).into_owned();
    let value: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(value["engine"]["run_metadata_schema"], 2);
    assert!(value["platform"].as_str().unwrap().contains('-'));
    for variable in ["HOME", "USERPROFILE"] {
        if let Ok(home) = std::env::var(variable) {
            assert!(home.is_empty() || !body.contains(&home), "no local paths");
        }
    }
    assert!(!body.contains(dir.to_string_lossy().as_ref()));
    assert_eq!(
        fs::read_dir(&dir).unwrap().count(),
        0,
        "version writes nothing"
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn doctor_separates_eplyx_requirements_from_the_developer_project() {
    let dir = temp("doctor");
    let missing = eplyx(&dir, &["doctor", "--offline"]);
    assert_eq!(missing.status.code(), Some(2));
    let missing = text(&missing);
    assert!(missing.contains("✓ Installed binary"), "{missing}");
    assert!(missing.contains("run `eplyx init`"), "{missing}");
    assert!(missing.contains("needs no Rust"), "{missing}");
    assert!(!missing.to_lowercase().contains("rust is required"));

    assert!(eplyx(&dir, &["init"]).status.success());
    let blank = text(&eplyx(&dir, &["doctor", "--offline"]));
    assert!(blank.contains("✗ Candidate program"), "{blank}");
    assert!(blank.contains("cargo build-sbf"), "{blank}");

    // A real checked-in candidate with blank addresses fails package validation only.
    let package = repo_root().join("examples/transitions/demo-fixed-ratio");
    fs::create_dir_all(dir.join("target/deploy")).unwrap();
    fs::copy(
        package.join("program.so"),
        dir.join("target/deploy/migration.so"),
    )
    .unwrap();
    let invalid = eplyx(&dir, &["doctor", "--offline"]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(text(&invalid).contains("✗ Transition package"));

    let manifest: Value =
        serde_json::from_slice(&fs::read(package.join("eplyx.json")).unwrap()).unwrap();
    let config: Value =
        serde_json::from_slice(&fs::read(package.join("config.json")).unwrap()).unwrap();
    let body = fs::read_to_string(dir.join("eplyx.toml")).unwrap();
    let body = body
        .replacen(
            "source_mint = \"\"",
            &format!(
                "source_mint = {:?}",
                manifest["sourceMint"].as_str().unwrap()
            ),
            1,
        )
        .replacen(
            "replacement_mint = \"\"",
            &format!(
                "replacement_mint = {:?}",
                manifest["replacementMint"].as_str().unwrap()
            ),
            1,
        )
        .replacen(
            "public_owner = \"\"",
            &format!(
                "public_owner = {:?}",
                config["publicOwner"].as_str().unwrap()
            ),
            1,
        )
        .replacen(
            "source_account = \"\"",
            &format!(
                "source_account = {:?}",
                config["sourceAccount"].as_str().unwrap()
            ),
            1,
        );
    fs::write(dir.join("eplyx.toml"), body).unwrap();
    let valid = eplyx(&dir, &["doctor", "--offline"]);
    let valid_text = text(&valid);
    assert_eq!(valid.status.code(), Some(0), "{valid_text}");
    assert!(valid_text.contains("✓ Transition package"));
    assert!(valid_text.contains("skipped (--offline)"));

    // Without --offline and without SOLANA_RPC_URL, doctor says what to set.
    let no_rpc = text(&eplyx(&dir, &["doctor"]));
    assert!(no_rpc.contains("set SOLANA_RPC_URL"), "{no_rpc}");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn doctor_never_echoes_the_rpc_url() {
    let dir = temp("doctor-rpc");
    let output = Command::new(env!("CARGO_BIN_EXE_eplyx"))
        .args(["doctor"])
        .current_dir(&dir)
        .env(
            "SOLANA_RPC_URL",
            "https://127.0.0.1:9/DOCTOR-SECRET-KEY?api-key=QUERY-SECRET",
        )
        .output()
        .unwrap();
    let output = text(&output);
    assert!(output.contains("✗ RPC"), "{output}");
    assert!(
        !output.contains("DOCTOR-SECRET-KEY") && !output.contains("QUERY-SECRET"),
        "{output}"
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn existing_milestone_16_store_opens_without_config() {
    let dir = temp("store");
    fn copy(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                copy(&entry.path(), &to.join(entry.file_name()));
            } else {
                fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
            }
        }
    }
    copy(
        &repo_root().join("fixtures/dashboard/transition-acceptance"),
        &dir,
    );
    for child in ["runs", "counterexamples", "cache"] {
        fs::create_dir_all(dir.join(".eplyx").join(child)).unwrap();
    }
    let runs = eplyx(&dir, &["runs", "--json"]);
    assert!(runs.status.success(), "{}", text(&runs));
    let rows: Value = serde_json::from_slice(&runs.stdout).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 4);
    assert_eq!(rows[3]["counterexamples"], 37);
    let show = text(&eplyx(
        &dir,
        &["show", "run_20260924123736483_ce7b4d55310c"],
    ));
    assert!(show.contains("Gate Block"), "{show}");
    fs::remove_dir_all(dir).unwrap();
}
