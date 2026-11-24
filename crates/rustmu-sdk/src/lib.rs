//! Game-facing contracts for the RustMU engine.
//!
//! This crate defines the stable boundary between game-specific code and
//! the evolving engine implementation. It intentionally excludes database
//! access, network transport details, and server internals so that game
//! code can depend on a small, durable API.
//!
//! The initial contract focuses on connected-session messages, the
//! character that acts within a command, command handling and registration,
//! and read-oriented world access. Portal translates client transport input
//! into [`ClientToServer`] messages, while the server produces
//! [`ServerToClient`] messages for Portal to deliver.

pub mod character;
pub mod command;
pub mod session;
pub mod world;

pub use character::{Character, CharacterId};
pub use command::{
    Command, CommandContext, CommandHandler, CommandHandlerResult, CommandRegistry,
    CommandRegistrationResult, LookCommand,
};
pub use session::{ClientToServer, ServerToClient, SessionId};
pub use world::World;
