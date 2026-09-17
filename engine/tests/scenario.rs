//! Phase 1: change dispatch preserves upgrade execution and rejects lifecycle
//! descriptions until a consequence model exists.

use eplyx_lifecycle_impact::{corpus, diff, executor, ChangeScenario, LifecycleChange};

#[test]
fn program_upgrade_matches_direct_execution_and_the_legacy_api() {
    let program_id = eplyx_lifecycle_impact::fixture_program_id();
    let fixture = corpus::generate(&program_id)
        .into_iter()
        .find(|fixture| fixture.id == "boundary-position-017")
        .unwrap();
    let (baseline, candidate) = eplyx_lifecycle_impact::load_versions(
        &eplyx_lifecycle_impact::default_artifact("v1"),
        &eplyx_lifecycle_impact::default_artifact("v2"),
    )
    .expect("run ./scripts/build-programs.sh before execution tests");

    // Independent execution establishes the expected inputs and observable
    // result, rather than comparing two wrappers around the same dispatch.
    let expected = diff::compare(
        &fixture,
        executor::execute(&fixture, &program_id, &baseline).unwrap(),
        executor::execute(&fixture, &program_id, &candidate).unwrap(),
    );
    assert!(expected.is_critical());

    let scenario = ChangeScenario::program_upgrade(&baseline, &candidate);
    let actual = scenario.compare_fixture(&fixture, &program_id).unwrap();
    let legacy =
        eplyx_lifecycle_impact::compare_fixture(&fixture, &program_id, &baseline, &candidate)
            .unwrap();
    assert_eq!(actual, expected);
    assert_eq!(legacy, expected);
}

#[test]
fn lifecycle_change_refuses_execution() {
    let scenario = ChangeScenario::LifecycleChange(LifecycleChange {
        description: "An asset changes lifecycle state".into(),
    });
    let program_id = eplyx_lifecycle_impact::fixture_program_id();
    let fixture = corpus::generate(&program_id).remove(0);

    // No program artefacts are loaded. Refuse the change before execution or
    // report construction.
    let fixture_error = scenario
        .compare_fixture(&fixture, &program_id)
        .unwrap_err()
        .to_string();
    assert!(fixture_error.contains("LifecycleChange is not supported"));
}
