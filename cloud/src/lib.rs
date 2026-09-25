//! Milestone 18: the optional hosted Eplyx workspace. It stores synced run
//! metadata, counterexamples and reproduction records, and renders them with
//! the engine's own dashboard view code. It never executes a candidate, never
//! calls an RPC provider and never replays: every page shows a synced result.
pub mod api;
pub mod auth;
pub mod db;
pub mod error;
pub mod views;
pub mod web;

use anyhow::{ensure, Context, Result};
use std::{net::SocketAddr, sync::Arc};

pub struct Config {
    pub database_url: String,
    /// Public origin, for example `https://eplyx-cloud.up.railway.app`.
    pub public_url: String,
    pub bind: SocketAddr,
    /// When set, creating an account requires this code.
    pub signup_code: Option<String>,
    /// The single project published read-only at `/demo`, if any.
    pub demo_project: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let var = |name: &str| std::env::var(name).ok().filter(|v| !v.trim().is_empty());
        let port: u16 = var("PORT").unwrap_or_else(|| "4300".into()).parse()?;
        let public_url = var("EPLYX_CLOUD_PUBLIC_URL")
            .or_else(|| var("RAILWAY_PUBLIC_DOMAIN").map(|d| format!("https://{d}")))
            .unwrap_or_else(|| format!("http://127.0.0.1:{port}"));
        let demo_project = var("EPLYX_CLOUD_DEMO_PROJECT");
        if let Some(id) = &demo_project {
            ensure!(
                eplyx_lifecycle_impact::cloud::contract::is_cloud_id(id, "prj_"),
                "EPLYX_CLOUD_DEMO_PROJECT must be a prj_ ID"
            );
        }
        Ok(Self {
            database_url: var("DATABASE_URL").context("set DATABASE_URL")?,
            public_url: eplyx_lifecycle_impact::cloud::credentials::normalize_server(&public_url)
                .context("EPLYX_CLOUD_PUBLIC_URL must be an https origin")?,
            bind: SocketAddr::from(([0, 0, 0, 0], port)),
            signup_code: var("EPLYX_CLOUD_SIGNUP_CODE"),
            demo_project,
        })
    }

    pub fn secure_cookies(&self) -> bool {
        self.public_url.starts_with("https://")
    }
}

pub struct AppState {
    pub db: deadpool_postgres::Pool,
    pub config: Config,
    pub limiter: auth::Limiter,
}

pub type Shared = Arc<AppState>;

/// Connect, apply migrations and build the router.
pub async fn app(config: Config) -> Result<axum::Router> {
    let db = db::connect(&config.database_url)?;
    db::migrate(&db).await?;
    let state = Arc::new(AppState {
        db,
        config,
        limiter: auth::Limiter::default(),
    });
    Ok(web::router(state))
}
