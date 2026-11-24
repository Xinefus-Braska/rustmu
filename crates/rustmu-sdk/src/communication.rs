//! Room-scoped communication contracts for the RustMU engine.
//!
//! This module defines the smallest speech-facing vocabulary needed by
//! command handlers. It treats speech as an explicit request: game code
//! describes the speaker and message, while the server owns audience
//! selection, visibility, policy, event delivery, persistence, and
//! formatting.

use crate::CharacterId;
use crate::command::{
    Command, CommandContext, CommandHandler, CommandHandlerResult, CommandOutput,
};
use crate::world::World;

/// A message spoken by a character within a room-scoped audience.
///
/// The SDK treats speech as a request rather than a completed delivery.
/// The server determines the audience, visibility, language handling,
/// moderation, event routing, and persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Speech {
    /// Character requesting to speak.
    pub speaker_id: CharacterId,

    /// Game-facing text supplied by the speaker.
    ///
    /// The server may normalize, reject, transform, or format this text
    /// according to game policy.
    pub text: String,
}

impl Speech {
    /// Creates a speech request for the supplied speaker and text.
    ///
    /// # Parameters
    ///
    /// - `speaker_id`: The character requesting to speak.
    /// - `text`: The game-facing text supplied by the speaker.
    pub fn new(speaker_id: CharacterId, text: impl Into<String>) -> Self {
        Self {
            speaker_id,
            text: text.into(),
        }
    }

    /// Returns the identifier of the character requesting to speak.
    pub fn speaker_id(&self) -> CharacterId {
        self.speaker_id
    }

    /// Returns the game-facing text supplied by the speaker.
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Result of a room-scoped speech request.
///
/// `Accepted` does not guarantee that every potential listener received the
/// message. It means the server accepted the request and owns delivery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeechResult {
    /// The server accepted the speech request.
    ///
    /// The text is the speaker-facing confirmation. It does not represent
    /// messages delivered to other characters.
    Accepted(String),

    /// The server rejected the speech request.
    ///
    /// The text explains the failure in game-facing terms. It does not
    /// replace server diagnostics or internal error handling.
    Rejected(String),
}

/// Handler for room-scoped speech.
///
/// The handler collects the remaining command arguments into one message
/// and submits it to the supplied [`World`] implementation. It does not
/// determine the audience, visibility rules, language policy, or delivery
/// behavior.
#[derive(Debug, Default)]
pub struct SayCommand;

impl SayCommand {
    /// Creates a room-scoped speech command handler.
    pub fn new() -> Self {
        Self
    }
}

impl CommandHandler for SayCommand {
    fn handle(
        &self,
        command: &Command,
        context: &CommandContext,
        world: &dyn World,
    ) -> CommandHandlerResult {
        // Preserve the words supplied after the command name. Argument
        // normalization and profanity, language, or formatting policy
        // belong to the server or game-specific rules.
        if command.arguments.is_empty() {
            return CommandHandlerResult::Error(String::from("Usage: say <message>"));
        }

        let text = command.arguments.join(" ");

        match world.say(Speech::new(context.character_id, text)) {
            SpeechResult::Accepted(text) => CommandHandlerResult::Ok(CommandOutput::new(text)),
            SpeechResult::Rejected(text) => CommandHandlerResult::Error(text),
        }
    }
}
