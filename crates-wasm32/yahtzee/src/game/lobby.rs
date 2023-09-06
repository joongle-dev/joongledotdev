use wasm_bindgen::prelude::*;
use std::collections::BTreeMap;
use crate::game::events::LobbyJoin;
use super::Context;

use super::events::{Event, WebSocketEvent, PeerNetworkEvent, WebSocketMessage, PeerMessage};
use crate::networks::{websocket::WebSocket, peernetwork::PeerNetwork};
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
        let display_container = Div::new(document).with_class("user-display-container");
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
    ui: Ui,
    display_users: Div,
    name: String,
    web_socket: WebSocket<WebSocketMessage>,
    peer_network: PeerNetwork<PeerMessage>,
    users_list: BTreeMap<u32, UserData>,
}
impl Lobby {
    pub fn new(ctx: &mut Context, lobby_join: LobbyJoin, web_socket: WebSocket<WebSocketMessage>, name: String) -> Self {
        let window = web_sys::window().unwrap_throw();
        let location = window.location();
        let protocol = location.protocol().unwrap_throw();
        let host = location.host().unwrap_throw();
        let path = location.pathname().unwrap_throw();
        let invite_link = format!("{protocol}//{host}{path}?lobby_id={}", lobby_join.lobby_id);
        let ui = Ui::new();
            ui.div().with_class("row heading").text("Yahtzee!");
        let row = ui.div().with_class("row");
            row.text("Invite code to lobby: ");
            row.anchor().with_text(invite_link.as_str()).with_link(invite_link.as_str());
        let display_users = ui.div();
        log::info!("Assigned id {} in lobby {} with {} users", lobby_join.user_id, lobby_join.lobby_id, lobby_join.peers_id.len());

        let mut peer_network = PeerNetwork::new();
        peer_network.set_user_id(lobby_join.user_id);
        let event_sender = ctx.event_sender.clone();
        peer_network.set_event_callback(move |message| {
            event_sender.queue(Event::PeerNetworkEvent(message));
        });
        for peer_id in lobby_join.peers_id {
            peer_network.initiate_handshake(peer_id);
        }

        let mut lobby_state = Self {
            ui,
            display_users,
            name,
            web_socket,
            peer_network,
            users_list: BTreeMap::new(),
        };
        let user = UserData::new();
        user.set_name(lobby_state.name.as_str());
        lobby_state.add_user(lobby_join.user_id);
        lobby_state
    }
    pub fn update(&mut self, timestamp: f64) {
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
                        self.peer_network.send(peer_id, &PeerMessage::Pong(self.name.clone()));
                    }
                    PeerMessage::Pong(name) => {
                        self.set_user_name(peer_id, name.as_str());
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
        self.users_list.insert(user_id, UserData::new());
        for peer in self.users_list.values() {
            self.display_users.append_child(&peer.display_container);
        }
    }
    fn remove_user(&mut self, user_id: u32) {
        if let Some(user) = self.users_list.remove(&user_id) {
            self.display_users.remove_child(&user.display_container);
        }
    }
    fn set_user_name(&self, user_id: u32, name: &str) {
        if let Some(user) = self.users_list.get(&user_id) {
            user.display_name.text(name);
        }
    }
}