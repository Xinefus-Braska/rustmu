use rustmu_sdk::{ClientToServer, ServerToClient};
use tokio::sync::mpsc;

pub mod world;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub welcome_message: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            welcome_message: "Welcome to Rust MU*.".to_string(),
        }
    }
}

pub struct GameServer {
    config: ServerConfig,
}

impl GameServer {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    pub async fn run(
        &self,
        mut client_rx: mpsc::Receiver<ClientToServer>,
        server_tx: mpsc::Sender<ServerToClient>,
    ) -> anyhow::Result<()> {
        tracing::info!("Game server started");

        while let Some(event) = client_rx.recv().await {
            match event {
                ClientToServer::Connected { session_id, peer } => {
                    tracing::info!(?session_id, %peer, "Client connected");

                    server_tx
                        .send(ServerToClient::Output {
                            session_id,
                            text: format!("{}\n", self.config.welcome_message),
                        })
                        .await?;
                }

                ClientToServer::Input { session_id, line } => {
                    tracing::debug!(?session_id, %line, "Input received");

                    // Temporary echo behavior.
                    // This will become the command dispatcher later.
                    server_tx
                        .send(ServerToClient::Output {
                            session_id,
                            text: format!("You said: {line}\n"),
                        })
                        .await?;
                }

                ClientToServer::Disconnected { session_id } => {
                    tracing::info!(?session_id, "Client disconnected");
                }
            }
        }

        tracing::info!("Game server stopped");
        Ok(())
    }
}
