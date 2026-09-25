//! Workspace, project, link, CI-token and sync endpoints. Sync verifies every
//! document with the engine's contract checks, binds it to its parent record
//! and stores it immutably: the same content twice is a no-op, and different
//! content under an existing identity is a conflict, never an overwrite.
use crate::{
    auth::{self, clean_name, parse_json, Principal},
    error::{ApiError, ApiResult},
    Shared,
};
use axum::{
    body::Bytes,
    extract::{rejection::BytesRejection, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use eplyx_lifecycle_impact::{
    cloud::contract::{
        self, is_cloud_id, CounterexampleDocument, ReproductionDocument, RunDocument,
    },
    dashboard::view::{self, RunBytes},
    local_store::{is_safe_id, SavedCounterexample},
};
use serde::Deserialize;
use serde_json::{json, Value};

pub struct Access {
    pub project_id: String,
    pub project_name: String,
    pub workspace_id: String,
    pub workspace_name: String,
    /// `owner` or `member`; `None` for a CI token.
    pub role: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Resolve a project the caller may see. Unknown and inaccessible projects
/// are indistinguishable (404), so project IDs cannot be probed.
pub async fn project_access(
    state: &Shared,
    principal: &Principal,
    project_id: &str,
) -> ApiResult<Access> {
    let unknown = || ApiError::not_found("unknown project");
    if !is_cloud_id(project_id, "prj_") {
        return Err(unknown());
    }
    let client = state.db.get().await?;
    let row = match principal {
        Principal::User { id, .. } => {
            client
                .query_opt(
                    "SELECT p.id, p.name, w.id, w.name, m.role, p.created_at FROM projects p
                     JOIN workspaces w ON w.id = p.workspace_id
                     JOIN workspace_members m ON m.workspace_id = p.workspace_id AND m.user_id = $2
                     WHERE p.id = $1",
                    &[&project_id, id],
                )
                .await?
        }
        Principal::Ci {
            project_id: scoped, ..
        } => {
            if scoped != project_id {
                return Err(unknown());
            }
            client
                .query_opt(
                    "SELECT p.id, p.name, w.id, w.name, NULL::TEXT, p.created_at FROM projects p
                     JOIN workspaces w ON w.id = p.workspace_id WHERE p.id = $1",
                    &[&project_id],
                )
                .await?
        }
    };
    let row = row.ok_or_else(unknown)?;
    Ok(Access {
        project_id: row.get(0),
        project_name: row.get(1),
        workspace_id: row.get(2),
        workspace_name: row.get(3),
        role: row.get(4),
        created_at: row.get(5),
    })
}

async fn workspace_role(state: &Shared, user_id: &str, workspace_id: &str) -> ApiResult<String> {
    if !is_cloud_id(workspace_id, "ws_") {
        return Err(ApiError::not_found("unknown workspace"));
    }
    let client = state.db.get().await?;
    client
        .query_opt(
            "SELECT role FROM workspace_members WHERE workspace_id = $1 AND user_id = $2",
            &[&workspace_id, &user_id],
        )
        .await?
        .map(|row| row.get(0))
        .ok_or_else(|| ApiError::not_found("unknown workspace"))
}

fn body(body: Result<Bytes, BytesRejection>) -> ApiResult<Bytes> {
    body.map_err(|rejection| {
        if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ApiError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "document exceeds the sync size bound",
            )
        } else {
            ApiError::bad_request("could not read the request body")
        }
    })
}

fn owner(role: &Option<String>) -> ApiResult<()> {
    if role.as_deref() == Some("owner") {
        Ok(())
    } else {
        Err(ApiError::forbidden("only a workspace owner can do this"))
    }
}

// --------------------------------------------------------------- workspaces

pub async fn list_workspaces(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let principal = auth::require_user(&state, &headers).await?;
    let user = principal.user_id().unwrap_or_default().to_owned();
    let client = state.db.get().await?;
    let rows = client
        .query(
            "SELECT w.id, w.name, m.role FROM workspaces w JOIN workspace_members m ON m.workspace_id = w.id
             WHERE m.user_id = $1 ORDER BY w.created_at, w.id",
            &[&user],
        )
        .await?;
    let mut workspaces = Vec::new();
    for row in rows {
        let id: String = row.get(0);
        let projects = client
            .query(
                "SELECT p.id, p.name, count(r.run_id),
                        (SELECT r2.gate_outcome FROM runs r2 WHERE r2.project_id = p.id ORDER BY r2.run_id DESC LIMIT 1)
                 FROM projects p LEFT JOIN runs r ON r.project_id = p.id
                 WHERE p.workspace_id = $1 GROUP BY p.id ORDER BY p.created_at, p.id",
                &[&id],
            )
            .await?
            .iter()
            .map(|p| {
                json!({"id": p.get::<_, String>(0), "name": p.get::<_, String>(1), "runs": p.get::<_, i64>(2), "latest_gate": p.get::<_, Option<String>>(3)})
            })
            .collect::<Vec<_>>();
        workspaces.push(json!({"id": id, "name": row.get::<_, String>(1), "role": row.get::<_, String>(2), "projects": projects}));
    }
    Ok(Json(json!({"workspaces": workspaces})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NameRequest {
    name: String,
}

pub async fn create_workspace(
    State(state): State<Shared>,
    headers: HeaderMap,
    raw: Result<Bytes, BytesRejection>,
) -> ApiResult<Response> {
    let principal = auth::require_user(&state, &headers).await?;
    auth::same_origin(&state, &headers, &principal)?;
    let request: NameRequest = parse_json(&body(raw)?)?;
    let name = clean_name(&request.name, "workspace name")?;
    let id = auth::new_id("ws_");
    let user = principal.user_id().unwrap_or_default().to_owned();
    let mut client = state.db.get().await?;
    let tx = client.transaction().await?;
    tx.execute(
        "INSERT INTO workspaces (id, name, created_by) VALUES ($1, $2, $3)",
        &[&id, &name, &user],
    )
    .await?;
    tx.execute(
        "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, 'owner')",
        &[&id, &user],
    )
    .await?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"workspace": {"id": id, "name": name, "role": "owner"}})),
    )
        .into_response())
}

