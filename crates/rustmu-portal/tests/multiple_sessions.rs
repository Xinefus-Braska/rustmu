use rustmu_portal::Portal;
use rustmu_sdk::{ClientToServer, ServerToClient};
use rustmu_server::{GameServer, ServerConfig};
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

    let server = GameServer::new(ServerConfig::default());
    tokio::spawn(async move {
        if let Err(error) = server.run(client_rx, server_tx).await {
            panic!("Game server failed: {error}");
        }
    });

    let mut client_a = TcpStream::connect(addr).await.unwrap();
    let mut client_b = TcpStream::connect(addr).await.unwrap();

    assert_eq!(read_line(&mut client_a).await, "Welcome to Rust MU*.");
    assert_eq!(read_line(&mut client_b).await, "Welcome to Rust MU*.");

    client_a.write_all(b"hello from A\n").await.unwrap();
    client_b.write_all(b"hello from B\n").await.unwrap();

    assert_eq!(read_line(&mut client_a).await, "You said: hello from A");
    assert_eq!(read_line(&mut client_b).await, "You said: hello from B");
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
