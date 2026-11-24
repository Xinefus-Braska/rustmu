//! Read-oriented world contracts for the RustMU engine.
//!
//! This module defines the smallest world-facing vocabulary needed by
//! initial command handlers. It intentionally exposes read access rather
//! than direct mutation: game code describes intent, while the server owns
//! world state, persistence, concurrency, and side effects.

use crate::{Character, CharacterId};

/// Read-oriented view of the world available to a command handler.
///
/// The server supplies this view when dispatching a command. It should
/// expose only the information and operations a handler legitimately needs;
/// it must not expose database connections, server internals, or unrestricted
/// mutation of game state.
///
/// The initial contract supports describing the character's current room and
/// listing the other characters present there. Movement, inventory, object
/// interaction, and write access should be added only when a concrete engine
/// feature requires them.
pub trait World {
    /// Returns a human-readable description of the room occupied by the
    /// character.
    ///
    /// Returns `None` when the character is not currently associated with a
    /// room. The server determines whether that condition is valid for the
    /// active game state.
    fn room_description(&self, character_id: CharacterId) -> Option<String>;

    /// Returns the characters present in the same room as the character,
    /// excluding the acting character.
    ///
    /// The server owns visibility rules. A future permissions or stealth
    /// system may change which characters appear in this result without
    /// changing the method's purpose.
    fn occupants(&self, character_id: CharacterId) -> Vec<Character>;
}
