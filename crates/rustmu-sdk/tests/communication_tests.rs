//! Integration tests for room-scoped speech requests.

use rustmu_sdk::{
    Character, CharacterId, Command, CommandContext, CommandHandler, CommandHandlerResult,
    Direction, MovementResult, SayCommand, Speech, SpeechResult, World,
};

/// Supplies deterministic speech behavior without a server or database.
struct TestWorld {
    /// Result returned when a valid speech request is submitted.
    result: SpeechResult,
}

impl World for TestWorld {
    fn room_description(&self, _: CharacterId) -> Option<String> {
        // Speech must not inspect the room description before submitting its
        // request.
        panic!("speech must not inspect the room description");
    }

    fn occupants(&self, _: CharacterId) -> Vec<Character> {
        // Speech must not inspect occupants before submitting its request.
        panic!("speech must not inspect occupants");
    }

    fn move_character(&self, _: CharacterId, _: Direction) -> MovementResult {
        // Speech must never request movement.
        panic!("speech must not request movement");
    }

    fn say(&self, speech: Speech) -> SpeechResult {
        // Verify that speech is requested for the acting character.
        assert_eq!(speech.speaker_id(), CharacterId(1));
        self.result.clone()
    }
}

/// Executes a speech command for an actor without a client session.
fn run_say(arguments: Vec<String>, result: SpeechResult) -> CommandHandlerResult {
    let world = TestWorld { result };
    let context = CommandContext::new(
        Character {
            id: CharacterId(1),
            name: String::from("Actor"),
        },
        None,
    );

    let command = Command {
        name: String::from("say"),
        arguments,
    };

    SayCommand::new().handle(&command, &context, &world)
}

/// Verifies that an accepted speech request produces command output.
#[test]
fn say_returns_accepted_output() {
    match run_say(
        vec![String::from("Hello"), String::from("there.")],
        SpeechResult::Accepted(String::from("You say, \"Hello there.\"")),
    ) {
        CommandHandlerResult::Ok(output) => {
            assert_eq!(output.text, "You say, \"Hello there.\"");
        }
        other => panic!("Expected output, got {other:?}"),
    }
}

/// Verifies that a rejected speech request becomes a handler error.
#[test]
fn say_returns_rejected_error() {
    match run_say(
        vec![String::from("Hello")],
        SpeechResult::Rejected(String::from("You cannot speak right now.")),
    ) {
        CommandHandlerResult::Error(message) => {
            assert_eq!(message, "You cannot speak right now.");
        }
        other => panic!("Expected an error, got {other:?}"),
    }
}

/// Verifies that speech requires at least one message argument.
#[test]
fn say_rejects_missing_message() {
    match run_say(vec![], SpeechResult::Accepted(String::from("Unused"))) {
        CommandHandlerResult::Error(message) => {
            assert_eq!(message, "Usage: say <message>");
        }
        other => panic!("Expected an error, got {other:?}"),
    }
}

/// Verifies that multiple arguments are joined into one spoken message.
#[test]
fn say_joins_multiple_arguments() {
    let world = TestWorld {
        result: SpeechResult::Accepted(String::from("You say, \"The guard is here.\"")),
    };

    let context = CommandContext::new(
        Character {
            id: CharacterId(1),
            name: String::from("Actor"),
        },
        None,
    );

    let command = Command {
        name: String::from("say"),
        arguments: vec![
            String::from("The"),
            String::from("guard"),
            String::from("is"),
            String::from("here."),
        ],
    };

    let result = SayCommand::new().handle(&command, &context, &world);

    // The world receives the complete message rather than separate tokens.
    match result {
        CommandHandlerResult::Ok(output) => {
            assert_eq!(output.text, "You say, \"The guard is here.\"");
        }
        other => panic!("Expected output, got {other:?}"),
    }
}
