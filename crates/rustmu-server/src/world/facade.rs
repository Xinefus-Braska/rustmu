//! Server-facing world contract for RustMU command dispatch.
//!
//! This module defines the world operations required by the initial
//! `rustmu-server` command dispatcher. It extends the SDK's read- and
//! request-oriented [`World`] contract rather than replacing it, preserving
//! the boundary between SDK command contracts, server orchestration, and
//! game-world state.

use rustmu_sdk::communication::{Speech, SpeechResult};
use rustmu_sdk::movement::{Direction, MovementResult};
use rustmu_sdk::world::World;
use rustmu_sdk::{Character, CharacterId};

/// Server-facing view of the game world available to command dispatch.
///
/// `WorldFacade` preserves the SDK's request-oriented world contract while
/// giving the server a stable integration point for world implementations.
/// A facade implementation owns the concrete state, concurrency strategy,
/// validation policy, event behavior, and persistence strategy.
///
/// The initial contract supports room description, occupant lookup,
/// movement requests, and room-scoped speech through [`World`]. It adds
/// character resolution for server orchestration. It intentionally does not
/// expose direct mutation, database connections, transport channels, or
/// server internals.
pub trait WorldFacade: World {
    /// Returns the character associated with the supplied identifier.
    ///
    /// Returns `None` when no character with that identifier exists.
    ///
    /// # Parameters
    ///
    /// - `character_id`: Identifier of the character to resolve.
    ///
    /// # Returns
    ///
    /// The character record, if present.
    fn character(&self, character_id: CharacterId) -> Option<Character>;
}
