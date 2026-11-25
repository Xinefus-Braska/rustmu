use rustmu_sdk::command::{Command, CommandContext, CommandHandlerResult};
use rustmu_sdk::{CharacterId, ClientToServer, ServerToClient, SessionId};
use tokio::sync::mpsc;

pub mod command;
pub mod session;
pub mod world;

use command::CommandDispatcher;
use session::SessionRegistry;
use world::{InMemoryWorld, WorldFacade};

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

/// Runs the RustMU server event loop.
///
/// The server owns session resolution, command parsing, dispatch, and output
/// delivery. It delegates game rules and world state to the world facade.
pub struct GameServer {
    /// Server configuration supplied by the host application.
    config: ServerConfig,

    /// The active in-memory world.
    world: InMemoryWorld,

    /// The command dispatcher used for client input.
    dispatcher: CommandDispatcher,

    /// Active session-to-character associations.
    sessions: SessionRegistry,
}

impl GameServer {
    /// Creates a server with an empty in-memory world and default handlers.
    pub fn new(config: ServerConfig) -> Self {
        Self::with_world(config, InMemoryWorld::new())
    }

    /// Creates a server with the supplied in-memory world and default handlers.
    ///
    /// This constructor is intended for tests and host applications that need
    /// to seed an initial world before starting the server.
    pub fn with_world(config: ServerConfig, world: InMemoryWorld) -> Self {
        Self {
            config,
            world,
            dispatcher: CommandDispatcher::with_default_handlers(),
            sessions: SessionRegistry::new(),
        }
    }

    /// Runs until the client message channel closes.
    ///
    /// Connected sessions receive the configured welcome message. Input is
    /// parsed into an SDK command, dispatched through the registered handlers,
    /// and its game-facing result is returned to the requesting session.
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

                    // Temporary bootstrap behavior: attach the first seeded
                    // character to every new session. This will be replaced by
                    // explicit authentication and character selection.
                    if let Some(character) = self.world.character(CharacterId(1)) {
                        self.sessions.attach_character(session_id, character.id);
                    }

                    server_tx
                        .send(ServerToClient::Output {
                            session_id,
                            text: format!("{}\n", self.config.welcome_message),
                        })
                        .await?;
                }

                ClientToServer::Input { session_id, line } => {
                    tracing::debug!(?session_id, %line, "Input received");

                    let output = self.handle_input(session_id, &line).await;

                    server_tx
                        .send(ServerToClient::Output {
                            session_id,
                            text: format!("{output}\n"),
                        })
                        .await?;
                }

                ClientToServer::Disconnected { session_id } => {
                    tracing::info!(?session_id, "Client disconnected");

                    self.sessions.remove_session(session_id);
                }
            }
        }

        tracing::info!("Game server stopped");
        Ok(())
    }

    /// Parses and dispatches one line of client input.
    ///
    /// Returns game-facing text for the requesting session. This method does
    /// not format transport framing or deliver output directly.
    async fn handle_input(&self, session_id: SessionId, line: &str) -> String {
        let Some(character_id) = self.sessions.character_id(session_id) else {
            return String::from("You do not have an active character.");
        };

        let Some(character) = self.world.character(character_id) else {
            return String::from("Your character is not available.");
        };

        let command = parse_command(line);
        let context = CommandContext::new(character, Some(session_id));

        match self.dispatcher.dispatch(&command, &context, &self.world) {
            CommandHandlerResult::Ok(output) => output.text,
            CommandHandlerResult::Error(message) => message,
        }
    }
}

/// Parses one line of client input into a structured SDK command.
///
/// The first whitespace-separated token is the command name. Remaining
/// tokens become command arguments. Empty input produces an empty command
/// name, which the dispatcher rejects as unknown.
fn parse_command(line: &str) -> Command {
    let mut parts = line.split_whitespace();

    let name = parts.next().unwrap_or_default().to_string();
    let arguments = parts.map(String::from).collect();

    Command { name, arguments }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_command_name_and_arguments() {
        let command = parse_command("move east");

        assert_eq!(command.name, "move");
        assert_eq!(command.arguments, vec![String::from("east")]);
    }

    #[test]
    fn parses_empty_input_as_an_empty_command() {
        let command = parse_command("   ");

        assert_eq!(command.name, "");
        assert!(command.arguments.is_empty());
    }
}
