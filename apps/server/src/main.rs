use anyhow::{Context, Result};
use tokio::net::TcpListener;

const ADDRESS: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind(ADDRESS)
        .await
        .with_context(|| format!("failed to bind {ADDRESS}"))?;

    println!("listening on http://{ADDRESS}");

    axum::serve(listener, server::router())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    println!("server stopped");

    Ok(())
}

/// Completes when the process receives Ctrl+C, which starts a graceful shutdown.
async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => println!("shutting down"),
        Err(err) => {
            eprintln!("cannot listen for Ctrl+C, graceful shutdown disabled: {err}");
            std::future::pending::<()>().await;
        }
    }
}
