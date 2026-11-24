//! Command contracts for the RustMU engine.
//!
//! This module defines the initial vocabulary for a character-initiated
//! command. It separates the parsed intent submitted by a client from the
//! context required to execute it, while leaving command dispatch, world
//! mutation, persistence, and transport delivery to the server.

use crate::world::World;
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
/// World access is supplied separately to [`CommandHandler::handle`] so the
/// context remains focused on actor and session identity.
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

/// Output produced by a successfully handled command.
///
/// The server owns delivery, formatting adjustments, and persistence. This
/// initial type carries game-facing text intended for the acting character's
/// session; broader audience and event semantics can be added later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    /// Game-facing text intended for delivery to the acting character.
    pub text: String,
}

impl CommandOutput {
    /// Creates output containing the supplied game-facing text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// Result returned by a command handler.
///
/// A successful handler returns [`CommandOutput`]. The server owns how that
/// output is delivered, whether it is persisted, and how failures are logged
/// or reported.
#[derive(Debug)]
pub enum CommandHandlerResult {
    /// The handler completed and produced output for the acting character.
    Ok(CommandOutput),

    /// The handler could not complete the command.
    ///
    /// The message is intended to describe the failure in game-facing terms.
    /// It does not replace engine diagnostics or internal error handling.
    Error(String),
}

/// Extension point for game-defined command behavior.
///
/// Implement this trait to handle a command on behalf of a [`Character`].
/// The server supplies the parsed [`Command`], [`CommandContext`], and a
/// read-oriented [`World`] implementation; the handler expresses its outcome
/// through [`CommandHandlerResult`].
///
/// The SDK intentionally does not define how handlers are registered,
/// scheduled, authorized, or granted write access to the world. Those
/// concerns belong to the server and will be introduced through explicit
/// contracts as the engine requires them.
pub trait CommandHandler {
    /// Handles a command submitted by a character.
    ///
    /// # Parameters
    ///
    /// - `command`: The parsed command intent.
    /// - `context`: The acting character and associated session information.
    /// - `world`: Read-oriented access to the game state relevant to the
    ///   command.
    ///
    /// # Returns
    ///
    /// A [`CommandHandlerResult`] describing either the output produced by
    /// the command or a handler-level error.
    fn handle(
        &self,
        command: &Command,
        context: &CommandContext,
        world: &dyn World,
    ) -> CommandHandlerResult;
}

/// Result returned when registering a command handler.
///
/// The server owns conflict detection and error reporting. This initial
/// result gives game code a simple success/failure boundary without exposing
/// the server's registry implementation.
#[derive(Debug)]
pub enum CommandRegistrationResult {
    /// The handler was accepted for the supplied command name.
    Ok,

    /// The server rejected the registration.
    ///
    /// Common causes include an invalid command name or a name that is
    /// already registered. The server determines the precise policy.
    Error(String),
}

/// Registration contract for game-defined command handlers.
///
/// Implementations associate a command name with a [`CommandHandler`]. The
/// server supplies the registry during game initialization; game code should
/// not construct or retain a registry beyond the registration phase.
///
/// The SDK does not define registry storage, lookup order, alias handling,
/// replacement rules, or authorization. Those behaviors belong to the server
/// and should be introduced only when a concrete feature requires them.
pub trait CommandRegistry {
    /// Registers a handler for a command name.
    ///
    /// # Parameters
    ///
    /// - `name`: The command name that will invoke the handler.
    /// - `handler`: The handler responsible for processing the command.
    ///
    /// # Returns
    ///
    /// A [`CommandRegistrationResult`] describing whether the server accepted
    /// the registration.
    fn register(
        &mut self,
        name: &str,
        handler: Box<dyn CommandHandler>,
    ) -> CommandRegistrationResult;
}

/// Handler for the `look` command.
///
/// The handler reads the acting character's current room description and
/// lists the other characters present. It does not mutate world state; the
/// server remains responsible for formatting, delivering, and persisting any
/// resulting output.
#[derive(Debug, Default)]
pub struct LookCommand;

impl LookCommand {
    /// Creates a `look` command handler.
    pub fn new() -> Self {
        Self
    }
}

impl CommandHandler for LookCommand {
    fn handle(
        &self,
        command: &Command,
        context: &CommandContext,
        world: &dyn World,
    ) -> CommandHandlerResult {
        // The initial look implementation accepts no arguments. Additional
        // syntax, such as "look <target>", requires a separate contract and
        // should not be silently ignored.
        if !command.arguments.is_empty() {
            return CommandHandlerResult::Error(String::from(
                "The look command does not accept arguments yet.",
            ));
        }

        let Some(description) = world.room_description(context.character_id) else {
            return CommandHandlerResult::Error(String::from(
                "You are not in a room.",
            ));
        };

        let occupants = world.occupants(context.character_id);

        // Build the room view as game-facing text. Keep formatting simple;
        // richer presentation belongs to a future presentation or output
        // formatting contract.
        let mut text = description;

        if !occupants.is_empty() {
            text.push_str("\n\nYou see:");

            for occupant in occupants {
                text.push_str("\n- ");
                text.push_str(&occupant.name);
            }
        }

        CommandHandlerResult::Ok(CommandOutput::new(text))
    }
}
