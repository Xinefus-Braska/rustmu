//! Read- and request-oriented world contracts for the RustMU engine.
//!
//! This module defines the smallest world-facing vocabulary needed by
//! initial command handlers. It exposes read access and explicit requests
//! rather than direct mutation: game code describes intent, while the
//! server owns world state, persistence, concurrency, and side effects.

use crate::communication::{Speech, SpeechResult};
use crate::movement::{Direction, MovementResult};
use crate::{Character, CharacterId};

/// Read- and request-oriented view of the world available to a command
/// handler.
///
/// The server supplies this view when dispatching a command. It should
/// expose only the information and operations a handler legitimately needs;
/// it must not expose database connections, server internals, or unrestricted
/// mutation of game state.
///
/// The initial contract supports describing the character's current room,
/// listing the other characters present there, requesting movement, and
/// requesting room-scoped speech. Inventory, object interaction, and
/// broader write access should be added only when a concrete engine feature
/// requires them.
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

    /// Requests movement for a character in the supplied direction.
    ///
    /// This is a request, not a direct mutation API. The server determines
    /// whether the destination exists, whether the character may move, how
    /// state changes, whether events occur, and how the change is persisted.
    ///
    /// # Parameters
    ///
    /// - `character_id`: The character requesting movement.
    /// - `direction`: The canonical direction in which to move.
    ///
    /// # Returns
    ///
    /// A [`MovementResult`] describing either successful movement or a
    /// game-facing reason the movement was blocked.
    fn move_character(
        &self,
        character_id: CharacterId,
        direction: Direction,
    ) -> MovementResult;

    /// Requests room-scoped speech from a character.
    ///
    /// This is a request, not a direct delivery API. The server determines
    /// which characters can hear the speech, applies game-specific policy,
    /// emits events, and persists any resulting history.
    ///
    /// # Parameters
    ///
    /// - `speech`: The speaker and spoken text.
    ///
    /// # Returns
    ///
    /// A [`SpeechResult`] describing whether the server accepted or rejected
    /// the request.
    fn say(&self, speech: Speech) -> SpeechResult;
}
