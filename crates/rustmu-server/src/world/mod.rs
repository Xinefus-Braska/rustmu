//! Server-facing world contracts for RustMU.
//!
//! This module contains the server's world-facing integration boundary. It
//! separates command dispatch and session orchestration from the concrete
//! game-world implementation.

pub mod facade;
pub mod memory;

pub use facade::WorldFacade;
pub use memory::{InMemoryWorld, RoomId};
