//! Routes, body bounds, security headers and the embedded browser assets.
//! The project pages reuse the local dashboard's modules unchanged, pointed at
//! the hosted view API; cloud-only pages (sign-in, device approval, workspaces
//! and project settings) are small extra modules.
use crate::{api, auth, error::ApiError, views, Shared};
use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::{header, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use eplyx_lifecycle_impact::{
    cloud::contract::{is_cloud_id, MAX_COUNTEREXAMPLE_BODY, MAX_REPRODUCTION_BODY, MAX_RUN_BODY},
    dashboard::assets as dashboard_assets,
};
use serde_json::json;

macro_rules! frontend {
    ($path:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend/", $path))
    };
}

const CLOUD_INDEX: &str = frontend!("cloud/index.html");
const CLOUD_ASSETS: &[(&str, &str, &str)] = &[
    (
        "cloud.js",
        "text/javascript; charset=utf-8",
        frontend!("cloud/cloud.js"),
    ),
    (
        "cloud.css",
        "text/css; charset=utf-8",
        frontend!("cloud/cloud.css"),
    ),
    (
        "settings.js",
        "text/javascript; charset=utf-8",
        frontend!("cloud/settings.js"),
    ),
];

const SMALL_BODY: usize = 64 * 1024;

pub fn router(state: Shared) -> Router {
    let project_views = Router::new()
        .route("/project", get(views::project))
        .route("/runs", get(views::runs))
        .route("/runs/{run}", get(views::run))
        .route("/counterexamples", get(views::counterexamples))
        .route("/counterexamples/{id}", get(views::counterexample))
        .route("/counterexamples/{id}/raw", get(views::counterexample_raw))
        .route("/compare", get(views::compare));
    let demo_views = Router::new()
        .route("/project", get(views::demo_project))
        .route("/runs", get(views::demo_runs))
        .route("/runs/{run}", get(views::demo_run))
        .route("/counterexamples", get(views::demo_counterexamples))
        .route("/counterexamples/{id}", get(views::demo_counterexample))
        .route(
            "/counterexamples/{id}/raw",
            get(views::demo_counterexample_raw),
        )
        .route("/compare", get(views::demo_compare));
    let api = Router::new()
        .route("/auth/signup", post(auth::signup))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/device", post(auth::device_start))
        .route("/auth/device/lookup", get(auth::device_lookup))
        .route("/auth/device/approve", post(auth::device_approve))
        .route("/auth/device/token", post(auth::device_token))
        .route("/auth/token", delete(auth::revoke_token))
        .route("/me", get(auth::me))
        .route(
            "/workspaces",
            get(api::list_workspaces).post(api::create_workspace),
        )
        .route(
            "/workspaces/{ws}/members",
            get(api::list_members).post(api::add_member),
        )
        .route(
            "/workspaces/{ws}/members/{user}",
            delete(api::remove_member),
        )
        .route("/workspaces/{ws}/projects", post(api::create_project))
        .route("/projects/{project}", get(api::get_project))
        .route("/projects/{project}/links", post(api::link_project))
        .route(
            "/projects/{project}/ci-tokens",
            get(api::list_ci_tokens).post(api::create_ci_token),
        )
        .route(
            "/projects/{project}/ci-tokens/{token}",
            delete(api::revoke_ci_token),
        )
        .route(
            "/projects/{project}/runs",
            get(views::runs)
                .post(api::sync_run)
                .layer(DefaultBodyLimit::max(MAX_RUN_BODY)),
        )
        .route(
            "/projects/{project}/counterexamples",
            get(views::counterexamples)
                .post(api::sync_counterexample)
                .layer(DefaultBodyLimit::max(MAX_COUNTEREXAMPLE_BODY)),
        )
        .route(
            "/projects/{project}/reproductions",
            post(api::sync_reproduction).layer(DefaultBodyLimit::max(MAX_REPRODUCTION_BODY)),
        )
        .nest("/projects/{project}/view", project_views)
        .nest("/demo/view", demo_views)
        .fallback(|| async { ApiError::not_found("unknown API route") });
    Router::new()
        .route("/healthz", get(health))
        .nest("/api/v1", api)
        .route("/assets/{name}", get(asset))
        .route("/p/{project}", get(project_shell))
        .route("/p/{project}/", get(project_shell))
        .route("/p/{project}/{*rest}", get(project_shell_nested))
        .route("/demo", get(demo_shell))
        .route("/demo/{*rest}", get(demo_shell))
        .fallback(cloud_shell)
        .layer(DefaultBodyLimit::max(SMALL_BODY))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_headers,
        ))
        .with_state(state)
}

