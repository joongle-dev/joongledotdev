use wasm_bindgen::prelude::*;
use crate::events::Event;
use crate::platform::EventHandlerProxy;
use crate::networks::{websocket::{WebSocket, WebSocketMessage}, peernetwork::PeerNetwork};
use crate::networks::peernetwork::{PeerHandshake, PeerMessage};
use crate::ui::Ui;

#[derive(serde::Serialize, serde::Deserialize)]
enum SocketMessage {
    ConnectSuccess {
        lobby_id: u64,
        user_id: u32,
        peers_id: Vec<u32>,
    },
    WebRtcHandshake {
        source_id: u32,
        target_id: u32,
        sdp_description: String,
        ice_candidates: Vec<(String, Option<String>, Option<u16>)>,
    }
}
pub struct Lobby {
    ui: Ui,
    name: String,
    peer_network: PeerNetwork,
    address: String,
}
impl Lobby {
    pub fn new(event_handler: EventHandlerProxy<Event>, name: String) -> Self {
        let window = web_sys::window().unwrap_throw();
        let location = window.location();
        let protocol = location.protocol().unwrap_throw();
        let host = location.host().unwrap_throw();
        let path = location.pathname().unwrap_throw();
        let search = location.search().unwrap_throw();
        let ws_protocol = if protocol.contains("https:") { "wss:" } else { "ws:" };
        let ws_address = format!("{ws_protocol}//{host}{path}ws{search}");

        let event_handler_clone = event_handler.clone();
        let web_socket = WebSocket::new(ws_address.as_str(), move |message| {
            event_handler_clone.call(Event::WebSocketMessage(message));
        });

        let peer_network = PeerNetwork::new();
            peer_network.set_handshake_callback(move |handshake| {
                let message = SocketMessage::WebRtcHandshake {
                    source_id: handshake.source_id,
                    target_id: handshake.target_id,
                    sdp_description: handshake.sdp_description,
                    ice_candidates: handshake.ice_candidates.into_iter().map(|v| v.into()).collect(),
                };
                let serialized = bincode::serialize(&message).unwrap_throw();
                web_socket.send_with_u8_array(serialized.as_slice());
            });
            peer_network.set_message_callback(move |message| {
                event_handler.call(Event::PeerMessage(message));
            });

        let ui = Ui::new();
            ui.div().with_class("row heading").text("Yahtzee!");
        Self {
            ui,
            name,
            peer_network,
            address: format!("{protocol}//{host}{path}?lobby_id="),
        }
    }
    pub fn web_socket_message(&mut self, message: WebSocketMessage) {
        match message {
            WebSocketMessage::String(_) => {}
            WebSocketMessage::Binary(data) => {
                log::info!("Received websocket message.");
                let message: SocketMessage = bincode::deserialize(data.as_slice()).unwrap_throw();
                match message {
                    SocketMessage::ConnectSuccess { lobby_id, user_id, peers_id } => {
                        let invite_link = format!("{}{}", self.address, lobby_id);
                        log::info!("Assigned id {} in lobby {} with {} users", user_id, lobby_id, peers_id.len());
                        let row = self.ui.div().with_class("row");
                            row.text("Invite code to lobby: ");
                            row.anchor().with_text(invite_link.as_str()).with_link(invite_link.as_str());

                        self.peer_network.set_user_id(user_id);
                        for peer_id in peers_id {
                            self.peer_network.initiate_handshake(peer_id);
                        }
                    }
                    SocketMessage::WebRtcHandshake { source_id, target_id, sdp_description, ice_candidates } => {
                        self.peer_network.receive_handshake(PeerHandshake {
                            source_id,
                            target_id,
                            sdp_description,
                            ice_candidates: ice_candidates.into_iter().map(|v| v.into()).collect(),
                        });
                    }
                }
            }
        }
    }
    pub fn peer_message(&mut self, message: PeerMessage) {
        match message {
            PeerMessage::String(data) => self.ui.div().text(data.as_str()),
            PeerMessage::Binary(_) => {}
        }
    }
    pub fn mousedown(&mut self, offset: (f32, f32)) {
        self.ui.div().text(format!("Sending click event at ({}, {}) to peers", offset.0, offset.1).as_str());
        self.peer_network.broadcast_str(format!("Click from {} at ({}, {})", self.name, offset.0, offset.1).as_str());
    }
}