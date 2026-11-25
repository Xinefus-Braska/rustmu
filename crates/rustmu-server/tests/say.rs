//! Integration tests for the SDK `say` command against `InMemoryWorld`.
//!
//! These tests verify that the server's in-memory world accepts room-scoped
//! speech requests and rejects speech from a character without a room. The
//! server consumes the SDK's existing speech command contract rather than
//! duplicating its behavior.

use rustmu_sdk::{
    Character, CharacterId, Command, CommandContext, CommandHandlerResult, SayCommand,
};

use rustmu_server::world::{InMemoryWorld, RoomId};

/// Creates a world with one room and two characters.
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

    world.place_character(CharacterId(1), RoomId(1));
    world.place_character(CharacterId(2), RoomId(1));

    world
}

#[test]
fn say_accepts_a_room_scoped_message() {
    let world = test_world();
    let character = world
        .character(CharacterId(1))
        .expect("Alice should be registered");
    let context = CommandContext::new(character, None);
    let command = Command {
        name: String::from("say"),
        arguments: vec![String::from("Hello,"), String::from("Bob!")],
    };

    let result = SayCommand::new().handle(&command, &context, &world);

    let CommandHandlerResult::Ok(output) = result else {
        panic!("say should succeed for a character in a room");
    };

    assert_eq!(output.text, "You say, \"Hello, Bob!\"");
}

#[test]
fn say_requires_a_message() {
    let world = test_world();
    let character = world
        .character(CharacterId(1))
        .expect("Alice should be registered");
    let context = CommandContext::new(character, None);
    let command = Command {
        name: String::from("say"),
        arguments: Vec::new(),
    };

    let result = SayCommand::new().handle(&command, &context, &world);

    let CommandHandlerResult::Error(message) = result else {
        panic!("say should require a message");
    };

    assert_eq!(message, "Usage: say <message>");
}

#[test]
fn say_rejects_a_character_without_a_room() {
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
        name: String::from("say"),
        arguments: vec![String::from("Hello.")],
    };

    let result = SayCommand::new().handle(&command, &context, &world);

    let CommandHandlerResult::Error(message) = result else {
        panic!("say should reject a character without a room");
    };

    assert_eq!(message, "You are not in a room.");
}
