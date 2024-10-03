use serde::{Serialize, Deserialize};

pub type RoomID = u64;
pub type PeerID = u16;

// Client to server message
#[derive(Serialize, Deserialize)]
pub enum PeerRequest {
    KeepAlive,
    Signal(PeerID, String),
}

// Server to client message
#[derive(Serialize, Deserialize)]
pub enum PeerEvent {
    IdAssigned(PeerID),
    PeerConnect(PeerID),
    PeerDisconnect(PeerID),
    Signal(PeerID, String),
}