use rustmu_sdk::{ClientToServer, ServerToClient, SessionId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

const MAX_LINE_LENGTH: usize = 2_000;

type SessionRegistry = Arc<Mutex<HashMap<SessionId, mpsc::Sender<ServerToClient>>>>;

pub struct Portal {
    listener: TcpListener,
    client_tx: mpsc::Sender<ClientToServer>,
    server_rx: mpsc::Receiver<ServerToClient>,
    sessions: SessionRegistry,
    next_session_id: u64,
}

impl Portal {
    pub async fn bind(
        addr: &str,
        client_tx: mpsc::Sender<ClientToServer>,
        server_rx: mpsc::Receiver<ServerToClient>,
    ) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!(%addr, "Telnet portal listening");

        Ok(Self {
            listener,
            client_tx,
            server_rx,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            next_session_id: 1,
        })
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                accepted = self.listener.accept() => {
                    let (stream, peer) = accepted?;
                    let session_id = SessionId(self.next_session_id);
                    self.next_session_id += 1;

                    let (output_tx, output_rx) = mpsc::channel::<ServerToClient>(256);
                    self.sessions
                        .lock()
                        .await
                        .insert(session_id, output_tx.clone());

                    let client_tx = self.client_tx.clone();
                    let sessions = Arc::clone(&self.sessions);

                    tokio::spawn(async move {
                        if let Err(error) = handle_connection(
                            stream,
                            peer,
                            session_id,
                            client_tx,
                            output_rx,
                            sessions,
                        )
                        .await
                        {
                            tracing::error!(?session_id, %error, "Telnet connection failed");
                        }
                    });
                }

                Some(message) = self.server_rx.recv() => {
                    route_message(&self.sessions, message).await;
                }
            }
        }
    }

    pub fn local_addr(&self) -> anyhow::Result<std::net::SocketAddr> {
        Ok(self.listener.local_addr()?)
    }
}

async fn route_message(sessions: &SessionRegistry, message: ServerToClient) {
    let session_id = match &message {
        ServerToClient::Output { session_id, .. } | ServerToClient::Disconnect { session_id } => {
            *session_id
        }
    };

    let sender = sessions.lock().await.get(&session_id).cloned();

    match sender {
        Some(sender) => {
            if sender.send(message).await.is_err() {
                tracing::warn!(?session_id, "Session output channel closed");
            }
        }
        None => {
            tracing::warn!(?session_id, "Unknown or disconnected session");
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    peer: std::net::SocketAddr,
    session_id: SessionId,
    client_tx: mpsc::Sender<ClientToServer>,
    mut output_rx: mpsc::Receiver<ServerToClient>,
    sessions: SessionRegistry,
) -> anyhow::Result<()> {
    client_tx
        .send(ClientToServer::Connected {
            session_id,
            peer: peer.to_string(),
        })
        .await?;

    let mut input_buffer: Vec<u8> = Vec::new();
    let mut read_buf = [0u8; 1024];

    let result = loop {
        tokio::select! {
            read = stream.read(&mut read_buf) => {
                match read {
                    Ok(0) => break Ok(()),

                    Ok(count) => {
                        for byte in &read_buf[..count] {
                            match byte {
                                // Minimal Telnet handling: ignore IAC sequences.
                                // A complete implementation must parse negotiation
                                // and subnegotiation correctly.
                                0xFF => continue,

                                b'\n' => {
                                    if input_buffer.len() > MAX_LINE_LENGTH {
                                        return Err(anyhow::anyhow!("input line too long"));
                                    }

                                    let mut line = std::mem::take(&mut input_buffer);

                                    // Normalize CRLF to LF.
                                    if line.last() == Some(&b'\r') {
                                        line.pop();
                                    }

                                    let line = match String::from_utf8(line) {
                                        Ok(line) => line,
                                        Err(error) =>  return Err(error.into()),
                                    };

                                    if client_tx
                                        .send(ClientToServer::Input {
                                            session_id,
                                            line,
                                        })
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }

                                byte => input_buffer.push(*byte),
                            }
                        }
                    }

                    Err(error) => break Err(error.into()),
                }
            }

            Some(message) = output_rx.recv() => {
                match message {
                    ServerToClient::Output { text, .. } => {
                        stream.write_all(text.as_bytes()).await?;
                        stream.flush().await?;
                    }

                    ServerToClient::Disconnect { .. } => {
                        break Ok(());
                    }
                }
            }
        }
    };

    sessions.lock().await.remove(&session_id);

    let _ = client_tx
        .send(ClientToServer::Disconnected { session_id })
        .await;

    result
}
