use rustmu_portal::Portal;
use rustmu_server::{GameServer, ServerConfig};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let (client_tx, client_rx) = mpsc::channel::<rustmu_sdk::ClientToServer>(1024);
    let (server_tx, server_rx) = mpsc::channel::<rustmu_sdk::ServerToClient>(1024);

    let portal = Portal::bind("0.0.0.0:4000", client_tx, server_rx).await?;

    let portal_task = tokio::spawn(async move {
        if let Err(error) = portal.run().await {
            tracing::error!(%error, "Portal stopped");
        }
    });

    let server = GameServer::new(ServerConfig::default());
    let server_task = tokio::spawn(async move {
        if let Err(error) = server.run(client_rx, server_tx).await {
            tracing::error!(%error, "Game server stopped");
        }
    });

    // Wait for either major component to stop.
    tokio::select! {
        result = portal_task => {
            result?;
        }
        result = server_task => {
            result?;
        }
    }

    Ok(())
}
