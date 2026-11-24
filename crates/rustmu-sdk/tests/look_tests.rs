//! Integration tests for room output and look-command validation.

use rustmu_sdk::{
    Character, CharacterId, Command, CommandContext, CommandHandler, CommandHandlerResult,
    Direction, LookCommand, MovementResult, Speech, SpeechResult, World,
};

/// Supplies deterministic world data without a server or database.
struct TestWorld {
    /// Room description returned for the test actor.
    description: Option<String>,

    /// Visible characters other than the test actor.
    occupants: Vec<Character>,
}

impl World for TestWorld {
    fn room_description(&self, character_id: CharacterId) -> Option<String> {
        // Verify that look queries the acting character rather than a session.
        assert_eq!(character_id, CharacterId(1));
        self.description.clone()
    }

    fn occupants(&self, character_id: CharacterId) -> Vec<Character> {
        assert_eq!(character_id, CharacterId(1));

        // Occupants must not be queried when no room exists.
        assert!(self.description.is_some());
        self.occupants.clone()
    }

    fn move_character(
        &self,
        _: CharacterId,
        _: Direction,
    ) -> MovementResult {
        // Look must never request movement.
        panic!("look must not request movement");
    }

    fn say(&self, _: Speech) -> SpeechResult {
        // Look must never request speech.
        panic!("look must not request speech");
    }
}

/// Executes look for an actor without a client session.
fn run_look(world: &dyn World, arguments: Vec<String>) -> CommandHandlerResult {
    let context = CommandContext::new(
        Character {
            id: CharacterId(1),
            name: String::from("Actor"),
        },
        None,
    );

    let command = Command {
        name: String::from("look"),
        arguments,
    };

    LookCommand::new().handle(&command, &context, world)
}

/// Verifies exact room output when no other characters are present.
#[test]
fn look_returns_description_without_occupants() {
    let world = TestWorld {
        description: Some(String::from("A quiet room.")),
        occupants: vec![],
    };

    match run_look(&world, vec![]) {
        CommandHandlerResult::Ok(output) => {
            assert_eq!(output.text, "A quiet room.");
        }
        other => panic!("Expected output, got {other:?}"),
    }
}

/// Verifies that occupant names appear in world-provided order.
#[test]
fn look_lists_occupants() {
    let world = TestWorld {
        description: Some(String::from("A busy room.")),
        occupants: vec![
            Character {
                id: CharacterId(2),
                name: String::from("Guard"),
            },
            Character {
                id: CharacterId(3),
                name: String::from("Merchant"),
            },
        ],
    };

    match run_look(&world, vec![]) {
        CommandHandlerResult::Ok(output) => {
            assert_eq!(
                output.text,
                "A busy room.\n\nYou see:\n- Guard\n- Merchant"
            );
        }
        other => panic!("Expected output, got {other:?}"),
    }
}

/// Verifies a game-facing error when the actor has no room.
#[test]
fn look_rejects_missing_room() {
    let world = TestWorld {
        description: None,
        occupants: vec![],
    };

    match run_look(&world, vec![]) {
        CommandHandlerResult::Error(message) => {
            assert_eq!(message, "You are not in a room.");
        }
        other => panic!("Expected an error, got {other:?}"),
    }
}

/// Fails immediately if validation unexpectedly accesses world state.
struct UnqueriedWorld;

impl World for UnqueriedWorld {
    fn room_description(&self, _: CharacterId) -> Option<String> {
        panic!("Invalid arguments must be rejected before querying the world");
    }

    fn occupants(&self, _: CharacterId) -> Vec<Character> {
        panic!("Invalid arguments must be rejected before querying occupants");
    }

    fn move_character(
        &self,
        _: CharacterId,
        _: Direction,
    ) -> MovementResult {
        panic!("Invalid arguments must be rejected before requesting movement");
    }

    fn say(&self, _: Speech) -> SpeechResult {
        panic!("Invalid arguments must be rejected before requesting speech");
    }
}

/// Verifies unsupported arguments are rejected before any world queries.
#[test]
fn look_rejects_arguments_before_world_access() {
    match run_look(&UnqueriedWorld, vec![String::from("guard")]) {
        CommandHandlerResult::Error(message) => {
            assert_eq!(
                message,
                "The look command does not accept arguments yet."
            );
        }
        other => panic!("Expected an error, got {other:?}"),
    }
}
