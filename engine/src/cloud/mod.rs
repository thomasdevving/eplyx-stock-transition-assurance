//! Milestone 18: optional cloud sync. Everything here is additive and runs only
//! from `eplyx login`, `logout`, `link` and `sync`. Preflight, search,
//! reproduce, replay, the local dashboard and the CI gate never depend on it,
//! and a synced result never becomes new evidence.
pub mod client;
pub mod commands;
pub mod contract;
pub mod credentials;
pub mod local;
pub mod privacy;

/// Environment variables the cloud commands read. Analysis commands remove
/// the token from their own environment before doing anything else.
pub const TOKEN_ENV: &str = "EPLYX_TOKEN";
pub const PROJECT_ENV: &str = "EPLYX_PROJECT_ID";
pub const SERVER_ENV: &str = "EPLYX_CLOUD_URL";
