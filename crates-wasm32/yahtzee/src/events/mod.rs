use crate::networks::peernetwork::PeerMessage;
use crate::networks::websocket::WebSocketMessage;

pub enum Event {
    JoinLobby(String),
    WebSocketMessage(WebSocketMessage),
    PeerMessage(PeerMessage),
}