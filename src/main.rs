use anyhow::Context;
use gitbroadcast::{config, server};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,gitbroadcast=debug")),
        )
        .init();

    let config_path = std::env::args()
        .skip(1)
        .fold(None::<PathBuf>, |acc, arg| {
            if arg == "--config" {
                acc
            } else if acc.is_none() && arg.starts_with("--config=") {
                Some(PathBuf::from(&arg[9..]))
            } else if acc.is_none() {
                Some(PathBuf::from(arg))
            } else {
                acc
            }
        })
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    let cfg = config::load(&config_path)
        .with_context(|| format!("loading config from {}", config_path.display()))?;

    let state = server::AppState::from_config(cfg)?;
    let bind = state.bind_addr;
    let router = server::router(state);

    tracing::info!(%bind, "starting gitbroadcast");
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .with_context(|| format!("binding to {bind}"))?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut s) = signal(SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("shutdown signal received");
}
