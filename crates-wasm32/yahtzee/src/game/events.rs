use std::cell::RefCell;
use std::rc::Rc;
use serde::{Serialize, Deserialize};

use crate::networks::peer_network::PeerHandshake;
use crate::util::fixed_ring_buffer::FixedRingBuffer;

pub struct LobbyJoin {
    pub lobby_id: u64,
    pub user_id: u32,
    pub peers_id: Vec<u32>,
}

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

pub type WebSocketEvent = crate::networks::web_socket::WebSocketEvent<WebSocketMessage>;
pub type PeerNetworkEvent = crate::networks::peer_network::PeerNetworkEvent<PeerMessage>;
pub enum Event {
    SubmitName(String),
    LobbyJoin(LobbyJoin),
    WebSocketEvent(WebSocketEvent),
    PeerNetworkEvent(PeerNetworkEvent),
}

#[derive(Clone)]
pub struct EventSender(Rc<RefCell<FixedRingBuffer<Event, 64>>>);
impl EventSender {
    pub fn queue(&self, event: Event) {
        self.0.borrow_mut().push_back(event).unwrap();
    }
}
pub struct EventQueue(Rc<RefCell<FixedRingBuffer<Event, 64>>>);
impl EventQueue {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(FixedRingBuffer::new())))
    }
    pub fn create_sender(&self) -> EventSender {
        EventSender(self.0.clone())
    }
    pub fn deque(&self) -> Option<Event> {
        self.0.borrow_mut().pop_front().ok()
    }
}
