//! Command dispatch contracts for the RustMU server.
//!
//! This module routes structured SDK commands to command handlers. It
//! separates command-name routing and handler registration from world state
//! and game rules.

pub mod dispatcher;

pub use dispatcher::CommandDispatcher;
