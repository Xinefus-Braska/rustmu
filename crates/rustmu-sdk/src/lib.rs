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

/// Messages sent from a connected client, through Portal, to the server.
///
/// Portal owns the network connection and is responsible for translating
/// transport-specific input into these messages. The server consumes them
/// and decides how they affect game state.
#[derive(Debug)]
pub enum ClientToServer {
    /// A client established a new session.
    ///
    /// `peer` is a transport-level description of the remote endpoint, such
    /// as an IP address and port. It is intended for diagnostics and must
    /// not be treated as a stable player identity.
    Connected {
        /// Identifier assigned by Portal for the new connection.
        session_id: SessionId,

        /// Transport-level description of the remote peer.
        peer: String,
    },

    /// A client submitted one complete line of input.
    ///
    /// Portal is responsible for line framing. The server receives the
    /// decoded text and determines whether it represents a command or
    /// another game interaction.
    Input {
        /// Session that submitted the input.
        session_id: SessionId,

        /// Decoded client input, with transport framing removed.
        line: String,
    },

    /// A client session ended.
    ///
    /// The server should treat this as notification that the connection is
    /// no longer available. It may still need to persist or clean up any
    /// character state associated with the session.
    Disconnected {
        /// Session that disconnected.
        session_id: SessionId,
    },
}

/// Messages sent from the server, through Portal, to a connected client.
///
/// These messages describe an outcome or transport action for a particular
/// session. Portal formats and delivers them using the active client
/// protocol; the SDK does not expose Telnet or other transport details.
#[derive(Debug)]
pub enum ServerToClient {
    /// Text intended for delivery to one client session.
    ///
    /// The text is already game-facing content. Portal may adapt line
    /// endings or encoding as required by its transport, but it should not
    /// reinterpret game semantics.
    Output {
        /// Session that should receive the text.
        session_id: SessionId,

        /// Game-facing text to deliver to the client.
        text: String,
    },

    /// A request for Portal to end the identified client session.
    ///
    /// This is an instruction to close the connection. Portal owns the
    /// actual disconnect procedure and any transport-specific cleanup.
    Disconnect {
        /// Session that should be disconnected.
        session_id: SessionId,
    },
}
