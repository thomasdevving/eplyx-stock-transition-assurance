//! Hosted dashboard views. Every payload has the same shape as the local
//! dashboard API and is produced by the engine's own `dashboard::view`
//! functions over synced bytes: the same summaries, gate views, joins and
//! Milestone 16 comparison semantics. Nothing is recomputed in the browser,
//! and no view executes, replays or calls a provider.
use crate::{
    api::{self, project_access, stored_run, Access, RUN_COLUMNS},
    auth,
    error::{ApiError, ApiResult},
    Shared,
};
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
    Json,
};
use eplyx_lifecycle_impact::{
    dashboard::view::{self, DetailContext},
    local_store::{is_safe_id, SavedCounterexample},
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Who is viewing: a workspace member, or anyone on the single demo project.
async fn viewer(state: &Shared, headers: &HeaderMap, project: Option<&str>) -> ApiResult<Access> {
    match project {
        Some(project) => {
            let principal = auth::require_user(state, headers).await?;
            project_access(state, &principal, project).await
        }
        None => {
            let id = state
                .config
                .demo_project
                .clone()
                .ok_or_else(|| ApiError::not_found("no public demo project on this server"))?;
            let client = state.db.get().await?;
            let row = client
                .query_opt(
                    "SELECT p.id, p.name, w.id, w.name, p.created_at FROM projects p JOIN workspaces w ON w.id = p.workspace_id WHERE p.id = $1",
                    &[&id],
                )
                .await?
                .ok_or_else(|| ApiError::not_found("no public demo project on this server"))?;
            Ok(Access {
                project_id: row.get(0),
                project_name: row.get(1),
                workspace_id: row.get(2),
                workspace_name: row.get(3),
                role: None,
                created_at: row.get(4),
            })
        }
    }
}

struct Snapshot {
    runs: Vec<Value>,
    files: Vec<Value>,
    reproductions: Vec<Value>,
}

/// Summaries for one project, joined by the engine exactly as locally.
async fn snapshot(state: &Shared, project: &str) -> ApiResult<Snapshot> {
    let client = state.db.get().await?;
    let run_rows = client
        .query(
            "SELECT summary, synced_by, synced_via, synced_at, local_project_id, search_synced_at FROM runs WHERE project_id = $1 ORDER BY run_id",
            &[&project],
        )
        .await?;
    let summaries = run_rows
        .iter()
        .map(|row| {
            let mut summary: Value =
                serde_json::from_str(&row.get::<_, String>(0)).unwrap_or(Value::Null);
            summary["synced"] = json!({
                "by": row.get::<_, String>(1),
                "via": row.get::<_, String>(2),
                "at": row.get::<_, chrono::DateTime<chrono::Utc>>(3),
                "local_project_id": row.get::<_, String>(4),
                "search_at": row.get::<_, Option<chrono::DateTime<chrono::Utc>>>(5),
            });
            summary
        })
        .collect();
    let cx_rows = client
        .query(
            "SELECT summary, synced_at FROM counterexamples WHERE project_id = $1 ORDER BY cx_id",
            &[&project],
        )
        .await?;
    let counterexamples = cx_rows
        .iter()
        .map(|row| {
            let mut summary: Value =
                serde_json::from_str(&row.get::<_, String>(0)).unwrap_or(Value::Null);
            // The local file's modification time does not travel; the sync
            // time is shown instead and labeled as such by the page.
            summary["saved_at_ms"] = Value::Null;
            summary["synced_at"] = json!(row.get::<_, chrono::DateTime<chrono::Utc>>(1));
            summary
        })
        .collect();
    let reproductions: Vec<Value> = client
        .query(
            "SELECT summary FROM reproductions WHERE project_id = $1 ORDER BY repro_id DESC",
            &[&project],
        )
        .await?
        .iter()
        .map(|row| serde_json::from_str(&row.get::<_, String>(0)).unwrap_or(Value::Null))
        .collect();
    let (runs, files) = view::assemble(summaries, counterexamples, &reproductions);
    Ok(Snapshot {
        runs,
        files,
        reproductions,
    })
}

fn latest_by_source(runs: &[Value], source: &str) -> Value {
    runs.iter()
        .find(|r| r["state"] == "Complete" && r["run_source"] == source)
        .map_or(Value::Null, |r| {
            json!({"id": r["id"], "number": r["number"], "gate": r["gate"]["outcome"], "timestamp": r["timestamp"],
                   "commit": r["git"]["commit"], "branch": r["git"]["branch"]})
        })
}

async fn project_view(state: &Shared, access: &Access, demo: bool) -> ApiResult<Value> {
    let snap = snapshot(state, &access.project_id).await?;
    let links = if demo {
        Value::Null
    } else {
        api::project_json(state, access).await?["links"].clone()
    };
    let mut payload = view::project_payload(
        json!({"name": access.project_name, "id": access.project_id}),
        json!({
            "config": {"state": "Cloud"},
            "cloud": {
                "workspace": {"id": access.workspace_id, "name": access.workspace_name},
                "project": {"id": access.project_id, "name": access.project_name, "visibility": "workspace", "created_at": access.created_at},
                "role": access.role,
                "demo": demo,
                "links": links,
            },
        }),
        json!({"path": "cloud", "root_display": "", "index": ""}),
        &snap.runs,
        &snap.files,
        &snap.reproductions,
        0,
    );
    let reproduced = snap
        .files
        .iter()
        .filter(|c| c["reproductions"]["succeeded"].as_u64().unwrap_or(0) > 0)
        .count();
    payload["cloud"] = json!({
        "latest_local": latest_by_source(&snap.runs, "local"),
        "latest_ci": latest_by_source(&snap.runs, "ci"),
        "counterexamples_reproduced": reproduced,
        "synced_note": "Synced results from local and CI Eplyx CLI runs. Viewing this workspace never reruns RPC, execution or replay.",
    });
    Ok(payload)
}

type ProjectPath = Path<String>;

pub async fn project(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(project): ProjectPath,
) -> ApiResult<Json<Value>> {
    let access = viewer(&state, &headers, Some(&project)).await?;
    Ok(Json(project_view(&state, &access, false).await?))
}

pub async fn demo_project(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let access = viewer(&state, &headers, None).await?;
    Ok(Json(project_view(&state, &access, true).await?))
}

async fn runs_payload(state: &Shared, access: &Access) -> ApiResult<Value> {
    let snap = snapshot(state, &access.project_id).await?;
    Ok(json!({"runs": snap.runs, "ignored_store_entries": 0}))
}

pub async fn runs(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(project): ProjectPath,
) -> ApiResult<Json<Value>> {
    let access = viewer(&state, &headers, Some(&project)).await?;
    Ok(Json(runs_payload(&state, &access).await?))
}

pub async fn demo_runs(State(state): State<Shared>, headers: HeaderMap) -> ApiResult<Json<Value>> {
    let access = viewer(&state, &headers, None).await?;
    Ok(Json(runs_payload(&state, &access).await?))
}

async fn run_payload(state: &Shared, access: &Access, run: &str) -> ApiResult<Value> {
    if !is_safe_id(run, "run_") {
        return Err(ApiError::not_found("unknown run"));
    }
    let client = state.db.get().await?;
    let row = client
        .query_opt(
            &format!("SELECT {RUN_COLUMNS}, r.artifact_sizes FROM runs r WHERE r.project_id = $1 AND r.run_id = $2"),
            &[&access.project_id, &run],
        )
        .await?
        .ok_or_else(|| ApiError::not_found("unknown run"))?;
    let snap = snapshot(state, &access.project_id).await?;
    let sizes: BTreeMap<String, Option<u64>> =
        serde_json::from_str(&row.get::<_, String>(7)).unwrap_or_default();
    let ordered: Vec<Option<u64>> = view::ARTIFACTS
        .iter()
        .map(|(name, ..)| sizes.get(*name).copied().flatten())
        .collect();
    let bindings: Option<Value> = serde_json::from_str(&row.get::<_, String>(2)).ok();
    let parsed = stored_run(&row, 0);
    let run_id = run.to_owned();
    let files = snap.files.clone();
    let mut detail = tokio::task::spawn_blocking(move || {
        view::run_detail_for(
            &parsed,
            DetailContext {
                provider: None,
                bindings,
                artifacts: view::artifact_rows(&run_id, &ordered),
            },
            &files,
        )
    })
    .await
    .map_err(ApiError::internal)?;
    if let Some(summary) = snap.runs.iter().find(|r| r["id"] == run) {
        detail["number"] = summary["number"].clone();
        detail["saved_counterexamples"] = summary["saved_counterexamples"].clone();
        detail["synced"] = summary["synced"].clone();
    }
    let position = snap.runs.iter().position(|r| r["id"] == run);
    detail["previous_run"] = position
        .and_then(|p| snap.runs.get(p + 1))
        .map_or(Value::Null, |r| r["id"].clone());
    // Captures, program bytes and the provider origin stay on the machine.
    detail["artifacts_local_only"] = json!(true);
    Ok(detail)
}

pub async fn run(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path((project, run)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    let access = viewer(&state, &headers, Some(&project)).await?;
    Ok(Json(run_payload(&state, &access, &run).await?))
}

pub async fn demo_run(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(run): Path<String>,
) -> ApiResult<Json<Value>> {
    let access = viewer(&state, &headers, None).await?;
    Ok(Json(run_payload(&state, &access, &run).await?))
}

async fn counterexamples_payload(state: &Shared, access: &Access) -> ApiResult<Value> {
    let snap = snapshot(state, &access.project_id).await?;
    Ok(json!({"counterexamples": snap.files}))
}

pub async fn counterexamples(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(project): ProjectPath,
) -> ApiResult<Json<Value>> {
    let access = viewer(&state, &headers, Some(&project)).await?;
    Ok(Json(counterexamples_payload(&state, &access).await?))
}

pub async fn demo_counterexamples(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let access = viewer(&state, &headers, None).await?;
    Ok(Json(counterexamples_payload(&state, &access).await?))
}

async fn counterexample_payload(state: &Shared, access: &Access, id: &str) -> ApiResult<Value> {
    if !is_safe_id(id, "cx_") {
        return Err(ApiError::not_found("unknown counterexample"));
    }
    let client = state.db.get().await?;
    let row = client
        .query_opt(
            &format!("SELECT {RUN_COLUMNS}, c.file_text FROM counterexamples c JOIN runs r ON r.project_id = c.project_id AND r.run_id = c.run_id
                      WHERE c.project_id = $1 AND c.cx_id = $2"),
            &[&access.project_id, &id],
        )
        .await?
        .ok_or_else(|| ApiError::not_found("unknown counterexample"))?;
    let snap = snapshot(state, &access.project_id).await?;
    let summary = snap
        .files
        .iter()
        .find(|c| c["id"] == id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("unknown counterexample"))?;
    let saved: SavedCounterexample =
        serde_json::from_str(&row.get::<_, String>(7)).map_err(ApiError::internal)?;
    let parent = stored_run(&row, 0);
    let summary_for_detail = summary.clone();
    let mut detail = tokio::task::spawn_blocking(move || {
        view::counterexample_detail_for(&saved, &summary_for_detail, &parent)
    })
    .await
    .map_err(ApiError::internal)?
    .map_err(ApiError::internal)?;
    detail["parent_summary"] = snap
        .runs
        .iter()
        .find(|r| r["id"] == summary["parent_run"])
        .cloned()
        .unwrap_or(Value::Null);
    Ok(detail)
}

pub async fn counterexample(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path((project, id)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    let access = viewer(&state, &headers, Some(&project)).await?;
    Ok(Json(counterexample_payload(&state, &access, &id).await?))
}

pub async fn demo_counterexample(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let access = viewer(&state, &headers, None).await?;
    Ok(Json(counterexample_payload(&state, &access, &id).await?))
}

async fn raw_counterexample(state: &Shared, access: &Access, id: &str) -> ApiResult<Response> {
    if !is_safe_id(id, "cx_") {
        return Err(ApiError::not_found("unknown counterexample"));
    }
    let client = state.db.get().await?;
    let text: String = client
        .query_opt(
            "SELECT file_text FROM counterexamples WHERE project_id = $1 AND cx_id = $2",
            &[&access.project_id, &id],
        )
        .await?
        .ok_or_else(|| ApiError::not_found("unknown counterexample"))?
        .get(0);
    let mut response = text.into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{id}.json\""))
            .map_err(ApiError::internal)?,
    );
    Ok(response)
}

pub async fn counterexample_raw(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path((project, id)): Path<(String, String)>,
) -> ApiResult<Response> {
    let access = viewer(&state, &headers, Some(&project)).await?;
    raw_counterexample(&state, &access, &id).await
}

pub async fn demo_counterexample_raw(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Response> {
    let access = viewer(&state, &headers, None).await?;
    raw_counterexample(&state, &access, &id).await
}

#[derive(Deserialize)]
pub struct CompareQuery {
    left: Option<String>,
    right: Option<String>,
}

async fn compare_payload(state: &Shared, access: &Access, query: CompareQuery) -> ApiResult<Value> {
    let (Some(left), Some(right)) = (query.left, query.right) else {
        return Err(ApiError::bad_request("choose two runs: left and right"));
    };
    if !is_safe_id(&left, "run_") || !is_safe_id(&right, "run_") {
        return Err(ApiError::not_found("unknown run"));
    }
    let client = state.db.get().await?;
    let mut parsed = Vec::new();
    for id in [&left, &right] {
        let row = client
            .query_opt(
                &format!(
                    "SELECT {RUN_COLUMNS} FROM runs r WHERE r.project_id = $1 AND r.run_id = $2"
                ),
                &[&access.project_id, id],
            )
            .await?
            .ok_or_else(|| ApiError::not_found("unknown run"))?;
        parsed.push(stored_run(&row, 0));
    }
    let snap = snapshot(state, &access.project_id).await?;
    let (b, a) = (
        parsed.pop().expect("two runs"),
        parsed.pop().expect("two runs"),
    );
    let mut comparison = tokio::task::spawn_blocking(move || view::compare_runs(&a, &b))
        .await
        .map_err(ApiError::internal)?
        .map_err(|e| ApiError::invalid(format!("{e:#}")))?;
    for (side, id) in [("left", &left), ("right", &right)] {
        if let Some(run) = snap.runs.iter().find(|r| r["id"] == id.as_str()) {
            comparison[side]["number"] = run["number"].clone();
            comparison[side]["synced"] = run["synced"].clone();
        }
    }
    Ok(comparison)
}

pub async fn compare(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(project): ProjectPath,
    Query(query): Query<CompareQuery>,
) -> ApiResult<Json<Value>> {
    let access = viewer(&state, &headers, Some(&project)).await?;
    Ok(Json(compare_payload(&state, &access, query).await?))
}

pub async fn demo_compare(
    State(state): State<Shared>,
    headers: HeaderMap,
    Query(query): Query<CompareQuery>,
) -> ApiResult<Json<Value>> {
    let access = viewer(&state, &headers, None).await?;
    Ok(Json(compare_payload(&state, &access, query).await?))
}
