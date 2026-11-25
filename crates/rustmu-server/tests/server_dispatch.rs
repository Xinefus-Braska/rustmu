//! End-to-end tests for the server's dispatcher integration.
//!
//! These tests exercise session attachment, command parsing, dispatch, world
//! mutation, and output delivery without involving Portal or a real network
//! transport.

use rustmu_sdk::{
    Character, CharacterId, ClientToServer, ServerToClient, SessionId,
};
use tokio::sync::mpsc;

use rustmu_server::{GameServer, ServerConfig};
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

#[tokio::test]
async fn dispatches_look_move_and_say() {
    let world = test_world();
    let server = GameServer::with_world(ServerConfig::default(), world);

    let (client_tx, client_rx) = mpsc::channel(16);
    let (server_tx, mut server_rx) = mpsc::channel(16);

    let server_task = tokio::spawn(async move {
        server.run(client_rx, server_tx).await
    });

    let session_id = SessionId(1);

    client_tx
        .send(ClientToServer::Connected {
            session_id,
            peer: String::from("test-client"),
        })
        .await
        .expect("connected message should be sent");

    client_tx
        .send(ClientToServer::Input {
            session_id,
            line: String::from("look"),
        })
        .await
        .expect("look input should be sent");

    client_tx
        .send(ClientToServer::Input {
            session_id,
            line: String::from("move east"),
        })
        .await
        .expect("movement input should be sent");

    client_tx
        .send(ClientToServer::Input {
            session_id,
            line: String::from("say Hello, Bob!"),
        })
        .await
        .expect("speech input should be sent");

    // Close input after all commands have been submitted.
    drop(client_tx);

    let mut outputs = Vec::new();
    while let Some(message) = server_rx.recv().await {
        if let ServerToClient::Output { text, .. } = message {
            outputs.push(text);
        }
    }

    server_task
        .await
        .expect("server task should complete")
        .expect("server should run successfully");

    assert_eq!(outputs.len(), 4);
    assert!(outputs[0].contains("Welcome to Rust MU*"));
    assert!(outputs[1].contains("Entry Hall"));
    assert!(outputs[1].contains("Bob"));
    assert_eq!(outputs[2], "You arrive in Library.\n");
    assert_eq!(outputs[3], "You say, \"Hello, Bob!\"\n");
}

#[tokio::test]
async fn rejects_input_without_an_active_character() {
    let server = GameServer::new(ServerConfig::default());

    let (client_tx, client_rx) = mpsc::channel(4);
    let (server_tx, mut server_rx) = mpsc::channel(4);

    let server_task = tokio::spawn(async move {
        server.run(client_rx, server_tx).await
    });

    let session_id = SessionId(9);

    client_tx
        .send(ClientToServer::Connected {
            session_id,
            peer: String::from("test-client"),
        })
        .await
        .expect("connected message should be sent");

    client_tx
        .send(ClientToServer::Input {
            session_id,
            line: String::from("look"),
        })
        .await
        .expect("look input should be sent");

    drop(client_tx);

    let mut outputs = Vec::new();
    while let Some(message) = server_rx.recv().await {
        if let ServerToClient::Output { text, .. } = message {
            outputs.push(text);
        }
    }

    server_task
        .await
        .expect("server task should complete")
        .expect("server should run successfully");

    assert_eq!(outputs.len(), 2);
    assert!(outputs[0].contains("Welcome to Rust MU*"));
    assert_eq!(outputs[1], "You do not have an active character.\n");
}
