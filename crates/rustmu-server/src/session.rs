//! Connected-session state for the RustMU server.
//!
//! This module tracks the association between an active client session and
//! the character acting through that session. It intentionally does not model
//! authentication, account ownership, network transport, or character
//! selection.

use std::collections::HashMap;

use rustmu_sdk::{CharacterId, SessionId};

/// Tracks active client sessions and their associated characters.
///
/// The server owns session-to-character assignment. A session may exist
/// before a character is selected; command dispatch requires an associated
/// character.
#[derive(Debug, Default)]
pub struct SessionRegistry {
    /// Active sessions, keyed by session identifier.
    sessions: HashMap<SessionId, CharacterId>,
}

impl SessionRegistry {
    /// Creates an empty session registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Associates a character with an active session.
    ///
    /// This replaces any character previously associated with the session.
    pub fn attach_character(&mut self, session_id: SessionId, character_id: CharacterId) {
        self.sessions.insert(session_id, character_id);
    }

    /// Removes a session and returns its associated character, if any.
    pub fn remove_session(&mut self, session_id: SessionId) -> Option<CharacterId> {
        self.sessions.remove(&session_id)
    }

    /// Returns the character associated with a session, if any.
    pub fn character_id(&self, session_id: SessionId) -> Option<CharacterId> {
        self.sessions.get(&session_id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attaches_and_resolves_a_character() {
        let mut sessions = SessionRegistry::new();
        let session_id = SessionId(1);
        let character_id = CharacterId(7);

        sessions.attach_character(session_id, character_id);

        assert_eq!(sessions.character_id(session_id), Some(character_id));
    }

    #[test]
    fn removes_a_session() {
        let mut sessions = SessionRegistry::new();
        let session_id = SessionId(2);
        let character_id = CharacterId(8);

        sessions.attach_character(session_id, character_id);
        let removed = sessions.remove_session(session_id);

        assert_eq!(removed, Some(character_id));
        assert_eq!(sessions.character_id(session_id), None);
    }
}
