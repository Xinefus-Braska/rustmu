//! Game-facing contracts for the RustMU engine.
//!
//! This crate defines the stable boundary between game-specific code and
//! the evolving engine implementation. It intentionally excludes database
//! access, network transport details, and server internals so that game
//! code can depend on a small, durable API.
//!
//! The initial contract focuses on connected-session messages. Portal
//! translates client transport input into [`ClientToServer`] messages, while
//! the server produces [`ServerToClient`] messages for Portal to deliver.

pub mod session;

pub use session::{ClientToServer, ServerToClient};

/// Identifies a connected client session.
///
/// A session represents an active connection managed by Portal. It is not
/// necessarily equivalent to a player account or an in-world character;
/// authentication and character selection remain separate concerns.
///
/// The numeric representation is public so Portal and the server can pass
/// session identifiers without depending on engine internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub u64);
