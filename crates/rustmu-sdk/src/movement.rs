//! Movement requests and direction vocabulary for the RustMU engine.
//!
//! This module defines the smallest movement-facing contract needed by
//! command handlers. It treats movement as an explicit request: game code
//! describes intent, while the server owns destination validation, world
//! mutation, persistence, concurrency, and event delivery.

use crate::command::{
    Command, CommandContext, CommandHandler, CommandHandlerResult, CommandOutput,
};
use crate::world::World;

/// A direction in which a character may request movement.
///
/// The SDK defines only the conventional six directions. Game-specific
/// aliases, diagonal movement, vehicles, teleportation, and custom travel
/// semantics should be implemented by game code or introduced through a
/// separate contract when a concrete feature requires them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Movement toward the north.
    North,

    /// Movement toward the south.
    South,

    /// Movement toward the east.
    East,

    /// Movement toward the west.
    West,

    /// Movement upward.
    Up,

    /// Movement downward.
    Down,
}

impl Direction {
    /// Returns the canonical command argument for this direction.
    ///
    /// Portal or the server may accept aliases, but the SDK exposes only
    /// these canonical names.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::North => "north",
            Self::South => "south",
            Self::East => "east",
            Self::West => "west",
            Self::Up => "up",
            Self::Down => "down",
        }
    }

    /// Parses a canonical direction name.
    ///
    /// Returns `None` for aliases, abbreviations, mixed case, or unknown
    /// input. Alias handling belongs to the component that parses commands.
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "north" => Some(Self::North),
            "south" => Some(Self::South),
            "east" => Some(Self::East),
            "west" => Some(Self::West),
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            _ => None,
        }
    }
}

/// Result of a movement request submitted by a command handler.
///
/// The server owns the actual world mutation, validation beyond the SDK
/// contract, persistence, concurrency, and event delivery. This result
/// describes only whether the request could be completed and provides
/// game-facing text for the acting character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovementResult {
    /// The character moved successfully.
    ///
    /// The text describes the destination from the acting character's
    /// perspective. The server may format, augment, or deliver this text.
    Moved(String),

    /// The character could not move in the requested direction.
    ///
    /// The text explains the failure in game-facing terms. It does not
    /// replace server diagnostics or internal error handling.
    Blocked(String),
}

/// Handler for directional movement commands.
///
/// The handler accepts exactly one canonical direction argument and asks
/// the supplied [`World`] implementation to process the movement request.
/// It does not decide whether movement is possible; that policy belongs to
/// the server and game-specific rules.
#[derive(Debug, Default)]
pub struct MoveCommand;

impl MoveCommand {
    /// Creates a directional movement command handler.
    pub fn new() -> Self {
        Self
    }
}

impl CommandHandler for MoveCommand {
    fn handle(
        &self,
        command: &Command,
        context: &CommandContext,
        world: &dyn World,
    ) -> CommandHandlerResult {
        // Require exactly one direction argument. Additional syntax, such
        // as "go north" or "move north quickly", requires a separate
        // contract and should not be silently interpreted.
        let [argument] = command.arguments.as_slice() else {
            return CommandHandlerResult::Error(String::from("Usage: <direction>"));
        };

        let Some(direction) = Direction::from_str(argument) else {
            return CommandHandlerResult::Error(String::from("You cannot move in that direction."));
        };

        match world.move_character(context.character_id, direction) {
            MovementResult::Moved(text) => CommandHandlerResult::Ok(CommandOutput::new(text)),
            MovementResult::Blocked(text) => CommandHandlerResult::Error(text),
        }
    }
}
