use rustmu_portal::Portal;
use rustmu_sdk::{Character, CharacterId, ClientToServer, ServerToClient};
use rustmu_server::world::{InMemoryWorld, RoomId};
use rustmu_server::{GameServer, ServerConfig};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_telnet_sessions_receive_only_their_own_output() {
    let (client_tx, client_rx) = mpsc::channel::<ClientToServer>(1024);
    let (server_tx, server_rx) = mpsc::channel::<ServerToClient>(1024);

    let portal = Portal::bind("127.0.0.1:0", client_tx, server_rx)
        .await
        .unwrap();

    let addr = portal.local_addr().unwrap();

    tokio::spawn(async move {
        if let Err(error) = portal.run().await {
            panic!("Portal failed: {error}");
        }
    });

    let server = GameServer::with_world(ServerConfig::default(), test_world());
    tokio::spawn(async move {
        if let Err(error) = server.run(client_rx, server_tx).await {
            panic!("Game server failed: {error}");
        }
    });

    let mut client_a = TcpStream::connect(addr).await.unwrap();
    let mut client_b = TcpStream::connect(addr).await.unwrap();

    assert_eq!(read_line(&mut client_a).await, "Welcome to Rust MU*.");
    assert_eq!(read_line(&mut client_b).await, "Welcome to Rust MU*.");

    // The portal and game server run in separate tasks. Waiting for both
    // welcome messages confirms that `Connected` messages reached the server,
    // but does not guarantee that the server has finished attaching both
    // sessions to characters. Yield briefly so both `Connected` events are
    // processed before sending input.
    wait_for_session_attachment().await;

    // `say` is the currently registered speech command. Each session should
    // receive only the response for the message it submitted.
    client_a.write_all(b"say hello from A\n").await.unwrap();
    client_b.write_all(b"say hello from B\n").await.unwrap();

    assert_eq!(read_line(&mut client_a).await, "You say, \"hello from A\"");
    assert_eq!(read_line(&mut client_b).await, "You say, \"hello from B\"");
}

/// Creates a world with two characters in the Entry Hall.
///
/// The server's temporary bootstrap behavior attaches `CharacterId(1)` to
/// every connected session. Seeding Alice and Bob gives the portal test a
/// valid world in which both sessions have an active character.
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

/// Allows the server task time to process both connection events.
///
/// This is intentionally a test-only synchronization aid. A production
/// client should not depend on timing; session readiness should eventually
/// be represented by an explicit protocol response.
async fn wait_for_session_attachment() {
    for _ in 0..100 {
        tokio::time::sleep(Duration::from_millis(10)).await;
        tokio::task::yield_now().await;
    }
}

async fn read_line(stream: &mut TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        let count = stream.read(&mut byte).await.unwrap();
        assert!(
            count > 0,
            "connection closed before a complete line arrived"
        );

        if byte[0] == b'\n' {
            if buffer.last() == Some(&b'\r') {
                buffer.pop();
            }

            return String::from_utf8(buffer).unwrap();
        }

        buffer.push(byte[0]);
    }
}
