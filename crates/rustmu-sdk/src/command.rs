//! Command contracts for the RustMU engine.
//!
//! This module defines the initial vocabulary for a character-initiated
//! command. It separates the parsed intent submitted by a client from the
//! context required to execute it, while leaving command dispatch, world
//! mutation, persistence, and transport delivery to the server.

use crate::{Character, CharacterId, SessionId};

/// A parsed command submitted by a character.
///
/// The SDK treats a command as structured intent rather than a raw input
/// line. Portal or the server is responsible for parsing transport input;
/// game code receives an already-structured command.
///
/// This initial representation supports a command name and optional
/// arguments. More structured parsing, aliases, permissions, and command
/// metadata should be added only when a concrete feature requires them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// Primary command name, normalized by the component that parsed it.
    pub name: String,

    /// Remaining arguments supplied after the command name.
    ///
    /// The SDK does not define argument syntax. The component that parses
    /// the command owns tokenization and validation rules.
    pub arguments: Vec<String>,
}

/// Context supplied to a command handler when the server dispatches a
/// command.
///
/// A context identifies the acting [`Character`] and, when available, the
/// client session that submitted the command. It deliberately does not
/// expose sockets, database connections, Tokio channels, or unrestricted
/// access to the game world.
///
/// World access and output capabilities will be added incrementally as the
/// server implements the features that require them.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Character performing the command.
    pub character: Character,

    /// Session associated with the command, if the actor is connected
    /// through Portal.
    ///
    /// NPC-initiated commands and other non-client actors may not have a
    /// session. Game code must not assume this value is always present.
    pub session_id: Option<SessionId>,

    /// Character identifier for the actor, duplicated for convenient access.
    ///
    /// This mirrors `character.id` so handlers can use the identifier without
    /// retaining or cloning the full character record.
    pub character_id: CharacterId,
}

impl CommandContext {
    /// Creates a context for a character-initiated command.
    ///
    /// The server supplies the character record and session association;
    /// game code does not construct command contexts directly.
    pub fn new(character: Character, session_id: Option<SessionId>) -> Self {
        // Keep the actor identifier adjacent to the full character record so
        // handlers can access it without cloning the character.
        let character_id = character.id;

        Self {
            character,
            session_id,
            character_id,
        }
    }

    /// Returns the identifier of the acting character.
    pub fn character_id(&self) -> CharacterId {
        self.character_id
    }

    /// Returns the session associated with the command, if one exists.
    pub fn session_id(&self) -> Option<SessionId> {
        self.session_id
    }
}

/// Result returned by a command handler.
///
/// The initial result distinguishes successful handling from a handler-level
/// failure. The server owns how failures are logged, reported to the acting
/// character, or converted into engine-level effects.
#[derive(Debug)]
pub enum CommandHandlerResult {
    /// The handler completed without a handler-level failure.
    Ok,

    /// The handler could not complete the command.
    ///
    /// The message is intended to describe the failure in game-facing terms.
    /// It does not replace engine diagnostics or internal error handling.
    Error(String),
}

/// Extension point for game-defined command behavior.
///
/// Implement this trait to handle a command on behalf of a [`Character`].
/// The server supplies the parsed [`Command`] and [`CommandContext`]; the
/// handler expresses the outcome through [`CommandHandlerResult`].
///
/// The SDK intentionally does not define how handlers are registered,
/// scheduled, authorized, or granted access to the world. Those concerns
/// belong to the server and will be introduced through explicit contracts
/// as the engine requires them.
pub trait CommandHandler {
    /// Handles a command submitted by a character.
    ///
    /// # Parameters
    ///
    /// - `command`: The parsed command intent.
    /// - `context`: The acting character and associated session information.
    ///
    /// # Returns
    ///
    /// A [`CommandHandlerResult`] describing whether the handler completed
    /// successfully or produced a handler-level error.
    fn handle(&self, command: &Command, context: &CommandContext) -> CommandHandlerResult;
}
