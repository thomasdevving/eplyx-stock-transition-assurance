//! The Milestone 18 sync contract: what one run, counterexample or
//! reproduction record looks like on its way to a cloud workspace, and the
//! checks both the CLI (before upload) and the server (on receipt) apply.
//!
//! Documents carry exact artifact bytes plus their SHA-256, so the cloud binds
//! to the same digests the local replay uses. Nothing here grants evidence:
//! a synced result is a copy of what the local engine concluded.
use super::privacy;
use crate::{
    conversion::package,
    dashboard::{
        store::{Store, CAPTURE_LIMIT},
        view::{self, RunBytes},
    },
    expansion::canonical,
    lifecycle::exposure::sha256,
    local_store::{
        counterexample_id, is_safe_id, replay_inputs, Metadata, Reproduction, SavedCounterexample,
    },
};
use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const RUN_SCHEMA: &str = "eplyx.cloud.run.v1";
pub const COUNTEREXAMPLE_SCHEMA: &str = "eplyx.cloud.counterexample.v1";
pub const REPRODUCTION_SCHEMA: &str = "eplyx.cloud.reproduction.v1";

/// Request body bounds. JSON string escaping can grow artifact text, so these
/// sit above the sum of the member bounds below.
pub const MAX_RUN_BODY: usize = 12 * 1024 * 1024;
pub const MAX_COUNTEREXAMPLE_BODY: usize = 3 * 1024 * 1024;
pub const MAX_REPRODUCTION_BODY: usize = 64 * 1024;

const METADATA_LIMIT: usize = 64 * 1024;
/// The package preflight worker's own bound for a report handoff.
const REPORT_LIMIT: usize = 4 * 1024 * 1024;
/// The engine reads bindings with a 32 KiB bound.
const BINDINGS_LIMIT: usize = 32 * 1024;
const SEARCH_LIMIT: usize = 4 * 1024 * 1024;
const COUNTEREXAMPLE_LIMIT: usize = 1024 * 1024;
const REPRODUCTION_LIMIT: usize = 16 * 1024;

/// Plain-language statement of the default sync scope, shown by the CLI.
pub const WHAT_IS_SYNCED: &str = "Eplyx sync uploads, for each complete run: metadata.json, result/report.json, result/bindings.json, package/eplyx.json, package/config.json and search/counterexamples.json when present, each as exact bytes with its SHA-256, plus the sizes of artifacts that stay local. It also uploads saved counterexample files and reproduction records. Public mint and token-account addresses in those files are included. It never uploads source code, the candidate .so, population/stress/wave/wallet captures, report.md, eplyx.toml, environment variables, RPC URLs, local absolute paths or your Eplyx token. Execution stays local; the cloud only displays synced results.";

/// Exact UTF-8 bytes of one small artifact and their SHA-256.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub sha256: String,
    pub text: String,
}

impl Artifact {
    pub fn new(bytes: Vec<u8>) -> Result<Self> {
        let text = String::from_utf8(bytes).context("artifact is not UTF-8")?;
        Ok(Self {
            sha256: sha256(text.as_bytes()),
            text,
        })
    }

    fn verify(&self, what: &str, limit: usize) -> Result<()> {
        ensure!(self.text.len() <= limit, "{what} exceeds its sync bound");
        ensure!(
            sha256(self.text.as_bytes()) == self.sha256,
            "{what} SHA-256 does not match its bytes"
        );
        privacy::scan(what, &self.text)
    }
}

