use wasm_bindgen::prelude::*;
use crate::game::events::{LobbyJoin, EventSender};
use super::Context;

use super::events::{Event, WebSocketEvent, WebSocketMessage};
use crate::networks::{websocket::WebSocket};

pub struct Connecting {
    event_sender: EventSender,
    pub web_socket: WebSocket<WebSocketMessage>,
    pub name: String,
}
impl Connecting {
    pub fn new(ctx: &mut Context, name: String) -> Self {
        let window = web_sys::window().unwrap_throw();
        let location = window.location();
        let protocol = location.protocol().unwrap_throw();
        let host = location.host().unwrap_throw();
        let path = location.pathname().unwrap_throw();
        let search = location.search().unwrap_throw();
        let ws_protocol = if protocol.contains("https:") { "wss:" } else { "ws:" };
        let ws_address = format!("{ws_protocol}//{host}{path}ws{search}");

        let event_sender = ctx.event_sender.clone();
        let web_socket = WebSocket::new(ws_address.as_str(), move |message| {
            event_sender.queue(Event::WebSocketEvent(message));
        });

        Self {
            event_sender: ctx.event_sender.clone(),
            web_socket,
            name
        }
    }
    pub fn web_socket_event(&mut self, message: WebSocketEvent) {
        match message {
            WebSocketEvent::Connect => {}
            WebSocketEvent::Disconnect => {}
            WebSocketEvent::Message(WebSocketMessage::ConnectSuccess { lobby_id, user_id, peers_id }) => {
                self.event_sender.queue(Event::LobbyJoin(LobbyJoin { lobby_id, user_id, peers_id }));
            }
            _ => {}
        }
    }
}