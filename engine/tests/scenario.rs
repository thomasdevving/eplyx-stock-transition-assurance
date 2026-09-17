//! Phase 1: change dispatch preserves upgrade execution and rejects lifecycle
//! descriptions until a consequence model exists.

use eplyx_engine::{corpus, diff, executor, ChangeScenario, LifecycleChange};

#[test]
fn program_upgrade_matches_direct_execution_and_the_legacy_api() {
    let program_id = eplyx_engine::fixture_program_id();
    let fixture = corpus::generate(&program_id)
        .into_iter()
        .find(|fixture| fixture.id == "boundary-position-017")
        .unwrap();
    let (baseline, candidate) = eplyx_engine::load_versions(
        &eplyx_engine::default_artifact("v1"),
        &eplyx_engine::default_artifact("v2"),
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
        eplyx_engine::compare_fixture(&fixture, &program_id, &baseline, &candidate).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(legacy, expected);
}

#[test]
fn lifecycle_change_refuses_fixture_and_replay_execution() {
    let scenario = ChangeScenario::LifecycleChange(LifecycleChange {
        description: "An asset changes lifecycle state".into(),
    });
    let program_id = eplyx_engine::fixture_program_id();
    let fixture = corpus::generate(&program_id).remove(0);

    // No program artefacts are loaded. Both entry points must refuse the change
    // before they attempt execution or publish a report.
    let fixture_error = scenario
        .compare_fixture(&fixture, &program_id)
        .unwrap_err()
        .to_string();
    let replay_error = scenario
        .compare_replay(&[], &eplyx_engine::replay::DependencyBundle::empty())
        .unwrap_err()
        .to_string();
    assert!(fixture_error.contains("LifecycleChange is not supported"));
    assert!(replay_error.contains("LifecycleChange is not supported"));
}
