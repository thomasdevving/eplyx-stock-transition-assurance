//! `eplyx-cloud`: the optional hosted Eplyx workspace server.
//!
//! Environment: `DATABASE_URL` (required), `PORT`, `EPLYX_CLOUD_PUBLIC_URL`
//! (or Railway's `RAILWAY_PUBLIC_DOMAIN`), `EPLYX_CLOUD_SIGNUP_CODE` and
//! `EPLYX_CLOUD_DEMO_PROJECT`. It needs no Solana RPC credential: it never
//! captures, executes or replays anything.
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = eplyx_cloud::Config::from_env()?;
    let bind = config.bind;
    let public = config.public_url.clone();
    let app = eplyx_cloud::app(config).await?;
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!(
        "eplyx-cloud {} listening on {bind} for {public}",
        eplyx_lifecycle_impact::build_info::VERSION
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