pub async fn list_members(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(workspace): Path<String>,
) -> ApiResult<Json<Value>> {
    let principal = auth::require_user(&state, &headers).await?;
    let role = workspace_role(&state, principal.user_id().unwrap_or_default(), &workspace).await?;
    let client = state.db.get().await?;
    let members = client
        .query(
            "SELECT u.id, u.email, u.name, m.role, m.added_at FROM workspace_members m JOIN users u ON u.id = m.user_id
             WHERE m.workspace_id = $1 ORDER BY m.added_at, u.email",
            &[&workspace],
        )
        .await?
        .iter()
        .map(|r| json!({"id": r.get::<_, String>(0), "email": r.get::<_, String>(1), "name": r.get::<_, String>(2), "role": r.get::<_, String>(3), "added_at": r.get::<_, chrono::DateTime<chrono::Utc>>(4)}))
        .collect::<Vec<_>>();
    Ok(Json(json!({"role": role, "members": members})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MemberRequest {
    email: String,
}

pub async fn add_member(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(workspace): Path<String>,
    raw: Result<Bytes, BytesRejection>,
) -> ApiResult<Json<Value>> {
    let principal = auth::require_user(&state, &headers).await?;
    auth::same_origin(&state, &headers, &principal)?;
    let role = workspace_role(&state, principal.user_id().unwrap_or_default(), &workspace).await?;
    owner(&Some(role))?;
    let request: MemberRequest = parse_json(&body(raw)?)?;
    let email = request.email.trim().to_ascii_lowercase();
    let client = state.db.get().await?;
    let user = client
        .query_opt("SELECT id FROM users WHERE email = $1", &[&email])
        .await?
        .ok_or_else(|| {
            ApiError::not_found("no Eplyx account uses this email; ask them to sign up first")
        })?;
    let user_id: String = user.get(0);
    client
        .execute(
            "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, 'member') ON CONFLICT DO NOTHING",
            &[&workspace, &user_id],
        )
        .await?;
    Ok(Json(json!({"added": email})))
}

pub async fn remove_member(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path((workspace, member)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    let principal = auth::require_user(&state, &headers).await?;
    auth::same_origin(&state, &headers, &principal)?;
    let role = workspace_role(&state, principal.user_id().unwrap_or_default(), &workspace).await?;
    owner(&Some(role))?;
    if principal.user_id() == Some(member.as_str()) {
        return Err(ApiError::bad_request("owners cannot remove themselves"));
    }
    let client = state.db.get().await?;
    let removed = client
        .execute(
            "DELETE FROM workspace_members WHERE workspace_id = $1 AND user_id = $2 AND role = 'member'",
            &[&workspace, &member],
        )
        .await?;
    if removed == 0 {
        return Err(ApiError::not_found("no such member"));
    }
    Ok(Json(json!({"removed": member})))
}

// ----------------------------------------------------------------- projects

pub async fn create_project(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(workspace): Path<String>,
    raw: Result<Bytes, BytesRejection>,
) -> ApiResult<Response> {
    let principal = auth::require_user(&state, &headers).await?;
    auth::same_origin(&state, &headers, &principal)?;
    workspace_role(&state, principal.user_id().unwrap_or_default(), &workspace).await?;
    let request: NameRequest = parse_json(&body(raw)?)?;
    let name = clean_name(&request.name, "project name")?;
    let id = auth::new_id("prj_");
    let client = state.db.get().await?;
    client
        .execute(
            "INSERT INTO projects (id, workspace_id, name, created_by) VALUES ($1, $2, $3, $4)",
            &[
                &id,
                &workspace,
                &name,
                &principal.user_id().unwrap_or_default(),
            ],
        )
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"project": {"id": id, "name": name, "workspace_id": workspace, "visibility": "workspace"}})),
    )
        .into_response())
}

