//! Integration tests for the SDK `look` command against `InMemoryWorld`.
//!
//! These tests verify that the server's in-memory world supplies the room
//! description and occupants required by the SDK's existing look handler.
//! The server does not duplicate look behavior; it consumes the SDK command
//! contract.

use rustmu_sdk::{
    Character, CharacterId, Command, CommandContext, CommandHandlerResult, LookCommand,
};

use rustmu_server::world::{InMemoryWorld, RoomId};

/// Creates a world with two connected rooms and two characters.
fn test_world() -> InMemoryWorld {
    let mut world = InMemoryWorld::new();

    world.insert_character(Character {
        id: CharacterId(1),
        name: String::from("Alice"),
    });

    world.insert_character(Character {
        id: CharacterId(2),
        name: String::from("Bob"),
    });

    world.insert_room(
        RoomId(1),
        "Entry Hall",
        "A small hall with a doorway to the east.",
    );

    world.insert_room(
        RoomId(2),
        "Library",
        "Shelves line the walls of a quiet library.",
    );

    world.connect_rooms(RoomId(1), rustmu_sdk::Direction::East, RoomId(2));
    world.connect_rooms(RoomId(2), rustmu_sdk::Direction::West, RoomId(1));

    world.place_character(CharacterId(1), RoomId(1));
    world.place_character(CharacterId(2), RoomId(1));

    world
}

#[test]
fn look_returns_room_description_and_other_occupants() {
    let world = test_world();
    let character = world
        .character(CharacterId(1))
        .expect("Alice should be registered");
    let context = CommandContext::new(character, None);
    let command = Command {
        name: String::from("look"),
        arguments: Vec::new(),
    };

    let result = LookCommand::new().handle(&command, &context, &world);

    let CommandHandlerResult::Ok(output) = result else {
        panic!("look should succeed for a character in a room");
    };

    assert!(output.text.contains("Entry Hall"));
    assert!(output.text.contains("A small hall with a doorway to the east."));
    assert!(output.text.contains("Exits: east"));
    assert!(output.text.contains("You see:"));
    assert!(output.text.contains("Bob"));
}

#[test]
fn look_rejects_arguments() {
    let world = test_world();
    let character = world
        .character(CharacterId(1))
        .expect("Alice should be registered");
    let context = CommandContext::new(character, None);
    let command = Command {
        name: String::from("look"),
        arguments: vec![String::from("north")],
    };

    let result = LookCommand::new().handle(&command, &context, &world);

    assert!(matches!(result, CommandHandlerResult::Error(_)));
}

#[test]
fn look_fails_for_a_character_without_a_room() {
    let mut world = InMemoryWorld::new();

    world.insert_character(Character {
        id: CharacterId(3),
        name: String::from("Carol"),
    });

    let character = world
        .character(CharacterId(3))
        .expect("Carol should be registered");
    let context = CommandContext::new(character, None);
    let command = Command {
        name: String::from("look"),
        arguments: Vec::new(),
    };

    let result = LookCommand::new().handle(&command, &context, &world);

    let CommandHandlerResult::Error(message) = result else {
        panic!("look should fail for a character without a room");
    };

    assert_eq!(message, "You are not in a room.");
}
