//! Production planner and artifact bindings; public update requires offline replay.
use eplyx_lifecycle_impact::{
    coverage::CoverageReport,
    expansion::{
        self as ex,
        discovery::VenueInventory,
        pipeline::{self, CaptureManifest, ExecutionEvidence, ExecutionIndex},
        Eligibility, ExpansionPlan,
    },
    lifecycle::LifecycleSnapshot,
    probe::ExitPathType,
    repo_root,
};
use std::sync::OnceLock;
fn world() -> &'static (
    LifecycleSnapshot,
    CoverageReport,
    VenueInventory,
    ExpansionPlan,
) {
    static W: OnceLock<(
        LifecycleSnapshot,
        CoverageReport,
        VenueInventory,
        ExpansionPlan,
    )> = OnceLock::new();
    W.get_or_init(|| {
        let r = repo_root();
        (
            LifecycleSnapshot::load(&r.join("snapshots/spacex-exposure.json")).unwrap(),
            ex::load(&r.join("reports/spacex-lifecycle-coverage.json")).unwrap(),
            ex::load(&r.join("probes/spacex-phase7-venues-verified.json")).unwrap(),
            ex::load(&r.join("probes/spacex-lifecycle-expansion-plan.json")).unwrap(),
        )
    })
}
#[test]
fn production_selector_is_deterministic_under_population_input_reordering() {
    let (s, b, i, p) = world();
    let first = ex::expand(s, b, i, &p.config).unwrap();
    assert_eq!(*p, first);
    drop(first);
    let mut s = s.clone();
    let mut b = b.clone();
    s.entities.reverse();
    b.entities.reverse();
    let reordered = ex::expand(&s, &b, i, &p.config).unwrap();
    assert_eq!(
        ex::canonical(p).unwrap(),
        ex::canonical(&reordered).unwrap()
    );
}
#[test]
fn bounded_plan_selects_three_distinct_balance_shapes_and_excludes_unsupported() {
    let (_, _, _, p) = world();
    assert_eq!(p.selected.len(), 6);
    let ids = p
        .selected
        .iter()
        .map(|g| &g.candidate.entity_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 3);
    assert_eq!(
        p.selected
            .iter()
            .take(3)
            .map(|g| g.candidate.balance_bucket)
            .collect::<Vec<_>>(),
        vec![0, 1, 3]
    );
    assert_eq!(
        p.selected
            .iter()
            .take(3)
            .map(|g| &g.candidate.state_shape_sha256)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
    assert!(p.selected.iter().all(|g| matches!(
        g.candidate.eligibility,
        Eligibility::CaptureRequired | Eligibility::ExecutableCandidate
    )));
    assert!(p
        .candidates
        .iter()
        .filter(|c| matches!(
            c.eligibility,
            Eligibility::Invalid | Eligibility::Unsupported
        ))
        .all(|c| c.initial_score.is_none()));
    assert!(p
        .gaps
        .iter()
        .any(|g| !g.execution_supported && g.represented_raw.parse::<u128>().unwrap() > 0));
    assert!(p.candidates.iter().all(|c| c.represented_raw != "0"));
}
#[test]
fn each_expected_entity_amount_is_counted_once_across_independent_paths() {
    let (_, _, _, p) = world();
    let mut per_entity = std::collections::BTreeMap::new();
    for g in &p.selected {
        per_entity
            .entry(&g.candidate.entity_id)
            .or_insert(g.candidate.represented_raw.parse::<u128>().unwrap());
    }
    assert_eq!(
        p.selected
            .iter()
            .map(|g| g.expected_gain.represented_raw.parse::<u128>().unwrap())
            .sum::<u128>(),
        per_entity.values().sum::<u128>()
    );
    assert_eq!(
        p.selected
            .iter()
            .map(|g| g.expected_gain.entities)
            .sum::<usize>(),
        3
    );
}
#[test]
fn hashes_and_selected_plan_tampering_fail_closed() {
    let (s, b, i, p) = world();
    let mut bad = p.clone();
    bad.selected[0].amount_matrix[0].raw = "99999999999999999".into();
    assert!(bad.validate(s, b, i).is_err());
    bad = p.clone();
    bad.coverage_sha256 = "wrong".into();
    assert!(bad.validate(s, b, i).is_err());
}
#[test]
fn real_multi_entity_transfer_and_second_venue_results_have_exact_content_identities() {
    let r = repo_root();
    let index: ExecutionIndex =
        ex::load(&r.join("reports/phase7-evidence/execution-index.json")).unwrap();
    let manifest: CaptureManifest =
        ex::load(&r.join("probes/phase7-captures/capture-manifest.json")).unwrap();
    assert_eq!(
        index.capture_manifest_sha256,
        ex::digest(&manifest).unwrap()
    );
    let mut executed = std::collections::BTreeSet::new();
    let mut paths = std::collections::BTreeSet::new();
    for reference in index.results {
        let bytes = std::fs::read(
            r.join("reports/phase7-evidence")
                .join(reference.result_file),
        )
        .unwrap();
        assert_eq!(
            eplyx_lifecycle_impact::lifecycle::exposure::sha256(&bytes),
            reference.result_sha256
        );
        let e: ExecutionEvidence = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(ex::digest(&e).unwrap(), reference.result_sha256);
        if e.execution.is_some() {
            executed.insert(e.entity_id.clone());
            paths.insert(e.path_type as u8);
        }
        if e.status == eplyx_lifecycle_impact::coverage::CaseStatus::Succeeded {
            assert!(e.deltas.as_ref().unwrap().reconciled);
            assert!(e.execution.as_ref().unwrap().success);
            assert!(!e.authority.signer_possession_known);
            assert!(e.authority.signer_assumed_locally);
            assert_eq!(
                e.authority.wording,
                "original owner locally assumed to sign"
            );
        }
        if e.status == eplyx_lifecycle_impact::coverage::CaseStatus::Failed {
            assert_eq!(e.rollback_verified, Some(true));
        }
    }
    assert!(executed.len() >= 3);
    assert!(paths.contains(&(ExitPathType::Transfer as u8)));
    assert!(paths.contains(&(ExitPathType::SecondaryMarketExit as u8)));
}

#[test]
fn public_update_rejects_changed_fixture_and_rehashed_fabricated_result() {
    let (s, b, i, p) = world();
    let root = repo_root();
    let capture_dir = root.join("probes/phase7-captures");
    let result_dir = root.join("reports/phase7-evidence");
    let manifest: CaptureManifest = ex::load(&capture_dir.join("capture-manifest.json")).unwrap();
    let mut index: ExecutionIndex = ex::load(&result_dir.join("execution-index.json")).unwrap();
    let tmp = std::env::temp_dir().join(format!(
        "eplyx-phase7-public-update-tamper-{}",
        std::process::id()
    ));
    std::fs::create_dir(&tmp).unwrap();
    let changed_fixture = tmp.join(&manifest.bindings[0].fixture_file);
    std::fs::create_dir_all(changed_fixture.parent().unwrap()).unwrap();
    let mut bytes = std::fs::read(capture_dir.join(&manifest.bindings[0].fixture_file)).unwrap();
    bytes.push(b' ');
    std::fs::write(changed_fixture, bytes).unwrap();
    let err = pipeline::update(s, b, p, i, &manifest, (&tmp, &result_dir), &index).unwrap_err();
    assert!(err.to_string().contains("fixture digest mismatch"), "{err}");

    let reference = &mut index.results[0];
    let mut evidence: ExecutionEvidence =
        ex::load(&result_dir.join(&reference.result_file)).unwrap();
    evidence.deltas.as_mut().unwrap().input_debited_raw = "999999999".into();
    reference.result_sha256 = ex::digest(&evidence).unwrap();
    reference.result_file = format!("results/{}.json", reference.result_sha256);
    ex::save(&evidence, &tmp.join(&reference.result_file)).unwrap();
    // Valid new digest/bindings are insufficient: actual fresh VM data must agree.
    let err = pipeline::update(s, b, p, i, &manifest, (&capture_dir, &tmp), &index).unwrap_err();
    assert!(
        err.to_string()
            .contains("result disagrees with fresh offline execution"),
        "{err}"
    );
    std::fs::remove_dir_all(tmp).unwrap();
}
