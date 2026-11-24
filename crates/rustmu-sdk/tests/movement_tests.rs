//! Integration tests for directional movement requests.

use rustmu_sdk::{
    Character, CharacterId, Command, CommandContext, CommandHandler, CommandHandlerResult,
    Direction, MoveCommand, MovementResult, Speech, SpeechResult, World,
};

/// Supplies deterministic movement behavior without a server or database.
struct TestWorld {
    /// Result returned when a valid movement request is submitted.
    result: MovementResult,
}

impl World for TestWorld {
    fn room_description(&self, _: CharacterId) -> Option<String> {
        // Movement must not inspect the current room before submitting its
        // request.
        panic!("movement must not inspect the room description");
    }

    fn occupants(&self, _: CharacterId) -> Vec<Character> {
        // Movement must not inspect occupants before submitting its request.
        panic!("movement must not inspect occupants");
    }

    fn move_character(&self, character_id: CharacterId, direction: Direction) -> MovementResult {
        // Verify that movement is requested for the acting character.
        assert_eq!(character_id, CharacterId(1));
        assert_eq!(direction, Direction::North);
        self.result.clone()
    }

    fn say(&self, _: Speech) -> SpeechResult {
        // Movement must never request speech.
        panic!("movement must not request speech");
    }
}

/// Executes a movement command for an actor without a client session.
fn run_move(arguments: Vec<String>, result: MovementResult) -> CommandHandlerResult {
    let world = TestWorld { result };
    let context = CommandContext::new(
        Character {
            id: CharacterId(1),
            name: String::from("Actor"),
        },
        None,
    );

    let command = Command {
        name: String::from("north"),
        arguments,
    };

    MoveCommand::new().handle(&command, &context, &world)
}

/// Verifies that a successful movement request produces command output.
#[test]
fn move_returns_moved_output() {
    match run_move(
        vec![String::from("north")],
        MovementResult::Moved(String::from("You walk north.")),
    ) {
        CommandHandlerResult::Ok(output) => {
            assert_eq!(output.text, "You walk north.");
        }
        other => panic!("Expected output, got {other:?}"),
    }
}

/// Verifies that a blocked movement request becomes a handler error.
#[test]
fn move_returns_blocked_error() {
    match run_move(
        vec![String::from("north")],
        MovementResult::Blocked(String::from("The way north is closed.")),
    ) {
        CommandHandlerResult::Error(message) => {
            assert_eq!(message, "The way north is closed.");
        }
        other => panic!("Expected an error, got {other:?}"),
    }
}

/// Verifies that movement requires exactly one direction argument.
#[test]
fn move_rejects_missing_argument() {
    match run_move(vec![], MovementResult::Moved(String::from("Unused"))) {
        CommandHandlerResult::Error(message) => {
            assert_eq!(message, "Usage: <direction>");
        }
        other => panic!("Expected an error, got {other:?}"),
    }
}

/// Verifies that extra arguments are rejected before movement is requested.
#[test]
fn move_rejects_extra_arguments() {
    match run_move(
        vec![String::from("north"), String::from("quickly")],
        MovementResult::Moved(String::from("Unused")),
    ) {
        CommandHandlerResult::Error(message) => {
            assert_eq!(message, "Usage: <direction>");
        }
        other => panic!("Expected an error, got {other:?}"),
    }
}

/// Verifies that unrecognized direction names are rejected.
#[test]
fn move_rejects_unrecognized_direction() {
    match run_move(
        vec![String::from("sideways")],
        MovementResult::Moved(String::from("Unused")),
    ) {
        CommandHandlerResult::Error(message) => {
            assert_eq!(message, "You cannot move in that direction.");
        }
        other => panic!("Expected an error, got {other:?}"),
    }
}
