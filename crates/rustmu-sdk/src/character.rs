//! In-world character contracts for the RustMU engine.
//!
//! This module defines the SDK vocabulary for the acting participant in a
//! command. A character may represent a player avatar, an NPC, or another
//! game-controlled actor; it intentionally does not model a connected
//! client, user account, authentication state, or network transport.

/// Identifies a character within the engine.
///
/// A character identifier is stable within the lifetime of the character
/// record it identifies. The server owns allocation, uniqueness, and
/// persistence; game code should treat identifiers as opaque values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CharacterId(pub u64);

/// The acting participant in a game command.
///
/// A `Character` represents an in-world actor, such as a player avatar or
/// NPC. It intentionally excludes account, authentication, session, and
/// transport concerns so those responsibilities remain with the server and
/// Portal layers.
///
/// This initial type contains only the identity and display information
/// needed to establish the SDK contract. Gameplay-specific state, such as
/// inventory, statistics, permissions, and combat data, should be added
/// only when a concrete engine feature requires it.
#[derive(Debug, Clone)]
pub struct Character {
    /// Stable identifier assigned and managed by the engine.
    pub id: CharacterId,

    /// Human-readable display name.
    ///
    /// The SDK does not guarantee uniqueness. The server owns naming,
    /// validation, and display policy.
    pub name: String,
}