pub async fn project_json(state: &Shared, access: &Access) -> ApiResult<Value> {
    let client = state.db.get().await?;
    let links = client
        .query(
            "SELECT local_project_id, linked_by, linked_via, linked_at FROM project_links WHERE project_id = $1 ORDER BY linked_at",
            &[&access.project_id],
        )
        .await?
        .iter()
        .map(|r| json!({"local_project_id": r.get::<_, String>(0), "linked_by": r.get::<_, String>(1), "linked_via": r.get::<_, String>(2), "linked_at": r.get::<_, chrono::DateTime<chrono::Utc>>(3)}))
        .collect::<Vec<_>>();
    Ok(json!({
        "project": {"id": access.project_id, "name": access.project_name, "visibility": "workspace", "created_at": access.created_at,
                    "demo": state.config.demo_project.as_deref() == Some(access.project_id.as_str())},
        "workspace": {"id": access.workspace_id, "name": access.workspace_name},
        "role": access.role,
        "links": links,
        "url": format!("{}/p/{}", state.config.public_url, access.project_id),
    }))
}

pub async fn get_project(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(project): Path<String>,
) -> ApiResult<Json<Value>> {
    let principal = auth::require(&state, &headers).await?;
    let access = project_access(&state, &principal, &project).await?;
    Ok(Json(project_json(&state, &access).await?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LinkRequest {
    local_project_id: String,
}

async fn record_link(
    state: &Shared,
    project: &str,
    local_project_id: &str,
    principal: &Principal,
) -> ApiResult<()> {
    let client = state.db.get().await?;
    client
        .execute(
            "INSERT INTO project_links (project_id, local_project_id, linked_by, linked_via) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
            &[&project, &local_project_id, &principal.label(), &principal.via()],
        )
        .await?;
    Ok(())
}

pub async fn link_project(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(project): Path<String>,
    raw: Result<Bytes, BytesRejection>,
) -> ApiResult<Json<Value>> {
    let principal = auth::require(&state, &headers).await?;
    // Linking binds a developer's local store; browser sessions never do it.
    if matches!(principal, Principal::User { session: true, .. }) {
        return Err(ApiError::forbidden(
            "link a local project with `eplyx link`",
        ));
    }
    let access = project_access(&state, &principal, &project).await?;
    let request: LinkRequest = parse_json(&body(raw)?)?;
    if !is_safe_id(&request.local_project_id, "project_") {
        return Err(ApiError::invalid("invalid local project ID"));
    }
    record_link(
        &state,
        &access.project_id,
        &request.local_project_id,
        &principal,
    )
    .await?;
    Ok(Json(project_json(&state, &access).await?))
}

// ---------------------------------------------------------------- CI tokens

pub async fn list_ci_tokens(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(project): Path<String>,
) -> ApiResult<Json<Value>> {
    let principal = auth::require_user(&state, &headers).await?;
    let access = project_access(&state, &principal, &project).await?;
    owner(&access.role)?;
    let client = state.db.get().await?;
    let tokens = client
        .query(
            "SELECT t.id, t.label, t.created_at, t.last_used_at, t.revoked_at, u.email FROM api_tokens t JOIN users u ON u.id = t.user_id
             WHERE t.project_id = $1 AND t.kind = 'ci' ORDER BY t.created_at DESC",
            &[&access.project_id],
        )
        .await?
        .iter()
        .map(|r| {
            type Time = Option<chrono::DateTime<chrono::Utc>>;
            json!({"id": r.get::<_, String>(0), "label": r.get::<_, String>(1), "created_at": r.get::<_, chrono::DateTime<chrono::Utc>>(2),
                   "last_used_at": r.get::<_, Time>(3), "revoked_at": r.get::<_, Time>(4), "created_by": r.get::<_, String>(5)})
        })
        .collect::<Vec<_>>();
    Ok(Json(json!({"tokens": tokens})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TokenRequest {
    label: String,
}

pub async fn create_ci_token(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(project): Path<String>,
    raw: Result<Bytes, BytesRejection>,
) -> ApiResult<Response> {
    let principal = auth::require_user(&state, &headers).await?;
    auth::same_origin(&state, &headers, &principal)?;
    let access = project_access(&state, &principal, &project).await?;
    owner(&access.role)?;
    let request: TokenRequest = parse_json(&body(raw)?)?;
    let label = clean_name(&request.label, "token label")?;
    let token = auth::random_token("eplyx_ci_");
    let id = auth::new_id("tok_");
    let client = state.db.get().await?;
    client
        .execute(
            "INSERT INTO api_tokens (id, token_sha256, kind, user_id, project_id, label) VALUES ($1, $2, 'ci', $3, $4, $5)",
            &[&id, &auth::digest(&token), &principal.user_id().unwrap_or_default(), &access.project_id, &label],
        )
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"id": id, "label": label, "token": token, "project_id": access.project_id,
                    "note": "Shown once. Store it as the EPLYX_TOKEN secret of trusted CI workflows; it can only sync runs to this project."})),
    )
        .into_response())
}

pub async fn revoke_ci_token(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path((project, token)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    let principal = auth::require_user(&state, &headers).await?;
    auth::same_origin(&state, &headers, &principal)?;
    let access = project_access(&state, &principal, &project).await?;
    owner(&access.role)?;
    let client = state.db.get().await?;
    let revoked = client
        .execute(
            "UPDATE api_tokens SET revoked_at = now() WHERE id = $1 AND project_id = $2 AND kind = 'ci' AND revoked_at IS NULL",
            &[&token, &access.project_id],
        )
        .await?;
    if revoked == 0 {
        return Err(ApiError::not_found("no such active token"));
    }
    Ok(Json(json!({"revoked": token})))
}

// --------------------------------------------------------------------- sync

async fn sync_principal(
    state: &Shared,
    headers: &HeaderMap,
    project: &str,
) -> ApiResult<(Principal, Access)> {
    let principal = auth::require(state, headers).await?;
    auth::same_origin(state, headers, &principal)?;
    let access = project_access(state, &principal, project).await?;
    Ok((principal, access))
}

async fn verify_blocking<T: Send + 'static>(
    job: impl FnOnce() -> anyhow::Result<T> + Send + 'static,
) -> ApiResult<T> {
    tokio::task::spawn_blocking(job)
        .await
        .map_err(ApiError::internal)?
        .map_err(|error| ApiError::invalid(format!("{error:#}")))
}

pub async fn sync_run(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(project): Path<String>,
    raw: Result<Bytes, BytesRejection>,
) -> ApiResult<Response> {
    let (principal, access) = sync_principal(&state, &headers, &project).await?;
    let document: RunDocument = parse_json(&body(raw)?)?;
    let (document, verified) = verify_blocking(move || {
        let verified = document.verify()?;
        Ok((document, verified))
    })
    .await?;
    let run_id = document.run_id.clone();
    let summary = serde_json::to_string(&verified.summary).map_err(ApiError::internal)?;
    let sizes =
        serde_json::to_string(&document.local_artifact_sizes).map_err(ApiError::internal)?;
    let run_source = verified
        .metadata
        .run_source
        .and_then(|s| serde_json::to_value(s).ok())
        .and_then(|v| v.as_str().map(str::to_owned));
    let search_text = document.search.as_ref().map(|s| s.text.clone());
    let mut client = state.db.get().await?;
    let tx = client.transaction().await?;
    let inserted = tx
        .execute(
            "INSERT INTO runs (project_id, run_id, local_project_id, core_sha256, search_sha256, run_source, run_timestamp, gate_outcome,
                               metadata_text, report_text, bindings_text, manifest_text, config_text, search_text, artifact_sizes, summary,
                               synced_by, synced_via, search_synced_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, CASE WHEN $5::TEXT IS NULL THEN NULL ELSE now() END)
             ON CONFLICT (project_id, run_id) DO NOTHING",
            &[
                &access.project_id, &run_id, &document.local_project_id, &verified.core_sha256, &verified.search_sha256,
                &run_source, &verified.metadata.timestamp, &verified.metadata.gate_outcome, &document.metadata.text,
                &document.report.text, &document.bindings.text, &document.manifest.text, &document.config.text,
                &search_text, &sizes, &summary, &principal.label(), &principal.via(),
            ],
        )
        .await?;
    let status = if inserted == 1 {
        "created"
    } else {
        let existing = tx
            .query_one(
                "SELECT core_sha256, search_sha256 FROM runs WHERE project_id = $1 AND run_id = $2 FOR UPDATE",
                &[&access.project_id, &run_id],
            )
            .await?;
        let core: String = existing.get(0);
        let search: Option<String> = existing.get(1);
        if core != verified.core_sha256 {
            return Err(ApiError::conflict(format!(
                "conflict: {run_id} is already synced to this project with different content (core digest {} ≠ {}). Synced runs are immutable; nothing was overwritten.",
                &core[..12], &verified.core_sha256[..12]
            )));
        }
        match (search, &verified.search_sha256) {
            (None, Some(_)) => {
                tx.execute(
                    "UPDATE runs SET search_sha256 = $3, search_text = $4, summary = $5, search_synced_at = now() WHERE project_id = $1 AND run_id = $2",
                    &[&access.project_id, &run_id, &verified.search_sha256, &search_text, &summary],
                )
                .await?;
                "search_attached"
            }
            (Some(saved), Some(new)) if &saved != new => {
                return Err(ApiError::conflict(format!(
                    "conflict: {run_id} already has a synced search result with a different digest. A synced search is immutable; nothing was overwritten."
                )));
            }
            _ => "unchanged",
        }
    };
    tx.execute(
        "INSERT INTO project_links (project_id, local_project_id, linked_by, linked_via) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
        &[&access.project_id, &document.local_project_id, &principal.label(), &principal.via()],
    )
    .await?;
    tx.commit().await?;
    let code = if status == "created" {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((
        code,
        Json(json!({
            "status": status,
            "run_id": run_id,
            "core_sha256": verified.core_sha256,
            "search_sha256": verified.search_sha256,
            "url": format!("{}/p/{}/runs/{run_id}", state.config.public_url, access.project_id),
        })),
    )
        .into_response())
}

/// Parse a stored run back into the engine's view model.
pub fn stored_run(row: &tokio_postgres::Row, offset: usize) -> view::Run {
    let text = |i: usize| row.get::<_, String>(offset + i);
    let search: Option<String> = row.get(offset + 5);
    let (metadata, report, manifest, config) = (text(0), text(1), text(3), text(4));
    view::from_bytes(
        &row.get::<_, String>(offset + 6),
        RunBytes {
            metadata: Some(metadata.as_bytes()),
            report: Some(report.as_bytes()),
            manifest: Some(manifest.as_bytes()),
            config: Some(config.as_bytes()),
            search: search.as_deref().map(str::as_bytes),
        },
    )
}

/// Column list matching [`stored_run`]'s offsets.
pub const RUN_COLUMNS: &str =
    "r.metadata_text, r.report_text, r.bindings_text, r.manifest_text, r.config_text, r.search_text, r.run_id";

pub async fn sync_counterexample(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(project): Path<String>,
    raw: Result<Bytes, BytesRejection>,
) -> ApiResult<Response> {
    let (principal, access) = sync_principal(&state, &headers, &project).await?;
    let document: CounterexampleDocument = parse_json(&body(raw)?)?;
    let (document, verified) = verify_blocking(move || {
        let verified = document.verify()?;
        Ok((document, verified))
    })
    .await?;
    let client = state.db.get().await?;
    let parent = client
        .query_opt(
            &format!("SELECT {RUN_COLUMNS}, r.local_project_id FROM runs r WHERE r.project_id = $1 AND r.run_id = $2"),
            &[&access.project_id, &verified.saved.parent_run],
        )
        .await?
        .ok_or_else(|| {
            ApiError::invalid(format!(
                "parent run {} is not synced to this project; sync the run first",
                verified.saved.parent_run
            ))
        })?;
    if parent.get::<_, String>(7) != document.local_project_id {
        return Err(ApiError::invalid(
            "counterexample comes from a different local project than its parent run",
        ));
    }
    let saved = verified.saved.clone();
    verify_blocking(move || contract::bind_counterexample(&saved, &stored_run(&parent, 0))).await?;
    let summary = serde_json::to_string(&verified.summary).map_err(ApiError::internal)?;
    let inserted = client
        .execute(
            "INSERT INTO counterexamples (project_id, cx_id, run_id, file_sha256, file_text, summary, synced_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (project_id, cx_id) DO NOTHING",
            &[&access.project_id, &document.counterexample_id, &verified.saved.parent_run, &document.file.sha256, &document.file.text, &summary, &principal.label()],
        )
        .await?;
    let status = if inserted == 1 {
        "created"
    } else {
        let existing: String = client
            .query_one(
                "SELECT file_sha256 FROM counterexamples WHERE project_id = $1 AND cx_id = $2",
                &[&access.project_id, &document.counterexample_id],
            )
            .await?
            .get(0);
        if existing != document.file.sha256 {
            return Err(ApiError::conflict(format!(
                "conflict: {} is already synced with different content; nothing was overwritten",
                document.counterexample_id
            )));
        }
        "unchanged"
    };
    Ok((
        if status == "created" {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(json!({"status": status, "counterexample_id": document.counterexample_id})),
    )
        .into_response())
}

pub async fn sync_reproduction(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(project): Path<String>,
    raw: Result<Bytes, BytesRejection>,
) -> ApiResult<Response> {
    let (principal, access) = sync_principal(&state, &headers, &project).await?;
    let document: ReproductionDocument = parse_json(&body(raw)?)?;
    let (document, record) = verify_blocking(move || {
        let record = document.verify()?;
        Ok((document, record))
    })
    .await?;
    let client = state.db.get().await?;
    let parent = client
        .query_opt(
            "SELECT c.file_text, r.local_project_id FROM counterexamples c JOIN runs r ON r.project_id = c.project_id AND r.run_id = c.run_id
             WHERE c.project_id = $1 AND c.cx_id = $2",
            &[&access.project_id, &document.counterexample_id],
        )
        .await?
        .ok_or_else(|| {
            ApiError::invalid(format!(
                "counterexample {} is not synced to this project; sync it first",
                document.counterexample_id
            ))
        })?;
    if parent.get::<_, String>(1) != document.local_project_id {
        return Err(ApiError::invalid(
            "reproduction comes from a different local project than its counterexample",
        ));
    }
    let saved: SavedCounterexample =
        serde_json::from_str(&parent.get::<_, String>(0)).map_err(ApiError::internal)?;
    contract::bind_reproduction(&record, &saved)
        .map_err(|e| ApiError::invalid(format!("{e:#}")))?;
    let summary = serde_json::to_string(&contract::reproduction_summary(&record))
        .map_err(ApiError::internal)?;
    let inserted = client
        .execute(
            "INSERT INTO reproductions (project_id, repro_id, cx_id, file_sha256, file_text, summary, synced_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (project_id, repro_id) DO NOTHING",
            &[&access.project_id, &document.reproduction_id, &document.counterexample_id, &document.file.sha256, &document.file.text, &summary, &principal.label()],
        )
        .await?;
    let status = if inserted == 1 {
        "created"
    } else {
        let existing: String = client
            .query_one(
                "SELECT file_sha256 FROM reproductions WHERE project_id = $1 AND repro_id = $2",
                &[&access.project_id, &document.reproduction_id],
            )
            .await?
            .get(0);
        if existing != document.file.sha256 {
            return Err(ApiError::conflict(format!(
                "conflict: {} is already synced with different content; nothing was overwritten",
                document.reproduction_id
            )));
        }
        "unchanged"
    };
    Ok((
        if status == "created" {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(json!({"status": status, "reproduction_id": document.reproduction_id})),
    )
        .into_response())
}
