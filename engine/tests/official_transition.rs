//! Public frozen-evidence replay and fail-closed investigation boundaries.
use eplyx_lifecycle_impact::{
    expansion,
    lifecycle::{decode, policy::LifecycleScenario, LifecycleSnapshot},
    probe::ExitPathType,
    repo_root,
    resolution::{LifecycleResolution, PathStatus},
    transition::research::{self, OfficialTransitionReport, ResearchManifest},
};
use std::{path::PathBuf, process::Command, sync::OnceLock};
const ENTITY: &str = "741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs";
fn inputs() -> &'static (LifecycleSnapshot, LifecycleScenario) {
    static C: OnceLock<(LifecycleSnapshot, LifecycleScenario)> = OnceLock::new();
    C.get_or_init(|| {
        let r = repo_root();
        (
            LifecycleSnapshot::load(&r.join("snapshots/spacex-exposure.json")).unwrap(),
            LifecycleScenario::load(&r.join("scenarios/spacex-transition.json")).unwrap(),
        )
    })
}
fn tmp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("eplyx-phase9-{name}-{}", std::process::id()));
    std::fs::create_dir(&p).unwrap();
    p
}
#[test]
fn successor_programs_and_real_samples_do_not_establish_official_conversion() {
    let r = repo_root();
    let report: OfficialTransitionReport =
        expansion::load(&r.join("reports/spacex-official-transition.json")).unwrap();
    assert_eq!(report.assessment.status, PathStatus::NotTested);
    assert_eq!(report.assessment.transition_exists_established, None);
    assert!(!report.assessment.execution_attempted);
    assert_eq!(report.exact_tested_input_raw, None);
    assert!(report.selected_mechanism.required_signers.is_empty());
    assert_eq!(report.entity_id, format!("solana-token-account:{ENTITY}"));
    assert_eq!(report.successor_verification.configuration.decimals, 8);
    assert!(report.successor_verification.configuration.is_token_2022);
    let reference = &report.successor_verification.evidence;
    let bytes = reference.artifact.read(&r.join("probes")).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let account = raw.pointer(&reference.pointer).unwrap();
    assert_eq!(
        decode::decode_mint(account).unwrap(),
        report.successor_verification.configuration
    );
    assert_eq!(
        report.successor_verification.creation_slot_established,
        None
    );
    assert!(report.shared_base_authorities.is_empty());
    assert!(report
        .transactions
        .iter()
        .any(|t| t.touches_source && t.touches_successor));
    assert!(report
        .transactions
        .iter()
        .any(|t| t.instruction_types.iter().any(|s| s == "burn")));
    assert!(report
        .transactions
        .iter()
        .any(|t| t.instruction_types.iter().any(|s| s == "mintTo")));
    assert!(report.transactions.iter().any(|t| t
        .instruction_types
        .iter()
        .any(|s| s == "withdrawWithheldTokensFromMint")));
    assert!(report
        .programs
        .iter()
        .filter(|p| p.programdata_address.is_some())
        .all(|p| p.loader_link_verified));
    assert!(report
        .programs
        .iter()
        .all(|p| !p.official_transition_identity_established));
    assert!(report
        .candidate_mechanisms
        .iter()
        .all(|m| !eplyx_lifecycle_impact::transition::official_identity_established(m)));
    let prior: LifecycleResolution =
        expansion::load(&r.join("reports/spacex-lifecycle-path-resolution.json")).unwrap();
    assert_eq!(
        report
            .updated_resolution
            .paths
            .iter()
            .filter(|p| p.path_type != ExitPathType::OfficialTransition)
            .collect::<Vec<_>>(),
        prior
            .paths
            .iter()
            .filter(|p| p.path_type != ExitPathType::OfficialTransition)
            .collect::<Vec<_>>()
    );
}
#[test]
fn offline_cli_reproduces_report_and_matrix_with_protected_portable_outputs() {
    let r = repo_root();
    let dir = tmp("cli");
    let out = dir.join("report.json");
    let matrix = dir.join("matrix.json");
    let discovery = dir.join("discovery.json");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_eplyx-lifecycle"));
    cmd.current_dir(&dir)
        .arg("investigate-transition")
        .arg("--snapshot")
        .arg(r.join("snapshots/spacex-exposure.json"))
        .arg("--scenario")
        .arg(r.join("scenarios/spacex-transition.json"))
        .arg("--entity")
        .arg(ENTITY)
        .arg("--research")
        .arg(r.join("probes/spacex-official-transition-research.json"))
        .args(["--format", "json", "--out"])
        .arg(&out)
        .arg("--out-resolution")
        .arg(&matrix)
        .arg("--out-discovery")
        .arg(&discovery);
    for key in [
        "SOLANA_RPC_URL",
        "EPLYX_RPC_URL",
        "EPLYX_ARCHIVE_RPC_URL",
        "SOLANA_ARCHIVE_RPC_URL",
    ] {
        cmd.env_remove(key);
    }
    let result = cmd.output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, std::fs::read(&out).unwrap());
    assert_eq!(
        result.stdout,
        std::fs::read(r.join("reports/spacex-official-transition.json")).unwrap()
    );
    assert_eq!(
        std::fs::read(&matrix).unwrap(),
        std::fs::read(r.join("reports/spacex-lifecycle-path-resolution-phase9.json")).unwrap()
    );
    assert_eq!(
        std::fs::read(&discovery).unwrap(),
        std::fs::read(r.join("probes/spacex-lifecycle-path-discovery-phase9.json")).unwrap()
    );
    let report: OfficialTransitionReport = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report.to_json().unwrap().as_bytes(), result.stdout);
    let retry = cmd.output().unwrap();
    assert!(!retry.status.success());
    assert!(String::from_utf8_lossy(&retry.stderr).contains("already exists"));
    assert_eq!(result.stdout, std::fs::read(&out).unwrap());
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn changed_research_artifacts_and_other_entity_are_rejected_before_replay() {
    let r = repo_root();
    let (snapshot, scenario) = inputs();
    let path = r.join("probes/spacex-official-transition-research.json");
    assert!(
        research::investigate(&path, snapshot, scenario, "another-holder")
            .unwrap_err()
            .to_string()
            .contains("inherit")
    );
    let dir = tmp("tamper");
    let mut plan: ResearchManifest = expansion::load(&path).unwrap();
    for reference in [
        &mut plan.prior_resolution,
        &mut plan.prior_discovery,
        &mut plan.evidence_bundle,
        &mut plan.coverage,
    ]
    .into_iter()
    .chain(plan.archives.iter_mut())
    .chain(plan.supplementary_artifacts.iter_mut())
    .chain(plan.external_sources.iter_mut().map(|s| &mut s.artifact))
    {
        reference.file = r
            .join("probes")
            .join(&reference.file)
            .to_string_lossy()
            .into_owned();
    }
    plan.supplementary_artifacts[0].sha256 = "0".repeat(64);
    let changed = dir.join("plan.json");
    expansion::save(&plan, &changed).unwrap();
    assert!(research::investigate(&changed, snapshot, scenario, ENTITY)
        .unwrap_err()
        .to_string()
        .contains("digest mismatch"));
    let mut invalid = serde_json::to_value(plan).unwrap();
    invalid["execution"] = serde_json::json!({"success":true});
    assert!(serde_json::from_value::<ResearchManifest>(invalid).is_err());
    std::fs::remove_dir_all(dir).unwrap();
}