async fn health(State(state): State<Shared>) -> Response {
    let db = match state.db.get().await {
        Ok(client) => client.query_one("SELECT 1", &[]).await.is_ok(),
        Err(_) => false,
    };
    let status = if db {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(json!({"ok": db, "database": if db { "reachable" } else { "unreachable" }, "version": eplyx_lifecycle_impact::build_info::VERSION, "executes": false})),
    )
        .into_response()
}

async fn security_headers(
    State(state): State<Shared>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    let set = |headers: &mut axum::http::HeaderMap, name: &'static str, value: &'static str| {
        headers.insert(name, HeaderValue::from_static(value));
    };
    set(headers, "cache-control", "no-store");
    set(headers, "x-content-type-options", "nosniff");
    set(headers, "x-frame-options", "DENY");
    set(headers, "referrer-policy", "no-referrer");
    set(headers, "cross-origin-resource-policy", "same-origin");
    set(headers, "cross-origin-opener-policy", "same-origin");
    set(
        headers,
        "content-security-policy",
        "default-src 'self'; img-src 'self' data:; style-src 'self'; style-src-attr 'unsafe-inline'; script-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'",
    );
    if state.config.secure_cookies() {
        set(headers, "strict-transport-security", "max-age=31536000");
    }
    response
}

async fn asset(Path(name): Path<String>) -> Response {
    let found = CLOUD_ASSETS
        .iter()
        .find(|(asset, ..)| *asset == name)
        .map(|(_, kind, body)| (*kind, body.as_bytes()))
        .or_else(|| dashboard_assets::get(&name));
    match found {
        Some((kind, body)) => ([(header::CONTENT_TYPE, kind)], body).into_response(),
        None => ApiError::not_found("unknown asset").into_response(),
    }
}

/// The local dashboard shell, pointed at a hosted project's view API.
fn dashboard_shell(base: &str, api: &str, project: &str, demo: bool) -> Html<String> {
    let attributes = format!(
        r#"<html lang="en" data-mode="overview" data-base="{base}" data-api="{api}" data-cloud="1" data-project="{project}"{}>"#,
        if demo { r#" data-demo="1""# } else { "" }
    );
    Html(
        dashboard_assets::INDEX
            .replacen(r#"<html lang="en" data-mode="overview">"#, &attributes, 1)
            .replacen("Eplyx — Local dashboard", "Eplyx — Cloud workspace", 1)
            .replacen(
                r#"<link rel="stylesheet" href="/assets/dashboard.css" />"#,
                r#"<link rel="stylesheet" href="/assets/dashboard.css" /><link rel="stylesheet" href="/assets/cloud.css" />"#,
                1,
            ),
    )
}

async fn project_shell(Path(project): Path<String>) -> Response {
    if !is_cloud_id(&project, "prj_") {
        return cloud_shell().await.into_response();
    }
    dashboard_shell(
        &format!("/p/{project}"),
        &format!("/api/v1/projects/{project}/view"),
        &project,
        false,
    )
    .into_response()
}

async fn project_shell_nested(Path((project, _)): Path<(String, String)>) -> Response {
    project_shell(Path(project)).await
}

async fn demo_shell() -> Html<String> {
    dashboard_shell("/demo", "/api/v1/demo/view", "", true)
}

async fn cloud_shell() -> Html<&'static str> {
    Html(CLOUD_INDEX)
}
