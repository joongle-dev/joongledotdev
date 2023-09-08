use serde::{Serialize, Deserialize};

use crate::network::peer_network::PeerHandshake;
use super::scene::GameScene;

#[derive(Serialize, Deserialize)]
pub enum WebSocketMessage {
    ConnectSuccess {
        lobby_id: u64,
        user_id: u32,
        peers_id: Vec<u32>,
    },
    PeerHandshake {
        source_id: u32,
        target_id: u32,
        sdp_description: String,
        ice_candidates: Vec<(String, Option<String>, Option<u16>)>,
    }
}
impl From<PeerHandshake> for WebSocketMessage {
    fn from(value: PeerHandshake) -> Self {
        Self::PeerHandshake {
            source_id: value.source_id,
            target_id: value.target_id,
            sdp_description: value.sdp_description,
            ice_candidates: value.ice_candidates,
        }
    }
}
impl From<WebSocketMessage> for PeerHandshake {
    fn from(value: WebSocketMessage) -> Self {
        if let WebSocketMessage::PeerHandshake { source_id, target_id, sdp_description, ice_candidates } = value {
            PeerHandshake {
                source_id,
                target_id,
                sdp_description,
                ice_candidates,
            }
        }
        else {
            PeerHandshake::default()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum PeerMessage {
    Ping,
    Pong(String),
}

pub type WebSocketEvent = crate::network::web_socket::WebSocketEvent<WebSocketMessage>;
pub type PeerNetworkEvent = crate::network::peer_network::PeerNetworkEvent<PeerMessage>;
pub enum GameEvent {
    ChangeGameScene(Box<dyn GameScene>),
    WebSocketEvent(WebSocketEvent),
    PeerNetworkEvent(PeerNetworkEvent),
}