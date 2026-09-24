//! Release identity of this binary. The semantic version is the only
//! compatibility identity; commit and target describe the build, and the
//! schema versions name the artifact formats this engine reads and writes.
use crate::{
    conversion::{package, search},
    dashboard::store::INDEX_VERSION,
    local_store::{METADATA_VERSION, REPRODUCTION_VERSION},
};
use serde_json::{json, Value};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const COMMIT: &str = env!("EPLYX_BUILD_COMMIT");
pub const TARGET: &str = env!("EPLYX_BUILD_TARGET");

pub fn short_commit() -> &'static str {
    COMMIT.get(..12).unwrap_or(COMMIT)
}

/// Platform name used in release artifact file names.
pub fn platform() -> String {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    };
    format!("{os}-{arch}")
}

pub fn long_version() -> String {
    format!(
        "{VERSION}\ncommit {}\ntarget {TARGET}\nengine eplyx-lifecycle-impact {VERSION} · package schema {}/{} · {} · run metadata {METADATA_VERSION}",
        short_commit(),
        package::VERSION,
        package::INVARIANT_PACKAGE_VERSION,
        search::VERSION,
    )
}

/// Machine-readable identity. Contains no paths, environment or secrets.
pub fn json() -> Value {
    json!({
        "schema_version": 1,
        "name": "eplyx",
        "version": VERSION,
        "commit": COMMIT,
        "target": TARGET,
        "platform": platform(),
        "os": std::env::consts::OS,
        "architecture": std::env::consts::ARCH,
        "engine": {
            "crate": "eplyx-lifecycle-impact",
            "version": VERSION,
            "transition_package_schemas": [package::VERSION, package::INVARIANT_PACKAGE_VERSION],
            "invariant_schema": package::INVARIANT_SCHEMA_VERSION,
            "counterexample_search": search::VERSION,
            "run_metadata_schema": METADATA_VERSION,
            "reproduction_schema": REPRODUCTION_VERSION,
            "dashboard_index": INDEX_VERSION,
        },
    })
}
