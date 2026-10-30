#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub u64);

#[derive(Debug)]
pub enum ClientToServer {
    Connected { session_id: SessionId, peer: String },
    Input { session_id: SessionId, line: String },
    Disconnected { session_id: SessionId },
}

#[derive(Debug)]
pub enum ServerToClient {
    Output { session_id: SessionId, text: String },
    Disconnect { session_id: SessionId },
}