pub fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Cloud workspace/project IDs: a fixed prefix and 20 lowercase hex digits.
pub fn is_cloud_id(id: &str, prefix: &str) -> bool {
    id.strip_prefix(prefix).is_some_and(|rest| {
        rest.len() == 20
            && rest
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunDocument {
    pub schema: String,
    pub local_project_id: String,
    pub run_id: String,
    pub metadata: Artifact,
    pub report: Artifact,
    pub bindings: Artifact,
    pub manifest: Artifact,
    pub config: Artifact,
    /// Present once `eplyx search` has run for this run. It may be attached to
    /// a synced run exactly once and never changed.
    #[serde(default)]
    pub search: Option<Artifact>,
    /// Sizes of allowlisted members that stay on the machine, for display
    /// only. Never part of the run identity.
    #[serde(default)]
    pub local_artifact_sizes: BTreeMap<String, Option<u64>>,
}

/// A run document that passed every check, with the engine's own summary.
pub struct VerifiedRun {
    pub core_sha256: String,
    pub search_sha256: Option<String>,
    pub metadata: Metadata,
    pub run: view::Run,
    pub summary: Value,
}

impl RunDocument {
    /// Identity of the immutable run: its IDs and the digests of every member
    /// except the append-only search attachment and the display-only sizes.
    pub fn core_sha256(&self) -> Result<String> {
        Ok(sha256(
            canonical(&json!({
                "schema": self.schema,
                "local_project_id": self.local_project_id,
                "run_id": self.run_id,
                "metadata_sha256": self.metadata.sha256,
                "report_sha256": self.report.sha256,
                "bindings_sha256": self.bindings.sha256,
                "manifest_sha256": self.manifest.sha256,
                "config_sha256": self.config.sha256,
            }))?
            .as_bytes(),
        ))
    }

    pub fn verify(&self) -> Result<VerifiedRun> {
        ensure!(self.schema == RUN_SCHEMA, "unsupported run document schema");
        ensure!(is_safe_id(&self.run_id, "run_"), "invalid run ID");
        ensure!(
            is_safe_id(&self.local_project_id, "project_"),
            "invalid local project ID"
        );
        self.metadata.verify("metadata.json", METADATA_LIMIT)?;
        self.report.verify("result/report.json", REPORT_LIMIT)?;
        self.bindings
            .verify("result/bindings.json", BINDINGS_LIMIT)?;
        self.manifest
            .verify("package/eplyx.json", package::MAX_MANIFEST_BYTES as usize)?;
        self.config
            .verify("package/config.json", package::MAX_CONFIG_BYTES as usize)?;
        if let Some(search) = &self.search {
            search.verify("search/counterexamples.json", SEARCH_LIMIT)?;
        }
        ensure!(
            self.local_artifact_sizes.len() <= view::ARTIFACTS.len()
                && self
                    .local_artifact_sizes
                    .keys()
                    .all(|name| view::artifact(name).is_some()),
            "unknown local artifact name"
        );
        let metadata: Metadata =
            serde_json::from_str(&self.metadata.text).context("invalid metadata.json")?;
        ensure!(
            matches!(metadata.schema_version, 1 | 2),
            "unsupported metadata schema"
        );
        ensure!(metadata.run_id == self.run_id, "metadata run ID mismatch");
        let (manifest, transition) =
            package::declared_identity(self.manifest.text.as_bytes(), self.config.text.as_bytes())?;
        ensure!(
            transition == metadata.transition_package_sha256
                && manifest.candidate_program.sha256 == metadata.candidate_program_sha256,
            "package identity does not match run metadata"
        );
        let report: Value =
            serde_json::from_str(&self.report.text).context("invalid report.json")?;
        let text = |value: &Value| value.as_str().map(str::to_owned);
        ensure!(
            text(&report["transition_package_sha256"]).as_deref() == Some(transition.as_str())
                && text(&report["candidate_program_sha256"]).as_deref()
                    == Some(metadata.candidate_program_sha256.as_str())
                && text(&report["config_sha256"]).as_deref() == Some(self.config.sha256.as_str()),
            "report.json is bound to a different package"
        );
        ensure!(
            text(&report["gate_outcome"]).as_deref() == Some(metadata.gate_outcome.as_str())
                && text(&report["gate_policy"]).as_deref() == Some(metadata.gate_policy.as_str()),
            "report.json gate differs from run metadata"
        );
        ensure!(
            report["official_transition"] == "NotTested" && report["funds_moved"] == false,
            "report.json claims an official transition or moved funds"
        );
        ensure!(
            report["run_id"].as_str().is_some_and(|id| !id.is_empty()),
            "report.json has no package run ID"
        );
        let bindings: Value =
            serde_json::from_str(&self.bindings.text).context("invalid bindings.json")?;
        ensure!(
            bindings["transition_package_sha256"] == report["transition_package_sha256"]
                && bindings["candidate_program_sha256"] == report["candidate_program_sha256"]
                && bindings["config_sha256"] == report["config_sha256"],
            "bindings.json is bound to a different package"
        );
        let run = view::from_bytes(
            &self.run_id,
            RunBytes {
                metadata: Some(self.metadata.text.as_bytes()),
                report: Some(self.report.text.as_bytes()),
                manifest: Some(self.manifest.text.as_bytes()),
                config: Some(self.config.text.as_bytes()),
                search: self.search.as_ref().map(|s| s.text.as_bytes()),
            },
        );
        if let Some(problem) = run.problems().first() {
            bail!("run is not consistent: {problem}");
        }
        ensure!(view::state(&run) == "Complete", "run is not complete");
        if self.search.is_some() {
            ensure!(run.search().is_some(), "search result is unreadable");
        }
        let summary = view::summary(&run);
        Ok(VerifiedRun {
            core_sha256: self.core_sha256()?,
            search_sha256: self.search.as_ref().map(|s| s.sha256.clone()),
            metadata,
            run,
            summary,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterexampleDocument {
    pub schema: String,
    pub local_project_id: String,
    pub counterexample_id: String,
    /// The saved `.eplyx/counterexamples/<id>.json` file, byte for byte.
    pub file: Artifact,
}

pub struct VerifiedCounterexample {
    pub saved: SavedCounterexample,
    pub summary: Value,
}

impl CounterexampleDocument {
    /// Checks that need no parent run: bounds, digest, privacy and the
    /// content-addressed `cx_` identity recomputed by the engine.
    pub fn verify(&self) -> Result<VerifiedCounterexample> {
        ensure!(
            self.schema == COUNTEREXAMPLE_SCHEMA,
            "unsupported counterexample document schema"
        );
        ensure!(
            is_safe_id(&self.local_project_id, "project_"),
            "invalid local project ID"
        );
        ensure!(
            is_safe_id(&self.counterexample_id, "cx_"),
            "invalid counterexample ID"
        );
        self.file
            .verify("saved counterexample", COUNTEREXAMPLE_LIMIT)?;
        let saved: SavedCounterexample =
            serde_json::from_str(&self.file.text).context("invalid saved counterexample")?;
        ensure!(
            saved.schema_version == 1
                && saved.id == self.counterexample_id
                && counterexample_id(&saved.counterexample)? == self.counterexample_id
                && is_safe_id(&saved.parent_run, "run_")
                && valid_digest(&saved.search_sha256)
                && saved
                    .replay_inputs
                    .as_ref()
                    .is_none_or(|inputs| inputs == &replay_inputs(&saved.parent_run)),
            "counterexample identity mismatch"
        );
        let summary = view::counterexample_fields(&saved, &self.counterexample_id);
        ensure!(
            summary["state"] == "Valid",
            "counterexample identity mismatch"
        );
        Ok(VerifiedCounterexample { saved, summary })
    }
}

/// A counterexample belongs to exactly one synced run: the run's saved search
/// must have the recorded digest and contain this exact engine counterexample.
pub fn bind_counterexample(saved: &SavedCounterexample, parent: &view::Run) -> Result<()> {
    ensure!(
        parent.id() == saved.parent_run,
        "counterexample belongs to a different run"
    );
    ensure!(
        parent.search_sha256() == Some(saved.search_sha256.as_str()),
        "counterexample search digest does not match the synced run's search"
    );
    let search = parent
        .search()
        .context("the parent run has no synced search result")?;
    ensure!(
        search.counterexamples.contains(&saved.counterexample),
        "counterexample is absent from the parent run's search result"
    );
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReproductionDocument {
    pub schema: String,
    pub local_project_id: String,
    pub reproduction_id: String,
    pub counterexample_id: String,
    /// The reproduction record. Its free-text error is sanitized of local
    /// roots and URLs before upload; every other field is unchanged.
    pub file: Artifact,
}

impl ReproductionDocument {
    pub fn verify(&self) -> Result<Reproduction> {
        ensure!(
            self.schema == REPRODUCTION_SCHEMA,
            "unsupported reproduction document schema"
        );
        ensure!(
            is_safe_id(&self.local_project_id, "project_")
                && is_safe_id(&self.reproduction_id, "repro_")
                && is_safe_id(&self.counterexample_id, "cx_"),
            "invalid reproduction IDs"
        );
        self.file
            .verify("reproduction record", REPRODUCTION_LIMIT)?;
        let record: Reproduction =
            serde_json::from_str(&self.file.text).context("invalid reproduction record")?;
        ensure!(
            record.id == self.reproduction_id
                && record.counterexample_id == self.counterexample_id
                && record.schema_version == 1
                && record
                    .id
                    .ends_with(self.counterexample_id.trim_start_matches("cx_"))
                && record
                    .parent_run
                    .as_deref()
                    .is_none_or(|run| is_safe_id(run, "run_"))
                && record.search_sha256.as_deref().is_none_or(valid_digest),
            "reproduction identity mismatch"
        );
        Ok(record)
    }
}

/// A reproduction record must name a synced counterexample, and any parent run
/// or search digest it recorded must be that counterexample's.
pub fn bind_reproduction(record: &Reproduction, saved: &SavedCounterexample) -> Result<()> {
    ensure!(
        record.counterexample_id == saved.id
            && record
                .parent_run
                .as_deref()
                .is_none_or(|run| run == saved.parent_run)
            && record
                .search_sha256
                .as_deref()
                .is_none_or(|digest| digest == saved.search_sha256),
        "reproduction record is bound to a different counterexample"
    );
    Ok(())
}

/// Summary the dashboards show for one reproduction record.
pub fn reproduction_summary(record: &Reproduction) -> Value {
    let mut value = json!(record);
    value["state"] = json!("Valid");
    value
}

// ------------------------------------------------------------ local builders

fn member(store: &Store, parts: &[&str], limit: usize) -> Result<Option<Artifact>> {
    store
        .read(parts, limit as u64)?
        .map(Artifact::new)
        .transpose()
}

/// Build the document for one complete local run from guarded store reads.
pub fn run_document(store: &Store, local_project_id: &str, run_id: &str) -> Result<RunDocument> {
    ensure!(is_safe_id(run_id, "run_"), "invalid run ID");
    let required = |relative: &str, limit: usize| -> Result<Artifact> {
        let mut parts = vec!["runs", run_id];
        parts.extend(relative.split('/'));
        member(store, &parts, limit)?
            .with_context(|| format!("{run_id} has no {relative}; only complete runs sync"))
    };
    let metadata = required("metadata.json", METADATA_LIMIT)?;
    let report = required("result/report.json", REPORT_LIMIT)?;
    let bindings = required("result/bindings.json", BINDINGS_LIMIT)?;
    let manifest = required("package/eplyx.json", package::MAX_MANIFEST_BYTES as usize)?;
    let config = required("package/config.json", package::MAX_CONFIG_BYTES as usize)?;
    let search = member(
        store,
        &["runs", run_id, "search", "counterexamples.json"],
        SEARCH_LIMIT,
    )?;
    let local_artifact_sizes = view::ARTIFACTS
        .iter()
        .map(|(name, path, ..)| {
            let mut parts = vec!["runs", run_id];
            parts.extend(path.split('/'));
            let size = store
                .open_file(&parts, CAPTURE_LIMIT)
                .ok()
                .flatten()
                .map(|(_, len)| len);
            ((*name).to_owned(), size)
        })
        .collect();
    Ok(RunDocument {
        schema: RUN_SCHEMA.into(),
        local_project_id: local_project_id.into(),
        run_id: run_id.into(),
        metadata,
        report,
        bindings,
        manifest,
        config,
        search,
        local_artifact_sizes,
    })
}

pub fn counterexample_document(
    store: &Store,
    local_project_id: &str,
    id: &str,
) -> Result<CounterexampleDocument> {
    ensure!(is_safe_id(id, "cx_"), "invalid counterexample ID");
    let file = format!("{id}.json");
    let file = member(store, &["counterexamples", &file], COUNTEREXAMPLE_LIMIT)?
        .context("counterexample file missing")?;
    Ok(CounterexampleDocument {
        schema: COUNTEREXAMPLE_SCHEMA.into(),
        local_project_id: local_project_id.into(),
        counterexample_id: id.into(),
        file,
    })
}

/// Build a reproduction document; `roots` are local path prefixes replaced in
/// the free-text error (for example the project root and home directory).
pub fn reproduction_document(
    store: &Store,
    local_project_id: &str,
    id: &str,
    roots: &[(String, &str)],
) -> Result<ReproductionDocument> {
    ensure!(is_safe_id(id, "repro_"), "invalid reproduction ID");
    let file = format!("{id}.json");
    let bytes = store
        .read(&["reproductions", &file], REPRODUCTION_LIMIT as u64)?
        .context("reproduction record missing")?;
    let mut record: Reproduction =
        serde_json::from_slice(&bytes).context("invalid reproduction record")?;
    ensure!(record.id == id, "reproduction record ID mismatch");
    let file = match &record.error {
        Some(error) if privacy::scan("error", error).is_err() => {
            record.error = Some(privacy::sanitize(error, roots));
            Artifact::new(serde_json::to_vec_pretty(&record)?)?
        }
        _ => Artifact::new(bytes)?,
    };
    Ok(ReproductionDocument {
        schema: REPRODUCTION_SCHEMA.into(),
        local_project_id: local_project_id.into(),
        reproduction_id: id.into(),
        counterexample_id: record.counterexample_id.clone(),
        file,
    })
}
