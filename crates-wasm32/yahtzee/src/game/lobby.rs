use wasm_bindgen::prelude::*;
use std::collections::BTreeMap;

use super::events::{GameEvent, WebSocketEvent, PeerNetworkEvent, WebSocketMessage, PeerMessage};
use crate::network::{web_socket::WebSocket, peer_network::PeerNetwork};
use crate::event_loop::EventSender;
use crate::ui::{Ui, div::Div};

struct UserData {
    display_container: Div,
    display_name: Div,
    display_ping: Div,
    ping_timestamp: Option<f64>,
}
impl UserData {
    fn new() -> Self {
        let document = web_sys::window().unwrap_throw().document().unwrap_throw();
        let display_container = Div::new(document).with_class("user-display");
        let display_name = display_container.div();
        let display_ping = display_container.div();
        Self {
            display_container,
            display_name,
            display_ping,
            ping_timestamp: None,
        }
    }
    fn set_name(&self, name: &str) {
        self.display_name.clear();
        self.display_name.text(name);
    }
}
impl Drop for UserData {
    fn drop(&mut self) {
        self.display_container.remove();
    }
}

pub struct Lobby {
    _ui: Ui,
    display_users: Div,
    username: String,
    web_socket: WebSocket<WebSocketMessage>,
    peer_network: PeerNetwork<PeerMessage>,
    users_list: BTreeMap<u32, UserData>,
}
impl Lobby {
    pub fn new(event_sender: EventSender<GameEvent>,
               web_socket: WebSocket<WebSocketMessage>,
               lobby_id: u64,
               username: String,
               user_id: u32,
               peers_id: Vec<u32>) -> Self {
        let window = web_sys::window().unwrap_throw();
        let location = window.location();
        let protocol = location.protocol().unwrap_throw();
        let host = location.host().unwrap_throw();
        let path = location.pathname().unwrap_throw();
        let invite_link = format!("{protocol}//{host}{path}?lobby_id={}", lobby_id);
        let ui = Ui::new();
            ui.div().with_class("row heading").text("Yahtzee!");
        let row = ui.div().with_class("row");
            row.text("Invite code to lobby: ");
            row.anchor().with_text(invite_link.as_str()).with_link(invite_link.as_str());
        let display_users = ui.div().with_class("user-display-list");
        log::info!("Assigned id {} in lobby {} with {} users", user_id, lobby_id, peers_id.len());

        let peer_network = PeerNetwork::new(user_id, move |message| {
            event_sender.send(GameEvent::PeerNetworkEvent(message));
        });
        for &peer_id in peers_id.iter() {
            peer_network.initiate_handshake(peer_id);
        }

        let mut lobby_state = Self {
            _ui: ui,
            display_users,
            username: username.clone(),
            web_socket,
            peer_network,
            users_list: BTreeMap::new(),
        };
        for peer_id in peers_id {
            lobby_state.add_user(peer_id)
        }
        lobby_state.add_user(user_id);
        lobby_state.update_user(user_id, username.as_str());
        lobby_state
    }
    pub fn update(&mut self, _timestamp: f64) {
    }
    pub fn web_socket_event(&mut self, message: WebSocketEvent) {
        match message {
            WebSocketEvent::Connect => {}
            WebSocketEvent::Disconnect => {}
            WebSocketEvent::Message(message) => if let WebSocketMessage::PeerHandshake { .. } = message {
                self.peer_network.receive_handshake(message.into());
            }
        }
    }
    pub fn peer_network_event(&mut self, message: PeerNetworkEvent) {
        match message {
            PeerNetworkEvent::Connect(peer_id) => {
                self.add_user(peer_id);
                self.peer_network.send(peer_id, &PeerMessage::Ping);
            },
            PeerNetworkEvent::Disconnect(peer_id) => {
                self.remove_user(peer_id)
            },
            PeerNetworkEvent::Message(peer_id, message) => {
                match message {
                    PeerMessage::Ping => {
                        log::info!("Received ping");
                        self.peer_network.send(peer_id, &PeerMessage::Pong(self.username.clone()));
                    }
                    PeerMessage::Pong(name) => {
                        log::info!("Received pong");
                        self.update_user(peer_id, name.as_str());
                    }
                }
            },
            PeerNetworkEvent::Handshake(handshake) => {
                self.web_socket.send(handshake.into());
            }
        }
    }
    pub fn mousedown(&mut self, offset: (f32, f32)) {

    }
    fn add_user(&mut self, user_id: u32) {
        let user = UserData::new();
            user.set_name("Connecting...");
        self.users_list.insert(user_id, user);
        for peer in self.users_list.values() {
            self.display_users.append_child(&peer.display_container);
        }
    }
    fn remove_user(&mut self, user_id: u32) {
        if let Some(user) = self.users_list.remove(&user_id) {
            self.display_users.remove_child(&user.display_container);
        }
    }
    fn update_user(&self, user_id: u32, name: &str) {
        if let Some(user) = self.users_list.get(&user_id) {
            user.display_name.clear();
            user.display_name.text(name);
        }
    }
}